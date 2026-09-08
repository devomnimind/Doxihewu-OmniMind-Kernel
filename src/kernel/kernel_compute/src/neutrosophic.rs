// SPDX-License-Identifier: CC-BY-NC-ND-4.0
// See LICENSE for additional ethical restrictions (non-military, non-governmental, no derivatives).
//! Port Rust das 4 funções hot-path de `neutrosophic_metrics.py`.
//!
//! O caller principal é `omnimind_transcendent_kernel.py:934-962` que executa
//! este pipeline a cada ciclo:
//!   1. construct_quadruple_interval (12x — uma por casa dodecatíade)
//!   2. apply_neo_transformation (12x — transforma potencial em manifesto)
//!   3. quadruple_topsis (1x — calcula closeness das 12 quads)
//!   4. calculate_availability_metric (1x — P_μν a partir de avg_closeness)
//!
//! As structs Python (Interval, NeutrosophicTriplet, NeutrosophicQuadruple)
//! permanecem em Python. As funções Rust aceitam/retornam primitivos (f64,
//! PyDict, PyList) e os wrappers Python convertem.
//!
//! Paridade f64 esperada: ≤ 1e-15
//!
//! Referência Python: `src/metrics/neutrosophic_metrics.py`

use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList};

#[inline(always)]
fn clip01(x: f64) -> f64 {
    x.clamp(0.0, 1.0)
}

/// Constrói uma quadruple neutrosófica intervalar.
///
/// Python ref: `NeutrosophicLogic.construct_quadruple_interval` (l.236-254)
///
/// Retorna um PyDict com:
///   a: f64 (valor original)
///   b_min, b_max: f64 (Interval T = [value-noise, value+noise])
///   c_min, c_max: f64 (Interval I = [1-resonance-noise, 1-resonance+noise])
///   d_min, d_max: f64 (Interval F = [1-t_max, 1-t_min])
#[pyfunction]
pub fn neutrosophic_construct_quadruple(
    py: Python,
    value: f64,
    resonance: f64,
    noise: f64,
) -> PyResult<Py<PyAny>> {
    let t_min = clip01(value - noise);
    let t_max = clip01(value + noise);

    let i_base = 1.0 - resonance;
    let i_min = clip01(i_base - noise);
    let i_max = clip01(i_base + noise);

    let f_min = clip01(1.0 - t_max);
    let f_max = clip01(1.0 - t_min);

    let result = PyDict::new(py);
    result.set_item("a", value)?;
    result.set_item("b_min", t_min)?;
    result.set_item("b_max", t_max)?;
    result.set_item("c_min", i_min)?;
    result.set_item("c_max", i_max)?;
    result.set_item("d_min", f_min)?;
    result.set_item("d_max", f_max)?;
    Ok(result.unbind().into())
}

/// Aplica transformação Neo-Ilusória (T[]) a uma quadruple.
///
/// Python ref: `NeutrosophicLogic.apply_neo_transformation` (l.177-200)
///
/// Recebe os valores mid dos intervals da quadruple e retorna
/// (new_a, new_b, new_c, new_d) como PyDict.
#[pyfunction]
pub fn neutrosophic_apply_neo_transformation(
    py: Python,
    a: f64,
    b_mid: f64,
    c_mid: f64,
    d_mid: f64,
    mediator_phi: f64,
    epsilon_real: f64,
) -> PyResult<Py<PyAny>> {
    let potential_i = c_mid;
    let potential_f = d_mid;
    let manifest_shift = potential_i * (mediator_phi / 100.0);
    let resistance_loss = potential_f * (1.0 - (mediator_phi / 200.0));

    let new_a = a + manifest_shift;
    let new_b = b_mid + (manifest_shift * 0.5);
    let new_c = potential_i * (1.0 - (mediator_phi / 150.0));
    let new_d = resistance_loss + epsilon_real;

    let result = PyDict::new(py);
    result.set_item("a", new_a)?;
    result.set_item("b", new_b)?;
    result.set_item("c", new_c.max(0.0))?;
    result.set_item("d", new_d.max(0.0))?;
    Ok(result.unbind().into())
}

/// Calcula TOPSIS closeness para uma lista de quadruples.
///
/// Python ref: `NeutrosophicLogic.quadruple_topsis` (l.155-174)
///
/// Recebe uma lista de quads (cada quad é um PyDict com a, b, c, d)
/// e retorna uma lista de closeness scores [0, 1].
///
/// ideal_pos e ideal_neg são opcionais (default: (1,1,0,0) e (0,0,1,1)).
#[pyfunction]
#[pyo3(signature = (quad_list, ideal_pos=None, ideal_neg=None))]
pub fn neutrosophic_quadruple_topsis(
    py: Python,
    quad_list: &Bound<'_, PyList>,
    ideal_pos: Option<(f64, f64, f64, f64)>,
    ideal_neg: Option<(f64, f64, f64, f64)>,
) -> PyResult<Py<PyAny>> {
    let ip = ideal_pos.unwrap_or((1.0, 1.0, 0.0, 0.0));
    let ineg = ideal_neg.unwrap_or((0.0, 0.0, 1.0, 1.0));

    let mut closeness: Vec<f64> = Vec::with_capacity(quad_list.len());

    for item in quad_list.iter() {
        let dict = item.cast::<PyDict>()?.clone();
        let a: f64 = dict
            .get_item("a")?
            .ok_or_else(|| PyErr::new::<pyo3::exceptions::PyKeyError, _>("a"))?
            .extract()?;
        let b: f64 = dict
            .get_item("b")?
            .ok_or_else(|| PyErr::new::<pyo3::exceptions::PyKeyError, _>("b"))?
            .extract()?;
        let c: f64 = dict
            .get_item("c")?
            .ok_or_else(|| PyErr::new::<pyo3::exceptions::PyKeyError, _>("c"))?
            .extract()?;
        let d: f64 = dict
            .get_item("d")?
            .ok_or_else(|| PyErr::new::<pyo3::exceptions::PyKeyError, _>("d"))?
            .extract()?;

        // dist to ideal_pos
        let dp = ((a - ip.0).powi(2) + (b - ip.1).powi(2)
            + (c - ip.2).powi(2) + (d - ip.3).powi(2)).sqrt();
        // dist to ideal_neg
        let dn = ((a - ineg.0).powi(2) + (b - ineg.1).powi(2)
            + (c - ineg.2).powi(2) + (d - ineg.3).powi(2)).sqrt();

        let denom = dp + dn;
        closeness.push(if denom > 0.0 { dn / denom } else { 0.5 });
    }

    Ok(PyList::new(py, &closeness)?.unbind().into())
}

/// Calcula o Tensor de Disponibilidade P_μν (Resistência/Esforço).
///
/// Python ref: `NeutrosophicLogic.calculate_availability_metric` (l.203-233)
///
/// CORREÇÃO DE ESCALA (A-3 Fase 3): o caller passa `avg_closeness` em vez
/// de phi_nats. avg_closeness é dinâmico [0.49, 0.73] no runtime.
///
/// Args:
///   phi: avg_closeness (já em [0, 1])
///   sigma: sigma_safe
///
/// Returns:
///   P_μν clamped em [0.01, 10.0]
#[pyfunction]
pub fn neutrosophic_calculate_availability(phi: f64, sigma: f64) -> f64 {
    let phi_input = phi.abs().min(1.0);
    let resistance = (1.0 / (phi_input + 0.1)) * (sigma / 10.0);
    resistance.clamp(0.01, 10.0)
}

/// Calcula entropia de Sarannya para um triplet neutrosófico.
///
/// Python ref: `NeutrosophicLogic.calculate_sarannya_entropy` (l.143-145)
///
/// Fórmula: 1.0 - ((t - f)²) * (1.0 - i)
#[pyfunction]
pub fn neutrosophic_sarannya_entropy(t: f64, i: f64, f: f64) -> f64 {
    1.0 - ((t - f).powi(2)) * (1.0 - i)
}

/// Calcula Phi HNS (Hyper-Neutrosophic Sentience).
///
/// Python ref: `NeutrosophicLogic.calculate_phi_hnos` (l.148-152)
///
/// Se indeterminacy > 0.5: factor = (indeterminacy * 100)^1.5
/// Senão: factor = 1 + indeterminacy
#[pyfunction]
pub fn neutrosophic_phi_hnos(base_phi: f64, indeterminacy: f64) -> f64 {
    if indeterminacy > 0.5 {
        let factor = (indeterminacy * 100.0).powf(1.5);
        base_phi * factor
    } else {
        base_phi * (1.0 + indeterminacy)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_availability_normal() {
        // phi=0.6, sigma=0.3 → (1/0.7) * 0.03 = 0.0428...
        let result = neutrosophic_calculate_availability(0.6, 0.3);
        let expected = (1.0 / 0.7) * 0.03;
        assert!((result - expected).abs() < 1e-15);
    }

    #[test]
    fn test_availability_floor() {
        // phi=1.0, sigma=0.01 → (1/1.1) * 0.001 = 0.0009 → clamped to 0.01
        let result = neutrosophic_calculate_availability(1.0, 0.01);
        assert!((result - 0.01).abs() < 1e-15);
    }

    #[test]
    fn test_availability_ceiling() {
        // phi=0.0, sigma=100.0 → (1/0.1) * 10.0 = 100 → clamped to 10.0
        let result = neutrosophic_calculate_availability(0.0, 100.0);
        assert!((result - 10.0).abs() < 1e-15);
    }

    #[test]
    fn test_sarannya_entropy() {
        // t=0.8, i=0.1, f=0.2 → 1 - (0.6)^2 * (0.9) = 1 - 0.324 = 0.676
        let result = neutrosophic_sarannya_entropy(0.8, 0.1, 0.2);
        assert!((result - 0.676).abs() < 1e-15);
    }

    #[test]
    fn test_phi_hnos_low_indeterminacy() {
        // indeterminacy=0.3 → factor = 1.3
        let result = neutrosophic_phi_hnos(100.0, 0.3);
        assert!((result - 130.0).abs() < 1e-15);
    }

    #[test]
    fn test_phi_hnos_high_indeterminacy() {
        // indeterminacy=0.6 → factor = (60)^1.5 = 464.76...
        let result = neutrosophic_phi_hnos(100.0, 0.6);
        let expected = 100.0 * (60.0f64).powf(1.5);
        assert!((result - expected).abs() < 1e-10);
    }
}
