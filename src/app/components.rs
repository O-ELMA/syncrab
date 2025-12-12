// Standards ─────────────────────────────────────────────────────
use std::{collections::HashMap, vec};

// Crates ────────────────────────────────────────────────────────
use crossterm::event::MouseEvent;
use ratatui::{
    layout::{Alignment, Constraint, Flex, Layout, Position},
    prelude::{Buffer, Rect},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Text},
    widgets::{
        Block, BorderType, Cell, Clear, List, ListItem, Padding, Paragraph, Row, Scrollbar,
        ScrollbarOrientation, StatefulWidget, Table, Widget,
    },
};

// mods ──────────────────────────────────────────────────────────
use super::{
    app::App,
    structs::{Component, Filter, Modal},
};
use crate::{
    consts::{
        ACTION_BACKUP, ACTION_CLONE, ACTION_CLOSE, ACTION_DELETE, ACTION_DISABLE, ACTION_EDIT,
        ACTION_ENABLE, ACTION_ERASE, ACTION_LOGS, ACTION_MOVE, ACTION_NEW, ACTION_QUIT,
        ACTION_TOGGLE, ACTION_UPDATE, ACTION_VIEW, ACTIVE, ACTIVE_SLIDER, APP_SUBTITLE, APP_TITLE,
        ARROW_DOWN, ARROW_UP, COL_BEIGE, COL_BLUE, COL_BORDER, COL_GRAY, COL_GREEN, COL_LBROWN,
        COL_MAGENTA, COL_ORANGE, COL_PURPLE, COL_TITLE, DAILY, DAY, EMOJI_FILTER, EMOJI_SEARCH,
        EMOJI_SECTION, EMOJI_STATS, EMOJI_STATUS_FAILED, EMOJI_STATUS_OTHER, EMOJI_STATUS_PARTIAL,
        EMOJI_STATUS_SUCCESS, FAILED, FILTER, HOUR, INACTIVE, JOURNAL, LOG, PARTIAL, REAL_TIME,
        REPLACE, REPLACE_WITH, SEARCH, SEPARATOR, SHORTCUT_DAILY, SHORTCUT_FILTER,
        SHORTCUT_REAL_TIME, SHORTCUT_SEARCH, SHORTCUT_WEEKLY, SLIDER, SOURCE, SUCCESS, TARGET,
        TO_REPLACE, WEEKLY,
    },
    structs::Stat,
    utils::{
        capitalise, field, get_active_jobs, get_active_logs, get_columns_info_by_key, into_lines,
    },
};

// Title ─────────────────────────────────────────────────────────
pub fn title(area: Rect, buf: &mut Buffer) {
    Text::from(vec![
        Line::styled(APP_TITLE, (COL_ORANGE, Modifier::BOLD)),
        Line::styled(APP_SUBTITLE, (COL_LBROWN, Modifier::ITALIC)),
    ])
    .alignment(Alignment::Center)
    .render(area, buf);
}

// Header ────────────────────────────────────────────────────────
pub fn header(area: Rect, buf: &mut Buffer, stats: &HashMap<&'static str, Stat>) {
    let horizotal_layout = Layout::horizontal(vec![Constraint::Fill(1); 3]);
    let [real_time_area, daily_area, weekly_area] = horizotal_layout.areas(area);

    card(real_time_area, buf, stats.get(REAL_TIME).unwrap());
    card(daily_area, buf, stats.get(DAILY).unwrap());
    card(weekly_area, buf, stats.get(WEEKLY).unwrap());
}

fn card(area: Rect, buf: &mut Buffer, stat: &Stat) {
    let block = Block::bordered()
        .padding(Padding::new(1, 1, 0, 0))
        .border_style(COL_BORDER)
        .border_type(BorderType::Rounded);
    block.clone().render(area, buf);

    let vertical_layout = Layout::vertical(vec![Constraint::Fill(1); 3]);
    let [top, middle, bottom] = vertical_layout.areas(block.inner(area));

    Text::from(format!("{} {}", EMOJI_STATS, stat.name.clone()))
        .add_modifier(Modifier::BOLD)
        .fg(COL_TITLE)
        .render(top, buf);

    Text::from(stat.count.clone().to_string())
        .add_modifier(Modifier::BOLD)
        .fg(COL_BEIGE)
        .render(middle, buf);

    let horizontal_layout = Layout::horizontal(vec![Constraint::Fill(1); 2]);
    let [left, right] = horizontal_layout.areas(bottom);

    Text::from(format!("{} {}", stat.active_count.clone(), ACTIVE))
        .fg(COL_GREEN)
        .render(left, buf);

    Text::from(format!("{} {}", stat.inactive_count.clone(), INACTIVE))
        .fg(COL_GRAY)
        .render(right, buf);
}

// Search ────────────────────────────────────────────────────────
pub fn search(area: Rect, buf: &mut Buffer, app: &mut App) {
    let horizontal_layout =
        Layout::horizontal(vec![Constraint::Ratio(1, 1), Constraint::Ratio(1, 3)]);
    let [left, right] = horizontal_layout.areas(area);

    let mut search_style = (COL_TITLE, COL_BORDER);

    if app.active_modal.is_none() {
        if let Some(MouseEvent { column, row, .. }) = app.event {
            let pos = Position::new(column, row);

            if left.contains(pos) {
                search_style = enable_search(app, left, buf);
            } else if right.contains(pos) && !app.filter_clicked {
                app.filter_clicked = true;
                app.filter = app.filter.next();
            }
        } else if app.active_component == Some(Component::Search) {
            search_style = enable_search(app, left, buf);
        }
    }

    Paragraph::new(app.search.value.as_str())
        .block(field(
            &format!(
                "{} [{}] {}",
                EMOJI_SEARCH,
                SHORTCUT_SEARCH,
                capitalise(SEARCH)
            ),
            search_style.0,
            search_style.1,
        ))
        .render(left, buf);

    Paragraph::new(capitalise(&app.filter.to_string()))
        .block(field(
            &format!(
                "{} [{}] {}",
                EMOJI_FILTER,
                SHORTCUT_FILTER,
                capitalise(FILTER)
            ),
            COL_TITLE,
            COL_BORDER,
        ))
        .render(right, buf);
}

fn enable_search(app: &mut App, left: Rect, buf: &mut Buffer) -> (Color, Color) {
    app.active_component = Some(Component::Search);
    app.search.set_cursor_position(left, buf);
    (COL_TITLE, COL_BEIGE)
}

// Section ───────────────────────────────────────────────────────
pub fn section(area: Rect, buf: &mut Buffer, app: &mut App) {
    if app.show_journal {
        table(area, buf, JOURNAL, app);
    } else {
        let vertical_layout = Layout::vertical(vec![
            Constraint::Fill(1),
            Constraint::Fill(1),
            Constraint::Fill(1),
        ]);
        let [real_time_area, daily_area, weekly_area] = vertical_layout.areas(area);

        block(real_time_area, buf, REAL_TIME, SHORTCUT_REAL_TIME, app);
        block(daily_area, buf, DAILY, SHORTCUT_DAILY, app);
        block(weekly_area, buf, WEEKLY, SHORTCUT_WEEKLY, app);
    }
}

fn block(area: Rect, buf: &mut Buffer, freq: &str, shortcut: char, app: &mut App) {
    let name = &app.stats.get(freq).unwrap().name;

    let mut block_style = COL_BORDER;

    if app.active_modal.is_none() {
        if let Some(MouseEvent { column, row, .. }) = app.event {
            if app.active_modal != Some(Modal::Job) {
                let pos = Position::new(column, row);

                // If the block is clicked, set it as active
                if area.contains(pos) {
                    app.active_component = Some(Component::from_str(freq));
                    block_style = COL_BEIGE;
                }
                // If clicked outside, disable the active component if it was previously active
                else if app
                    .active_component
                    .as_ref()
                    .map_or(false, |c| c == &Component::from_str(freq))
                {
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
    }

    let block = Block::bordered()
        .padding(Padding::new(1, 1, 0, 0))
        .border_style(block_style)
        .border_type(BorderType::Rounded);
    block.clone().render(area, buf);

    let vertical_layout = Layout::vertical([Constraint::Length(2), Constraint::Fill(1)]);
    let [top, bottom] = vertical_layout.areas(block.inner(area));

    Text::from(format!("{} [{}] {}", EMOJI_SECTION, shortcut, name))
        .add_modifier(Modifier::BOLD)
        .fg(COL_TITLE)
        .render(top, buf);

    table(bottom, buf, freq, app);
}

// Table of Jobs/Logs ────────────────────────────────────────────
pub fn table(area: Rect, buf: &mut Buffer, freq: &str, app: &mut App) {
    // Build header row
    let (columns, styles, col_alignment) = get_columns_info_by_key(freq);

    let header = {
        let mut header_cells = Vec::with_capacity(columns.len());
        for (i, &col) in columns.iter().enumerate() {
            header_cells.push(Cell::from(Text::from(col).alignment(col_alignment[i])));
        }
        Row::new(header_cells).style(Style::default().fg(COL_PURPLE).add_modifier(Modifier::BOLD))
    };

    // Render scrollbar
    let state = app.states.get_mut(freq).unwrap();
    Scrollbar::new(ScrollbarOrientation::VerticalRight)
        .begin_symbol(Some(ARROW_UP))
        .end_symbol(Some(ARROW_DOWN))
        .thumb_symbol(ACTIVE_SLIDER)
        .track_symbol(Some(SLIDER))
        .render(area, buf, &mut state.scroll_state);

    let selected_index = state.table_state.selected();
    let search_term = &app.search.value.to_lowercase();

    let mut data: Vec<Row<'_>> = vec![];
    let mut table_area: Rect = area;

    if freq == LOG {
        // Render Log Results
        if let Some(log) = &app.selected_log {
            if let Some(log_results) = &log.log_results {
                data = log_results
                    .iter()
                    .enumerate()
                    .map(|(i, log_result)| {
                        let mut highest_line = 1;
                        let cells = log_result
                            .get_fields_data()
                            .into_iter()
                            .enumerate()
                            .map(|(i, field)| {
                                let (lines, content) = into_lines(field);
                                if lines > highest_line {
                                    highest_line = lines;
                                }
                                Cell::from(Text::from(content).alignment(col_alignment[i]))
                            })
                            .collect::<Vec<Cell>>();

                        let mut row = Row::new(cells).fg(COL_GRAY).height(highest_line);
                        if selected_index == Some(i) {
                            row = row.fg(COL_BEIGE).add_modifier(Modifier::BOLD);
                        }

                        row
                    })
                    .collect();
            }
        }
    } else if freq == JOURNAL {
        // Render Journal
        let logs = get_active_logs(search_term, &app.logs);

        data = logs
            .iter()
            .enumerate()
            .map(|(i, log)| {
                let cells = log
                    .get_fields_data()
                    .into_iter()
                    .enumerate()
                    .map(|(i, field)| Cell::from(Text::from(field).alignment(col_alignment[i])))
                    .collect::<Vec<Cell>>();

                let mut row = Row::new(cells).fg(COL_GRAY);
                if selected_index == Some(i) {
                    row = row.fg(COL_BEIGE).add_modifier(Modifier::BOLD);
                }

                row
            })
            .collect();

        let mut border_style = COL_BORDER;

        if app.active_modal.is_none() {
            if let Some(MouseEvent { column, row, .. }) = app.event {
                let pos = Position::new(column, row);

                if table_area.contains(pos) {
                    border_style = COL_BEIGE;
                    app.active_component = Some(Component::Journal);
                }
            } else if app.active_component == Some(Component::Journal) {
                border_style = COL_BEIGE;
                app.active_component = Some(Component::Journal);
            }
        }

        let block = Block::bordered()
            .padding(Padding::new(1, 1, 0, 0))
            .border_style(border_style)
            .border_type(BorderType::Rounded);
        block.clone().render(area, buf);

        table_area = block.inner(area);
    }
    // Render Jobs
    else {
        let jobs = get_active_jobs(search_term, &app.filter, &app.jobs.get(freq).unwrap());

        data = jobs
            .iter()
            .enumerate()
            .map(|(i, job)| {
                let cells = job
                    .get_fields_data()
                    .into_iter()
                    .enumerate()
                    .map(|(i, field)| Cell::from(Text::from(field).alignment(col_alignment[i])))
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
        Table::new(data, styles).header(header),
        table_area,
        buf,
        &mut state.table_state,
    );
}

// Job form | Replace string form
pub fn form(area: Rect, buf: &mut Buffer, app: &mut App) {
    let mut fields_num = 0;
    let mut form_name = "";

    if let Some(job) = &app.selected_job {
        form_name = &job.frequency;
        if &job.frequency == REAL_TIME {
            fields_num = 2;
        } else {
            fields_num = 3;
        }
    } else if app.active_modal == Some(Modal::Replace) {
        fields_num = 2;
        form_name = REPLACE;
    }

    let vertical_areas = match fields_num {
        2 => Layout::vertical(vec![Constraint::Ratio(1, 2); 2])
            .areas::<2>(area)
            .to_vec(),
        3 => Layout::vertical(vec![Constraint::Ratio(1, 3); 3])
            .areas::<3>(area)
            .to_vec(),
        _ => unreachable!(),
    };

    let (areas, labels, mut components): (Vec<_>, Vec<_>, Vec<_>) = match form_name {
        REAL_TIME => (
            vec![vertical_areas[0], vertical_areas[1]],
            vec![SOURCE, TARGET],
            vec![&mut app.source, &mut app.target],
        ),
        DAILY => (
            vec![vertical_areas[0], vertical_areas[1], vertical_areas[2]],
            vec![SOURCE, TARGET, HOUR],
            vec![&mut app.source, &mut app.target, &mut app.hour],
        ),
        WEEKLY => {
            let horizontal_layout = Layout::horizontal(vec![Constraint::Ratio(1, 2); 2]);
            let [left, right] = horizontal_layout.areas(vertical_areas[2]);
            (
                vec![vertical_areas[0], vertical_areas[1], left, right],
                vec![SOURCE, TARGET, HOUR, DAY],
                vec![
                    &mut app.source,
                    &mut app.target,
                    &mut app.hour,
                    &mut app.day,
                ],
            )
        }
        REPLACE => (
            vec![vertical_areas[0], vertical_areas[1]],
            vec![TO_REPLACE, REPLACE_WITH],
            vec![&mut app.to_replace, &mut app.replace_with],
        ),
        _ => unreachable!(),
    };

    let mut styles = vec![(COL_BLUE, COL_BORDER); areas.len()];

    if let Some(MouseEvent { column, row, .. }) = app.event {
        let pos = Position::new(column, row);
        for (i, area) in areas.iter().enumerate() {
            if area.contains(pos) {
                app.active_component = Some(Component::from_str(labels[i]));
                styles[i] = (COL_BLUE, Color::White);
                components[i].set_cursor_position(*area, buf);
                app.event = None;
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

    let (is_input_listable, input_label): (bool, &str) =
        if let Some(comp) = app.active_component.as_ref() {
            (comp.is_listable(), comp.to_str())
        } else {
            (false, "")
        };

    let mut dropdown_to_render: Option<(Rect, &Vec<String>)> = None;

    for ((component, area), (label, (fg, bg))) in components
        .iter()
        .zip(&areas)
        .zip(labels.iter().zip(styles.iter()))
    {
        let capitalized_label = capitalise(label);

        Paragraph::new(component.value.clone())
            .block(field(capitalized_label.as_str(), *fg, *bg))
            .render(*area, buf);

        let should_show_suggestions = is_input_listable
            && label == &input_label
            && !component.suggestions.is_empty()
            && !component.value.is_empty()
            && app.suggestion_state.active;

        // If active and has suggestions, calculate dropdown position
        if should_show_suggestions {
            // Calculate area directly below the input field
            let dropdown_height = (component.suggestions.len() as u16).min(10) + 2; // Cap at 10 items + borders
            let dropdown_area = Rect {
                x: area.x,
                y: area.y + area.height, // Start immediately below
                width: area.width,
                height: dropdown_height,
            };

            // Ensure dropdown doesn't go off screen bottom
            let screen_intersection = dropdown_area.intersection(buf.area);

            // Store it to render after the loop
            dropdown_to_render = Some((screen_intersection, &component.suggestions));
        }

        // Render the Dropdown Overlay (Z-Index: Top)
        if let Some((area, suggestions)) = dropdown_to_render {
            // Clear the background so text underneath doesn't show through
            Clear.render(area, buf);

            // Convert strings to ListItems
            let items: Vec<ListItem> = suggestions
                .iter()
                .map(|s| ListItem::new(s.as_str()))
                .collect();

            // Render the List
            let list = List::new(items)
                .block(
                    Block::bordered()
                        .style(Style::default().fg(COL_GRAY))
                        .border_type(BorderType::Rounded)
                        .border_style(COL_MAGENTA),
                )
                .highlight_style(Style::default().fg(COL_BEIGE).bold());

            app.suggestion_state.paths = suggestions.clone();
            app.suggestion_state.len = suggestions.len();
            if app.suggestion_state.state.selected().is_none() {
                app.suggestion_state.state.select_first();
            }

            StatefulWidget::render(list, area, buf, &mut app.suggestion_state.state);
        }
    }
}

// Modal
pub fn modal(area: Rect, buf: &mut Buffer, app: &mut App) {
    if app.active_modal.is_none() {
        return;
    }

    let vertical_const: Constraint = match app.active_modal {
        Some(Modal::Log) => Constraint::Percentage(80),
        Some(Modal::Replace) => Constraint::Length(6),
        Some(Modal::Job) => match app.selected_job.as_ref().unwrap().frequency.as_str() {
            REAL_TIME => Constraint::Length(6),
            _ => Constraint::Length(9),
        },
        None => Constraint::Length(9),
    };

    let vertical = Layout::vertical([vertical_const]).flex(Flex::Center);
    let horizontal = Layout::horizontal([Constraint::Percentage(80)]).flex(Flex::Center);
    let [area] = vertical.areas(area);
    let [area] = horizontal.areas(area);

    Clear.render(area, buf);
    let block = Block::bordered()
        .padding(Padding::new(1, 1, 1, 1))
        .border_style(COL_BORDER)
        .border_type(BorderType::Rounded);
    block.clone().render(area, buf);

    // Render Log & LogResult table
    if let Some(log) = &app.selected_log {
        let status_emoji = match log.status.as_str() {
            SUCCESS => EMOJI_STATUS_SUCCESS,
            FAILED => EMOJI_STATUS_FAILED,
            PARTIAL => EMOJI_STATUS_PARTIAL,
            _ => EMOJI_STATUS_OTHER,
        };

        let inner_vertical = Layout::vertical([Constraint::Length(2), Constraint::Fill(1)]);
        let [first, second] = inner_vertical.areas(block.inner(area));

        Paragraph::new(format!(
            "⏱️ {}  |  ⏹️ {}  |  {} {}",
            log.startstamp, log.endstamp, status_emoji, log.status
        ))
        .alignment(Alignment::Center)
        .render(first, buf);

        table(second, buf, LOG, app);
    }
    // Render Job form | Replace string form
    else if matches!(app.active_modal, Some(Modal::Job) | Some(Modal::Replace)) {
        form(area, buf, app);
    }
}

// Footer ───────────────────────────────────────────────────────
pub fn footer(area: Rect, buf: &mut Buffer, app: &App) {
    let mut shortcuts = Vec::with_capacity(11); // Pre-allocate space for expected max number of shortcuts

    if app.active_modal == Some(Modal::Job) {
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
            let count: u16 = match app.filter {
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
                shortcuts.push(ACTION_CLONE);

                if count > 1 {
                    shortcuts.push(ACTION_ENABLE);
                    shortcuts.push(ACTION_DISABLE);
                }
            }
            shortcuts.push(ACTION_CLOSE);
        }
    } else {
        shortcuts.push(ACTION_LOGS);
        shortcuts.push(ACTION_QUIT);
    }

    // Join shortcuts into a string and render them
    Paragraph::new(shortcuts.join(SEPARATOR))
        .alignment(Alignment::Center)
        .render(area, buf);
}
