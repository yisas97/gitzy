// ai/ollama.rs - Integracion con Ollama CLI

use std::io::Write;
use std::process::{Command, Stdio};

use super::{clean_ai_response, get_commit_prompt, get_limited_diff};

/// Genera un mensaje de commit usando Ollama
pub fn generate(model: &str) -> Result<String, String> {
    let diff = get_limited_diff()?;
    let prompt = get_commit_prompt(&diff);

    // ollama run <model> lee de stdin
    #[cfg(windows)]
    let mut child = Command::new("cmd")
        .args(["/c", "ollama", "run", model])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!(
            "Error ejecutando ollama: {}. Asegurate de tener ollama instalado (https://ollama.ai)",
            e
        ))?;

    #[cfg(not(windows))]
    let mut child = Command::new("ollama")
        .args(["run", model])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!(
            "Error ejecutando ollama: {}. Asegurate de tener ollama instalado (https://ollama.ai)",
            e
        ))?;

    // Escribir el prompt a stdin
    if let Some(mut stdin) = child.stdin.take() {
        stdin.write_all(prompt.as_bytes())
            .map_err(|e| format!("Error escribiendo a stdin: {}", e))?;
    }

    // Esperar resultado
    let output = child.wait_with_output()
        .map_err(|e| format!("Error esperando ollama: {}", e))?;

    if output.status.success() {
        let raw = String::from_utf8_lossy(&output.stdout).to_string();

        if raw.trim().is_empty() {
            Err("Ollama no genero respuesta".to_string())
        } else {
            Ok(clean_ai_response(&raw))
        }
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(format!("Ollama fallo: {}", stderr))
    }
}
