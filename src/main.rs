// main.rs - Punto de entrada de tgit

mod app;
mod git;
mod ui;

use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::prelude::*;
use std::io::{stdout, Result};

use app::App;

fn main() -> Result<()> {
    // Verificar que estamos en un repo git
    if !git::is_git_repo() {
        eprintln!("Error: No estas en un repositorio git");
        eprintln!("Ejecuta 'git init' o navega a un repositorio existente");
        return Ok(());
    }

    // Setup terminal
    stdout().execute(EnterAlternateScreen)?;
    enable_raw_mode()?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;

    // Crear app
    let mut app = App::new();

    // Contador para limpiar mensajes
    let mut message_timer = 0u8;

    // Main loop
    while app.running {
        // Render
        terminal.draw(|frame| ui::render(frame, &app))?;

        // Limpiar mensaje despues de unos ciclos
        if app.message.is_some() {
            message_timer += 1;
            if message_timer > 20 {  // ~2 segundos
                app.clear_message();
                message_timer = 0;
            }
        }

        // Handle events
        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                // IMPORTANTE: Solo procesar eventos de tecla "Press", ignorar "Release" y "Repeat"
                // En Windows, las teclas generan multiples eventos
                if key.kind != KeyEventKind::Press {
                    continue;
                }

                // Manejar diferente segun el modo
                match app.mode {
                    app::Mode::Commit => {
                        // Modo commit: capturar texto
                        match key.code {
                            KeyCode::Esc => app.exit_commit_mode(),
                            KeyCode::Enter => app.do_commit(),
                            KeyCode::Backspace => app.commit_delete_char(),
                            KeyCode::Left => app.commit_cursor_left(),
                            KeyCode::Right => app.commit_cursor_right(),
                            KeyCode::Tab => app.generate_commit_with_ai(),
                            KeyCode::Char(c) => app.commit_input_char(c),
                            _ => {}
                        }
                    }
                    app::Mode::Log => {
                        // Modo log: navegar historial
                        match key.code {
                            KeyCode::Esc | KeyCode::Char('q') => app.exit_log_mode(),
                            KeyCode::Char('j') | KeyCode::Down => app.log_next(),
                            KeyCode::Char('k') | KeyCode::Up => app.log_previous(),
                            _ => {}
                        }
                    }
                    app::Mode::Branches => {
                        // Modo branches: seleccionar rama
                        match key.code {
                            KeyCode::Esc | KeyCode::Char('q') => app.exit_branches_mode(),
                            KeyCode::Char('j') | KeyCode::Down => app.branch_next(),
                            KeyCode::Char('k') | KeyCode::Up => app.branch_previous(),
                            KeyCode::Enter | KeyCode::Char(' ') => app.checkout_selected_branch(),
                            KeyCode::Char('n') => {
                                app.exit_branches_mode();
                                app.enter_create_branch_mode();
                            }
                            _ => {}
                        }
                    }
                    app::Mode::CreateBranch => {
                        // Modo crear rama: escribir nombre
                        match key.code {
                            KeyCode::Esc => app.exit_create_branch_mode(),
                            KeyCode::Enter => app.do_create_branch(),
                            KeyCode::Backspace => app.branch_name_delete_char(),
                            KeyCode::Left => app.branch_name_cursor_left(),
                            KeyCode::Right => app.branch_name_cursor_right(),
                            KeyCode::Char(c) => app.branch_name_input_char(c),
                            _ => {}
                        }
                    }
                    app::Mode::Normal => {
                        // Modo normal: navegacion y comandos
                        match key.code {
                            // Salir
                            KeyCode::Char('q') | KeyCode::Esc => app.quit(),

                            // Navegacion
                            KeyCode::Char('j') | KeyCode::Down => {
                                if app.active_panel == app::Panel::Files {
                                    app.next();
                                } else {
                                    app.scroll_down();
                                }
                            }
                            KeyCode::Char('k') | KeyCode::Up => {
                                if app.active_panel == app::Panel::Files {
                                    app.previous();
                                } else {
                                    app.scroll_up();
                                }
                            }

                            // Cambiar panel
                            KeyCode::Tab | KeyCode::Char('l') | KeyCode::Right => app.toggle_panel(),
                            KeyCode::Char('h') | KeyCode::Left => app.toggle_panel(),

                            // Acciones git
                            KeyCode::Char(' ') | KeyCode::Enter => app.toggle_stage(),
                            KeyCode::Char('a') => app.stage_all(),
                            KeyCode::Char('u') => app.unstage_all(),
                            KeyCode::Char('d') => app.discard_selected(),
                            KeyCode::Char('c') => app.enter_commit_mode(),

                            // Log y Branches
                            KeyCode::Char('g') => app.enter_log_mode(),      // g = git log
                            KeyCode::Char('b') => app.enter_branches_mode(), // b = branches

                            // Refrescar
                            KeyCode::Char('r') => app.refresh(),

                            // Page Up/Down para scroll rapido
                            KeyCode::PageDown => {
                                for _ in 0..10 {
                                    app.scroll_down();
                                }
                            }
                            KeyCode::PageUp => {
                                for _ in 0..10 {
                                    app.scroll_up();
                                }
                            }

                            _ => {}
                        }
                    }
                }
            }
        }
    }

    // Cleanup
    disable_raw_mode()?;
    stdout().execute(LeaveAlternateScreen)?;
    Ok(())
}
