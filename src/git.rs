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

/// Información de una rama
#[derive(Debug, Clone)]
pub struct BranchInfo {
    pub name: String,
    pub is_remote: bool,
    pub is_current: bool,
    pub remote_name: Option<String>, // ej: "origin" para origin/main
}

/// Obtiene todas las ramas (locales y remotas)
pub fn get_all_branches() -> Vec<BranchInfo> {
    let mut branches = Vec::new();
    let current = get_current_branch();

    // Ramas locales
    if let Ok(output) = Command::new("git")
        .args(["branch", "--format=%(refname:short)"])
        .output()
    {
        let stdout = String::from_utf8_lossy(&output.stdout);
        for line in stdout.lines() {
            if !line.is_empty() {
                branches.push(BranchInfo {
                    name: line.to_string(),
                    is_remote: false,
                    is_current: line == current,
                    remote_name: None,
                });
            }
        }
    }

    // Ramas remotas
    if let Ok(output) = Command::new("git")
        .args(["branch", "-r", "--format=%(refname:short)"])
        .output()
    {
        let stdout = String::from_utf8_lossy(&output.stdout);
        for line in stdout.lines() {
            // Ignorar HEAD -> origin/main
            if line.is_empty() || line.contains("HEAD") {
                continue;
            }
            // Extraer nombre del remote y la rama (origin/main -> remote=origin, name=main)
            let parts: Vec<&str> = line.splitn(2, '/').collect();
            if parts.len() == 2 {
                let remote = parts[0].to_string();
                let branch_name = parts[1].to_string();

                // No agregar si ya existe como rama local
                let exists_local = branches.iter().any(|b| !b.is_remote && b.name == branch_name);
                if !exists_local {
                    branches.push(BranchInfo {
                        name: branch_name,
                        is_remote: true,
                        is_current: false,
                        remote_name: Some(remote),
                    });
                }
            }
        }
    }

    branches
}

/// Hace checkout de una rama remota (crea rama local con tracking)
pub fn checkout_remote_branch(remote: &str, branch: &str) -> Result<(), String> {
    // git checkout -b <branch> <remote>/<branch>
    let remote_ref = format!("{}/{}", remote, branch);
    let output = Command::new("git")
        .args(["checkout", "-b", branch, &remote_ref])
        .output()
        .map_err(|e| e.to_string())?;

    if output.status.success() {
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        // Si la rama ya existe, intentar checkout normal
        if stderr.contains("already exists") {
            checkout_branch(branch)
        } else {
            Err(format!("git checkout fallo: {}", stderr))
        }
    }
}

/// Hace merge de una rama hacia la rama actual
pub fn merge_branch(branch: &str) -> Result<String, String> {
    let output = Command::new("git")
        .args(["merge", branch])
        .output()
        .map_err(|e| e.to_string())?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    if output.status.success() {
        if stdout.contains("Already up to date") {
            Ok("Ya esta actualizado".to_string())
        } else {
            Ok(format!("Merge exitoso de {}", branch))
        }
    } else {
        if stderr.contains("CONFLICT") || stdout.contains("CONFLICT") {
            Err("Conflictos detectados. Resuelve manualmente.".to_string())
        } else {
            Err(format!("Merge fallo: {}", stderr))
        }
    }
}

/// Hace push de una rama local al remoto (con -u para tracking)
pub fn push_branch_to_remote(remote: &str, branch: &str) -> Result<String, String> {
    let output = Command::new("git")
        .args(["push", "-u", remote, branch])
        .output()
        .map_err(|e| e.to_string())?;

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        Ok(format!("{}{}", stdout, stderr).trim().to_string())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(format!("git push fallo: {}", stderr))
    }
}

/// Crea una nueva rama y cambia a ella
pub fn create_branch(name: &str) -> Result<(), String> {
    let output = Command::new("git")
        .args(["checkout", "-b", name])
        .output()
        .map_err(|e| e.to_string())?;

    if output.status.success() {
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(format!("git checkout -b fallo: {}", stderr))
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

// === Funciones de Remotes ===

/// Información de un remote
#[derive(Debug, Clone)]
pub struct RemoteInfo {
    pub name: String,
    pub fetch_url: String,
    pub push_url: String,
}

/// Obtiene la lista de remotes configurados
pub fn get_remotes() -> Vec<RemoteInfo> {
    let output = Command::new("git")
        .args(["remote", "-v"])
        .output();

    match output {
        Ok(o) => {
            let stdout = String::from_utf8_lossy(&o.stdout);
            let mut remotes: Vec<RemoteInfo> = Vec::new();

            for line in stdout.lines() {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    let name = parts[0].to_string();
                    let url = parts[1].to_string();
                    let is_fetch = line.contains("(fetch)");

                    // Buscar si ya existe este remote
                    if let Some(remote) = remotes.iter_mut().find(|r| r.name == name) {
                        if is_fetch {
                            remote.fetch_url = url;
                        } else {
                            remote.push_url = url;
                        }
                    } else {
                        remotes.push(RemoteInfo {
                            name,
                            fetch_url: if is_fetch { url.clone() } else { String::new() },
                            push_url: if !is_fetch { url } else { String::new() },
                        });
                    }
                }
            }
            remotes
        }
        Err(_) => Vec::new(),
    }
}

/// Cambia la URL de un remote
pub fn set_remote_url(name: &str, url: &str) -> Result<(), String> {
    let output = Command::new("git")
        .args(["remote", "set-url", name, url])
        .output()
        .map_err(|e| e.to_string())?;

    if output.status.success() {
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(format!("git remote set-url fallo: {}", stderr))
    }
}

/// Agrega un nuevo remote
pub fn add_remote(name: &str, url: &str) -> Result<(), String> {
    let output = Command::new("git")
        .args(["remote", "add", name, url])
        .output()
        .map_err(|e| e.to_string())?;

    if output.status.success() {
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(format!("git remote add fallo: {}", stderr))
    }
}

/// Elimina un remote
pub fn remove_remote(name: &str) -> Result<(), String> {
    let output = Command::new("git")
        .args(["remote", "remove", name])
        .output()
        .map_err(|e| e.to_string())?;

    if output.status.success() {
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(format!("git remote remove fallo: {}", stderr))
    }
}

/// Hace git push
pub fn push(remote: &str, branch: &str) -> Result<String, String> {
    let output = Command::new("git")
        .args(["push", remote, branch])
        .output()
        .map_err(|e| e.to_string())?;

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        // Git push escribe el progreso en stderr
        Ok(format!("{}{}", stdout, stderr).trim().to_string())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(format!("git push fallo: {}", stderr))
    }
}

/// Hace git push con -u para establecer upstream
pub fn push_set_upstream(remote: &str, branch: &str) -> Result<String, String> {
    let output = Command::new("git")
        .args(["push", "-u", remote, branch])
        .output()
        .map_err(|e| e.to_string())?;

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        Ok(format!("{}{}", stdout, stderr).trim().to_string())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(format!("git push -u fallo: {}", stderr))
    }
}

/// Hace git pull
pub fn pull(remote: &str, branch: &str) -> Result<String, String> {
    let output = Command::new("git")
        .args(["pull", remote, branch])
        .output()
        .map_err(|e| e.to_string())?;

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        Ok(stdout.trim().to_string())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(format!("git pull fallo: {}", stderr))
    }
}

/// Hace git fetch
pub fn fetch(remote: &str) -> Result<String, String> {
    let output = Command::new("git")
        .args(["fetch", remote])
        .output()
        .map_err(|e| e.to_string())?;

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        let result = format!("{}{}", stdout, stderr).trim().to_string();
        if result.is_empty() {
            Ok("Fetch completado".to_string())
        } else {
            Ok(result)
        }
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(format!("git fetch fallo: {}", stderr))
    }
}

/// Obtiene cuantos commits adelante/atras esta la rama respecto al upstream
pub fn get_ahead_behind() -> (usize, usize) {
    let output = Command::new("git")
        .args(["rev-list", "--left-right", "--count", "HEAD...@{upstream}"])
        .output();

    match output {
        Ok(o) if o.status.success() => {
            let stdout = String::from_utf8_lossy(&o.stdout);
            let parts: Vec<&str> = stdout.trim().split_whitespace().collect();
            if parts.len() >= 2 {
                let ahead = parts[0].parse().unwrap_or(0);
                let behind = parts[1].parse().unwrap_or(0);
                (ahead, behind)
            } else {
                (0, 0)
            }
        }
        _ => (0, 0),
    }
}

/// Verifica si la rama actual tiene upstream configurado
pub fn has_upstream() -> bool {
    Command::new("git")
        .args(["rev-parse", "--abbrev-ref", "@{upstream}"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Obtiene el nombre del remote por defecto (generalmente "origin")
pub fn get_default_remote() -> Option<String> {
    let remotes = get_remotes();
    if remotes.is_empty() {
        None
    } else if let Some(origin) = remotes.iter().find(|r| r.name == "origin") {
        Some(origin.name.clone())
    } else {
        Some(remotes[0].name.clone())
    }
}

