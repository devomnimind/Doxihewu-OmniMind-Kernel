import math
import numpy as np

def python_phi_reference(
    total_processes: int,
    system_scale_factor: float,
    thermodynamic_factor: float,
    structural_factor: float,
    topology_potentiator: float,
    triad_boost: float = 1.0,
    vital_flame_boost: float = 1.0,
    quantum_phi_adjustment: float = 1.0
) -> float:
    process_density_factor = min(1.0, total_processes / 512.0)
    base_dynamic = (
        0.18
        + (system_scale_factor * 0.18)
        + (thermodynamic_factor * 0.16)
        + (structural_factor * 0.14)
        + (process_density_factor * 0.14)
    )
    range_dynamic = (
        0.30
        + (process_density_factor * 0.40)
        + (thermodynamic_factor * 0.15)
        + (structural_factor * 0.10)
    )

    combined_factor = (
        system_scale_factor * 0.40
        + thermodynamic_factor * 0.35
        + structural_factor * 0.25
    )

    local_phi = (base_dynamic + (combined_factor * range_dynamic)) * topology_potentiator
    local_phi *= triad_boost
    local_phi *= vital_flame_boost
    
    integrated_phi = local_phi * quantum_phi_adjustment
    return integrated_phi

if __name__ == "__main__":
    try:
        from omnimind_kernel_compute import phi_engine
    except ImportError:
        print("omnimind_kernel_compute not compiled yet.")
        exit(1)
        
    # Setup test vectors
    p_total = 250
    s_scale = 0.85
    t_factor = 0.90
    str_factor = 0.88
    top_pot = 2.45
    
    py_phi = python_phi_reference(p_total, s_scale, t_factor, str_factor, top_pot)
    rs_phi = phi_engine.calculate_shadow_phi(p_total, s_scale, t_factor, str_factor, top_pot, 1.0, 1.0, 1.0)
    
    diff = abs(py_phi - rs_phi)
    print(f"Python Phi: {py_phi:.12f}")
    print(f"Rust Phi:   {rs_phi:.12f}")
    print(f"Difference: {diff:.12e}")
    
    if diff <= 1e-12:
        print("✅ Parity Test PASSED")
    else:
        print("❌ Parity Test FAILED")
        exit(1)
