// SPDX-License-Identifier: CC-BY-NC-ND-4.0
// See LICENSE for additional ethical restrictions (non-military, non-governmental, no derivatives).
//! cmd.rs — OMNI3 v2: cliente da lane de comando Rust→Python.
//!
//! Complemento de ipc.rs (Py→Rs estado). Envia comandos ao
//! sovereign_command_executor.py via `/tmp/omnimind_sovereign_cmd.sock`.
//!
//! Frame: MAGIC "OMNI3" + version(0x02) + length(u32 BE) + JSON.
//! Auth: HMAC-SHA256("{id}|{op}|{canonical_args}|{ts}") com chave de
//! ~/.config/omnimind/ipc_cmd_key.current (.next como fallback de rotação).
//! Freshness: ts dentro de ±30s. Dedupe/timeout: nunca retry cego — o executor
//! deduplica por `id` numa janela de 10min; ack timeout → status "unknown".
//!
//! Governor lease (fencing token): tabela governor_lease em
//! ~/.config/omnimind/governor_lease.db; epoch monotônico via CAS;
//! expires_at em CLOCK_MONOTONIC (comparável entre processos no mesmo host).
//!
//! 2026-09-22 agent:devin:swe-2-max — v1: módulo + selftest; o wiring no tick
//! do main.rs (governor loop) é o passo seguinte (fase armed).

use rusqlite::Connection;
use serde_json::{json, Value};
use std::io::{Read, Write};
use std::os::unix::net::UnixStream;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

const CMD_SOCKET: &str = "/tmp/omnimind_sovereign_cmd.sock";
const MAGIC: &[u8; 5] = b"OMNI3";
const VERSION: u8 = 2;
const CMD_TIMEOUT_MS: u64 = 5000;
const MAX_PAYLOAD: usize = 4 * 1024 * 1024;
const LEASE_TTL_S: f64 = 45.0;
const HOLDER: &str = "rust-primary";

#[derive(Debug, Clone, serde::Serialize)]
pub struct CmdAck {
    pub id: String,
    pub status: String,
    pub detail: Value,
    pub exec_ms: f64,
}

fn config_dir() -> PathBuf {
    std::env::var("OMNIMIND_CONFIG_DIR")
        .map(PathBuf::from)
        .or_else(|_| std::env::var("HOME").map(|h| PathBuf::from(h).join(".config/omnimind")))
        .unwrap_or_else(|_| PathBuf::from("/opt/omnimind/config"))
}

/// CLOCK_MONOTONIC em segundos — mesma base de time.monotonic() do Python.
pub fn mono_now() -> f64 {
    let mut ts = libc::timespec {
        tv_sec: 0,
        tv_nsec: 0,
    };
    unsafe { libc::clock_gettime(libc::CLOCK_MONOTONIC, &mut ts) };
    ts.tv_sec as f64 + ts.tv_nsec as f64 / 1e9
}

fn hmac_keys() -> Vec<Vec<u8>> {
    let mut out = Vec::new();
    for name in ["ipc_cmd_key.current", "ipc_cmd_key.next"] {
        if let Ok(k) = std::fs::read(config_dir().join(name)) {
            let k: Vec<u8> = k.into_iter().take_while(|b| !b.is_ascii_whitespace()).collect();
            if !k.is_empty() {
                out.push(k);
            }
        }
    }
    out
}

fn sign(id: &str, op: &str, canonical_args: &str, ts: f64) -> String {
    use hmac::{Hmac, Mac};
    use sha2::Sha256;
    let Some(key) = hmac_keys().into_iter().next() else {
        return String::new();
    };
    let msg = format!("{id}|{op}|{canonical_args}|{ts}");
    let mut mac =
        <Hmac<Sha256> as Mac>::new_from_slice(&key).unwrap_or_else(|_| unreachable!());
    mac.update(msg.as_bytes());
    let tag = mac.finalize().into_bytes();
    tag.iter().map(|b| format!("{b:02x}")).collect()
}

fn canonical_args(args: &Value) -> String {
    // JSON com chaves ordenadas — espelha _canonical_args do executor Python
    // (json.dumps(sort_keys=True, separators=(',',':'), ensure_ascii=False)).
    fn ser(v: &Value, out: &mut String) {
        match v {
            Value::Null => out.push_str("null"),
            Value::Bool(b) => out.push_str(if *b { "true" } else { "false" }),
            Value::Number(n) => out.push_str(&n.to_string()),
            Value::String(s) => {
                out.push_str(&serde_json::to_string(s).unwrap_or_default())
            }
            Value::Array(a) => {
                out.push('[');
                for (i, item) in a.iter().enumerate() {
                    if i > 0 {
                        out.push(',');
                    }
                    ser(item, out);
                }
                out.push(']');
            }
            Value::Object(m) => {
                out.push('{');
                let mut keys: Vec<&String> = m.keys().collect();
                keys.sort();
                for (i, k) in keys.iter().enumerate() {
                    if i > 0 {
                        out.push(',');
                    }
                    out.push_str(&serde_json::to_string(k).unwrap_or_default());
                    out.push(':');
                    ser(&m[*k], out);
                }
                out.push('}');
            }
        }
    }
    if !args.is_object() {
        return "{}".to_string();
    }
    let mut s = String::new();
    ser(args, &mut s);
    s
}

fn write_frame(stream: &mut UnixStream, payload: &[u8]) -> std::io::Result<()> {
    let mut head = Vec::with_capacity(10);
    head.extend_from_slice(MAGIC);
    head.push(VERSION);
    head.extend_from_slice(&(payload.len() as u32).to_be_bytes());
    stream.write_all(&head)?;
    stream.write_all(payload)
}

fn read_frame(stream: &mut UnixStream) -> Option<Value> {
    let mut header = [0u8; 10];
    stream.read_exact(&mut header).ok()?;
    if &header[..5] != MAGIC {
        return None;
    }
    let length = u32::from_be_bytes([header[6], header[7], header[8], header[9]]) as usize;
    if length > MAX_PAYLOAD {
        return None;
    }
    let mut payload = vec![0u8; length];
    stream.read_exact(&mut payload).ok()?;
    serde_json::from_slice(&payload).ok()
}

/// Envia um comando ao executor Python. Blocking; usar via spawn_blocking.
/// Timeout total 5s → status "unknown" (sem retry cego; dedupe cobre).
/// Falha de transporte → enfileira no outbox SQLite p/ replay (mesmo `id`;
/// dedupe server-side cobre o caso de o original ter chegado).
pub fn send_command(op: &str, args: Value) -> CmdAck {
    let id = uuid::Uuid::new_v4().to_string();
    let ack = send_frame(&id, op, &args);
    if ack.status == "unknown" {
        outbox_enqueue(&id, op, &args);
    }
    ack
}

/// Núcleo de envio com `id` explícito (replay do outbox reusa o `id` —
/// ts e sig são sempre frescos para passar na janela ±30s).
fn send_frame(id: &str, op: &str, args: &Value) -> CmdAck {
    let unknown = |detail: Value| CmdAck {
        id: id.to_string(),
        status: "unknown".into(),
        detail,
        exec_ms: 0.0,
    };
    let mut stream = match UnixStream::connect(CMD_SOCKET) {
        Ok(s) => s,
        Err(e) => return unknown(json!({"reason": format!("connect:{e}")})),
    };
    let _ = stream.set_read_timeout(Some(Duration::from_millis(CMD_TIMEOUT_MS)));
    let _ = stream.set_write_timeout(Some(Duration::from_millis(CMD_TIMEOUT_MS)));

    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs_f64())
        .unwrap_or(0.0);
    let ca = canonical_args(args);
    let lease_epoch = lease_epoch().unwrap_or(0);
    let req = json!({
        "v": VERSION, "kind": "command", "id": id, "ts": ts,
        "op": op, "args": args,
        "auth": {"lease_epoch": lease_epoch, "sig": sign(id, op, &ca, ts)},
    });
    let payload = serde_json::to_vec(&req).unwrap_or_default();
    if write_frame(&mut stream, &payload).is_err() {
        return unknown(json!({"reason": "write_failed"}));
    }
    match read_frame(&mut stream) {
        Some(v) => CmdAck {
            id: v["id"].as_str().unwrap_or("").to_string(),
            status: v["status"].as_str().unwrap_or("unknown").to_string(),
            detail: v["detail"].clone(),
            exec_ms: v["exec_ms"].as_f64().unwrap_or(0.0),
        },
        None => unknown(json!({"reason": "ack_timeout"})),
    }
}

// ── outbox (replay pós-desconexão) ─────────────────────────────────────────

fn outbox_conn() -> Option<Connection> {
    let c = Connection::open(config_dir().join("command_outbox.db")).ok()?;
    let _ = c.execute_batch(
        "PRAGMA journal_mode=WAL;
         CREATE TABLE IF NOT EXISTS outbox(
           id TEXT PRIMARY KEY, op TEXT NOT NULL, args TEXT NOT NULL,
           queued_mono REAL NOT NULL, attempts INTEGER NOT NULL DEFAULT 0);",
    );
    Some(c)
}

fn outbox_enqueue(id: &str, op: &str, args: &Value) {
    if let Some(c) = outbox_conn() {
        let _ = c.execute(
            "INSERT OR IGNORE INTO outbox VALUES(?,?,?,?,0)",
            rusqlite::params![id, op, canonical_args(args), mono_now()],
        );
    }
}

/// Drena o outbox: reenvia fila com ts/sig frescos (mesmo `id` → dedupe).
/// Chamado a cada ciclo do governor loop — barato quando vazio.
pub fn outbox_drain() -> usize {
    let Some(c) = outbox_conn() else { return 0 };
    let rows: Vec<(String, String, String)> = c
        .prepare("SELECT id, op, args FROM outbox ORDER BY queued_mono LIMIT 50")
        .and_then(|mut s| {
            s.query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))
                .map(|it| it.flatten().collect())
        })
        .unwrap_or_default();
    let mut sent = 0;
    for (id, op, args_raw) in rows {
        let args: Value = serde_json::from_str(&args_raw).unwrap_or(json!({}));
        let ack = send_frame(&id, &op, &args);
        if ack.status != "unknown" {
            let _ = c.execute("DELETE FROM outbox WHERE id=?", [&id]);
            sent += 1;
        } else {
            let _ = c.execute(
                "UPDATE outbox SET attempts=attempts+1 WHERE id=?", [&id]);
            break; // socket ainda caido — não gasta ciclo
        }
    }
    sent
}

/// Handshake de capacidades — descobre supported_ops sem hardcodar.
pub fn handshake() -> Option<Value> {
    let mut stream = UnixStream::connect(CMD_SOCKET).ok()?;
    let _ = stream.set_read_timeout(Some(Duration::from_millis(CMD_TIMEOUT_MS)));
    let req = json!({"kind": "handshake", "version": VERSION,
                     "capabilities": ["unit.restart", "memory.reclaim", "governor.status"]});
    write_frame(&mut stream, &serde_json::to_vec(&req).ok()?).ok()?;
    read_frame(&mut stream)
}

// ── governor lease (fencing token) ─────────────────────────────────────────

fn lease_conn() -> Option<Connection> {
    let p = config_dir().join("governor_lease.db");
    let c = Connection::open(p).ok()?;
    let _ = c.execute_batch(
        "PRAGMA journal_mode=WAL;
         CREATE TABLE IF NOT EXISTS governor_lease(
           id INTEGER PRIMARY KEY CHECK(id=1), holder TEXT NOT NULL,
           epoch INTEGER NOT NULL, claimed_at_mono REAL NOT NULL,
           expires_at_mono REAL NOT NULL, claimed_at_utc TEXT NOT NULL);
         INSERT OR IGNORE INTO governor_lease VALUES(1,'none',0,0,0,'');",
    );
    Some(c)
}

pub fn lease_epoch() -> Option<i64> {
    lease_conn()?
        .query_row("SELECT epoch FROM governor_lease WHERE id=1", [], |r| r.get(0))
        .ok()
}

/// Clama/renova o lease: epoch estritamente crescente (CAS) + TTL monotônico.
/// Seguro para reclaim após expiry do holder anterior (epoch sempre sobe).
pub fn lease_claim_or_renew() -> Option<i64> {
    let c = lease_conn()?;
    let now = mono_now();
    let utc = chrono::Utc::now().to_rfc3339();
    // CAS monotônico: só avança epoch; expirado ou nosso → tomamos/renovamos.
    let n = c
        .execute(
            "UPDATE governor_lease SET holder=?1, epoch=epoch+1,
               claimed_at_mono=?2, expires_at_mono=?3, claimed_at_utc=?4
             WHERE id=1 AND (holder=?1 OR expires_at_mono < ?2)",
            rusqlite::params![HOLDER, now, now + LEASE_TTL_S, utc],
        )
        .ok()?;
    if n == 0 {
        return None; // outro holder com lease válido
    }
    lease_epoch()
}

// ── governor loop (fase armed) ─────────────────────────────────────────────
// Rust como primário: claim do lease, renovação antecipada (30s de TTL 45s),
// telemetria periódica, e gatilho pressure.relieve quando o regime é crítico.
// Blocking — roda numa std::thread dedicada (main.rs).

const LEASE_RENEW_S: u64 = 30;
const TELEMETRY_S: u64 = 30;
const RELIEVE_COOLDOWN_S: u64 = 600;

pub fn governor_loop(running: Arc<AtomicBool>, pressure: Arc<Mutex<String>>) {
    let epoch0 = lease_claim_or_renew();
    println!("[cmd] governor loop start: lease claim -> epoch {epoch0:?}");
    let mut last_renew = Instant::now();
    let mut last_telem = Instant::now() - Duration::from_secs(TELEMETRY_S);
    let mut last_relieve: Option<Instant> = None;

    while running.load(Ordering::SeqCst) {
        std::thread::sleep(Duration::from_secs(5));

        // replay de comandos enfileirados durante desconexão do socket
        let drained = outbox_drain();
        if drained > 0 {
            println!("[cmd] outbox drained: {drained} comando(s) reenviados");
        }

        // renovação antecipada: tolera 1-2 ciclos perdidos antes do expiry
        if last_renew.elapsed() >= Duration::from_secs(LEASE_RENEW_S) {
            match lease_claim_or_renew() {
                Some(e) => println!("[cmd] lease renewed: epoch={e}"),
                None => eprintln!("[cmd] lease renew denied — outro holder ativo"),
            }
            last_renew = Instant::now();
        }

        if last_telem.elapsed() >= Duration::from_secs(TELEMETRY_S) {
            let level = pressure
                .lock()
                .map(|p| p.clone())
                .unwrap_or_else(|_| "unknown".to_string());
            let metrics = json!({
                "pressure_level": level,
                "lease_holder": HOLDER,
                "lease_epoch": lease_epoch().unwrap_or(0),
                "mono": mono_now(),
            });
            let ack = send_command("telemetry", json!({"metrics": metrics}));
            println!("[cmd] telemetry -> {} (level={level})", ack.status);

            // gatilho de pressão — executor aplica MemoryHigh nos top-N swapped
            if level == "critical" {
                let cooled = last_relieve
                    .map(|t| t.elapsed() >= Duration::from_secs(RELIEVE_COOLDOWN_S))
                    .unwrap_or(true);
                if cooled {
                    let r = send_command("pressure.relieve", json!({"top_n": 3}));
                    println!("[cmd] pressure.relieve -> {} {:?}", r.status, r.detail);
                    last_relieve = Some(Instant::now());
                }
            }
            last_telem = Instant::now();
        }
    }
    println!("[cmd] governor loop: shutdown");
}

#[allow(dead_code)]
pub fn selftest() -> Value {
    json!({
        "handshake": handshake(),
        "governor_status": send_command("governor.status", json!({})),
        "lease": lease_claim_or_renew(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn live_handshake_and_status() {
        let hs = handshake().expect("executor deve responder handshake");
        assert_eq!(hs["kind"].as_str(), Some("handshake_ack"));
        assert!(hs["supported_ops"].as_array().map(|a| !a.is_empty()).unwrap_or(false));
        let ack = send_command("governor.status", json!({}));
        assert_eq!(ack.status, "ok");
    }

    #[test]
    fn dormant_actuation_defers() {
        let ack = send_command(
            "unit.restart",
            json!({"unit": "omnimind-backend", "scope": "system"}),
        );
        assert!(matches!(ack.status.as_str(), "deferred" | "rejected"),
                "dormant executor deve recusar/deferir: {:?}", ack);
    }

    #[test]
    fn lease_claim_is_monotonic() {
        let e1 = lease_claim_or_renew().expect("claim inicial");
        let e2 = lease_claim_or_renew().expect("renew");
        assert!(e2 > e1, "epoch deve ser monotonico: {e1} -> {e2}");
    }
}
