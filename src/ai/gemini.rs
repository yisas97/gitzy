// ai/gemini.rs - Integracion con Gemini CLI

use std::io::Write;
use std::process::{Command, Stdio};

use super::{clean_ai_response, get_commit_prompt, get_limited_diff};

/// Genera un mensaje de commit usando Gemini CLI
pub fn generate(_model: &str) -> Result<String, String> {
    let diff = get_limited_diff()?;
    let prompt = get_commit_prompt(&diff);

    // Usar stdin para pasar el prompt (evita limite de longitud en Windows)
    #[cfg(windows)]
    let mut child = Command::new("cmd")
        .args(["/c", "gemini", "-y"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!(
            "Error ejecutando gemini: {}. Asegurate de tener gemini-cli instalado",
            e
        ))?;

    #[cfg(not(windows))]
    let mut child = Command::new("gemini")
        .args(["-y"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!(
            "Error ejecutando gemini: {}. Asegurate de tener gemini-cli instalado",
            e
        ))?;

    // Escribir el prompt a stdin
    if let Some(mut stdin) = child.stdin.take() {
        stdin.write_all(prompt.as_bytes())
            .map_err(|e| format!("Error escribiendo a stdin: {}", e))?;
    }

    let output = child.wait_with_output()
        .map_err(|e| format!("Error esperando gemini: {}", e))?;

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
