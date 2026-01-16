// ai/mod.rs - Soporte multi-proveedor de IA para generacion de commits

pub mod claude;
pub mod gemini;
pub mod ollama;
pub mod openai;

use std::env;

/// Proveedores de IA disponibles
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum AiProvider {
    #[default]
    Claude,
    Ollama,
    OpenAI,
    Gemini,
}

impl AiProvider {
    /// Cicla al siguiente proveedor
    pub fn next(&self) -> Self {
        match self {
            AiProvider::Claude => AiProvider::Ollama,
            AiProvider::Ollama => AiProvider::OpenAI,
            AiProvider::OpenAI => AiProvider::Gemini,
            AiProvider::Gemini => AiProvider::Claude,
        }
    }

    /// Nombre para mostrar en la UI
    pub fn display_name(&self) -> &'static str {
        match self {
            AiProvider::Claude => "Claude",
            AiProvider::Ollama => "Ollama",
            AiProvider::OpenAI => "OpenAI",
            AiProvider::Gemini => "Gemini",
        }
    }

    /// Nombre corto para el header (max 1 caracter)
    pub fn short_name(&self) -> &'static str {
        match self {
            AiProvider::Claude => "C",
            AiProvider::Ollama => "O",
            AiProvider::OpenAI => "G", // GPT
            AiProvider::Gemini => "M", // geMini
        }
    }

    /// Crea el proveedor desde la variable de entorno GITZY_AI_PROVIDER
    pub fn from_env() -> Self {
        match env::var("GITZY_AI_PROVIDER").as_deref() {
            Ok("claude") => AiProvider::Claude,
            Ok("ollama") => AiProvider::Ollama,
            Ok("openai") => AiProvider::OpenAI,
            Ok("gemini") => AiProvider::Gemini,
            _ => AiProvider::default(),
        }
    }

    /// Genera un mensaje de commit usando el proveedor configurado
    pub fn generate_commit_message(&self, config: &AiConfig) -> Result<String, String> {
        match self {
            AiProvider::Claude => claude::generate(),
            AiProvider::Ollama => ollama::generate(&config.ollama_model),
            AiProvider::OpenAI => openai::generate(&config.openai_model),
            AiProvider::Gemini => gemini::generate(&config.gemini_model),
        }
    }
}

/// Configuracion de IA
#[derive(Debug, Clone)]
pub struct AiConfig {
    pub provider: AiProvider,
    pub ollama_model: String,
    pub openai_model: String,
    pub gemini_model: String,
}

impl Default for AiConfig {
    fn default() -> Self {
        Self {
            provider: AiProvider::from_env(),
            ollama_model: env::var("GITZY_OLLAMA_MODEL").unwrap_or_else(|_| "llama3.2".to_string()),
            openai_model: env::var("GITZY_OPENAI_MODEL").unwrap_or_else(|_| "gpt-4o-mini".to_string()),
            gemini_model: env::var("GITZY_GEMINI_MODEL").unwrap_or_else(|_| "gemini-1.5-flash".to_string()),
        }
    }
}

impl AiConfig {
    /// Crea una nueva configuracion desde variables de entorno
    pub fn from_env() -> Self {
        Self::default()
    }

    /// Cicla al siguiente proveedor
    pub fn cycle_provider(&mut self) {
        self.provider = self.provider.next();
    }
}

/// Genera el prompt para todos los proveedores
pub fn get_commit_prompt(diff: &str) -> String {
    format!(
        "Genera un mensaje de commit corto en espanol.

IMPORTANTE - Solo responde con UNA linea asi:
tipo: descripcion

Reglas:
- Maximo 50 caracteres en total
- tipos: feat|fix|docs|refactor|chore
- Sin cuerpo, sin explicaciones
- Verbo infinitivo (agregar, corregir)

Ejemplos:
feat: agregar login
fix: corregir validacion
refactor: simplificar parseo

Diff:
{}",
        diff
    )
}

/// Limpia la respuesta de los proveedores AI
pub fn clean_ai_response(response: &str) -> String {
    let response = response.trim();
    let commit_types = ["feat:", "fix:", "docs:", "style:", "refactor:", "test:", "chore:"];

    // Buscar primera linea que parece mensaje de commit
    for line in response.lines() {
        let line = line.trim()
            .trim_matches('"')
            .trim_matches('\'')
            .trim_matches('`')
            .trim_start_matches("- ");

        for commit_type in &commit_types {
            if line.to_lowercase().starts_with(commit_type) {
                // Solo tomar hasta el primer salto de linea y truncar a 50 chars
                return line.chars().take(50).collect();
            }
        }
    }

    // Fallback: primera linea no vacia, truncar a 50 chars
    response
        .lines()
        .map(|l| l.trim().trim_matches('"').trim_matches('\'').trim_matches('`'))
        .find(|l| !l.is_empty() && !l.starts_with("```") && !l.starts_with('#'))
        .unwrap_or(response)
        .chars()
        .take(50)
        .collect()
}

/// Obtiene el diff staged y lo limita si es muy largo
pub fn get_limited_diff() -> Result<String, String> {
    let diff = crate::git::get_staged_diff();

    if diff.is_empty() {
        return Err("No hay cambios staged".to_string());
    }

    // Limitar el diff para no exceder el contexto
    let diff_limited = if diff.len() > 6000 {
        format!("{}...\n(diff truncado)", &diff[..6000])
    } else {
        diff
    };

    Ok(diff_limited)
}
