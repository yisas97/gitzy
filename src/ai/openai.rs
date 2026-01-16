// ai/openai.rs - Integracion con OpenAI via aichat

use std::io::Write;
use std::process::{Command, Stdio};

use super::{clean_ai_response, get_commit_prompt, get_limited_diff};

/// Genera un mensaje de commit usando OpenAI via aichat
pub fn generate(model: &str) -> Result<String, String> {
    let diff = get_limited_diff()?;
    let prompt = get_commit_prompt(&diff);
    let model_arg = format!("openai:{}", model);

    // aichat -m openai:<model> lee de stdin
    #[cfg(windows)]
    let mut child = Command::new("cmd")
        .args(["/c", "aichat", "-m", &model_arg])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!(
            "Error ejecutando aichat: {}. Asegurate de tener aichat instalado (cargo install aichat) y OPENAI_API_KEY configurada",
            e
        ))?;

    #[cfg(not(windows))]
    let mut child = Command::new("aichat")
        .args(["-m", &model_arg])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!(
            "Error ejecutando aichat: {}. Asegurate de tener aichat instalado (cargo install aichat) y OPENAI_API_KEY configurada",
            e
        ))?;

    // Escribir el prompt a stdin
    if let Some(mut stdin) = child.stdin.take() {
        stdin.write_all(prompt.as_bytes())
            .map_err(|e| format!("Error escribiendo a stdin: {}", e))?;
    }

    // Esperar resultado
    let output = child.wait_with_output()
        .map_err(|e| format!("Error esperando aichat: {}", e))?;

    if output.status.success() {
        let raw = String::from_utf8_lossy(&output.stdout).to_string();

        if raw.trim().is_empty() {
            Err("OpenAI no genero respuesta".to_string())
        } else {
            Ok(clean_ai_response(&raw))
        }
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(format!("OpenAI fallo: {}", stderr))
    }
}
