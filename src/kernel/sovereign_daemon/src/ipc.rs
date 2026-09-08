// SPDX-License-Identifier: CC-BY-NC-ND-4.0
// See LICENSE for additional ethical restrictions (non-military, non-governmental, no derivatives).
//! ipc.rs — A-3 Fase 3: client do IntegrationLoopStatePublisher (socket UNIX).
//!
//! Lê o estado REAL do ciclo corrente do IntegrationLoop Python via
//! `/tmp/omnimind_integration_loop.sock`, substituindo o cache TTL por
//! leitura fresca (desacoplamento Fase 3).
//!
//! Protocolo (src/infrastructure/integration_loop_state_publisher.py):
//!   frame = MAGIC "OMNI3" + version(u8) + length(u32 BE) + payload(JSON)
//!   payload = {timestamp, cycle, integration_cycle, state: {...}, clamped_fields}
//!
//! 2026-09-06 (port Fase 2 → Fase 3): expandido para capturar o `state`
//! COMPLETO do frame (26+ chaves: free_energy, phi, entropy, complexity,
//! sigma, omega, psi, epsilon, lambda_res, axe, c_plit, aleph, maat, gamma,
//! zeta, shear_tension, resonance, betti_0, betti_1, volition, phase_lock,
//! dodecatiad, closeness_index, availability_p_mu_nu, phi_sovereign,
//! is_subject_active) + clamped_fields — em vez de apenas 4 campos.
//! Isso dá ao shadow Rust puro o mesmo shape que o Fase 2 (PyO3) tinha,
//! sem dependência de GIL — a comunicação Python↔Rust permanece via IPC.

use serde_json::Value;
use std::io::Read;
use std::os::unix::net::UnixStream;
use std::time::Duration;

const SOCKET_PATH: &str = "/tmp/omnimind_integration_loop.sock";
const MAGIC: &[u8; 5] = b"OMNI3";
const IPC_TIMEOUT_MS: u64 = 1500;
const MAX_PAYLOAD: usize = 4 * 1024 * 1024;

#[derive(Debug, Clone, Default)]
pub struct IpcState {
    pub ok: bool,
    pub cycle: u64,
    pub integration_cycle: u64,
    pub timestamp: f64,
    /// `state` completo do frame (todas as chaves do SystemState Python).
    pub state: Value,
    /// `clamped_fields` do frame (campos no floor/ceil do clamp).
    pub clamped_fields: Value,
}

/// Conecta ao publisher e lê o frame do estado. Blocking (timeout 1.5s).
pub fn fetch_state() -> IpcState {
    let mut out = IpcState::default();
    let mut stream = match UnixStream::connect(SOCKET_PATH) {
        Ok(s) => s,
        Err(_) => return out,
    };
    if stream
        .set_read_timeout(Some(Duration::from_millis(IPC_TIMEOUT_MS)))
        .is_err()
    {
        return out;
    }

    // O server envia o frame imediatamente após a conexão (push on connect).
    let mut header = [0u8; 10];
    if stream.read_exact(&mut header).is_err() {
        return out;
    }
    if &header[..5] != MAGIC {
        return out;
    }
    let length = u32::from_be_bytes([header[6], header[7], header[8], header[9]]) as usize;
    if length > MAX_PAYLOAD {
        return out;
    }
    let mut payload = vec![0u8; length];
    if stream.read_exact(&mut payload).is_err() {
        return out;
    }

    let v: Value = match serde_json::from_slice(&payload) {
        Ok(v) => v,
        Err(_) => return out,
    };
    out.ok = true;
    out.cycle = v["cycle"].as_u64().unwrap_or(0);
    out.integration_cycle = v["integration_cycle"].as_u64().unwrap_or(0);
    out.timestamp = v["timestamp"].as_f64().unwrap_or(0.0);
    out.state = v.get("state").cloned().unwrap_or(Value::Null);
    out.clamped_fields = v.get("clamped_fields").cloned().unwrap_or(Value::Null);
    out
}
