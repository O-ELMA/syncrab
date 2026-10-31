// Crates ────────────────────────────────────────────────────────
use crossterm::event::MouseEvent;
use ratatui::{
    layout::{Alignment, Position},
    prelude::{Buffer, Rect},
    style::{Modifier, Style, Stylize},
    text::Text,
    widgets::{
        Block, BorderType, Cell, Padding, Row, Scrollbar, ScrollbarOrientation, ScrollbarState,
        StatefulWidget, Table, Widget,
    },
};

// mods ──────────────────────────────────────────────────────────
use crate::{
    app::{
        app::App,
        structs::{Component, Filter},
    },
    consts::{
        ACTIVE_SLIDER, ARROW_DOWN, ARROW_UP, COL_BEIGE, COL_BORDER, COL_GRAY, COL_PURPLE, JOURNAL,
        LOG, SLIDER,
    },
    structs::{Job, Log},
    utils::{get_active_jobs, get_active_logs, get_columns_info_by_key, into_lines},
};

// Table of Jobs/Logs ────────────────────────────────────────────
pub fn table(area: Rect, buf: &mut Buffer, freq: &str, app: &mut App) {
    let (columns, styles, col_alignment) = get_columns_info_by_key(freq);
    let header = table_header(columns, col_alignment);

    let selected_index = app
        .states
        .get(freq)
        .and_then(|state| state.table_state.selected());

    let search_term = app.search.value.to_lowercase();
    let mut table_area = area;

    let data: Vec<Row<'_>> = if freq == LOG {
        match &app.selected_log {
            Some(log) => log_result_rows(log, selected_index, col_alignment),
            None => Vec::new(),
        }
    } else if freq == JOURNAL {
        // Render the journal border before borrowing the logs, it may set focus
        table_area = render_journal_border(area, buf, app);
        journal_rows(&app.logs, &search_term, selected_index, col_alignment)
    } else {
        job_rows(
            app.jobs.get(freq).unwrap(),
            &app.filter,
            &search_term,
            selected_index,
            col_alignment,
        )
    };

    let state = app.states.get_mut(freq).unwrap();
    render_scrollbar(area, buf, &mut state.scroll_state);
    state.scroll_state = state.scroll_state.position(data.len());

    StatefulWidget::render(
        Table::new(data, styles).header(header),
        table_area,
        buf,
        &mut state.table_state,
    );
}

fn table_header(columns: &[&'static str], col_alignment: &[Alignment]) -> Row<'static> {
    let mut header_cells = Vec::with_capacity(columns.len());
    for (i, &col) in columns.iter().enumerate() {
        header_cells.push(Cell::from(Text::from(col).alignment(col_alignment[i])));
    }

    Row::new(header_cells).style(Style::default().fg(COL_PURPLE).add_modifier(Modifier::BOLD))
}

fn render_scrollbar(area: Rect, buf: &mut Buffer, state: &mut ScrollbarState) {
    Scrollbar::new(ScrollbarOrientation::VerticalRight)
        .begin_symbol(Some(ARROW_UP))
        .end_symbol(Some(ARROW_DOWN))
        .thumb_symbol(ACTIVE_SLIDER)
        .track_symbol(Some(SLIDER))
        .render(area, buf, state);
}

fn log_result_rows<'a>(
    log: &'a Log,
    selected_index: Option<usize>,
    col_alignment: &[Alignment],
) -> Vec<Row<'a>> {
    let Some(log_results) = &log.log_results else {
        return Vec::new();
    };

    log_results
        .iter()
        .enumerate()
        .map(|(i, log_result)| {
            let mut highest_line = 1;
            let cells = log_result
                .get_fields_data()
                .into_iter()
                .enumerate()
                .map(|(i, field)| {
                    let (lines, content) = into_lines(&field);
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
        .collect()
}

fn journal_rows<'a>(
    logs: &'a [Log],
    search_term: &str,
    selected_index: Option<usize>,
    col_alignment: &[Alignment],
) -> Vec<Row<'a>> {
    get_active_logs(search_term, logs)
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
        .collect()
}

fn job_rows<'a>(
    jobs: &'a [Job],
    filter: &Filter,
    search_term: &str,
    selected_index: Option<usize>,
    col_alignment: &[Alignment],
) -> Vec<Row<'a>> {
    get_active_jobs(search_term, filter, jobs)
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
        .collect()
}

fn render_journal_border(area: Rect, buf: &mut Buffer, app: &mut App) -> Rect {
    let mut border_style = COL_BORDER;

    if app.active_modal.is_none() {
        if let Some(MouseEvent { column, row, .. }) = app.event {
            let pos = Position::new(column, row);

            if area.contains(pos) {
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

    block.inner(area)
}
