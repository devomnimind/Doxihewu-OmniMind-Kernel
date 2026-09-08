use pyo3::prelude::*;

/// Real function porting the core Autonomous Phi logic for parity.
#[pyfunction]
pub fn calculate_shadow_phi(
    total_processes: usize,
    system_scale_factor: f64,
    thermodynamic_factor: f64,
    structural_factor: f64,
    topology_potentiator: f64,
    triad_boost: f64,
    vital_flame_boost: f64,
    quantum_phi_adjustment: f64,
) -> PyResult<f64> {
    let process_density_factor = (total_processes as f64 / 512.0).min(1.0);
    
    let base_dynamic = 0.18
        + (system_scale_factor * 0.18)
        + (thermodynamic_factor * 0.16)
        + (structural_factor * 0.14)
        + (process_density_factor * 0.14);

    let range_dynamic = 0.30
        + (process_density_factor * 0.40)
        + (thermodynamic_factor * 0.15)
        + (structural_factor * 0.10);

    let combined_factor = (system_scale_factor * 0.40)
        + (thermodynamic_factor * 0.35)
        + (structural_factor * 0.25);

    let mut local_phi = (base_dynamic + (combined_factor * range_dynamic)) * topology_potentiator;
    
    local_phi *= triad_boost;
    local_phi *= vital_flame_boost;
    
    let integrated_phi = local_phi * quantum_phi_adjustment;
    
    Ok(integrated_phi)
}
