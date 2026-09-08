use pyo3::prelude::*;
use std::fs;
use std::os::unix::fs::PermissionsExt;

/// Enforce kernel behaviors for Tupi particles
pub fn enforce(path: &str, lexeme: &str) -> PyResult<()> {
    // Check if path exists
    if !std::path::Path::new(path).exists() {
        return Err(PyErr::new::<pyo3::exceptions::PyFileNotFoundError, _>(
            format!("File not found: {}", path)
        ));
    }
    
    // Parse particles from lexeme
    let particles: Vec<&str> = lexeme.split('-')
        .filter(|p| p.chars().all(|c| c.is_lowercase() || c == '\'' || c == 'ẽ'))
        .collect();
    
    for particle in particles {
        match particle {
            "atã" => enforce_ata(path)?,
            "pema" => enforce_pema(path)?,
            "me'ẽ" | "meẽ" => enforce_mee(path)?,
            "katu" => enforce_katu(path)?,
            _ => {}, // Other particles don't have filesystem enforcement
        }
    }
    
    Ok(())
}

/// -atã: Immutable (read-only)
fn enforce_ata(path: &str) -> PyResult<()> {
    let metadata = fs::metadata(path)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyIOError, _>(e.to_string()))?;
    
    let mut perms = metadata.permissions();
    perms.set_mode(0o444); // Read-only for all (owner, group, others)
    
    fs::set_permissions(path, perms)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyIOError, _>(
            format!("Failed to set immutable permissions: {}", e)
        ))?;
    
    // TODO: chattr +i (requires CAP_LINUX_IMMUTABLE)
    // Would need libc::ioctl with FS_IOC_SETFLAGS
    
    Ok(())
}

/// -pema: Hidden (mark with extended attribute)
fn enforce_pema(path: &str) -> PyResult<()> {
    // Mark as hidden using extended attributes
    // This is a soft enforcement (doesn't require root)
    
    #[cfg(target_os = "linux")]
    {
        use std::process::Command;
        
        // Use setfattr to mark as hidden
        let output = Command::new("setfattr")
            .arg("-n")
            .arg("user.langue_visibility")
            .arg("-v")
            .arg("hidden")
            .arg(path)
            .output();
        
        if let Ok(out) = output {
            if !out.status.success() {
                // Fallback: just log warning
                eprintln!("⚠️ Warning: Could not set xattr for -pema on {}", path);
            }
        }
    }
    
    Ok(())
}

/// -me'ẽ: Exportable (mark with extended attribute)
fn enforce_mee(path: &str) -> PyResult<()> {
    #[cfg(target_os = "linux")]
    {
        use std::process::Command;
        
        let output = Command::new("setfattr")
            .arg("-n")
            .arg("user.langue_exportable")
            .arg("-v")
            .arg("true")
            .arg(path)
            .output();
        
        if let Ok(out) = output {
            if !out.status.success() {
                eprintln!("⚠️ Warning: Could not set xattr for -me'ẽ on {}", path);
            }
        }
    }
    
    Ok(())
}

/// -katu: Validated (mark with extended attribute)
fn enforce_katu(path: &str) -> PyResult<()> {
    #[cfg(target_os = "linux")]
    {
        use std::process::Command;
        
        let output = Command::new("setfattr")
            .arg("-n")
            .arg("user.langue_validated")
            .arg("-v")
            .arg("true")
            .arg(path)
            .output();
        
        if let Ok(out) = output {
            if !out.status.success() {
                eprintln!("⚠️ Warning: Could not set xattr for -katu on {}", path);
            }
        }
    }
    
    Ok(())
}
