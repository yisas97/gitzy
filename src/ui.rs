// ui.rs - Renderizado de la interfaz

use ratatui::{
    prelude::*,
    widgets::*,
};
use crate::app::{App, Mode, Panel};
use crate::git::FileStatus;

/// Renderiza toda la UI
pub fn render(frame: &mut Frame, app: &App) {
    // Layout principal: header, contenido, footer
    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Header
            Constraint::Min(0),     // Contenido
            Constraint::Length(3),  // Footer
        ])
        .split(frame.area());

    render_header(frame, app, main_chunks[0]);
    render_content(frame, app, main_chunks[1]);
    render_footer(frame, app, main_chunks[2]);

    // Mostrar popups segun el modo
    match app.mode {
        Mode::Commit => render_commit_popup(frame, app),
        Mode::Log => render_log_popup(frame, app),
        Mode::Branches => render_branches_popup(frame, app),
        Mode::CreateBranch => render_create_branch_popup(frame, app),
        Mode::Remotes => render_remotes_popup(frame, app),
        Mode::SetRemoteUrl => render_set_url_popup(frame, app),
        Mode::AddRemote => render_add_remote_popup(frame, app),
        Mode::Normal => {}
    }
}

/// Header con info del repo
fn render_header(frame: &mut Frame, app: &App, area: Rect) {
    let status_count = app.files.iter().filter(|f| !f.staged).count();
    let staged_count = app.files.iter().filter(|f| f.staged).count();

    // Indicador de ahead/behind
    let sync_indicator = if app.ahead > 0 || app.behind > 0 {
        format!(" [{}{}]",
            if app.ahead > 0 { format!("{}+", app.ahead) } else { String::new() },
            if app.behind > 0 { format!("{}-", app.behind) } else { String::new() }
        )
    } else {
        String::new()
    };

    let header_text = format!(
        " gitzy   {}{}   {} staged, {} unstaged",
        app.branch,
        sync_indicator,
        staged_count,
        status_count
    );

    let header = Paragraph::new(header_text)
        .style(Style::default().fg(Color::Cyan).bold())
        .block(Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::DarkGray)));

    frame.render_widget(header, area);
}

/// Contenido principal: lista de archivos + diff
fn render_content(frame: &mut Frame, app: &App, area: Rect) {
    // Dividir en dos paneles
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(35),  // Lista de archivos
            Constraint::Percentage(65),  // Diff
        ])
        .split(area);

    render_file_list(frame, app, chunks[0]);
    render_diff(frame, app, chunks[1]);
}

/// Lista de archivos modificados
fn render_file_list(frame: &mut Frame, app: &App, area: Rect) {
    let is_active = app.active_panel == Panel::Files;

    // Crear items directamente de los archivos (sin separadores)
    let items: Vec<ListItem> = app.files
        .iter()
        .enumerate()
        .map(|(idx, file)| {
            let is_selected = idx == app.selected;
            let style = get_file_style(&file.status, is_selected);
            let icon = get_status_icon(&file.status);

            // Indicador de staged: [S] para staged, [ ] para unstaged
            let staged_indicator = if file.staged { "+" } else { " " };
            let staged_color = if file.staged { Color::Green } else { Color::DarkGray };

            let selector = if is_selected { ">>" } else { "  " };

            ListItem::new(Line::from(vec![
                Span::styled(selector, Style::default().fg(Color::Cyan).bold()),
                Span::styled(format!("{}", staged_indicator), Style::default().fg(staged_color)),
                Span::styled(format!(" {} ", icon), style),
                Span::styled(&file.path, style),
            ]))
        })
        .collect();

    // Si no hay archivos
    let items = if items.is_empty() {
        vec![ListItem::new(Line::from(vec![
            Span::styled("  No hay cambios", Style::default().fg(Color::DarkGray)),
        ]))]
    } else {
        items
    };

    let border_color = if is_active { Color::Cyan } else { Color::DarkGray };

    // Contar staged/unstaged para el titulo
    let staged_count = app.files.iter().filter(|f| f.staged).count();
    let unstaged_count = app.files.iter().filter(|f| !f.staged).count();
    let title = format!(" Archivos (+{} staged, {} changes) ", staged_count, unstaged_count);

    let list = List::new(items)
        .block(Block::default()
            .title(title)
            .borders(Borders::ALL)
            .border_style(Style::default().fg(border_color)));

    frame.render_widget(list, area);
}

/// Panel de diff
fn render_diff(frame: &mut Frame, app: &App, area: Rect) {
    let is_active = app.active_panel == Panel::Diff;
    let border_color = if is_active { Color::Cyan } else { Color::DarkGray };

    // Parsear el diff y extraer numeros de linea
    let lines: Vec<Line> = app.current_diff
        .lines()
        .skip(app.diff_scroll as usize)
        .enumerate()
        .map(|(idx, line)| {
            let line_num = app.diff_scroll as usize + idx + 1;

            // Determinar tipo de linea y su estilo
            let (line_style, prefix, bg_color) = if line.starts_with('+') && !line.starts_with("+++") {
                (Style::default().fg(Color::Green), "+", Some(Color::Rgb(0, 40, 0)))
            } else if line.starts_with('-') && !line.starts_with("---") {
                (Style::default().fg(Color::Red), "-", Some(Color::Rgb(40, 0, 0)))
            } else if line.starts_with("@@") {
                (Style::default().fg(Color::Cyan).bold(), "@", None)
            } else if line.starts_with("diff ") {
                (Style::default().fg(Color::Yellow).bold(), "=", None)
            } else if line.starts_with("index ") || line.starts_with("---") || line.starts_with("+++") {
                (Style::default().fg(Color::DarkGray), " ", None)
            } else {
                (Style::default().fg(Color::White), " ", None)
            };

            // Construir la linea con numero, prefijo y contenido
            let num_style = Style::default().fg(Color::DarkGray);
            let prefix_style = line_style;

            let content = if line.len() > 1 && (line.starts_with('+') || line.starts_with('-')) {
                &line[1..]  // Quitar el +/- del contenido ya que lo ponemos como prefijo
            } else {
                line
            };

            // Aplicar fondo si es linea de cambio
            let content_style = if let Some(bg) = bg_color {
                line_style.bg(bg)
            } else {
                line_style
            };

            Line::from(vec![
                Span::styled(format!("{:4} ", line_num), num_style),
                Span::styled(format!("{} ", prefix), prefix_style),
                Span::styled(content, content_style),
            ])
        })
        .collect();

    // Titulo con nombre de archivo y estadisticas
    let title = if let Some(file) = app.files.get(app.selected) {
        let additions = app.current_diff.lines().filter(|l| l.starts_with('+') && !l.starts_with("+++")).count();
        let deletions = app.current_diff.lines().filter(|l| l.starts_with('-') && !l.starts_with("---")).count();
        format!(" {} | +{} -{} ", file.path, additions, deletions)
    } else {
        " Diff ".to_string()
    };

    let total_lines = app.current_diff.lines().count();
    let scroll_info = if total_lines > 0 {
        format!(" L{}/{} ", app.diff_scroll + 1, total_lines)
    } else {
        String::new()
    };

    // Titulo combinado: nombre a la izquierda, scroll a la derecha
    let full_title = format!("{}{}{}",
        title,
        " ".repeat(20),  // Espacio para separar
        scroll_info
    );

    let diff = Paragraph::new(lines)
        .block(Block::default()
            .title(full_title)
            .borders(Borders::ALL)
            .border_style(Style::default().fg(border_color)));

    frame.render_widget(diff, area);
}

/// Footer con ayuda
fn render_footer(frame: &mut Frame, app: &App, area: Rect) {
    let help_text = if let Some(msg) = &app.message {
        msg.clone()
    } else {
        "q:Salir j/k:Nav Space:Stage a:All u:Unstage d:Discard c:Commit g:Log b:Ramas s:Remotes p:Push P:Pull f:Fetch r:Refresh".to_string()
    };

    let footer = Paragraph::new(help_text)
        .style(Style::default().fg(Color::DarkGray))
        .alignment(Alignment::Center)
        .block(Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::DarkGray)));

    frame.render_widget(footer, area);
}

/// Popup para escribir mensaje de commit
fn render_commit_popup(frame: &mut Frame, app: &App) {
    let area = frame.area();

    // Calcular area del popup (centrado)
    let popup_width = 70.min(area.width.saturating_sub(4));
    let popup_height = 12;
    let popup_x = (area.width.saturating_sub(popup_width)) / 2;
    let popup_y = (area.height.saturating_sub(popup_height)) / 2;

    let popup_area = Rect::new(popup_x, popup_y, popup_width, popup_height);

    // Fondo semi-transparente (limpiar area)
    frame.render_widget(Clear, popup_area);

    // Layout del popup
    let popup_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),  // Info
            Constraint::Length(1),  // Espacio
            Constraint::Length(3),  // Input
            Constraint::Length(1),  // Espacio
            Constraint::Length(1),  // Boton AI
            Constraint::Min(0),     // Ayuda
        ])
        .margin(1)
        .split(popup_area);

    // Borde del popup
    let block = Block::default()
        .title(" Commit ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Green))
        .style(Style::default().bg(Color::Black));
    frame.render_widget(block, popup_area);

    // Info de archivos staged
    let staged_count = app.files.iter().filter(|f| f.staged).count();
    let info = Paragraph::new(format!("{} archivo(s) para commit", staged_count))
        .style(Style::default().fg(Color::Cyan));
    frame.render_widget(info, popup_chunks[0]);

    // Input del mensaje
    let input_text = if app.generating_ai {
        "Generando con Claude AI...".to_string()
    } else if app.commit_message.is_empty() {
        "Escribe tu mensaje o presiona Tab para generar con AI...".to_string()
    } else {
        app.commit_message.clone()
    };

    let input_style = if app.generating_ai {
        Style::default().fg(Color::Magenta)
    } else if app.commit_message.is_empty() {
        Style::default().fg(Color::DarkGray)
    } else {
        Style::default().fg(Color::White)
    };

    let input = Paragraph::new(input_text)
        .style(input_style)
        .block(Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Yellow))
            .title(" Mensaje "));
    frame.render_widget(input, popup_chunks[2]);

    // Posicionar cursor (solo si no esta generando)
    if !app.generating_ai && (!app.commit_message.is_empty() || app.cursor_position == 0) {
        frame.set_cursor_position((
            popup_chunks[2].x + app.cursor_position as u16 + 1,
            popup_chunks[2].y + 1,
        ));
    }

    // Boton AI
    let ai_button = Paragraph::new("[ Tab ] Generar mensaje con Claude AI")
        .style(Style::default().fg(Color::Magenta))
        .alignment(Alignment::Center);
    frame.render_widget(ai_button, popup_chunks[4]);

    // Ayuda
    let help = Paragraph::new("Enter: Confirmar | Esc: Cancelar | Tab: AI")
        .style(Style::default().fg(Color::DarkGray))
        .alignment(Alignment::Center);
    frame.render_widget(help, popup_chunks[5]);
}

/// Popup para ver el historial de commits
fn render_log_popup(frame: &mut Frame, app: &App) {
    let area = frame.area();

    // Calcular area del popup (casi toda la pantalla)
    let popup_width = (area.width - 4).min(100);
    let popup_height = (area.height - 4).min(30);
    let popup_x = (area.width.saturating_sub(popup_width)) / 2;
    let popup_y = (area.height.saturating_sub(popup_height)) / 2;

    let popup_area = Rect::new(popup_x, popup_y, popup_width, popup_height);

    // Limpiar area
    frame.render_widget(Clear, popup_area);

    // Crear items del log
    let items: Vec<ListItem> = app.logs
        .iter()
        .enumerate()
        .map(|(idx, log)| {
            let is_selected = idx == app.log_selected;
            let selector = if is_selected { ">> " } else { "   " };

            let style = if is_selected {
                Style::default().bg(Color::DarkGray).bold()
            } else {
                Style::default()
            };

            ListItem::new(Line::from(vec![
                Span::styled(selector, Style::default().fg(Color::Cyan)),
                Span::styled(&log.hash, Style::default().fg(Color::Yellow)),
                Span::styled(" ", Style::default()),
                Span::styled(&log.message, style.fg(Color::White)),
                Span::styled(" - ", Style::default().fg(Color::DarkGray)),
                Span::styled(&log.author, Style::default().fg(Color::Green)),
                Span::styled(" (", Style::default().fg(Color::DarkGray)),
                Span::styled(&log.date, Style::default().fg(Color::Cyan)),
                Span::styled(")", Style::default().fg(Color::DarkGray)),
            ]))
        })
        .collect();

    let items = if items.is_empty() {
        vec![ListItem::new(Line::from(vec![
            Span::styled("  No hay commits", Style::default().fg(Color::DarkGray)),
        ]))]
    } else {
        items
    };

    let list = List::new(items)
        .block(Block::default()
            .title(format!(" Log - {} commits ", app.logs.len()))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Magenta))
            .style(Style::default().bg(Color::Black)));

    frame.render_widget(list, popup_area);

    // Ayuda en la parte inferior
    let help_area = Rect::new(popup_x, popup_y + popup_height - 1, popup_width, 1);
    let help = Paragraph::new(" j/k: Navegar | Esc: Cerrar ")
        .style(Style::default().fg(Color::DarkGray))
        .alignment(Alignment::Center);
    frame.render_widget(help, help_area);
}

/// Popup para seleccionar rama
fn render_branches_popup(frame: &mut Frame, app: &App) {
    let area = frame.area();

    // Calcular area del popup
    let popup_width = 50.min(area.width.saturating_sub(4));
    let popup_height = (app.branches.len() as u16 + 4).min(area.height.saturating_sub(4)).max(6);
    let popup_x = (area.width.saturating_sub(popup_width)) / 2;
    let popup_y = (area.height.saturating_sub(popup_height)) / 2;

    let popup_area = Rect::new(popup_x, popup_y, popup_width, popup_height);

    // Limpiar area
    frame.render_widget(Clear, popup_area);

    // Crear items de branches
    let items: Vec<ListItem> = app.branches
        .iter()
        .enumerate()
        .map(|(idx, branch)| {
            let is_selected = idx == app.branch_selected;
            let is_current = branch == &app.branch;

            let selector = if is_selected { ">> " } else { "   " };
            let current_marker = if is_current { " *" } else { "" };

            let style = if is_selected {
                Style::default().bg(Color::DarkGray).bold()
            } else if is_current {
                Style::default().fg(Color::Green).bold()
            } else {
                Style::default().fg(Color::White)
            };

            ListItem::new(Line::from(vec![
                Span::styled(selector, Style::default().fg(Color::Cyan)),
                Span::styled(branch, style),
                Span::styled(current_marker, Style::default().fg(Color::Green)),
            ]))
        })
        .collect();

    let items = if items.is_empty() {
        vec![ListItem::new(Line::from(vec![
            Span::styled("  No hay ramas", Style::default().fg(Color::DarkGray)),
        ]))]
    } else {
        items
    };

    let list = List::new(items)
        .block(Block::default()
            .title(format!(" Ramas ({}) ", app.branches.len()))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Yellow))
            .style(Style::default().bg(Color::Black)));

    frame.render_widget(list, popup_area);

    // Ayuda
    let help_area = Rect::new(popup_x, popup_y + popup_height - 1, popup_width, 1);
    let help = Paragraph::new(" j/k: Nav | Enter: Cambiar | n: Nueva | Esc: Cerrar ")
        .style(Style::default().fg(Color::DarkGray))
        .alignment(Alignment::Center);
    frame.render_widget(help, help_area);
}

/// Popup para crear nueva rama
fn render_create_branch_popup(frame: &mut Frame, app: &App) {
    let area = frame.area();

    // Calcular area del popup
    let popup_width = 50.min(area.width.saturating_sub(4));
    let popup_height = 8;
    let popup_x = (area.width.saturating_sub(popup_width)) / 2;
    let popup_y = (area.height.saturating_sub(popup_height)) / 2;

    let popup_area = Rect::new(popup_x, popup_y, popup_width, popup_height);

    // Limpiar area
    frame.render_widget(Clear, popup_area);

    // Layout del popup
    let popup_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),  // Info
            Constraint::Length(1),  // Espacio
            Constraint::Length(3),  // Input
            Constraint::Min(0),     // Ayuda
        ])
        .margin(1)
        .split(popup_area);

    // Borde del popup
    let block = Block::default()
        .title(" Nueva Rama ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Green))
        .style(Style::default().bg(Color::Black));
    frame.render_widget(block, popup_area);

    // Info
    let info = Paragraph::new(format!("Crear desde: {}", app.branch))
        .style(Style::default().fg(Color::Cyan));
    frame.render_widget(info, popup_chunks[0]);

    // Input del nombre
    let input_text = if app.new_branch_name.is_empty() {
        "feature/mi-nueva-rama".to_string()
    } else {
        app.new_branch_name.clone()
    };

    let input_style = if app.new_branch_name.is_empty() {
        Style::default().fg(Color::DarkGray)
    } else {
        Style::default().fg(Color::White)
    };

    let input = Paragraph::new(input_text)
        .style(input_style)
        .block(Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Yellow))
            .title(" Nombre "));
    frame.render_widget(input, popup_chunks[2]);

    // Posicionar cursor
    frame.set_cursor_position((
        popup_chunks[2].x + app.new_branch_cursor as u16 + 1,
        popup_chunks[2].y + 1,
    ));

    // Ayuda
    let help = Paragraph::new("Enter: Crear | Esc: Cancelar")
        .style(Style::default().fg(Color::DarkGray))
        .alignment(Alignment::Center);
    frame.render_widget(help, popup_chunks[3]);
}

/// Popup para gestionar remotes
fn render_remotes_popup(frame: &mut Frame, app: &App) {
    let area = frame.area();

    // Calcular area del popup
    let popup_width = 70.min(area.width.saturating_sub(4));
    let popup_height = (app.remotes.len() as u16 + 6).min(area.height.saturating_sub(4)).max(8);
    let popup_x = (area.width.saturating_sub(popup_width)) / 2;
    let popup_y = (area.height.saturating_sub(popup_height)) / 2;

    let popup_area = Rect::new(popup_x, popup_y, popup_width, popup_height);

    // Limpiar area
    frame.render_widget(Clear, popup_area);

    // Crear items de remotes
    let items: Vec<ListItem> = app.remotes
        .iter()
        .enumerate()
        .map(|(idx, remote)| {
            let is_selected = idx == app.remote_selected;
            let selector = if is_selected { ">> " } else { "   " };

            let style = if is_selected {
                Style::default().bg(Color::DarkGray).bold()
            } else {
                Style::default().fg(Color::White)
            };

            // Mostrar nombre y URL (truncada si es muy larga)
            let url = if remote.push_url.len() > 45 {
                format!("{}...", &remote.push_url[..42])
            } else if !remote.push_url.is_empty() {
                remote.push_url.clone()
            } else {
                remote.fetch_url.clone()
            };

            ListItem::new(Line::from(vec![
                Span::styled(selector, Style::default().fg(Color::Cyan)),
                Span::styled(&remote.name, Style::default().fg(Color::Yellow).bold()),
                Span::styled(" -> ", Style::default().fg(Color::DarkGray)),
                Span::styled(url, style),
            ]))
        })
        .collect();

    let items = if items.is_empty() {
        vec![ListItem::new(Line::from(vec![
            Span::styled("  No hay remotes configurados", Style::default().fg(Color::DarkGray)),
        ]))]
    } else {
        items
    };

    let list = List::new(items)
        .block(Block::default()
            .title(format!(" Remotes ({}) ", app.remotes.len()))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Blue))
            .style(Style::default().bg(Color::Black)));

    frame.render_widget(list, popup_area);

    // Ayuda
    let help_area = Rect::new(popup_x, popup_y + popup_height - 1, popup_width, 1);
    let help = Paragraph::new(" j/k: Nav | e: Editar URL | a: Agregar | d: Eliminar | Esc: Cerrar ")
        .style(Style::default().fg(Color::DarkGray))
        .alignment(Alignment::Center);
    frame.render_widget(help, help_area);
}

/// Popup para editar URL de un remote
fn render_set_url_popup(frame: &mut Frame, app: &App) {
    let area = frame.area();

    // Calcular area del popup
    let popup_width = 70.min(area.width.saturating_sub(4));
    let popup_height = 9;
    let popup_x = (area.width.saturating_sub(popup_width)) / 2;
    let popup_y = (area.height.saturating_sub(popup_height)) / 2;

    let popup_area = Rect::new(popup_x, popup_y, popup_width, popup_height);

    // Limpiar area
    frame.render_widget(Clear, popup_area);

    // Layout del popup
    let popup_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),  // Info
            Constraint::Length(1),  // Espacio
            Constraint::Length(3),  // Input
            Constraint::Min(0),     // Ayuda
        ])
        .margin(1)
        .split(popup_area);

    // Borde del popup
    let remote_name = app.remotes.get(app.remote_selected)
        .map(|r| r.name.as_str())
        .unwrap_or("?");
    let block = Block::default()
        .title(format!(" Editar URL - {} ", remote_name))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Yellow))
        .style(Style::default().bg(Color::Black));
    frame.render_widget(block, popup_area);

    // Info
    let info = Paragraph::new("Ingresa la nueva URL del remote")
        .style(Style::default().fg(Color::Cyan));
    frame.render_widget(info, popup_chunks[0]);

    // Input de la URL
    let input_text = if app.edit_remote_url.is_empty() {
        "git@github.com:user/repo.git".to_string()
    } else {
        app.edit_remote_url.clone()
    };

    let input_style = if app.edit_remote_url.is_empty() {
        Style::default().fg(Color::DarkGray)
    } else {
        Style::default().fg(Color::White)
    };

    let input = Paragraph::new(input_text)
        .style(input_style)
        .block(Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Blue))
            .title(" URL "));
    frame.render_widget(input, popup_chunks[2]);

    // Posicionar cursor
    frame.set_cursor_position((
        popup_chunks[2].x + app.edit_remote_cursor as u16 + 1,
        popup_chunks[2].y + 1,
    ));

    // Ayuda
    let help = Paragraph::new("Enter: Guardar | Esc: Cancelar")
        .style(Style::default().fg(Color::DarkGray))
        .alignment(Alignment::Center);
    frame.render_widget(help, popup_chunks[3]);
}

/// Popup para agregar un nuevo remote
fn render_add_remote_popup(frame: &mut Frame, app: &App) {
    let area = frame.area();

    // Calcular area del popup
    let popup_width = 70.min(area.width.saturating_sub(4));
    let popup_height = 12;
    let popup_x = (area.width.saturating_sub(popup_width)) / 2;
    let popup_y = (area.height.saturating_sub(popup_height)) / 2;

    let popup_area = Rect::new(popup_x, popup_y, popup_width, popup_height);

    // Limpiar area
    frame.render_widget(Clear, popup_area);

    // Layout del popup
    let popup_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),  // Info
            Constraint::Length(3),  // Input nombre
            Constraint::Length(3),  // Input URL
            Constraint::Min(0),     // Ayuda
        ])
        .margin(1)
        .split(popup_area);

    // Borde del popup
    let block = Block::default()
        .title(" Agregar Remote ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Green))
        .style(Style::default().bg(Color::Black));
    frame.render_widget(block, popup_area);

    // Info
    let info = Paragraph::new("Tab para cambiar entre campos")
        .style(Style::default().fg(Color::Cyan));
    frame.render_widget(info, popup_chunks[0]);

    // Input del nombre
    let name_text = if app.new_remote_name.is_empty() {
        "origin".to_string()
    } else {
        app.new_remote_name.clone()
    };

    let name_style = if app.new_remote_name.is_empty() {
        Style::default().fg(Color::DarkGray)
    } else {
        Style::default().fg(Color::White)
    };

    let name_border = if app.new_remote_field == 0 {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default().fg(Color::Blue)
    };

    let name_input = Paragraph::new(name_text)
        .style(name_style)
        .block(Block::default()
            .borders(Borders::ALL)
            .border_style(name_border)
            .title(" Nombre "));
    frame.render_widget(name_input, popup_chunks[1]);

    // Input de la URL
    let url_text = if app.new_remote_url.is_empty() {
        "git@github.com:user/repo.git".to_string()
    } else {
        app.new_remote_url.clone()
    };

    let url_style = if app.new_remote_url.is_empty() {
        Style::default().fg(Color::DarkGray)
    } else {
        Style::default().fg(Color::White)
    };

    let url_border = if app.new_remote_field == 1 {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default().fg(Color::Blue)
    };

    let url_input = Paragraph::new(url_text)
        .style(url_style)
        .block(Block::default()
            .borders(Borders::ALL)
            .border_style(url_border)
            .title(" URL "));
    frame.render_widget(url_input, popup_chunks[2]);

    // Posicionar cursor en el campo activo
    if app.new_remote_field == 0 {
        frame.set_cursor_position((
            popup_chunks[1].x + app.new_remote_cursor as u16 + 1,
            popup_chunks[1].y + 1,
        ));
    } else {
        frame.set_cursor_position((
            popup_chunks[2].x + app.new_remote_cursor as u16 + 1,
            popup_chunks[2].y + 1,
        ));
    }

    // Ayuda
    let help = Paragraph::new("Enter: Agregar | Tab: Cambiar campo | Esc: Cancelar")
        .style(Style::default().fg(Color::DarkGray))
        .alignment(Alignment::Center);
    frame.render_widget(help, popup_chunks[3]);
}

/// Obtiene el icono segun el estado del archivo
fn get_status_icon(status: &FileStatus) -> &'static str {
    match status {
        FileStatus::Modified => "M",
        FileStatus::Added => "A",
        FileStatus::Deleted => "D",
        FileStatus::Renamed => "R",
        FileStatus::Untracked => "?",
    }
}

/// Obtiene el estilo segun el estado del archivo
fn get_file_style(status: &FileStatus, selected: bool) -> Style {
    let base_color = match status {
        FileStatus::Modified => Color::Yellow,
        FileStatus::Added => Color::Green,
        FileStatus::Deleted => Color::Red,
        FileStatus::Renamed => Color::Magenta,
        FileStatus::Untracked => Color::Gray,
    };

    let style = Style::default().fg(base_color);

    if selected {
        style.bg(Color::DarkGray).bold()
    } else {
        style
    }
}
