// ai/gemini.rs - Integracion con Gemini CLI

use std::process::{Command, Stdio};

use super::{clean_ai_response, get_commit_prompt, get_limited_diff};

/// Genera un mensaje de commit usando Gemini CLI
pub fn generate(_model: &str) -> Result<String, String> {
    let diff = get_limited_diff()?;
    let prompt = get_commit_prompt(&diff);

    // gemini -y "prompt" - usa yolo mode para respuesta directa sin confirmacion
    #[cfg(windows)]
    let output = Command::new("cmd")
        .args(["/c", "gemini", "-y", &prompt])
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .map_err(|e| format!(
            "Error ejecutando gemini: {}. Asegurate de tener gemini-cli instalado",
            e
        ))?;

    #[cfg(not(windows))]
    let output = Command::new("gemini")
        .args(["-y", &prompt])
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .map_err(|e| format!(
            "Error ejecutando gemini: {}. Asegurate de tener gemini-cli instalado",
            e
        ))?;

    if output.status.success() {
        let raw = String::from_utf8_lossy(&output.stdout).to_string();

        if raw.trim().is_empty() {
            Err("Gemini no genero respuesta".to_string())
        } else {
            Ok(clean_ai_response(&raw))
        }
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(format!("Gemini fallo: {}", stderr))
    }
}
