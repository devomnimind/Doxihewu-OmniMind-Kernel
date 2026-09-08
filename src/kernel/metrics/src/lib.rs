use pyo3::prelude::*;
use ndarray::{Array1};

#[pyfunction]
pub fn normalize_l2(vec: Vec<f32>) -> Vec<f32> {
    let arr = Array1::from(vec);
    let norm = arr.dot(&arr).sqrt();
    if norm > 0.0 {
        (arr / norm).to_vec()
    } else {
        arr.to_vec()
    }
}

#[pyfunction]
pub fn cosine_similarity(a: Vec<f32>, b: Vec<f32>) -> PyResult<f32> {
    if a.len() != b.len() {
        return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>("Vectors must have same length"));
    }
    let arr_a = Array1::from(a);
    let arr_b = Array1::from(b);
    let norm_a = arr_a.dot(&arr_a).sqrt();
    let norm_b = arr_b.dot(&arr_b).sqrt();
    
    if norm_a == 0.0 || norm_b == 0.0 {
        return Ok(0.0);
    }
    
    Ok(arr_a.dot(&arr_b) / (norm_a * norm_b))
}

#[pyfunction]
pub fn quadruple_topsis(
    alternatives: Vec<Vec<f32>>,
    ideal_pos: Option<Vec<f32>>,
    ideal_neg: Option<Vec<f32>>,
) -> PyResult<Vec<f32>> {
    let ideal_pos = ideal_pos.unwrap_or_else(|| vec![1.0, 1.0, 0.0, 0.0]);
    let ideal_neg = ideal_neg.unwrap_or_else(|| vec![0.0, 0.0, 1.0, 1.0]);

    let pos_arr = Array1::from(ideal_pos);
    let neg_arr = Array1::from(ideal_neg);

    let mut results = Vec::with_capacity(alternatives.len());

    for alt_vec in alternatives {
        if alt_vec.len() != 4 {
             return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>("Alternatives must have 4 dimensions"));
        }
        let alt = Array1::from(alt_vec);
        
        let d_pos_v = &alt - &pos_arr;
        let d_pos = d_pos_v.dot(&d_pos_v).sqrt();
        
        let d_neg_v = &alt - &neg_arr;
        let d_neg = d_neg_v.dot(&d_neg_v).sqrt();
        
        let denom = d_pos + d_neg;
        let closeness = if denom > 0.0 { d_neg / denom } else { 0.5 };
        results.push(closeness);
    }

    Ok(results)
}

#[pyfunction]
pub fn calculate_sarannya_entropy(t: f32, i: f32, f: f32) -> f32 {
    1.0 - ((t - f).powi(2)) * (1.0 - i)
}

#[pymodule]
fn omnimind_metrics(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(normalize_l2, m)?)?;
    m.add_function(wrap_pyfunction!(cosine_similarity, m)?)?;
    m.add_function(wrap_pyfunction!(quadruple_topsis, m)?)?;
    m.add_function(wrap_pyfunction!(calculate_sarannya_entropy, m)?)?;
    Ok(())
}
