// Crates ────────────────────────────────────────────────────────
use crossterm::event::MouseEvent;
use ratatui::{
    layout::{Constraint, Layout, Position},
    prelude::{Buffer, Rect},
    style::{Color, Style, Stylize},
    widgets::{Block, BorderType, Clear, List, ListItem, Paragraph, StatefulWidget, Widget},
};

// mods ──────────────────────────────────────────────────────────
use crate::{
    app::{
        app::App,
        structs::{Component, Modal, SuggestionState},
    },
    consts::{
        COL_BEIGE, COL_BLUE, COL_BORDER, COL_GRAY, COL_MAGENTA, DAILY, DAY, HOUR, REAL_TIME,
        REPLACE, REPLACE_WITH, SOURCE, TARGET, TO_REPLACE, WEEKLY,
    },
    utils::{capitalise, field},
};

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

        if should_show_suggestions {
            render_dropdown(*area, buf, &mut app.suggestion_state, &component.suggestions);
        }
    }
}

fn render_dropdown(
    area: Rect,
    buf: &mut Buffer,
    suggestion_state: &mut SuggestionState,
    suggestions: &[String],
) {
    // Calculate area directly below the input field, capped at 10 items + borders
    let dropdown_height = (suggestions.len() as u16).min(10) + 2;
    let dropdown_area = Rect {
        x: area.x,
        y: area.y + area.height, // Start immediately below
        width: area.width,
        height: dropdown_height,
    };

    // Ensure dropdown doesn't go off screen bottom
    let screen_intersection = dropdown_area.intersection(buf.area);

    // Clear the background so text underneath doesn't show through
    Clear.render(screen_intersection, buf);

    let items: Vec<ListItem> = suggestions
        .iter()
        .map(|s| ListItem::new(s.as_str()))
        .collect();

    let list = List::new(items)
        .block(
            Block::bordered()
                .style(Style::default().fg(COL_GRAY))
                .border_type(BorderType::Rounded)
                .border_style(COL_MAGENTA),
        )
        .highlight_style(Style::default().fg(COL_BEIGE).bold());

    suggestion_state.paths = suggestions.to_vec();
    suggestion_state.len = suggestions.len();
    if suggestion_state.state.selected().is_none() {
        suggestion_state.state.select_first();
    }

    StatefulWidget::render(list, screen_intersection, buf, &mut suggestion_state.state);
}
