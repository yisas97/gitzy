// ai/claude.rs - Integracion con Claude CLI

use std::io::Write;
use std::process::{Command, Stdio};

use super::{clean_ai_response, get_commit_prompt, get_limited_diff};

/// Genera un mensaje de commit usando Claude CLI
pub fn generate() -> Result<String, String> {
    let diff = get_limited_diff()?;
    let prompt = get_commit_prompt(&diff);

    // Usar stdin para pasar el prompt (evita limite de longitud de comando en Windows)
    #[cfg(windows)]
    let mut child = Command::new("cmd")
        .args(["/c", "claude", "-p", "-"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!(
            "Error ejecutando claude: {}. Asegurate de tener claude instalado (npm i -g @anthropic-ai/claude-code)",
            e
        ))?;

    #[cfg(not(windows))]
    let mut child = Command::new("claude")
        .args(["-p", "-"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!(
            "Error ejecutando claude: {}. Asegurate de tener claude instalado (npm i -g @anthropic-ai/claude-code)",
            e
        ))?;

    // Escribir el prompt a stdin
    if let Some(mut stdin) = child.stdin.take() {
        stdin.write_all(prompt.as_bytes())
            .map_err(|e| format!("Error escribiendo a stdin: {}", e))?;
    }

    // Esperar resultado
    let output = child.wait_with_output()
        .map_err(|e| format!("Error esperando claude: {}", e))?;

    if output.status.success() {
        let raw = String::from_utf8_lossy(&output.stdout).to_string();

        if raw.trim().is_empty() {
            Err("Claude no genero respuesta".to_string())
        } else {
            Ok(clean_ai_response(&raw))
        }
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(format!("Claude fallo: {}", stderr))
    }
}
