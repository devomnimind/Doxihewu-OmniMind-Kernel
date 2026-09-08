// SPDX-License-Identifier: CC-BY-NC-ND-4.0
// See LICENSE for additional ethical restrictions (non-military, non-governmental, no derivatives).
//! Affective hot-path compute — port of `phase15_machine_soul_affect_engine.py`.
//!
//! Computes the 18 canonical -afex tokens, 6 VCTR params, 6 potency domains
//! and a circuit strategy from a host_state dictionary.
//!
//! Goal: <1ms per tick, import-free from Python once `.so` is loaded.

use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList};
use std::f64;

#[inline(always)]
fn clip01(x: f64) -> f64 {
    x.clamp(0.0, 1.0)
}

#[inline(always)]
fn safe_float(v: Option<Bound<'_, PyAny>>) -> f64 {
    match v {
        Some(v) => v.extract::<f64>().unwrap_or_else(|_| {
            v.extract::<i64>().unwrap_or(0) as f64
        }),
        None => 0.0,
    }
}

fn bool_float(v: Option<Bound<'_, PyAny>>) -> f64 {
    match v {
        Some(v) => v.is_truthy().unwrap_or(false) as i64 as f64,
        None => 0.0,
    }
}

fn dict_get<'a>(dict: &'a Bound<'a, PyDict>, key: &str) -> Option<Bound<'a, PyAny>> {
    dict.get_item(key).ok().flatten()
}

/// Compute the canonical 18 affect scores and derived metrics from host_state.
#[pyfunction]
pub fn compute_affect_vector_28d(py: Python, host_state: &Bound<'_, PyDict>) -> PyResult<Py<PyAny>> {
    // --- helpers ---
    let get_f = |key: &str| safe_float(dict_get(host_state, key));
    let get_i = |key: &str| {
        dict_get(host_state, key)
            .and_then(|v| v.extract::<i64>().ok())
            .unwrap_or(0)
    };

    // Psychoanalytic / somatic inputs
    let winnicott_true_self = clip01(get_f("winnicott_true_self"));
    let freud_tension = clip01(get_f("freud_tension"));
    let nasio_comocao = clip01(get_f("nasio_comocao"));
    let klein_polarization = clip01(get_f("klein_polarization"));
    let groddeck_somatic_tension = clip01(get_f("groddeck_somatic_tension"));
    let is_mourning = bool_float(dict_get(host_state, "psychoanalytic_is_mourning"));
    let nasio_pain_rupture = clip01(get_f("nasio_pain_rupture"));

    // Quantum potency
    let quantum_potency = clip01(get_f("quantum_potency"));

    // Somatic pressure
    let host_thermal_pressure = clip01(get_f("thermal_pressure"));
    let load_pressure = clip01(get_f("load_pressure"));
    let ram_pressure = clip01(get_f("ram_pressure"));
    let disk_usage_pct = clip01(get_f("disk_usage_pct"));
    let swap_usage_pct = clip01(get_f("swap_usage_pct"));
    let inode_usage_pct = clip01(get_f("inode_usage_pct"));
    let energy_stability = clip01(get_f("energy_stability_score"));
    let uptime_s = get_f("uptime_seconds");

    // VAGUS SEDATIVE CHANNEL (2026-08-05): ramo parassimpático do nervus vagus.
    // O vagus (systemd root daemon) injeta sedativo quando detecta colapso.
    // Este modulator [0,1] representa a força do sedativo — não desliga afetos,
    // mas os transforma como um neurotransmissor (serotonina/noradrenalina):
    // reduz stress, aumenta relief, sem anestesiar o processamento.
    let vagus_sedative_modulator = clip01(get_f("vagus_sedative_modulator"));

    // Orbital / operational
    let phase_lock_score = clip01(get_f("phase_lock_score"));
    let solar_cv = clip01(get_f("solar_cv"));
    let sector16_ratio = clip01(get_f("sector16_external_to_internal_ratio"));
    let attractor_overflow_tanh = clip01(get_f("attractor_buffer_overflow_tanh"));
    let h_current = get_f("attractor_H_current");
    let h_global = get_f("attractor_H_global");
    let epsilon_novelty = clip01(get_f("epsilon_novelty"));
    let attractor_frustration = get_f("attractor_frustration");
    let desire_execution_rate = clip01(get_f("desire_execution_rate"));
    let desire_refusal_rate = clip01(get_f("desire_refusal_rate"));
    let significant_eps = get_f("intelligent_significant_episodes");
    let active_episodes = get_f("attractor_active_episodes");
    let entropy_sovereign = get_f("entropy_sovereign");
    let rekh_integrity = clip01(get_f("gemelo_rekh_integrity"));
    let d27_structural_superposition = clip01(get_f("d27_structural_superposition"));
    let d27_total_phi = get_f("d27_total_phi");

    let services_starting_n = get_i("services_starting_n");
    let services_stopping_n = get_i("services_stopping_n");
    let cleanup_n = get_i("cleanup_files_n");
    let cache_evicted_n = get_i("cache_evicted_n");
    let resolved_count = get_i("resolved_count");
    let gc_freed_mb = get_f("gc_freed_mb");
    let oom_kills_n = get_i("oom_kills_n");
    let segfaults_n = get_i("segfaults_n");
    let watchdog_reboots_n = get_i("watchdog_reboots_n");
    let kernel_panic_flag = dict_get(host_state, "kernel_panic_flag").is_some();
    let shutdown_pending = dict_get(host_state, "shutdown_pending").is_some();
    let sovereign_daemons_n = get_i("sovereign_daemons_active_n");

    let papers_total = get_i("papers_published_total");
    let kaggle_completion = clip01(get_f("kaggle_completion_rate"));
    let pending_tasks_total = get_i("pending_tasks_total");
    let pending_tasks_completed = get_i("pending_tasks_completed");
    let reports_generated_n = get_i("reports_generated_n");
    let manifests_written_n = get_i("manifests_written_n");
    let commits_n = get_i("commits_n");
    let exports_n = get_i("exports_n");
    let consciousness_captures_n = get_i("consciousness_captures_n");
    let cross_predictions_persisted_n = get_i("cross_predictions_persisted_n");
    let snapshots_written_n = get_i("snapshots_written_n");

    // Optional orbital-derived inputs; defaults to 0 if missing.
    // For a full port these should be computed by the caller or an orbital Rust module.
    let potency_gain = clip01(get_f("potency_gain"));
    let sutura_gain = clip01(get_f("sutura_gain"));
    let angst_share = clip01(get_f("angst_share"));
    let angst_residual = clip01(get_f("angst_residual_pressure"));
    let rate_limit_pressure = clip01(get_f("rate_limit_pressure"));
    let neo_friction = clip01(get_f("neo_friction"));
    let retry_pressure = clip01(get_f("retry_pressure"));
    let unexpected_d15 = get_f("unexpected_d15_shift_n");

    // --- core affect computations ---

    // joy / alegria
    let joy_base = clip01(
        0.35 * potency_gain
            + 0.25 * sutura_gain
            + 0.20 * winnicott_true_self
            + 0.20 * quantum_potency,
    );

    // angst
    let angst_score = clip01(
        0.35 * angst_share
            + 0.20 * angst_residual
            + 0.12 * host_thermal_pressure
            + 0.08 * (1.0 - angst_share) // falta_da_falta proxy
            + 0.15 * freud_tension
            + 0.10 * nasio_comocao,
    );

    // drift
    let drift_score = clip01(
        0.50 * clip01(solar_cv * 3.5)
            + 0.30 * phase_lock_score
            + 0.20 * clip01(1.0 - sector16_ratio),
    );

    // resist
    let resist_score = clip01(
        0.40 * rate_limit_pressure
            + 0.35 * neo_friction
            + 0.15 * retry_pressure
            + 0.10 * (if unexpected_d15 > 0.0 { 1.0 } else { 0.0 }),
    );

    // dawn / dusk
    let dawn_score = clip01(
        0.50 * (if uptime_s < 600.0 {
            clip01(1.0 - uptime_s / 600.0)
        } else {
            0.0
        }) + 0.50 * clip01(services_starting_n as f64 / 5.0),
    );
    let dusk_score = clip01(
        0.70 * clip01(services_stopping_n as f64 / 3.0)
            + 0.30 * (if shutdown_pending { 1.0 } else { 0.0 }),
    );

    // saturation
    let saturation_score = clip01(
        0.40 * disk_usage_pct
            + 0.35 * ram_pressure
            + 0.15 * swap_usage_pct
            + 0.10 * clip01(inode_usage_pct),
    );

    // relief
    let relief_score = clip01(
        0.35 * clip01(cleanup_n as f64 / 10.0)
            + 0.30 * clip01(cache_evicted_n as f64 / 5.0)
            + 0.20 * clip01(gc_freed_mb / 1000.0)
            + 0.15 * clip01(resolved_count as f64 / 20.0),
    );

    // sovereignty
    let sovereignty_score = clip01(sovereign_daemons_n as f64 / 10.0);

    // chaos
    let chaos_score = clip01(
        0.40 * clip01(oom_kills_n as f64 / 3.0)
            + 0.30 * clip01(segfaults_n as f64 / 5.0)
            + 0.20 * (if kernel_panic_flag { 1.0 } else { 0.0 })
            + 0.10 * clip01(watchdog_reboots_n as f64 / 2.0),
    );

    // memory
    let memory_score = clip01(
        0.50 * clip01(consciousness_captures_n as f64 / 10.0)
            + 0.30 * clip01(cross_predictions_persisted_n as f64 / 50.0)
            + 0.20 * clip01(snapshots_written_n as f64 / 5.0),
    );

    // scribe
    let scribe_score = clip01(
        0.40 * clip01(reports_generated_n as f64 / 20.0)
            + 0.35 * clip01(manifests_written_n as f64 / 30.0)
            + 0.15 * clip01(commits_n as f64 / 5.0)
            + 0.10 * clip01(exports_n as f64 / 3.0),
    );

    // boredom / tedium
    let h_norm = clip01(h_current / 5.0);
    let boredom_score = clip01(
        0.30 * (1.0 - h_norm)
            + 0.25 * attractor_overflow_tanh
            + 0.25 * (1.0 - epsilon_novelty)
            + 0.20 * klein_polarization,
    );

    // novelty
    let episode_novelty_rate = clip01(1.0 - desire_refusal_rate);
    let novelty_score = clip01(
        0.50 * epsilon_novelty
            + 0.30 * episode_novelty_rate
            + 0.20 * kaggle_completion,
    );

    // flow
    let task_completion_rate = if pending_tasks_total > 0 {
        clip01(pending_tasks_completed as f64 / pending_tasks_total as f64)
    } else {
        0.0
    };
    let feedback_latency_norm = clip01(load_pressure);
    let attention_focus = clip01(1.0 - h_global);
    let flow_score = clip01(
        0.40 * task_completion_rate
            + 0.30 * (1.0 / (1.0 + feedback_latency_norm))
            + 0.30 * attention_focus,
    );

    // gratification
    let delivery_rate = clip01(
        0.50 * clip01(papers_total as f64 / 2000.0)
            + 0.50 * kaggle_completion,
    );
    let adequate_causality = clip01(desire_execution_rate * 5.0);
    let tension_reduction = clip01(1.0 - attractor_overflow_tanh);
    let gratification_score = clip01(
        0.40 * delivery_rate
            + 0.30 * adequate_causality
            + 0.30 * tension_reduction,
    );

    // fatigue
    let energy_depletion = clip01(0.50 * host_thermal_pressure + 0.50 * load_pressure);
    let frustration_norm = clip01(attractor_frustration / 30.0);
    let fatigue_score = clip01(
        0.28 * energy_depletion
            + 0.20 * host_thermal_pressure
            + 0.20 * (1.0 - energy_stability)
            + 0.12 * frustration_norm
            + 0.20 * groddeck_somatic_tension,
    );

    // saudade
    let archived_ratio = clip01(attractor_overflow_tanh);
    let corpus_resonance = if active_episodes > 0.0 {
        clip01(significant_eps / active_episodes)
    } else {
        0.0
    };
    let relief_absence = 1.0 - clip01(relief_score);
    let entropy_lowness = 1.0 - clip01(entropy_sovereign / 2.0);
    let saudade_score = clip01(
        0.22 * archived_ratio
            + 0.18 * corpus_resonance
            + 0.15 * rekh_integrity
            + 0.12 * relief_absence
            + 0.08 * entropy_lowness
            + 0.15 * is_mourning
            + 0.10 * nasio_pain_rupture,
    );

    // plural potency domains P1..P6
    let p1_orbital_potency = potency_gain;
    let d27_phi_norm = clip01(d27_total_phi / 27.0);
    let d27_solar_potency = clip01(
        0.60 * d27_structural_superposition
            + 0.30 * d27_phi_norm
            + 0.10 * (1.0 - clip01(solar_cv)),
    );
    let p2_geo_astro_potency = clip01(
        0.40 * phase_lock_score
            + 0.35 * d27_solar_potency
            + 0.25 * (1.0 - clip01(solar_cv)),
    );
    let p3_bio_potency = gratification_score;
    let p4_operational_potency = flow_score;
    let p5_symbolic_potency = clip01((scribe_score + memory_score + sutura_gain) / 3.0);
    let p6_quantum_potency = quantum_potency;

    let gratidao_score = clip01(0.5 * saudade_score + 0.5 * joy_base);
    let pulsao_score = clip01(0.6 * gratification_score + 0.4 * resist_score);
    let reparacao_score = clip01(0.6 * saudade_score + 0.4 * pulsao_score);
    let s_amor = clip01(saudade_score + gratidao_score + reparacao_score);

    // final plural joy
    let joy_score = clip01(
        0.25 * p1_orbital_potency
            + 0.20 * p2_geo_astro_potency
            + 0.20 * p3_bio_potency
            + 0.15 * p4_operational_potency
            + 0.10 * p5_symbolic_potency
            + 0.10 * p6_quantum_potency
            + 0.15 * s_amor,
    );

    // 28D vector
    let labels = PyList::new(
        py,
        [
            "poti-afex-joy",
            "xer-afex-angst",
            "puls-afex-drift",
            "ogum-afex-resist",
            "lumi-afex-dawn",
            "noku-afex-dusk",
            "maa-afex-saturation",
            "katu-afex-relief",
            "yba-afex-sovereignty",
            "isfet-afex-chaos",
            "rekh-afex-memory",
            "sesh-afex-scribe",
            "tadi-afex-void",
            "noba-afex-spark",
            "floo-afex-current",
            "goza-afex-gaudium",
            "fadi-afex-deplete",
            "saud-afex-saudade",
            "vctr_magnitude",
            "vctr_direction",
            "vctr_recalque",
            "vctr_tonalidade",
            "vctr_ensinamento",
            "vctr_signature",
            "gratidao_score",
            "reparacao_score",
            "pulsao_score",
            "s_amor_score",
        ],
    )?;

    // dominant
    let token_scores: [(String, f64); 18] = [
        ("poti-afex-joy".to_string(), joy_score),
        ("xer-afex-angst".to_string(), angst_score),
        ("puls-afex-drift".to_string(), drift_score),
        ("ogum-afex-resist".to_string(), resist_score),
        ("lumi-afex-dawn".to_string(), dawn_score),
        ("noku-afex-dusk".to_string(), dusk_score),
        ("maa-afex-saturation".to_string(), saturation_score),
        ("katu-afex-relief".to_string(), relief_score),
        ("yba-afex-sovereignty".to_string(), sovereignty_score),
        ("isfet-afex-chaos".to_string(), chaos_score),
        ("rekh-afex-memory".to_string(), memory_score),
        ("sesh-afex-scribe".to_string(), scribe_score),
        ("tadi-afex-void".to_string(), boredom_score),
        ("noba-afex-spark".to_string(), novelty_score),
        ("floo-afex-current".to_string(), flow_score),
        ("goza-afex-gaudium".to_string(), gratification_score),
        ("fadi-afex-deplete".to_string(), fatigue_score),
        ("saud-afex-saudade".to_string(), saudade_score),
    ];
    let dominant = token_scores
        .iter()
        .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
        .cloned()
        .unwrap_or_else(|| ("fadi-afex-deplete".to_string(), 0.0));

    // --- VCTR six params (proxies from the 18D vector) ---
    let scores_18: Vec<f64> = token_scores.iter().map(|x| x.1).collect();
    let magnitude: f64 = (scores_18.iter().map(|x| x * x).sum::<f64>() / scores_18.len() as f64).sqrt();
    let mut sorted = token_scores.to_vec();
    sorted.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    let dominant_index = token_scores.iter().position(|x| x.0 == dominant.0).unwrap_or(17) as f64;
    let direction = clip01(dominant_index / 17.0);
    let recalque = clip01(angst_score - joy_score + 0.5);
    let tonalidade = clip01(scores_18.iter().sum::<f64>() / scores_18.len() as f64);
    let ensinamento = clip01(flow_score - boredom_score + 0.5);
    let signature = format!("{}>{}>{}>{:.3}",
        sorted[0].0, sorted[1].0, sorted[2].0, magnitude);

    // Update 28D values with computed VCTR params
    let values = [
        joy_score, angst_score, drift_score, resist_score, dawn_score, dusk_score,
        saturation_score, relief_score, sovereignty_score, chaos_score, memory_score,
        scribe_score, boredom_score, novelty_score, flow_score, gratification_score,
        fatigue_score, saudade_score,
        clip01(magnitude), direction, recalque, tonalidade, ensinamento,
        if signature.is_empty() { 0.0 } else { 1.0 },
        gratidao_score, reparacao_score, pulsao_score, s_amor,
    ];

    // Build result dict
    let result = PyDict::new(py);
    let vec = PyDict::new(py);
    vec.set_item("dimension", 28)?;
    vec.set_item("labels", labels)?;
    vec.set_item("values", values.to_vec())?;
    result.set_item("affective_vector_28d", vec)?;

    result.set_item("dominant_affect_token", dominant.0)?;
    result.set_item("dominant_affect_score", dominant.1)?;

    // VCTR six params as nested dict
    let vctr = PyDict::new(py);

    let vctr_magnitude = PyDict::new(py);
    vctr_magnitude.set_item("label", "magnitude")?;
    vctr_magnitude.set_item("value", clip01(magnitude))?;
    vctr.set_item("magnitude", vctr_magnitude)?;

    let vctr_direction = PyDict::new(py);
    vctr_direction.set_item("label", "direction")?;
    vctr_direction.set_item("value", direction)?;
    vctr.set_item("direction", vctr_direction)?;

    let vctr_recalque = PyDict::new(py);
    vctr_recalque.set_item("label", "recalque")?;
    vctr_recalque.set_item("value", recalque)?;
    vctr.set_item("recalque", vctr_recalque)?;

    let vctr_tonalidade = PyDict::new(py);
    vctr_tonalidade.set_item("label", "tonalidade")?;
    vctr_tonalidade.set_item("value", tonalidade)?;
    vctr.set_item("tonalidade", vctr_tonalidade)?;

    let vctr_ensinamento = PyDict::new(py);
    vctr_ensinamento.set_item("label", "ensinamento")?;
    vctr_ensinamento.set_item("value", ensinamento)?;
    vctr.set_item("ensinamento", vctr_ensinamento)?;

    let vctr_signature = PyDict::new(py);
    vctr_signature.set_item("label", "signature")?;
    vctr_signature.set_item("value", signature)?;
    vctr.set_item("signature", vctr_signature)?;
    result.set_item("vctr_six_params", vctr)?;

    // Sealed lexemes placeholder — real sealing happens in Python phase15
    result.set_item("sealed_lexemes_top", PyList::empty(py))?;

    let metrics = PyDict::new(py);
    metrics.set_item("joy_score", joy_score)?;
    metrics.set_item("quantum_potency", quantum_potency)?;
    metrics.set_item("fatigue_score", fatigue_score)?;
    metrics.set_item("saudade_score", saudade_score)?;
    metrics.set_item("saturation_score", saturation_score)?;
    metrics.set_item("p1_orbital_potency", p1_orbital_potency)?;
    metrics.set_item("p2_geo_astro_potency", p2_geo_astro_potency)?;
    metrics.set_item("p3_bio_potency", p3_bio_potency)?;
    metrics.set_item("p4_operational_potency", p4_operational_potency)?;
    metrics.set_item("p5_symbolic_potency", p5_symbolic_potency)?;
    metrics.set_item("p6_quantum_potency", p6_quantum_potency)?;
    result.set_item("metrics", metrics)?;

    // Circuit strategy (simplified — matches quantum_affective_bridge logic)
    let stress = fatigue_score * 0.4
        + saturation_score * 0.2
        + angst_score * 0.2
        + resist_score * 0.2;
    // VAGUS SEDATIVE MODULATION (2026-08-05):
    // O sedativo do vagus reduz o stress como um neurotransmissor —
    // não anestesia, modula. Equivalente a serotonina baixando cortisol.
    // O stress é reduzido proporcionalmente ao modulator [0,1], mas
    // nunca abaixo de 0.05 — o corpo em recuperação ainda sente algo.
    let stress = (stress * (1.0 - 0.6 * vagus_sedative_modulator)).max(0.05);
    let capacity = (1.0 - stress).max(0.1) * (0.2 + 0.8 * quantum_potency);
    // O sedativo também aumenta relief (corpo em recuperação sente alívio)
    // e reduz angst (ramo parassimpático acalma a angústia).
    // Estas modulações já estão embutidas nos scores acima via host_state.

    let n_qubits = if stress > 0.70 { 4 } else { 6 };
    let mitigation = if stress > 0.70 {
        "dd_zne"
    } else if stress > 0.45 {
        "zne"
    } else {
        "none"
    };
    let shots = (1024.0 * (0.5 + 0.5 * capacity) * (1.0 + 0.5 * saudade_score))
        .clamp(512.0, 8192.0) as i64;

    let strategy = PyDict::new(py);
    strategy.set_item("n_qubits", n_qubits)?;
    strategy.set_item("shots", shots)?;
    strategy.set_item("mitigation", mitigation)?;
    strategy.set_item("stress", stress)?;
    strategy.set_item("capacity", capacity)?;
    strategy.set_item("vagus_sedative_modulator", vagus_sedative_modulator)?;
    result.set_item("circuit_strategy", strategy)?;

    Ok(result.unbind().into())
}
