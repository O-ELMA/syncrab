// Standards ─────────────────────────────────────────────────────
use std::{collections::HashMap, vec};
use crate::{
     structs::Stat,
     utils::{field, get_columns_info_by_key}
};
use super::{
    app::App,
    structs::{Filter, Component}
};

// Crates ────────────────────────────────────────────────────────
use ratatui::{
    layout::{Alignment, Constraint, Flex, Layout, Position},
    prelude::{Buffer, Rect},
    style::{Color, Modifier, Style, Stylize}, 
    text::{Line, Text},
    widgets::{Block, BorderType, Cell, Clear, Padding, Paragraph, Row, Scrollbar, ScrollbarOrientation, StatefulWidget, Table, Widget}
};
use crossterm::event::MouseEvent;

// Constants ─────────────────────────────────────────────────────
const COL_GREEN: Color = Color::Rgb(125, 176, 136);
const COL_CYAN: Color = Color::Rgb(135, 173, 161);
const COL_BROWN: Color = Color::Rgb(118, 114, 104);
const COL_LBROWN: Color = Color::Rgb(189, 163, 124);
const COL_BEIGE: Color = Color::Rgb(210, 201, 174);
const COL_ORANGE: Color = Color::Rgb(247, 161, 108);
const COL_PURPLE: Color = Color::Rgb(159, 132, 181);
const COL_BLUE: Color = Color::Rgb(116, 142, 195);
const _COL_LBLUE: Color = Color::Rgb(140, 185, 201);
const _COL_RED: Color = Color::Rgb(227, 113, 122);
const _COL_MAGENTA: Color = Color::Rgb(196, 126, 145);
const COL_GRAY: Color = Color::Rgb(107, 114, 128);

const COL_TITLE: Color = COL_CYAN;
const COL_BORDER: Color = COL_BROWN;

const ACTION_NEW: &str = "🆕 [n] New";
const ACTION_MOVE: &str = "🧭 [↑↓] Move";
const ACTION_ERASE: &str = "🗑️ [Ctrl+W] Delete Word";
const ACTION_DELETE: &str = "🗑️ [Del] Delete";
const ACTION_TOGGLE: &str = "⏯️ [Space] Toggle";
const ACTION_DISABLE: &str = "🛑 [Alt+Space] Disable All";
const ACTION_ENABLE: &str = "✅ [Ctrl+Space] Enable All";
const ACTION_UPDATE: &str = "💾 [Enter] Update";
const ACTION_EDIT: &str = "📝 [Enter] Edit";
const ACTION_VIEW: &str = "👀 [Enter] View";
const ACTION_QUIT: &str = "❌ [q] Quit";
const ACTION_CLOSE: &str = "❌ [Esc] Close";
const ACTION_LOGS: &str = "📜 [Tab] Logs";
const ACTION_BACKUP: &str = "📦 [Tab] Backup Menu";

// Title ─────────────────────────────────────────────────────────
pub fn title(area: Rect, buf: &mut Buffer) {
    Text::from(vec![
        Line::styled(
            "🦀 Syncrab",
            (COL_ORANGE, Modifier::BOLD)
        ),
        Line::styled(
            "Manage and monitor your backup jobs",
            (COL_LBROWN, Modifier::ITALIC)
        ),
    ])
    .alignment(Alignment::Center)
    .render(area, buf);
}

// Header ────────────────────────────────────────────────────────
pub fn header(area: Rect, buf: &mut Buffer, stats: &HashMap<&'static str, Stat>) {
    let horizotal_layout = Layout::horizontal(vec![Constraint::Fill(1); 3]);
    let [
        daily_area,
        weekly_area,
        monthly_area,
    ] = horizotal_layout.areas(area);

    card(daily_area, buf, stats.get("daily").unwrap());
    card(weekly_area, buf, stats.get("weekly").unwrap());
    card(monthly_area, buf, stats.get("monthly").unwrap());
}

fn card(area: Rect, buf: &mut Buffer, stat: &Stat) {
    let block = Block::bordered()
        .padding(Padding::new(1, 1, 0, 0))
        .border_style(COL_BORDER)
        .border_type(BorderType::Rounded);
    block.clone().render(area, buf);

    let vertical_layout = Layout::vertical(vec![Constraint::Fill(1); 3]);
    let [top, middle, bottom] = vertical_layout.areas(block.inner(area));

    Text::from(format!("🗓️ {}", stat.name.clone()))
        .add_modifier(Modifier::BOLD)
        .fg(COL_TITLE)
        .render(top, buf);

    Text::from(stat.count.clone().to_string())
        .add_modifier(Modifier::BOLD)
        .fg(COL_BEIGE)
        .render(middle, buf);

    let horizontal_layout = Layout::horizontal(vec![Constraint::Fill(1); 2]);
    let [left ,right] = horizontal_layout.areas(bottom);

    Text::from(format!("{} active", stat.active_count.clone()))
        .fg(COL_GREEN)
        .render(left, buf);

    Text::from(format!("{} inactive", stat.inactive_count.clone()))
        .fg(COL_GRAY)
        .render(right, buf);
}

// Search ────────────────────────────────────────────────────────
pub fn search(area: Rect, buf: &mut Buffer, app: &mut App) {
    let horizontal_layout = Layout::horizontal(vec![
        Constraint::Ratio(1, 1),
        Constraint::Ratio(1, 3)
    ]);
    let [left ,right] = horizontal_layout.areas(area);

    let mut search_style = (COL_TITLE, COL_BORDER);
    
    if let Some(MouseEvent { column, row, .. }) = app.event {
        let pos = Position::new(column, row);

        if left.contains(pos) {
            search_style = enable_search(app, left, buf);
        } else if right.contains(pos) && !app.filter_clicked {
            app.filter_clicked = true;
            app.filter = app.filter.next();
        }
    } else if matches!(app.active_component, Some(Component::Search))  {
        search_style = enable_search(app, left, buf);
    }

    Paragraph::new(app.search.value.as_str())
        .block(field("🔭 [s] Search", search_style.0, search_style.1))
        .render(left, buf);

    Paragraph::new (app.filter.to_string())
        .block(field("🔍 [f] Filter", COL_TITLE, COL_BORDER))
        .render(right, buf);
}

fn enable_search(app: &mut App, left: Rect, buf: &mut Buffer) -> (Color, Color) {
    app.active_component = Some(Component::Search);
    app.search.set_cursor_position(left, buf);
    (COL_TITLE, COL_BEIGE)
}

// Section ───────────────────────────────────────────────────────
pub fn section(area: Rect, buf: &mut Buffer, app: &mut App) {
    let vertical_layout = Layout::vertical(vec![
        Constraint::Fill(1),
        Constraint::Fill(1),
        Constraint::Fill(1),
    ]);
    let [
        daily_area,
        weekly_area,
        monthly_area,
    ] = vertical_layout.areas(area);

    block(daily_area, buf, "daily", app);
    block(weekly_area, buf, "weekly", app);
    block(monthly_area, buf, "monthly", app);
}

fn block(area: Rect, buf: &mut Buffer, freq: &str, app: &mut App) {
    let name = &app.stats.get(freq).unwrap().name;
    let shortcut = freq.chars().next().unwrap();

    let mut block_style = COL_BORDER;
    if let Some(MouseEvent { column, row, .. }) = app.event {
        if !app.show_form {
            let pos = Position::new(column, row);
    
            // If the block is clicked, set it as active
            if area.contains(pos) {
                app.active_component = Some(Component::from_str(freq));
                block_style = COL_BEIGE;
            } 
            // If clicked outside, disable the active component if it was previously active
            else if app.active_component.as_ref().map_or(false, |c| c == &Component::from_str(freq)) {
                app.active_component = None;
            }
        }
    }

    // Check if the component is already active from keymaps
    if let Some(comp) = app.active_component.as_ref() {
        if comp == &Component::from_str(freq) {
            block_style = COL_BEIGE;
        }
    }

    let block = Block::bordered()
        .padding(Padding::new(1, 1, 0, 0))
        .border_style(block_style)
        .border_type(BorderType::Rounded);
    block.clone().render(area, buf);

    let vertical_layout = Layout::vertical([
        Constraint::Length(2),
        Constraint::Fill(1)
    ]);
    let [top, bottom] = vertical_layout.areas(block.inner(area));

    Text::from(format!("🕐 [{}] {}", shortcut, name))
        .add_modifier(Modifier::BOLD)
        .fg(COL_TITLE)
        .render(top, buf);

    table(bottom, buf, freq, app);
}

// Table of Jobs/Logs ────────────────────────────────────────────
pub fn table(area: Rect, buf: &mut Buffer, freq: &str, app: &mut App) { 
    // Build header row
    let (
        columns,
        styles,
        col_alignment
    ) = get_columns_info_by_key(freq);

    let header = {
        let mut header_cells = Vec::with_capacity(columns.len());
        for (i, &col) in columns.iter().enumerate() {
            header_cells.push(Cell::from(Text::from(col).alignment(col_alignment[i])));
        }
        Row::new(header_cells)
            .style(Style::default().fg(COL_PURPLE).add_modifier(Modifier::BOLD))
    };

    // Render scrollbar
    let state = app.states.get_mut(freq).unwrap();
    Scrollbar::new(ScrollbarOrientation::VerticalRight)
        .begin_symbol(Some("⏶"))
        .end_symbol(Some("⏷"))
        .thumb_symbol("┃")
        .track_symbol(Some("│"))
        .render(area, buf, &mut state.scroll_state);
    
    let selected_index = state.table_state.selected();
    let search_term = &app.search.value.to_lowercase();

    let mut data: Vec<Row<'_>> = vec![];
    let mut table_area: Rect = area; 

    if app.show_journal {
        
        // Render Log Results
        if let Some (log) = &app.selected_log {
            if let Some(log_results) = &log.log_results {
                data = log_results
                    .iter()
                    .map(|log_result| {
                        let cells = log_result.get_fields_data()
                            .into_iter()
                            .enumerate()
                            .map(|(i, field)| {
                                Cell::from(Text::from(field).alignment(col_alignment[i]))
                            })
                            .collect::<Vec<Cell>>();
                        
                        Row::new(cells).fg(COL_GRAY)
                    })
                    .collect();
            }
        }
        
        // Render Journal
        else {
            let mut logs = app.logs.clone();
    
            if search_term != "" {
                logs.retain(|log| 
                    log.startstamp.to_lowercase().contains(search_term) 
                    || log.endstamp.to_lowercase().contains(search_term)
                    || log.status.to_lowercase().contains(search_term)
                    || log.success_count.to_string().contains(search_term)
                    || log.failed_count.to_string().contains(search_term)
                    || log.id.unwrap().to_string().contains(search_term));
            }
    
            data = logs
                .iter()
                .enumerate()
                .map(|(i, log)| {
                    let cells = log.get_fields_data()
                        .into_iter()
                        .enumerate()
                        .map(|(i, field)| {
                            Cell::from(Text::from(field).alignment(col_alignment[i]))
                        })
                        .collect::<Vec<Cell>>();
                    
                    let mut row = Row::new(cells).fg(COL_GRAY);
                    if selected_index == Some(i) {
                        row = row.fg(COL_BEIGE).add_modifier(Modifier::BOLD);
                    }
    
                    row
                })
                .collect();
    
            let block = Block::bordered()
                .padding(Padding::new(1, 1, 0, 0))
                .border_style(COL_BORDER)
                .border_type(BorderType::Rounded);
            block.clone().render(area, buf);
    
            table_area = block.inner(area)
        }
    } 

    // Render Jobs    
    else {
        let mut jobs = app.jobs.get(freq).unwrap().clone();

        // Filter active jobs
        match app.filter {
            Filter::All => {},
            Filter::Active => { jobs.retain(|job| job.active == 1); },
            Filter::Inactive => { jobs.retain(|job| job.active == 0); },
        }

        // Filter jobs via search term
        if search_term != "" {
            jobs.retain(|job| 
                job.source.to_lowercase().contains(search_term) 
                || job.target.to_lowercase().contains(search_term)
                || job.id.unwrap().to_string().contains(search_term));
        }
    
        data = jobs
            .iter()
            .enumerate()
            .map(|(i, job)| {
                let cells = job
                    .get_fields_data()
                    .into_iter()
                    .enumerate()
                    .map(|(i, field)| {
                        Cell::from(Text::from(field).alignment(col_alignment[i]))
                    })
                    .collect::<Vec<Cell>>();
                
                let mut row = Row::new(cells).fg(COL_GRAY);
                if selected_index == Some(i) {
                    row = row.fg(COL_BEIGE).add_modifier(Modifier::BOLD);
                }
    
                row
            })
            .collect();
    }

    state.scroll_state = state.scroll_state.position(data.len());

    StatefulWidget::render(
        Table::new(
            data,
            styles
        ).header(header),
        table_area,
        buf,
        &mut state.table_state,
    );

}

// Job Form
pub fn form(area: Rect, buf: &mut Buffer, app: &mut App) {
    let vertical = Layout::vertical([Constraint::Length(11)]).flex(Flex::Center);
    let horizontal = Layout::horizontal([Constraint::Percentage(70)]).flex(Flex::Center);
    let [area] = vertical.areas(area);
    let [area] = horizontal.areas(area);

    Clear.render(area, buf);
    let block = Block::bordered()
        .padding(Padding::new(1, 1, 0, 0))
        .title_style(COL_TITLE)
        .add_modifier(Modifier::BOLD)
        .border_style(COL_BEIGE)
        .border_type(BorderType::Rounded);
    
    if let Some(job) = &app.selected_job {
        let vertical_layout = Layout::vertical(vec![Constraint::Ratio(1, 3); 3]);
        let [ first, second, third ] = vertical_layout.areas(block.inner(area));

        let (areas, labels, mut components): (Vec<_>, Vec<_>, Vec<_>) = match job.frequency.as_str() {
            "daily" => (
                vec![first, second, third],
                vec!["source", "target", "hour"],
                vec![&mut app.source, &mut app.target, &mut app.hour],
            ),
            "weekly" | "monthly" => {
                let horizontal_layout = Layout::horizontal(vec![Constraint::Ratio(1, 2); 2]);
                let [left, right] = horizontal_layout.areas(third);
                (
                    vec![first, second, left, right],
                    vec!["source", "target", "hour", "day"],
                    vec![&mut app.source, &mut app.target, &mut app.hour, &mut app.day],
                )
            },
            _ => unreachable!()
        };

        let mut styles = vec![(COL_BLUE, COL_BORDER); areas.len()];

        if let Some(MouseEvent { column, row, .. }) = app.event {
            let pos = Position::new(column, row);
            for (i, area) in areas.iter().enumerate() {
                if area.contains(pos) {
                    app.active_component = Some(Component::from_str(labels[i]));
                    styles[i] = (COL_BLUE, Color::White);
                    components[i].set_cursor_position(*area, buf);
                    break;
                }
            }
        } else if let Some(comp) = app.active_component.as_ref() {
            for (i, label) in labels.iter().enumerate() {
                if comp == &Component::from_str(label) {
                    styles[i] = (COL_BLUE, Color::White);
                    components[i].set_cursor_position(areas[i], buf);
                    break;
                }
            } 
        }

        for ((component, area), (label, (fg, bg))) in components.iter().zip(&areas).zip(labels.iter().zip(styles.iter())) {
            let capitalized_label = label
                .chars()
                .next()
                .map(|c| c.to_uppercase().collect::<String>() + &label[1..])
                .unwrap();

            Paragraph::new(component.value.clone())
                .block(field(capitalized_label.as_str(), *fg, *bg))
                .render(*area, buf);
        }
    }
}

// LogResult Modal
pub fn modal(area: Rect, buf: &mut Buffer, app: &mut App) {
    let vertical = Layout::vertical([Constraint::Percentage(80)]).flex(Flex::Center);
    let horizontal = Layout::horizontal([Constraint::Percentage(80)]).flex(Flex::Center);
    let [area] = vertical.areas(area);
    let [area] = horizontal.areas(area);

    Clear.render(area, buf);
    let block = Block::bordered()
        .padding(Padding::new(1, 1, 1, 1))
        .border_style(COL_BORDER)
        .border_type(BorderType::Rounded);
    block.clone().render(area, buf);

    if let Some(log) = &app.selected_log {
        let status_emoji = match log.status.as_str() {
            "success" => "✅",
            "failed" => "❌",
            "Partial" => "⚠️",
            _ => "📊",
        };

        let inner_vertical = Layout::vertical([Constraint::Length(2), Constraint::Fill(1)]);
        let [first, second] = inner_vertical.areas(block.inner(area));

        Paragraph::new(format!("⏱️ {}  |  ⏹️ {}  |  {} {}", log.startstamp, log.endstamp, status_emoji, log.status))
            .alignment(Alignment::Center)
            .render(first, buf);
        
        table(second, buf, "log", app);
    }
}

// Footer ───────────────────────────────────────────────────────
pub fn footer(area: Rect, buf: &mut Buffer, app: &App) {
    let mut shortcuts = Vec::with_capacity(6); // Pre-allocate space for expected max number of shortcuts

    if app.show_form {
        shortcuts.push(ACTION_CLOSE);
        shortcuts.push(ACTION_MOVE);
        shortcuts.push(ACTION_UPDATE);
    } else if let Some(comp) = &app.active_component {
        if comp.is_field() {
            shortcuts.push(ACTION_ERASE);
            shortcuts.push(ACTION_CLOSE);
        } else if app.show_journal {
            if app.selected_log.is_some() {
                shortcuts.push(ACTION_BACKUP);
                shortcuts.push(ACTION_CLOSE);
                shortcuts.push(ACTION_QUIT);
            } else {
                shortcuts.push(ACTION_MOVE);
                shortcuts.push(ACTION_BACKUP);
                shortcuts.push(ACTION_VIEW);
                shortcuts.push(ACTION_QUIT);
            }
        } else {
            let stat = app.stats.get(comp.to_str()).unwrap();
            let count: u8 = match app.filter {
                Filter::All => stat.count,
                Filter::Active => stat.active_count,
                Filter::Inactive => stat.active_count,
            };

            // Base shortcuts for non-field components
            shortcuts.push(ACTION_NEW);
            shortcuts.push(ACTION_LOGS);
            shortcuts.push(ACTION_QUIT);

            if count > 0 {
                shortcuts.push(ACTION_MOVE);
                shortcuts.push(ACTION_DELETE);
                shortcuts.push(ACTION_EDIT);
                shortcuts.push(ACTION_TOGGLE);

                if count > 1 {
                    shortcuts.push(ACTION_ENABLE);
                    shortcuts.push(ACTION_DISABLE);
                    shortcuts.push(ACTION_CLOSE);
                }

                shortcuts.push(ACTION_CLOSE);
            }
        }
    } else {
        shortcuts.push(ACTION_LOGS);
        shortcuts.push(ACTION_QUIT);
    }

    // Join shortcuts into a string and render them
    Paragraph::new(shortcuts.join(" | "))
        .alignment(Alignment::Center)
        .render(area, buf);
}
