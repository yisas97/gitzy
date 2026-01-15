// app.rs - Estado de la aplicacion

use crate::git::{self, ChangedFile, LogEntry, RemoteInfo};

/// Paneles de la aplicacion
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Panel {
    Files,  // Lista de archivos
    Diff,   // Vista del diff
}

/// Modo de la aplicacion
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Mode {
    Normal,       // Navegacion normal
    Commit,       // Escribiendo mensaje de commit
    Log,          // Viendo historial de commits
    Branches,     // Seleccionando rama
    CreateBranch, // Creando nueva rama
    Remotes,      // Panel de remotes/configuracion
    SetRemoteUrl, // Editando URL de un remote
    AddRemote,    // Agregando nuevo remote
}

/// Estado principal de la aplicacion
pub struct App {
    pub running: bool,
    pub files: Vec<ChangedFile>,
    pub selected: usize,
    pub active_panel: Panel,
    pub diff_scroll: u16,
    pub current_diff: String,
    pub branch: String,
    pub message: Option<String>,
    // Commit
    pub mode: Mode,
    pub commit_message: String,
    pub cursor_position: usize,
    pub generating_ai: bool,  // Indica si estamos generando mensaje con AI
    // Log
    pub logs: Vec<LogEntry>,
    pub log_selected: usize,
    // Branches
    pub branches: Vec<String>,
    pub branch_selected: usize,
    // Create Branch
    pub new_branch_name: String,
    pub new_branch_cursor: usize,
    // Remotes
    pub remotes: Vec<RemoteInfo>,
    pub remote_selected: usize,
    // Estado de sync con remote
    pub ahead: usize,
    pub behind: usize,
    // Editar URL de remote
    pub edit_remote_url: String,
    pub edit_remote_cursor: usize,
    // Agregar nuevo remote
    pub new_remote_name: String,
    pub new_remote_url: String,
    pub new_remote_field: usize, // 0 = name, 1 = url
    pub new_remote_cursor: usize,
}

impl App {
    pub fn new() -> Self {
        let files = git::get_changed_files();
        let branch = git::get_current_branch();
        let current_diff = if !files.is_empty() {
            git::get_file_diff(&files[0].path, files[0].staged)
        } else {
            String::new()
        };
        let (ahead, behind) = git::get_ahead_behind();

        Self {
            running: true,
            files,
            selected: 0,
            active_panel: Panel::Files,
            diff_scroll: 0,
            current_diff,
            branch,
            message: None,
            mode: Mode::Normal,
            commit_message: String::new(),
            cursor_position: 0,
            generating_ai: false,
            logs: Vec::new(),
            log_selected: 0,
            branches: Vec::new(),
            branch_selected: 0,
            new_branch_name: String::new(),
            new_branch_cursor: 0,
            remotes: Vec::new(),
            remote_selected: 0,
            ahead,
            behind,
            edit_remote_url: String::new(),
            edit_remote_cursor: 0,
            new_remote_name: String::new(),
            new_remote_url: String::new(),
            new_remote_field: 0,
            new_remote_cursor: 0,
        }
    }

    /// Refresca la lista de archivos desde git
    pub fn refresh(&mut self) {
        self.files = git::get_changed_files();
        self.branch = git::get_current_branch();
        let (ahead, behind) = git::get_ahead_behind();
        self.ahead = ahead;
        self.behind = behind;

        // Ajustar seleccion si es necesario
        if self.selected >= self.files.len() && !self.files.is_empty() {
            self.selected = self.files.len() - 1;
        }

        self.update_diff();
        self.message = Some("Refrescado".to_string());
    }

    /// Mueve la seleccion hacia abajo
    pub fn next(&mut self) {
        if !self.files.is_empty() {
            self.selected = (self.selected + 1) % self.files.len();
            self.update_diff();
            self.diff_scroll = 0;
        }
    }

    /// Mueve la seleccion hacia arriba
    pub fn previous(&mut self) {
        if !self.files.is_empty() {
            self.selected = self.selected
                .checked_sub(1)
                .unwrap_or(self.files.len() - 1);
            self.update_diff();
            self.diff_scroll = 0;
        }
    }

    /// Actualiza el diff del archivo seleccionado
    fn update_diff(&mut self) {
        if let Some(file) = self.files.get(self.selected) {
            self.current_diff = git::get_file_diff(&file.path, file.staged);
        } else {
            self.current_diff = String::new();
        }
    }

    /// Cambia el panel activo
    pub fn toggle_panel(&mut self) {
        self.active_panel = match self.active_panel {
            Panel::Files => Panel::Diff,
            Panel::Diff => Panel::Files,
        };
    }

    /// Scroll del diff hacia abajo
    pub fn scroll_down(&mut self) {
        self.diff_scroll = self.diff_scroll.saturating_add(1);
    }

    /// Scroll del diff hacia arriba
    pub fn scroll_up(&mut self) {
        self.diff_scroll = self.diff_scroll.saturating_sub(1);
    }

    /// Stage/Unstage el archivo seleccionado
    pub fn toggle_stage(&mut self) {
        if let Some(file) = self.files.get(self.selected) {
            let result = if file.staged {
                git::unstage_file(&file.path)
            } else {
                git::stage_file(&file.path)
            };

            match result {
                Ok(_) => {
                    self.message = Some(if file.staged {
                        format!("Unstaged: {}", file.path)
                    } else {
                        format!("Staged: {}", file.path)
                    });
                    self.refresh();
                }
                Err(e) => {
                    self.message = Some(format!("Error: {}", e));
                }
            }
        }
    }

    /// Stage todos los archivos
    pub fn stage_all(&mut self) {
        for file in &self.files {
            if !file.staged {
                let _ = git::stage_file(&file.path);
            }
        }
        self.message = Some("Todos los archivos staged".to_string());
        self.refresh();
    }

    /// Unstage todos los archivos
    pub fn unstage_all(&mut self) {
        for file in &self.files {
            if file.staged {
                let _ = git::unstage_file(&file.path);
            }
        }
        self.message = Some("Todos los archivos unstaged".to_string());
        self.refresh();
    }

    /// Descarta cambios del archivo seleccionado
    pub fn discard_selected(&mut self) {
        if let Some(file) = self.files.get(self.selected) {
            if !file.staged {
                match git::discard_changes(&file.path) {
                    Ok(_) => {
                        self.message = Some(format!("Descartado: {}", file.path));
                        self.refresh();
                    }
                    Err(e) => {
                        self.message = Some(format!("Error: {}", e));
                    }
                }
            } else {
                self.message = Some("Primero haz unstage del archivo".to_string());
            }
        }
    }

    // === Funciones de Commit ===

    /// Entra en modo commit
    pub fn enter_commit_mode(&mut self) {
        let staged_count = self.files.iter().filter(|f| f.staged).count();
        if staged_count == 0 {
            self.message = Some("No hay archivos staged para commit".to_string());
            return;
        }
        self.mode = Mode::Commit;
        self.commit_message.clear();
        self.cursor_position = 0;
    }

    /// Sale del modo commit
    pub fn exit_commit_mode(&mut self) {
        self.mode = Mode::Normal;
        self.commit_message.clear();
        self.cursor_position = 0;
    }

    /// Agrega un caracter al mensaje de commit
    pub fn commit_input_char(&mut self, c: char) {
        self.commit_message.insert(self.cursor_position, c);
        self.cursor_position += 1;
    }

    /// Borra el caracter anterior (backspace)
    pub fn commit_delete_char(&mut self) {
        if self.cursor_position > 0 {
            self.cursor_position -= 1;
            self.commit_message.remove(self.cursor_position);
        }
    }

    /// Mueve el cursor a la izquierda
    pub fn commit_cursor_left(&mut self) {
        if self.cursor_position > 0 {
            self.cursor_position -= 1;
        }
    }

    /// Mueve el cursor a la derecha
    pub fn commit_cursor_right(&mut self) {
        if self.cursor_position < self.commit_message.len() {
            self.cursor_position += 1;
        }
    }

    /// Ejecuta el commit
    pub fn do_commit(&mut self) {
        if self.commit_message.trim().is_empty() {
            self.message = Some("El mensaje no puede estar vacio".to_string());
            return;
        }

        match git::commit(&self.commit_message) {
            Ok(_) => {
                self.message = Some(format!("Commit exitoso: {}", self.commit_message));
                self.exit_commit_mode();
                self.refresh();
            }
            Err(e) => {
                self.message = Some(format!("Error en commit: {}", e));
            }
        }
    }

    /// Genera mensaje de commit con Claude AI
    pub fn generate_commit_with_ai(&mut self) {
        self.generating_ai = true;
        self.message = Some("Generando mensaje con Claude...".to_string());

        match git::generate_commit_message_with_claude() {
            Ok(msg) => {
                self.commit_message = msg;
                self.cursor_position = self.commit_message.len();
                self.message = Some("Mensaje generado con AI".to_string());
            }
            Err(e) => {
                self.message = Some(format!("Error AI: {}", e));
            }
        }
        self.generating_ai = false;
    }

    pub fn quit(&mut self) {
        self.running = false;
    }

    /// Limpia el mensaje despues de mostrarlo
    pub fn clear_message(&mut self) {
        self.message = None;
    }

    // === Funciones de Log ===

    /// Entra en modo log
    pub fn enter_log_mode(&mut self) {
        self.logs = git::get_log(50);  // Ultimos 50 commits
        self.log_selected = 0;
        self.mode = Mode::Log;
    }

    /// Sale del modo log
    pub fn exit_log_mode(&mut self) {
        self.mode = Mode::Normal;
    }

    /// Navega hacia abajo en el log
    pub fn log_next(&mut self) {
        if !self.logs.is_empty() {
            self.log_selected = (self.log_selected + 1) % self.logs.len();
        }
    }

    /// Navega hacia arriba en el log
    pub fn log_previous(&mut self) {
        if !self.logs.is_empty() {
            self.log_selected = self.log_selected
                .checked_sub(1)
                .unwrap_or(self.logs.len() - 1);
        }
    }

    // === Funciones de Branches ===

    /// Entra en modo branches
    pub fn enter_branches_mode(&mut self) {
        self.branches = git::get_branches();
        // Seleccionar la rama actual
        self.branch_selected = self.branches
            .iter()
            .position(|b| b == &self.branch)
            .unwrap_or(0);
        self.mode = Mode::Branches;
    }

    /// Sale del modo branches
    pub fn exit_branches_mode(&mut self) {
        self.mode = Mode::Normal;
    }

    /// Navega hacia abajo en branches
    pub fn branch_next(&mut self) {
        if !self.branches.is_empty() {
            self.branch_selected = (self.branch_selected + 1) % self.branches.len();
        }
    }

    /// Navega hacia arriba en branches
    pub fn branch_previous(&mut self) {
        if !self.branches.is_empty() {
            self.branch_selected = self.branch_selected
                .checked_sub(1)
                .unwrap_or(self.branches.len() - 1);
        }
    }

    /// Cambia a la rama seleccionada
    pub fn checkout_selected_branch(&mut self) {
        if let Some(branch) = self.branches.get(self.branch_selected) {
            if branch == &self.branch {
                self.message = Some("Ya estas en esa rama".to_string());
                return;
            }

            match git::checkout_branch(branch) {
                Ok(_) => {
                    self.message = Some(format!("Cambiado a rama: {}", branch));
                    self.exit_branches_mode();
                    self.refresh();
                }
                Err(e) => {
                    self.message = Some(format!("Error: {}", e));
                }
            }
        }
    }

    // === Funciones de Create Branch ===

    /// Entra en modo crear rama
    pub fn enter_create_branch_mode(&mut self) {
        self.new_branch_name.clear();
        self.new_branch_cursor = 0;
        self.mode = Mode::CreateBranch;
    }

    /// Sale del modo crear rama
    pub fn exit_create_branch_mode(&mut self) {
        self.mode = Mode::Normal;
        self.new_branch_name.clear();
        self.new_branch_cursor = 0;
    }

    /// Agrega un caracter al nombre de la rama
    pub fn branch_name_input_char(&mut self, c: char) {
        // Solo permitir caracteres validos para nombres de rama
        if c.is_alphanumeric() || c == '-' || c == '_' || c == '/' {
            self.new_branch_name.insert(self.new_branch_cursor, c);
            self.new_branch_cursor += 1;
        }
    }

    /// Borra el caracter anterior
    pub fn branch_name_delete_char(&mut self) {
        if self.new_branch_cursor > 0 {
            self.new_branch_cursor -= 1;
            self.new_branch_name.remove(self.new_branch_cursor);
        }
    }

    /// Mueve el cursor a la izquierda
    pub fn branch_name_cursor_left(&mut self) {
        if self.new_branch_cursor > 0 {
            self.new_branch_cursor -= 1;
        }
    }

    /// Mueve el cursor a la derecha
    pub fn branch_name_cursor_right(&mut self) {
        if self.new_branch_cursor < self.new_branch_name.len() {
            self.new_branch_cursor += 1;
        }
    }

    /// Crea la nueva rama
    pub fn do_create_branch(&mut self) {
        let name = self.new_branch_name.trim();

        if name.is_empty() {
            self.message = Some("El nombre no puede estar vacio".to_string());
            return;
        }

        match git::create_branch(name) {
            Ok(_) => {
                self.message = Some(format!("Rama creada: {}", name));
                self.exit_create_branch_mode();
                self.refresh();
            }
            Err(e) => {
                self.message = Some(format!("Error: {}", e));
            }
        }
    }

    // === Funciones de Remotes ===

    /// Entra en modo remotes
    pub fn enter_remotes_mode(&mut self) {
        self.remotes = git::get_remotes();
        self.remote_selected = 0;
        self.mode = Mode::Remotes;
    }

    /// Sale del modo remotes
    pub fn exit_remotes_mode(&mut self) {
        self.mode = Mode::Normal;
    }

    /// Navega hacia abajo en remotes
    pub fn remote_next(&mut self) {
        if !self.remotes.is_empty() {
            self.remote_selected = (self.remote_selected + 1) % self.remotes.len();
        }
    }

    /// Navega hacia arriba en remotes
    pub fn remote_previous(&mut self) {
        if !self.remotes.is_empty() {
            self.remote_selected = self.remote_selected
                .checked_sub(1)
                .unwrap_or(self.remotes.len() - 1);
        }
    }

    /// Elimina el remote seleccionado
    pub fn delete_selected_remote(&mut self) {
        if let Some(remote) = self.remotes.get(self.remote_selected) {
            let name = remote.name.clone();
            match git::remove_remote(&name) {
                Ok(_) => {
                    self.message = Some(format!("Remote eliminado: {}", name));
                    self.remotes = git::get_remotes();
                    if self.remote_selected >= self.remotes.len() && !self.remotes.is_empty() {
                        self.remote_selected = self.remotes.len() - 1;
                    }
                }
                Err(e) => {
                    self.message = Some(format!("Error: {}", e));
                }
            }
        }
    }

    // === Funciones de SetRemoteUrl ===

    /// Entra en modo editar URL de remote
    pub fn enter_set_url_mode(&mut self) {
        if let Some(remote) = self.remotes.get(self.remote_selected) {
            self.edit_remote_url = remote.push_url.clone();
            if self.edit_remote_url.is_empty() {
                self.edit_remote_url = remote.fetch_url.clone();
            }
            self.edit_remote_cursor = self.edit_remote_url.len();
            self.mode = Mode::SetRemoteUrl;
        }
    }

    /// Sale del modo editar URL
    pub fn exit_set_url_mode(&mut self) {
        self.mode = Mode::Remotes;
        self.edit_remote_url.clear();
        self.edit_remote_cursor = 0;
    }

    /// Agrega un caracter a la URL
    pub fn set_url_input_char(&mut self, c: char) {
        self.edit_remote_url.insert(self.edit_remote_cursor, c);
        self.edit_remote_cursor += 1;
    }

    /// Borra el caracter anterior
    pub fn set_url_delete_char(&mut self) {
        if self.edit_remote_cursor > 0 {
            self.edit_remote_cursor -= 1;
            self.edit_remote_url.remove(self.edit_remote_cursor);
        }
    }

    /// Mueve el cursor a la izquierda
    pub fn set_url_cursor_left(&mut self) {
        if self.edit_remote_cursor > 0 {
            self.edit_remote_cursor -= 1;
        }
    }

    /// Mueve el cursor a la derecha
    pub fn set_url_cursor_right(&mut self) {
        if self.edit_remote_cursor < self.edit_remote_url.len() {
            self.edit_remote_cursor += 1;
        }
    }

    /// Aplica el cambio de URL
    pub fn do_set_remote_url(&mut self) {
        let url = self.edit_remote_url.trim();
        if url.is_empty() {
            self.message = Some("La URL no puede estar vacia".to_string());
            return;
        }

        if let Some(remote) = self.remotes.get(self.remote_selected) {
            let name = remote.name.clone();
            match git::set_remote_url(&name, url) {
                Ok(_) => {
                    self.message = Some(format!("URL actualizada para {}", name));
                    self.remotes = git::get_remotes();
                    self.exit_set_url_mode();
                }
                Err(e) => {
                    self.message = Some(format!("Error: {}", e));
                }
            }
        }
    }

    // === Funciones de AddRemote ===

    /// Entra en modo agregar remote
    pub fn enter_add_remote_mode(&mut self) {
        self.new_remote_name.clear();
        self.new_remote_url.clear();
        self.new_remote_field = 0;
        self.new_remote_cursor = 0;
        self.mode = Mode::AddRemote;
    }

    /// Sale del modo agregar remote
    pub fn exit_add_remote_mode(&mut self) {
        self.mode = Mode::Remotes;
        self.new_remote_name.clear();
        self.new_remote_url.clear();
        self.new_remote_field = 0;
        self.new_remote_cursor = 0;
    }

    /// Cambia entre campo nombre y URL
    pub fn add_remote_toggle_field(&mut self) {
        self.new_remote_field = if self.new_remote_field == 0 { 1 } else { 0 };
        self.new_remote_cursor = if self.new_remote_field == 0 {
            self.new_remote_name.len()
        } else {
            self.new_remote_url.len()
        };
    }

    /// Agrega un caracter al campo activo
    pub fn add_remote_input_char(&mut self, c: char) {
        if self.new_remote_field == 0 {
            // Solo caracteres validos para nombre de remote
            if c.is_alphanumeric() || c == '-' || c == '_' {
                self.new_remote_name.insert(self.new_remote_cursor, c);
                self.new_remote_cursor += 1;
            }
        } else {
            self.new_remote_url.insert(self.new_remote_cursor, c);
            self.new_remote_cursor += 1;
        }
    }

    /// Borra el caracter anterior
    pub fn add_remote_delete_char(&mut self) {
        if self.new_remote_cursor > 0 {
            self.new_remote_cursor -= 1;
            if self.new_remote_field == 0 {
                self.new_remote_name.remove(self.new_remote_cursor);
            } else {
                self.new_remote_url.remove(self.new_remote_cursor);
            }
        }
    }

    /// Mueve el cursor a la izquierda
    pub fn add_remote_cursor_left(&mut self) {
        if self.new_remote_cursor > 0 {
            self.new_remote_cursor -= 1;
        }
    }

    /// Mueve el cursor a la derecha
    pub fn add_remote_cursor_right(&mut self) {
        let max_len = if self.new_remote_field == 0 {
            self.new_remote_name.len()
        } else {
            self.new_remote_url.len()
        };
        if self.new_remote_cursor < max_len {
            self.new_remote_cursor += 1;
        }
    }

    /// Agrega el nuevo remote
    pub fn do_add_remote(&mut self) {
        let name = self.new_remote_name.trim();
        let url = self.new_remote_url.trim();

        if name.is_empty() {
            self.message = Some("El nombre no puede estar vacio".to_string());
            return;
        }
        if url.is_empty() {
            self.message = Some("La URL no puede estar vacia".to_string());
            return;
        }

        match git::add_remote(name, url) {
            Ok(_) => {
                self.message = Some(format!("Remote agregado: {}", name));
                self.remotes = git::get_remotes();
                self.exit_add_remote_mode();
            }
            Err(e) => {
                self.message = Some(format!("Error: {}", e));
            }
        }
    }

    // === Funciones de Push/Pull/Fetch ===

    /// Hace push al remote
    pub fn do_push(&mut self) {
        let remote = match git::get_default_remote() {
            Some(r) => r,
            None => {
                self.message = Some("No hay remotes configurados".to_string());
                return;
            }
        };

        self.message = Some(format!("Pushing a {}...", remote));

        // Si no hay upstream, usar push -u
        if !git::has_upstream() {
            match git::push_set_upstream(&remote, &self.branch) {
                Ok(output) => {
                    let msg = if output.is_empty() {
                        "Push exitoso (upstream configurado)".to_string()
                    } else {
                        format!("Push exitoso: {}", output.lines().next().unwrap_or(""))
                    };
                    self.message = Some(msg);
                    self.refresh();
                }
                Err(e) => {
                    self.message = Some(format!("Error: {}", e));
                }
            }
        } else {
            match git::push(&remote, &self.branch) {
                Ok(output) => {
                    let msg = if output.is_empty() {
                        "Push exitoso".to_string()
                    } else {
                        format!("Push: {}", output.lines().next().unwrap_or("exitoso"))
                    };
                    self.message = Some(msg);
                    self.refresh();
                }
                Err(e) => {
                    self.message = Some(format!("Error: {}", e));
                }
            }
        }
    }

    /// Hace pull del remote
    pub fn do_pull(&mut self) {
        let remote = match git::get_default_remote() {
            Some(r) => r,
            None => {
                self.message = Some("No hay remotes configurados".to_string());
                return;
            }
        };

        self.message = Some(format!("Pulling de {}...", remote));

        match git::pull(&remote, &self.branch) {
            Ok(output) => {
                let msg = if output.contains("Already up to date") {
                    "Ya estas actualizado".to_string()
                } else if output.is_empty() {
                    "Pull exitoso".to_string()
                } else {
                    format!("Pull: {}", output.lines().next().unwrap_or("exitoso"))
                };
                self.message = Some(msg);
                self.refresh();
            }
            Err(e) => {
                self.message = Some(format!("Error: {}", e));
            }
        }
    }

    /// Hace fetch del remote
    pub fn do_fetch(&mut self) {
        let remote = match git::get_default_remote() {
            Some(r) => r,
            None => {
                self.message = Some("No hay remotes configurados".to_string());
                return;
            }
        };

        self.message = Some(format!("Fetching de {}...", remote));

        match git::fetch(&remote) {
            Ok(output) => {
                self.message = Some(output);
                self.refresh();
            }
            Err(e) => {
                self.message = Some(format!("Error: {}", e));
            }
        }
    }
}
