// main.rs - Punto de entrada de gitzy

mod ai;
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

        // Si hay generacion de AI pendiente, ejecutarla (despues del render para mostrar "Generando...")
        if app.generating_ai {
            app.execute_ai_generation();
            continue; // Volver a renderizar inmediatamente
        }

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
                            KeyCode::Tab => app.request_ai_generation(),
                            // 'i' cambia el proveedor de AI (solo si el mensaje esta vacio)
                            KeyCode::Char('i') if app.commit_message.is_empty() => {
                                app.cycle_ai_provider();
                            }
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
                            KeyCode::Char('p') => app.push_selected_branch(),  // Push rama al remote
                            KeyCode::Char('m') => app.merge_selected_branch(), // Merge rama a la actual
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
                    app::Mode::Remotes => {
                        // Modo remotes: gestionar remotes
                        match key.code {
                            KeyCode::Esc | KeyCode::Char('q') => app.exit_remotes_mode(),
                            KeyCode::Char('j') | KeyCode::Down => app.remote_next(),
                            KeyCode::Char('k') | KeyCode::Up => app.remote_previous(),
                            KeyCode::Char('e') | KeyCode::Enter => app.enter_set_url_mode(),
                            KeyCode::Char('a') => app.enter_add_remote_mode(),
                            KeyCode::Char('d') => app.delete_selected_remote(),
                            _ => {}
                        }
                    }
                    app::Mode::SetRemoteUrl => {
                        // Modo editar URL de remote
                        match key.code {
                            KeyCode::Esc => app.exit_set_url_mode(),
                            KeyCode::Enter => app.do_set_remote_url(),
                            KeyCode::Backspace => app.set_url_delete_char(),
                            KeyCode::Left => app.set_url_cursor_left(),
                            KeyCode::Right => app.set_url_cursor_right(),
                            KeyCode::Char(c) => app.set_url_input_char(c),
                            _ => {}
                        }
                    }
                    app::Mode::AddRemote => {
                        // Modo agregar remote
                        match key.code {
                            KeyCode::Esc => app.exit_add_remote_mode(),
                            KeyCode::Enter => app.do_add_remote(),
                            KeyCode::Tab => app.add_remote_toggle_field(),
                            KeyCode::Backspace => app.add_remote_delete_char(),
                            KeyCode::Left => app.add_remote_cursor_left(),
                            KeyCode::Right => app.add_remote_cursor_right(),
                            KeyCode::Char(c) => app.add_remote_input_char(c),
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

                            // Log, Branches y Remotes
                            KeyCode::Char('g') => app.enter_log_mode(),      // g = git log
                            KeyCode::Char('b') => app.enter_branches_mode(), // b = branches
                            KeyCode::Char('s') => app.enter_remotes_mode(),  // s = settings/remotes

                            // Push, Pull, Fetch
                            KeyCode::Char('p') => app.do_push(),             // p = push
                            KeyCode::Char('P') => app.do_pull(),             // P = pull
                            KeyCode::Char('f') => app.do_fetch(),            // f = fetch

                            // Stash
                            KeyCode::Char('S') => app.do_stash(),            // S = stash (guardar)
                            KeyCode::Char('z') => app.do_stash_pop(),        // z = stash pop (recuperar)

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
