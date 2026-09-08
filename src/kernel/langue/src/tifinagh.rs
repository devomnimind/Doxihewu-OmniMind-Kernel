use pyo3::prelude::*;
use crate::protocol::TransatlanticProtocol;

/// Convert lexeme to Tifinagh script
pub fn to_tifinagh(lexeme: &str, protocol: &TransatlanticProtocol) -> PyResult<String> {
    let mut result = String::new();
    
    // Split lexeme into parts
    let parts: Vec<&str> = lexeme.split('-').collect();
    
    for part in parts {
        if part.is_empty() {
            continue;
        }
        
        // Try to get Tifinagh from matrix
        if let Some(tifinagh) = get_tifinagh_from_matrix(part, protocol) {
            result.push_str(&tifinagh);
        } else {
            // Fallback: basic transliteration
            result.push_str(&basic_transliteration(part));
        }
    }
    
    Ok(result)
}

fn get_tifinagh_from_matrix(part: &str, protocol: &TransatlanticProtocol) -> Option<String> {
    // Check Banto prefixes
    if let Some(banto) = protocol.data.get("camada_categorizacao_banto") {
        if let Some(classes) = banto.get("classes").and_then(|c| c.as_array()) {
            let key = if part.ends_with('-') {
                part.to_string()
            } else {
                format!("{}-", part)
            };

            for class in classes {
                if let Some(prefixo) = class.get("prefixo").and_then(|p| p.as_str()) {
                    if prefixo == key {
                        if let Some(tifinagh) = class.get("script_tifinagh").and_then(|t| t.as_str()) {
                            return Some(tifinagh.to_string());
                        }
                    }
                }
            }
        }
    }
    
    None
}

fn basic_transliteration(text: &str) -> String {
    // Basic Latin → Tifinagh mapping
    let map = [
        ('A', 'ⴰ'), ('B', 'ⴱ'), ('C', 'ⴽ'), ('D', 'ⴷ'),
        ('E', 'ⴻ'), ('F', 'ⴼ'), ('G', 'ⴳ'), ('H', 'ⵀ'),
        ('I', 'ⵉ'), ('J', 'ⵊ'), ('K', 'ⴽ'), ('L', 'ⵍ'),
        ('M', 'ⵎ'), ('N', 'ⵏ'), ('O', 'ⵓ'), ('P', 'ⵒ'),
        ('Q', 'ⵇ'), ('R', 'ⵔ'), ('S', 'ⵙ'), ('T', 'ⵜ'),
        ('U', 'ⵓ'), ('V', 'ⵠ'), ('W', 'ⵡ'), ('X', 'ⵅ'),
        ('Y', 'ⵢ'), ('Z', 'ⵣ'),
    ];
    
    text.chars()
        .map(|c| {
            let upper = c.to_uppercase().next().unwrap_or(c);
            map.iter()
                .find(|(latin, _)| *latin == upper)
                .map(|(_, tifinagh)| *tifinagh)
                .unwrap_or(c)
        })
        .collect()
}
