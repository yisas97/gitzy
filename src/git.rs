// git.rs - Funciones para interactuar con git via comandos

use std::process::Command;

/// Estado de un archivo en git
#[derive(Debug, Clone, PartialEq)]
pub enum FileStatus {
    Modified,    // Modificado
    Added,       // Nuevo archivo
    Deleted,     // Eliminado
    Renamed,     // Renombrado
    Untracked,   // No rastreado
}

/// Representa un archivo con cambios
#[derive(Debug, Clone)]
pub struct ChangedFile {
    pub path: String,
    pub status: FileStatus,
    pub staged: bool,
}

/// Obtiene la lista de archivos con cambios (staged y unstaged)
pub fn get_changed_files() -> Vec<ChangedFile> {
    let mut files = Vec::new();

    // Obtener archivos staged (en el index)
    if let Ok(output) = Command::new("git")
        .args(["diff", "--cached", "--name-status"])
        .output()
    {
        let stdout = String::from_utf8_lossy(&output.stdout);
        for line in stdout.lines() {
            if let Some(file) = parse_status_line(line, true) {
                files.push(file);
            }
        }
    }

    // Obtener archivos modificados (no staged)
    if let Ok(output) = Command::new("git")
        .args(["diff", "--name-status"])
        .output()
    {
        let stdout = String::from_utf8_lossy(&output.stdout);
        for line in stdout.lines() {
            if let Some(file) = parse_status_line(line, false) {
                // Evitar duplicados si ya esta en staged
                if !files.iter().any(|f| f.path == file.path) {
                    files.push(file);
                }
            }
        }
    }

    // Obtener archivos untracked
    if let Ok(output) = Command::new("git")
        .args(["ls-files", "--others", "--exclude-standard"])
        .output()
    {
        let stdout = String::from_utf8_lossy(&output.stdout);
        for line in stdout.lines() {
            if !line.is_empty() {
                files.push(ChangedFile {
                    path: line.to_string(),
                    status: FileStatus::Untracked,
                    staged: false,
                });
            }
        }
    }

    files
}

/// Parsea una linea del output de git diff --name-status
fn parse_status_line(line: &str, staged: bool) -> Option<ChangedFile> {
    let parts: Vec<&str> = line.split('\t').collect();
    if parts.len() < 2 {
        return None;
    }

    let status_char = parts[0].chars().next()?;
    let path = parts.last()?.to_string();

    let status = match status_char {
        'M' => FileStatus::Modified,
        'A' => FileStatus::Added,
        'D' => FileStatus::Deleted,
        'R' => FileStatus::Renamed,
        '?' => FileStatus::Untracked,
        _ => FileStatus::Modified,
    };

    Some(ChangedFile { path, status, staged })
}

/// Obtiene el diff de un archivo especifico
pub fn get_file_diff(path: &str, staged: bool) -> String {
    let args = if staged {
        vec!["diff", "--cached", "--", path]
    } else {
        vec!["diff", "--", path]
    };

    match Command::new("git").args(&args).output() {
        Ok(output) => {
            let diff = String::from_utf8_lossy(&output.stdout).to_string();
            if diff.is_empty() {
                // Si no hay diff, el archivo es untracked - mostrar contenido
                if let Ok(content) = std::fs::read_to_string(path) {
                    format!("(Archivo nuevo - untracked)\n\n{}", content)
                } else {
                    "(No se puede leer el archivo)".to_string()
                }
            } else {
                diff
            }
        }
        Err(e) => format!("Error obteniendo diff: {}", e),
    }
}

/// Hace git add de un archivo
pub fn stage_file(path: &str) -> Result<(), String> {
    let output = Command::new("git")
        .args(["add", "--", path])
        .output()
        .map_err(|e| e.to_string())?;

    if output.status.success() {
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(format!("git add fallo: {}", stderr))
    }
}

/// Hace git reset de un archivo (unstage)
pub fn unstage_file(path: &str) -> Result<(), String> {
    let output = Command::new("git")
        .args(["reset", "HEAD", "--", path])
        .output()
        .map_err(|e| e.to_string())?;

    if output.status.success() {
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(format!("git reset fallo: {}", stderr))
    }
}

/// Descarta cambios de un archivo
pub fn discard_changes(path: &str) -> Result<(), String> {
    let output = Command::new("git")
        .args(["checkout", "--", path])
        .output()
        .map_err(|e| e.to_string())?;

    if output.status.success() {
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(format!("git checkout fallo: {}", stderr))
    }
}

/// Obtiene el nombre de la rama actual
pub fn get_current_branch() -> String {
    Command::new("git")
        .args(["branch", "--show-current"])
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_else(|_| "???".to_string())
}

/// Verifica si estamos en un repositorio git
pub fn is_git_repo() -> bool {
    Command::new("git")
        .args(["rev-parse", "--git-dir"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Hace git commit con el mensaje dado
pub fn commit(message: &str) -> Result<(), String> {
    let output = Command::new("git")
        .args(["commit", "-m", message])
        .output()
        .map_err(|e| e.to_string())?;

    if output.status.success() {
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(format!("git commit fallo: {}", stderr))
    }
}

/// Obtiene el diff completo de todos los archivos staged
pub fn get_staged_diff() -> String {
    Command::new("git")
        .args(["diff", "--cached"])
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).to_string())
        .unwrap_or_default()
}

/// Estructura para un commit del log
#[derive(Debug, Clone)]
pub struct LogEntry {
    pub hash: String,      // Hash corto
    pub message: String,   // Mensaje del commit
    pub author: String,    // Autor
    pub date: String,      // Fecha relativa
}

/// Obtiene el log de commits
pub fn get_log(limit: usize) -> Vec<LogEntry> {
    let output = Command::new("git")
        .args([
            "log",
            &format!("-{}", limit),
            "--pretty=format:%h\t%s\t%an\t%ar",
        ])
        .output();

    match output {
        Ok(o) => {
            String::from_utf8_lossy(&o.stdout)
                .lines()
                .filter_map(|line| {
                    let parts: Vec<&str> = line.split('\t').collect();
                    if parts.len() >= 4 {
                        Some(LogEntry {
                            hash: parts[0].to_string(),
                            message: parts[1].to_string(),
                            author: parts[2].to_string(),
                            date: parts[3].to_string(),
                        })
                    } else {
                        None
                    }
                })
                .collect()
        }
        Err(_) => Vec::new(),
    }
}

/// Obtiene la lista de ramas locales
pub fn get_branches() -> Vec<String> {
    let output = Command::new("git")
        .args(["branch", "--format=%(refname:short)"])
        .output();

    match output {
        Ok(o) => {
            String::from_utf8_lossy(&o.stdout)
                .lines()
                .map(|s| s.to_string())
                .collect()
        }
        Err(_) => Vec::new(),
    }
}

/// Cambia a una rama
pub fn checkout_branch(branch: &str) -> Result<(), String> {
    let output = Command::new("git")
        .args(["checkout", branch])
        .output()
        .map_err(|e| e.to_string())?;

    if output.status.success() {
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(format!("git checkout fallo: {}", stderr))
    }
}

/// Genera un mensaje de commit usando Claude CLI
pub fn generate_commit_message_with_claude() -> Result<String, String> {
    let diff = get_staged_diff();

    if diff.is_empty() {
        return Err("No hay cambios staged".to_string());
    }

    // Limitar el diff para no exceder el contexto
    let diff_limited = if diff.len() > 8000 {
        format!("{}...\n(diff truncado)", &diff[..8000])
    } else {
        diff
    };

    let prompt = format!(
        "Genera un mensaje de commit conciso y descriptivo en español para estos cambios. \
        Solo responde con el mensaje, sin explicaciones ni formato markdown. \
        Usa el formato: tipo: descripcion (ej: feat: agregar login, fix: corregir bug en api). \
        Maximo 72 caracteres.\n\nDiff:\n{}",
        diff_limited
    );

    // Llamar a Claude CLI
    // En Windows, los comandos npm son .cmd, necesitamos usar cmd /c
    #[cfg(windows)]
    let output = Command::new("cmd")
        .args(["/c", "claude", "-p", &prompt])
        .output()
        .map_err(|e| format!("Error ejecutando claude: {}. Asegurate de tener claude instalado (npm i -g @anthropic-ai/claude-code)", e))?;

    #[cfg(not(windows))]
    let output = Command::new("claude")
        .args(["-p", &prompt])
        .output()
        .map_err(|e| format!("Error ejecutando claude: {}. Asegurate de tener claude instalado (npm i -g @anthropic-ai/claude-code)", e))?;

    if output.status.success() {
        let message = String::from_utf8_lossy(&output.stdout)
            .trim()
            .to_string();

        if message.is_empty() {
            Err("Claude no genero respuesta".to_string())
        } else {
            Ok(message)
        }
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(format!("Claude fallo: {}", stderr))
    }
}
