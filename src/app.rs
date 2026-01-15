// app.rs - Estado de la aplicacion

use crate::git::{self, ChangedFile, LogEntry};

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
        }
    }

    /// Refresca la lista de archivos desde git
    pub fn refresh(&mut self) {
        self.files = git::get_changed_files();
        self.branch = git::get_current_branch();

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
}
