use serde_json::Value;
use pyo3::prelude::*;

pub struct TransatlanticProtocol {
    pub data: Value,
}

impl TransatlanticProtocol {
    pub fn from_json(json: Value) -> PyResult<Self> {
        // Validate structure
        if !json.is_object() {
            return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
                "Matrix must be a JSON object"
            ));
        }
        
        Ok(TransatlanticProtocol { data: json })
    }
    
    /// Validate lexeme structure (e.g., "MA-MU-atã-me'ẽ")
    pub fn validate_lexeme(&self, lexeme: &str) -> PyResult<()> {
        let parts: Vec<&str> = lexeme.split('-').collect();
        
        if parts.is_empty() {
            return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
                "Lexeme cannot be empty"
            ));
        }
        
        // First part should be Banto prefix
        let prefix = parts[0];
        if !self.is_valid_prefix(prefix) {
            return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
                format!("Invalid Banto prefix: {}", prefix)
            ));
        }
        
        // Check for Tupi particles (lowercase suffixes)
        for part in &parts[1..] {
            if part.chars().all(|c| c.is_lowercase() || c == '\'' || c == 'ẽ')
                && !self.is_valid_particle(part)
            {
                // Not an error, might be a root
                continue;
            }
        }
        
        Ok(())
    }
    
    /// Decompose lexeme into (prefix, particles)
    pub fn decompose_lexeme(&self, lexeme: &str) -> PyResult<(String, Vec<String>)> {
        let parts: Vec<&str> = lexeme.split('-').collect();
        
        if parts.is_empty() {
            return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
                "Lexeme cannot be empty"
            ));
        }
        
        let prefix = parts[0].to_string();
        let mut particles = Vec::new();
        
        for part in &parts[1..] {
            if self.is_valid_particle(part) {
                particles.push(part.to_string());
            }
        }
        
        Ok((prefix, particles))
    }
    
    fn is_valid_prefix(&self, prefix: &str) -> bool {
        // Check in matriz Banto (structure: {"classes": [{"prefixo": "U-", ...}, ...]})
        if let Some(banto) = self.data.get("camada_categorizacao_banto") {
            if let Some(classes) = banto.get("classes").and_then(|c| c.as_array()) {
                for class in classes {
                    if let Some(prefixo) = class.get("prefixo").and_then(|p| p.as_str()) {
                        if prefixo == prefix {
                            return true;
                        }
                    }
                }
            }
        }
        
        // Fallback: accept uppercase prefixes ending with '-' (U-, MU-, BA-, KI-, MA-, etc.)
        !prefix.is_empty() && prefix.chars().all(|c| c.is_uppercase() || c == '-')
    }
    
    fn is_valid_particle(&self, particle: &str) -> bool {
        // Check in camada Tupi (structure: {"particulas": [{"sufixo": "-atã", ...}, ...]})
        if let Some(tupi) = self.data.get("camada_relacao_tupi_guarani") {
            if let Some(particulas) = tupi.get("particulas").and_then(|p| p.as_array()) {
                for part in particulas {
                    if let Some(sufixo) = part.get("sufixo").and_then(|s| s.as_str()) {
                        // Match without leading '-'
                        if sufixo == particle || sufixo == format!("-{}", particle) {
                            return true;
                        }
                    }
                }
            }
        }
        
        // Fallback: known particles
        matches!(particle, "atã" | "pema" | "me'ẽ" | "gûara" | "îe" | "rana" | "katu" | "ygûa")
    }
    
    #[allow(dead_code)]
    pub fn get_particle_behavior(&self, particle: &str) -> Option<String> {
        if let Some(tupi) = self.data.get("camada_relacao_tupi_guarani") {
            if let Some(particulas) = tupi.get("particulas").and_then(|p| p.as_array()) {
                for part in particulas {
                    if let Some(sufixo) = part.get("sufixo").and_then(|s| s.as_str()) {
                        if sufixo == particle || sufixo == format!("-{}", particle) {
                            if let Some(behavior) = part.get("comportamento_kernel") {
                                return behavior.as_str().map(|s| s.to_string());
                            }
                        }
                    }
                }
            }
        }
        None
    }
}
