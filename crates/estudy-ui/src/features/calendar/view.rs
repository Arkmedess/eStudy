//! Renderização do Calendário Unificado (`Ano / Mês / Semana / Dia`).

use chrono::{Datelike, Duration, Months, NaiveDate};
use estudy_domain::{CalendarOccurrence, CalendarViewMode, Project, RecurrenceRule, Task};
use estudy_ui_kit::widgets::{
    card, color_dot, empty_placeholder, hairline_divider, icon_svg, panel, pill_chip,
    section_header, segmented_switch, soft_surface, status_badge,
};
use estudy_ui_kit::{
    EstudyTheme, Icon, RADIUS_CONTROL, RADIUS_PILL, SPACE_1, SPACE_2, SPACE_3, SPACE_4,
};
use iced::widget::{button, column, container, row, scrollable, text, text_input};
use iced::{Alignment, Border, Element, Length};

use super::state::{CalendarMessage, CalendarState};
use crate::features::projects::CONTEXT_SIDEBAR_WIDTH;

/// Renderiza a tela completa do Calendário Unificado.
pub fn view_calendar<'a, Message: Clone + 'a>(
    state: &'a CalendarState,
    projects: &'a [Project],
    tasks: &'a [Task],
    today: NaiveDate,
    theme: EstudyTheme,
    map_msg: impl Fn(CalendarMessage) -> Message + Clone + 'a,
) -> Element<'a, Message> {
    let main_area = view_calendar_main(state, tasks, today, theme, map_msg.clone());

    if state.sidebar_visible {
        let sidebar = view_calendar_sidebar(state, projects, today, theme, map_msg);
        row![sidebar, main_area]
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    } else {
        main_area
    }
}

fn view_calendar_sidebar<'a, Message: Clone + 'a>(
    state: &'a CalendarState,
    projects: &'a [Project],
    today: NaiveDate,
    theme: EstudyTheme,
    map_msg: impl Fn(CalendarMessage) -> Message + 'a,
) -> Element<'a, Message> {
    let create_btn = button(
        row![
            icon_svg(Icon::Plus, 14.0, theme.on_accent),
            text("Criar evento").size(12).color(theme.on_accent),
        ]
        .spacing(SPACE_1)
        .align_y(Alignment::Center),
    )
    .on_press(map_msg(CalendarMessage::ToggleCreateForm))
    .width(Length::Fill)
    .padding([SPACE_2, SPACE_3])
    .style(move |_, status| theme.primary_button_style(status));

    let today_card = panel(
        column![
            section_header(Icon::Calendar, "Data ativa", theme),
            text(format!(
                "{:02}/{:02}/{}",
                state.cursor_date.day(),
                state.cursor_date.month(),
                state.cursor_date.year()
            ))
            .size(13)
            .color(theme.text_primary),
            button(text("Ir para Hoje").size(11).color(theme.accent))
                .on_press(map_msg(CalendarMessage::SelectDate(today)))
                .padding([3.0, 8.0])
                .style(move |_, status| theme.ghost_button_style(status)),
        ]
        .spacing(SPACE_1),
        theme,
    );

    let project_checks = view_project_filter_list(state, projects, theme, &map_msg);
    let tags_row = view_tag_filter_list(state, theme, &map_msg);

    container(
        scrollable(
            column![
                create_btn,
                today_card,
                panel(
                    column![
                        section_header(Icon::FolderKanban, "Projetos", theme),
                        project_checks,
                    ]
                    .spacing(SPACE_2),
                    theme
                ),
                panel(
                    column![section_header(Icon::Tag, "Tags", theme), tags_row].spacing(SPACE_2),
                    theme
                ),
            ]
            .spacing(SPACE_3),
        )
        .height(Length::Fill),
    )
    .width(Length::Fixed(CONTEXT_SIDEBAR_WIDTH))
    .height(Length::Fill)
    .padding(SPACE_3)
    .style(move |_| container::Style {
        background: Some(theme.panel.into()),
        border: Border {
            color: theme.border_subtle,
            width: 1.0,
            radius: 0.0.into(),
        },
        ..container::Style::default()
    })
    .into()
}

fn view_project_filter_list<'a, Message: Clone + 'a>(
    state: &'a CalendarState,
    projects: &'a [Project],
    theme: EstudyTheme,
    map_msg: &impl Fn(CalendarMessage) -> Message,
) -> Element<'a, Message> {
    let mut project_checks = column![].spacing(SPACE_1);
    for proj in projects {
        let checked = state.selected_project_ids.contains(&proj.id);
        let icon = if checked {
            Icon::CheckSquare
        } else {
            Icon::LayoutGrid
        };
        let tint = if checked {
            theme.accent
        } else {
            theme.text_muted
        };
        project_checks = project_checks.push(
            button(
                row![
                    icon_svg(icon, 13.0, tint),
                    text(&proj.title).size(12).color(theme.text_primary),
                ]
                .spacing(SPACE_2)
                .align_y(Alignment::Center),
            )
            .on_press(map_msg(CalendarMessage::ToggleProjectFilter(
                proj.id.clone(),
            )))
            .width(Length::Fill)
            .padding([SPACE_1, SPACE_2])
            .style(move |_, status| theme.ghost_button_style(status)),
        );
    }
    project_checks.into()
}

fn view_tag_filter_list<'a, Message: Clone + 'a>(
    state: &'a CalendarState,
    theme: EstudyTheme,
    map_msg: &impl Fn(CalendarMessage) -> Message,
) -> Element<'a, Message> {
    let mut tags_row = column![].spacing(SPACE_1);
    for tag in state.available_tags() {
        let active = state.active_tag_filter.as_deref() == Some(tag.as_str());
        tags_row = tags_row.push(pill_chip(
            format!("#{tag}"),
            Some(theme.accent),
            None,
            active,
            map_msg(CalendarMessage::ToggleTagFilter(if active {
                None
            } else {
                Some(tag)
            })),
            theme,
        ));
    }
    tags_row.into()
}

fn view_calendar_main<'a, Message: Clone + 'a>(
    state: &'a CalendarState,
    tasks: &'a [Task],
    today: NaiveDate,
    theme: EstudyTheme,
    map_msg: impl Fn(CalendarMessage) -> Message + Clone + 'a,
) -> Element<'a, Message> {
    let toolbar = view_calendar_toolbar(state, theme, map_msg.clone());
    let occurrences = state.visible_occurrences(tasks);

    let mut content = column![toolbar].spacing(SPACE_3);
    if state.create_draft.open {
        content = content.push(view_create_event_form(state, theme, map_msg.clone()));
    }

    let body: Element<'a, Message> = match state.view_mode {
        CalendarViewMode::Year => view_year_grid(state, &occurrences, theme, &map_msg),
        CalendarViewMode::Month => view_month_grid(state, &occurrences, today, theme, &map_msg),
        CalendarViewMode::Week => view_week_columns(state, &occurrences, today, theme, &map_msg),
        CalendarViewMode::Day => view_day_agenda(state, &occurrences, theme),
    };

    content = content.push(body);

    container(content.width(Length::Fill).height(Length::Fill))
        .padding(SPACE_4)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

fn view_calendar_toolbar<'a, Message: Clone + 'a>(
    state: &'a CalendarState,
    theme: EstudyTheme,
    map_msg: impl Fn(CalendarMessage) -> Message + Clone + 'a,
) -> Element<'a, Message> {
    const VIEW_MODES: [(CalendarViewMode, &str); 4] = [
        (CalendarViewMode::Year, "Ano"),
        (CalendarViewMode::Month, "Mês"),
        (CalendarViewMode::Week, "Semana"),
        (CalendarViewMode::Day, "Dia"),
    ];

    let modes = segmented_switch(
        &VIEW_MODES,
        state.view_mode,
        {
            let map_msg = map_msg.clone();
            move |mode| map_msg(CalendarMessage::SetViewMode(mode))
        },
        theme,
    );

    let nav = row![
        button(icon_svg(Icon::PanelLeft, 15.0, theme.text_primary))
            .on_press(map_msg(CalendarMessage::ToggleSidebar))
            .padding(SPACE_2)
            .style(move |_, status| theme.subtle_button_style(status)),
        button(icon_svg(Icon::ChevronLeft, 14.0, theme.text_primary))
            .on_press(map_msg(CalendarMessage::ShiftPeriod(-1)))
            .padding(SPACE_1)
            .style(move |_, status| theme.subtle_button_style(status)),
        text(format!(
            "{:02}/{}",
            state.cursor_date.month(),
            state.cursor_date.year()
        ))
        .size(16)
        .color(theme.text_primary),
        button(icon_svg(Icon::ChevronRight, 14.0, theme.text_primary))
            .on_press(map_msg(CalendarMessage::ShiftPeriod(1)))
            .padding(SPACE_1)
            .style(move |_, status| theme.subtle_button_style(status)),
    ]
    .spacing(SPACE_2)
    .align_y(Alignment::Center);

    let today_btn = pill_chip(
        "Timeline Hoje",
        None,
        Some(Icon::Clock),
        true,
        map_msg(CalendarMessage::OpenTodayDrawer),
        theme,
    );

    row![
        nav,
        container(
            row![modes, today_btn]
                .spacing(SPACE_3)
                .align_y(Alignment::Center)
        )
        .align_x(Alignment::End)
        .width(Length::Fill),
    ]
    .align_y(Alignment::Center)
    .into()
}

fn view_create_event_form<'a, Message: Clone + 'a>(
    state: &'a CalendarState,
    theme: EstudyTheme,
    map_msg: impl Fn(CalendarMessage) -> Message + Clone + 'a,
) -> Element<'a, Message> {
    let title_input = text_input("Título do evento...", &state.create_draft.title)
        .on_input({
            let map_msg = map_msg.clone();
            move |val| map_msg(CalendarMessage::NewTitleChanged(val))
        })
        .padding(SPACE_2)
        .size(13);

    let start_input = text_input("Início HH:MM", &state.create_draft.start_hhmm)
        .on_input({
            let map_msg = map_msg.clone();
            move |val| map_msg(CalendarMessage::NewStartChanged(val))
        })
        .width(Length::Fixed(96.0))
        .padding(SPACE_2)
        .size(12);

    let end_input = text_input("Fim HH:MM", &state.create_draft.end_hhmm)
        .on_input({
            let map_msg = map_msg.clone();
            move |val| map_msg(CalendarMessage::NewEndChanged(val))
        })
        .width(Length::Fixed(96.0))
        .padding(SPACE_2)
        .size(12);

    let rec_options = [
        (None, "Único"),
        (Some(RecurrenceRule::Daily), "Diário"),
        (Some(RecurrenceRule::Weekly), "Semanal"),
        (Some(RecurrenceRule::Monthly), "Mensal"),
    ];
    let mut rec_row = row![].spacing(SPACE_1);
    for (opt, label) in rec_options {
        let active = state.create_draft.recurrence == opt;
        rec_row = rec_row.push(
            button(text(label).size(11).color(if active {
                theme.on_accent
            } else {
                theme.text_primary
            }))
            .on_press(map_msg(CalendarMessage::SetNewRecurrence(opt)))
            .padding([3.0, 8.0])
            .style(move |_, status| {
                if active {
                    theme.primary_button_style(status)
                } else {
                    theme.ghost_button_style(status)
                }
            }),
        );
    }

    card(
        column![
            row![title_input, start_input, end_input].spacing(SPACE_2),
            row![
                rec_row,
                container(
                    button(text("Salvar evento").size(12).color(theme.on_accent))
                        .on_press(map_msg(CalendarMessage::CreateEvent))
                        .padding([SPACE_1, SPACE_3])
                        .style(move |_, status| theme.primary_button_style(status)),
                )
                .align_x(Alignment::End)
                .width(Length::Fill),
            ]
            .align_y(Alignment::Center),
        ]
        .spacing(SPACE_2),
        theme,
    )
    .into()
}

fn view_year_grid<'a, Message: Clone + 'a>(
    state: &'a CalendarState,
    occurrences: &[CalendarOccurrence],
    theme: EstudyTheme,
    map_msg: &impl Fn(CalendarMessage) -> Message,
) -> Element<'a, Message> {
    let year = state.cursor_date.year();
    let mut rows = column![].spacing(SPACE_3);

    for chunk in [1_u32, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12].chunks(4) {
        let mut line = row![].spacing(SPACE_3).width(Length::Fill);
        for &month in chunk {
            let count = occurrences
                .iter()
                .filter(|occ| occ.date.year() == year && occ.date.month() == month)
                .count();
            let target = NaiveDate::from_ymd_opt(year, month, 1).unwrap_or(state.cursor_date);
            line = line.push(
                button(
                    column![
                        text(format!("Mês {month:02}"))
                            .size(14)
                            .color(theme.text_primary),
                        text(format!("{count} itens"))
                            .size(12)
                            .color(theme.text_muted),
                    ]
                    .spacing(SPACE_1),
                )
                .on_press(map_msg(CalendarMessage::SelectDate(target)))
                .width(Length::Fill)
                .padding(SPACE_3)
                .style(move |_, status| theme.ghost_button_style(status)),
            );
        }
        rows = rows.push(line);
    }
    scrollable(rows).height(Length::Fill).into()
}

fn view_month_grid<'a, Message: Clone + 'a>(
    state: &'a CalendarState,
    occurrences: &[CalendarOccurrence],
    today: NaiveDate,
    theme: EstudyTheme,
    map_msg: &impl Fn(CalendarMessage) -> Message,
) -> Element<'a, Message> {
    let first = NaiveDate::from_ymd_opt(state.cursor_date.year(), state.cursor_date.month(), 1)
        .unwrap_or(state.cursor_date);
    let next_month = first
        .checked_add_months(Months::new(1))
        .unwrap_or(first + Duration::days(30));
    let days_count = u32::try_from((next_month - first).num_days()).unwrap_or(30);
    let offset = first.weekday().num_days_from_monday();

    let mut cells: Vec<Option<NaiveDate>> = vec![None; offset as usize];
    for day in 1..=days_count {
        cells.push(NaiveDate::from_ymd_opt(first.year(), first.month(), day));
    }

    let weekdays = ["SEG", "TER", "QUA", "QUI", "SEX", "SÁB", "DOM"];
    let mut weekday_header = row![].spacing(SPACE_2).width(Length::Fill);
    for day_name in weekdays {
        weekday_header = weekday_header.push(
            container(text(day_name).size(11).color(theme.text_muted))
                .width(Length::Fill)
                .padding([0.0, SPACE_1]),
        );
    }

    let mut grid = column![weekday_header, hairline_divider(theme)].spacing(SPACE_2);
    for week in cells.chunks(7) {
        let mut week_row = row![].spacing(SPACE_2).width(Length::Fill);
        for cell in week {
            if let Some(date) = cell {
                week_row = week_row.push(view_month_day_cell(
                    state,
                    *date,
                    today,
                    occurrences,
                    theme,
                    map_msg,
                ));
            } else {
                week_row = week_row.push(container(text("")).width(Length::Fill));
            }
        }
        grid = grid.push(week_row);
    }

    let legend = row![
        color_dot(theme.accent),
        text("Evento / Hoje").size(11).color(theme.text_muted),
        color_dot(theme.warning),
        text("Prazo de tarefa").size(11).color(theme.text_muted),
        color_dot(theme.success),
        text("Concluída").size(11).color(theme.text_muted),
    ]
    .spacing(SPACE_2)
    .align_y(Alignment::Center);

    column![scrollable(grid).height(Length::Fill), legend]
        .spacing(SPACE_2)
        .height(Length::Fill)
        .into()
}

fn view_month_day_cell<'a, Message: Clone + 'a>(
    state: &'a CalendarState,
    date: NaiveDate,
    today: NaiveDate,
    occurrences: &[CalendarOccurrence],
    theme: EstudyTheme,
    map_msg: &impl Fn(CalendarMessage) -> Message,
) -> Element<'a, Message> {
    let day_items: Vec<&CalendarOccurrence> = occurrences
        .iter()
        .filter(|occ| occ.date == date)
        .take(3)
        .collect();

    let is_today = date == today;
    let is_selected = date == state.cursor_date;
    let day_badge: Element<'a, Message> = if is_today {
        container(
            text(format!("{:02}", date.day()))
                .size(11)
                .color(theme.on_accent),
        )
        .padding([2.0, 7.0])
        .style(move |_| container::Style {
            background: Some(theme.accent.into()),
            border: Border {
                radius: RADIUS_PILL.into(),
                ..Border::default()
            },
            ..container::Style::default()
        })
        .into()
    } else {
        text(format!("{:02}", date.day()))
            .size(12)
            .color(theme.text_primary)
            .into()
    };

    let mut list = column![day_badge].spacing(3.0);
    for occ in day_items {
        list = list.push(view_occurrence_pill(occ, theme));
    }

    button(list)
        .on_press(map_msg(CalendarMessage::SelectDate(date)))
        .width(Length::Fill)
        .height(Length::Fixed(96.0))
        .padding(SPACE_2)
        .style(move |_, status| button::Style {
            background: Some(
                if is_selected {
                    theme.accent_subtle
                } else if matches!(status, button::Status::Hovered) {
                    theme.card_hover
                } else {
                    theme.card
                }
                .into(),
            ),
            text_color: theme.text_primary,
            border: Border {
                color: if is_selected || is_today {
                    theme.accent
                } else {
                    theme.border_subtle
                },
                width: 1.0,
                radius: RADIUS_CONTROL.into(),
            },
            ..button::Style::default()
        })
        .into()
}

fn view_week_columns<'a, Message: Clone + 'a>(
    state: &'a CalendarState,
    occurrences: &[CalendarOccurrence],
    today: NaiveDate,
    theme: EstudyTheme,
    map_msg: &impl Fn(CalendarMessage) -> Message,
) -> Element<'a, Message> {
    let (start, _) = state.visible_range();
    let mut cols = row![]
        .spacing(SPACE_2)
        .width(Length::Fill)
        .height(Length::Fill);

    for offset in 0..7_i64 {
        let date = start + Duration::days(offset);
        let is_today = date == today;
        let day_items: Vec<&CalendarOccurrence> =
            occurrences.iter().filter(|occ| occ.date == date).collect();

        let mut day_col = column![
            button(
                text(format!("{:02}/{:02}", date.day(), date.month()))
                    .size(13)
                    .color(if is_today {
                        theme.accent
                    } else {
                        theme.text_primary
                    })
            )
            .on_press(map_msg(CalendarMessage::SelectDate(date)))
            .padding(SPACE_1)
            .style(move |_, status| theme.subtle_button_style(status)),
            hairline_divider(theme),
        ]
        .spacing(SPACE_2);

        if day_items.is_empty() {
            day_col = day_col.push(text("Dia livre por aqui").size(11).color(theme.text_muted));
        } else {
            for occ in day_items {
                day_col = day_col.push(view_occurrence_pill(occ, theme));
            }
        }

        let col_card = if is_today {
            card(scrollable(day_col), theme)
        } else {
            soft_surface(scrollable(day_col), theme)
        };

        cols = cols.push(col_card.width(Length::Fill).height(Length::Fill));
    }

    cols.into()
}

fn view_day_agenda<'a, Message: 'a>(
    state: &'a CalendarState,
    occurrences: &[CalendarOccurrence],
    theme: EstudyTheme,
) -> Element<'a, Message> {
    let day_items: Vec<&CalendarOccurrence> = occurrences
        .iter()
        .filter(|occ| occ.date == state.cursor_date)
        .collect();

    if day_items.is_empty() {
        return empty_placeholder(
            Icon::Calendar,
            "Dia livre na agenda",
            "Nenhum evento ou prazo marcado para esta data.",
            None,
            theme,
        );
    }

    let mut list = column![].spacing(SPACE_2);
    for occ in day_items {
        let time_text = if occ.all_day {
            "Dia todo / Sem horário".to_string()
        } else {
            format!(
                "{:02}:{:02} – {:02}:{:02}",
                occ.start_minute / 60,
                occ.start_minute % 60,
                occ.end_minute / 60,
                occ.end_minute % 60
            )
        };

        let badge_text = if occ.is_task_deadline && occ.is_completed {
            "Concluída"
        } else if occ.is_task_deadline {
            "Prazo"
        } else {
            "Evento"
        };

        list = list.push(card(
            row![
                icon_svg(
                    if occ.is_task_deadline {
                        Icon::Flag
                    } else {
                        Icon::Clock
                    },
                    14.0,
                    theme.accent,
                ),
                column![
                    text(occ.title.clone()).size(14).color(theme.text_primary),
                    text(time_text).size(11).color(theme.text_muted),
                ]
                .spacing(2.0)
                .width(Length::Fill),
                status_badge(badge_text, theme.accent, theme),
            ]
            .spacing(SPACE_2)
            .align_y(Alignment::Center),
            theme,
        ));
    }
    scrollable(list).height(Length::Fill).into()
}

fn view_occurrence_pill<'a, Message: 'a>(
    occ: &CalendarOccurrence,
    theme: EstudyTheme,
) -> Element<'a, Message> {
    let icon = if occ.is_task_deadline {
        Icon::Flag
    } else {
        Icon::Calendar
    };
    let label = if occ.is_completed {
        format!("✓ {}", occ.title)
    } else {
        occ.title.clone()
    };

    row![
        icon_svg(icon, 11.0, theme.accent),
        text(label).size(11).color(if occ.is_completed {
            theme.text_muted
        } else {
            theme.text_primary
        }),
    ]
    .spacing(4.0)
    .align_y(Alignment::Center)
    .into()
}
