//! Renderização limpa da `Agenda` (`Calendário | Lista` + modal de novo compromisso, sem segunda sidebar).

use chrono::{Datelike, Duration, Months, NaiveDate};
use estudy_domain::{CalendarOccurrence, CalendarViewMode, Project, RecurrenceRule, Task};
use estudy_ui_kit::widgets::{
    card, color_dot, editorial_page_header, empty_placeholder, hairline_divider, icon_svg,
    modal_surface, pill_chip, segmented_switch, shaped_text, status_badge,
};
use estudy_ui_kit::{
    CategoryColor, EstudyTheme, FONT_MONO, FONT_SANS, FONT_SANS_MEDIUM, FONT_SANS_SEMIBOLD,
    FONT_SERIF_ITALIC, FONT_SIZE_BASE, FONT_SIZE_LG, FONT_SIZE_MD, FONT_SIZE_SM, FONT_SIZE_XS,
    Icon, RADIUS_CONTROL, RADIUS_PILL, SPACE_1, SPACE_2, SPACE_3, SPACE_4, SPACE_5, SPACE_6,
};
use iced::widget::{button, column, container, row, scrollable, stack, text, text_input};
use iced::{Alignment, Border, Color, Element, Length};

use super::state::{CalendarMessage, CalendarState};
use crate::features::projects::modals::month_name_pt;

/// Renderiza a tela completa da Agenda sem segunda sidebar (estilo Vídeo 2 `00:12`).
pub fn view_calendar<'a, Message: Clone + 'a>(
    state: &'a CalendarState,
    projects: &'a [Project],
    tasks: &'a [Task],
    today: NaiveDate,
    theme: EstudyTheme,
    map_msg: impl Fn(CalendarMessage) -> Message + Clone + 'a,
) -> Element<'a, Message> {
    let new_event_btn = button(
        row![
            icon_svg(Icon::Plus, 16.0, theme.text_primary),
            shaped_text(
                "Novo compromisso",
                FONT_SIZE_BASE,
                FONT_SANS_MEDIUM,
                theme.text_primary,
            ),
        ]
        .spacing(6.0)
        .align_y(Alignment::Center),
    )
    .on_press(map_msg(CalendarMessage::ToggleCreateForm))
    .padding([8.0, 16.0])
    .style(move |_, status| button::Style {
        background: Some(
            if matches!(status, button::Status::Hovered) {
                theme.card_hover
            } else {
                theme.card
            }
            .into(),
        ),
        text_color: theme.text_primary,
        border: Border {
            color: theme.border_subtle,
            width: 1.0,
            radius: RADIUS_PILL.into(),
        },
        shadow: theme.card_style().shadow,
        snap: true,
    });

    let header = editorial_page_header(
        "ESTUDY  /  AGENDA",
        "Agenda",
        "Compromissos",
        Some(new_event_btn.into()),
        theme,
    );

    let toolbar = view_agenda_toolbar(state, projects, theme, map_msg.clone());
    let occurrences = state.visible_occurrences(tasks);

    let body: Element<'a, Message> = match state.view_mode {
        CalendarViewMode::Day => view_agenda_list(state, &occurrences, theme),
        CalendarViewMode::Year | CalendarViewMode::Month | CalendarViewMode::Week => {
            view_month_grid(state, &occurrences, today, theme, &map_msg)
        }
    };

    let footer = view_agenda_footer_legend(theme);

    let page_shell = container(
        column![header, toolbar, body, footer]
            .spacing(SPACE_4)
            .width(Length::Fill)
            .height(Length::Fill),
    )
    .padding([SPACE_5, SPACE_6])
    .width(Length::Fill)
    .height(Length::Fill);

    if state.create_draft.open {
        stack![page_shell, view_create_event_modal(state, theme, map_msg)].into()
    } else {
        page_shell.into()
    }
}

fn view_agenda_toolbar<'a, Message: Clone + 'a>(
    state: &'a CalendarState,
    projects: &'a [Project],
    theme: EstudyTheme,
    map_msg: impl Fn(CalendarMessage) -> Message + Clone + 'a,
) -> Element<'a, Message> {
    const MODES: [(CalendarViewMode, &str); 2] = [
        (CalendarViewMode::Month, "Calendário"),
        (CalendarViewMode::Day, "Lista"),
    ];
    let active_mode = if state.view_mode == CalendarViewMode::Day {
        CalendarViewMode::Day
    } else {
        CalendarViewMode::Month
    };

    let on_mode = map_msg.clone();
    let mode_switch = segmented_switch(
        &MODES,
        active_mode,
        move |mode| on_mode(CalendarMessage::SetViewMode(mode)),
        theme,
    );

    let raw_m = month_name_pt(state.cursor_date.month());
    let mut chars = raw_m.chars();
    let cap_m = chars.next().map_or_else(String::new, |c| {
        format!("{}{}", c.to_uppercase(), chars.as_str())
    });
    let month_label = format!("{cap_m} de {}", state.cursor_date.year());

    let period_nav = row![
        button(icon_svg(Icon::ChevronLeft, 16.0, theme.text_primary))
            .on_press(map_msg(CalendarMessage::ShiftPeriod(-1)))
            .padding(6.0)
            .style(move |_, status| theme.subtle_button_style(status)),
        shaped_text(
            month_label,
            FONT_SIZE_MD,
            FONT_SANS_MEDIUM,
            theme.text_primary
        ),
        button(icon_svg(Icon::ChevronRight, 16.0, theme.text_primary))
            .on_press(map_msg(CalendarMessage::ShiftPeriod(1)))
            .padding(6.0)
            .style(move |_, status| theme.subtle_button_style(status)),
    ]
    .spacing(SPACE_2)
    .align_y(Alignment::Center);

    let mut proj_filters = row![].spacing(SPACE_1).align_y(Alignment::Center);
    for (idx, proj) in projects.iter().enumerate() {
        let checked = state.selected_project_ids.contains(&proj.id);
        proj_filters = proj_filters.push(pill_chip(
            &proj.title,
            Some(CategoryColor::from_index(idx).solid(theme)),
            None,
            checked,
            map_msg(CalendarMessage::ToggleProjectFilter(proj.id.clone())),
            theme,
        ));
    }

    row![
        row![mode_switch, period_nav]
            .spacing(SPACE_4)
            .align_y(Alignment::Center),
        container(proj_filters)
            .align_x(Alignment::End)
            .width(Length::Fill),
    ]
    .align_y(Alignment::Center)
    .width(Length::Fill)
    .into()
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
    let offset = first.weekday().num_days_from_sunday();

    let mut cells: Vec<Option<NaiveDate>> = vec![None; offset as usize];
    for day in 1..=days_count {
        cells.push(NaiveDate::from_ymd_opt(first.year(), first.month(), day));
    }

    let weekdays = ["DOM", "SEG", "TER", "QUA", "QUI", "SEX", "SÁB"];
    let mut weekday_header = row![].spacing(SPACE_2).width(Length::Fill);
    for day_name in weekdays {
        weekday_header = weekday_header.push(
            container(shaped_text(
                day_name,
                FONT_SIZE_XS,
                FONT_SANS_SEMIBOLD,
                theme.text_muted,
            ))
            .center_x(Length::Fill)
            .width(Length::FillPortion(1)),
        );
    }

    let mut grid = column![weekday_header].spacing(SPACE_2);
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
                week_row = week_row.push(
                    container(text(""))
                        .height(Length::Fixed(104.0))
                        .width(Length::FillPortion(1))
                        .style(move |_| container::Style {
                            background: Some(
                                Color {
                                    a: 0.35,
                                    ..theme.surface_soft
                                }
                                .into(),
                            ),
                            border: Border {
                                color: theme.border_subtle,
                                width: 1.0,
                                radius: RADIUS_CONTROL.into(),
                            },
                            ..container::Style::default()
                        }),
                );
            }
        }
        for _ in week.len()..7 {
            week_row = week_row.push(
                container(text(""))
                    .height(Length::Fixed(104.0))
                    .width(Length::FillPortion(1)),
            );
        }
        grid = grid.push(week_row);
    }

    scrollable(grid).height(Length::Fill).into()
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
        container(shaped_text(
            format!("{}", date.day()),
            FONT_SIZE_XS,
            FONT_MONO,
            theme.on_accent,
        ))
        .padding([2.0, 7.0])
        .style(move |_| container::Style {
            background: Some(CategoryColor::Terracotta.solid(theme).into()),
            border: Border {
                radius: RADIUS_PILL.into(),
                ..Border::default()
            },
            ..container::Style::default()
        })
        .into()
    } else {
        shaped_text(
            format!("{}", date.day()),
            FONT_SIZE_SM,
            FONT_MONO,
            theme.text_primary,
        )
        .into()
    };

    let mut list = column![day_badge].spacing(4.0);
    for occ in day_items {
        let dot_color = if occ.is_task_deadline {
            CategoryColor::Ochre.solid(theme)
        } else {
            CategoryColor::Sage.solid(theme)
        };
        list = list.push(
            row![
                color_dot(dot_color),
                shaped_text(
                    occ.title.clone(),
                    FONT_SIZE_XS,
                    FONT_SANS_MEDIUM,
                    if occ.is_completed {
                        theme.text_muted
                    } else {
                        theme.text_primary
                    },
                ),
            ]
            .spacing(5.0)
            .align_y(Alignment::Center),
        );
    }

    button(list)
        .on_press(map_msg(CalendarMessage::SelectDate(date)))
        .width(Length::FillPortion(1))
        .height(Length::Fixed(104.0))
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
                color: if is_today {
                    CategoryColor::Terracotta.solid(theme)
                } else if is_selected {
                    theme.accent
                } else {
                    theme.border_subtle
                },
                width: if is_today || is_selected { 1.5 } else { 1.0 },
                radius: RADIUS_CONTROL.into(),
            },
            ..button::Style::default()
        })
        .into()
}

fn view_agenda_list<'a, Message: 'a>(
    state: &'a CalendarState,
    occurrences: &[CalendarOccurrence],
    theme: EstudyTheme,
) -> Element<'a, Message> {
    if occurrences.is_empty() {
        return empty_placeholder(
            Icon::Calendar,
            "Agenda livre neste período",
            "Clique em + Novo compromisso no canto superior direito para agendar.",
            None,
            theme,
        );
    }

    let mut list = column![].spacing(SPACE_2);
    for occ in occurrences {
        let time_text = if occ.all_day {
            format!("{:02}/{:02} · Dia todo", occ.date.day(), occ.date.month())
        } else {
            format!(
                "{:02}/{:02} · {:02}:{:02} – {:02}:{:02}",
                occ.date.day(),
                occ.date.month(),
                occ.start_minute / 60,
                occ.start_minute % 60,
                occ.end_minute / 60,
                occ.end_minute % 60
            )
        };
        let badge_label = if occ.is_task_deadline {
            "Prazo"
        } else {
            "Compromisso"
        };
        let tint = if occ.is_task_deadline {
            CategoryColor::Ochre.solid(theme)
        } else {
            CategoryColor::Sage.solid(theme)
        };

        list = list.push(card(
            row![
                color_dot(tint),
                column![
                    shaped_text(
                        occ.title.clone(),
                        FONT_SIZE_LG,
                        FONT_SANS_SEMIBOLD,
                        theme.text_primary,
                    ),
                    shaped_text(time_text, FONT_SIZE_SM, FONT_MONO, theme.text_muted),
                ]
                .spacing(2.0)
                .width(Length::Fill),
                status_badge(badge_label, tint, theme),
            ]
            .spacing(SPACE_3)
            .align_y(Alignment::Center),
            theme,
        ));
    }

    let _ = state;
    scrollable(list).height(Length::Fill).into()
}

fn view_agenda_footer_legend<'a, Message: 'a>(theme: EstudyTheme) -> Element<'a, Message> {
    let cats = [
        ("Pessoal", CategoryColor::Terracotta.solid(theme)),
        ("Estudos", CategoryColor::Slate.solid(theme)),
        ("Projetos", CategoryColor::Sage.solid(theme)),
        ("Prazos", CategoryColor::Ochre.solid(theme)),
        ("Revisão", CategoryColor::Lavender.solid(theme)),
    ];
    let mut left = row![].spacing(SPACE_3).align_y(Alignment::Center);
    for (name, c) in cats {
        left = left.push(
            row![
                color_dot(c),
                shaped_text(name, FONT_SIZE_XS, FONT_SANS_MEDIUM, theme.text_muted),
            ]
            .spacing(5.0)
            .align_y(Alignment::Center),
        );
    }

    row![
        left,
        container(
            text("Clique num dia para ver ou criar um compromisso")
                .size(FONT_SIZE_SM)
                .font(FONT_SERIF_ITALIC)
                .color(theme.text_muted)
                .shaping(text::Shaping::Advanced),
        )
        .align_x(Alignment::End)
        .width(Length::Fill),
    ]
    .align_y(Alignment::Center)
    .width(Length::Fill)
    .into()
}

fn view_create_event_modal<'a, Message: Clone + 'a>(
    state: &'a CalendarState,
    theme: EstudyTheme,
    map_msg: impl Fn(CalendarMessage) -> Message + Clone + 'a,
) -> Element<'a, Message> {
    let close_btn = button(icon_svg(Icon::X, 16.0, theme.text_muted))
        .on_press(map_msg(CalendarMessage::ToggleCreateForm))
        .padding(SPACE_1)
        .style(move |_, status| theme.subtle_button_style(status));

    let title_input = text_input("Nome do compromisso...", &state.create_draft.title)
        .on_input({
            let map_msg = map_msg.clone();
            move |val| map_msg(CalendarMessage::NewTitleChanged(val))
        })
        .on_submit(map_msg(CalendarMessage::CreateEvent))
        .padding([SPACE_2, SPACE_3])
        .size(FONT_SIZE_MD)
        .style(move |_, status| theme.soft_input_style(status));

    let start_input = text_input("Início HH:MM", &state.create_draft.start_hhmm)
        .on_input({
            let map_msg = map_msg.clone();
            move |val| map_msg(CalendarMessage::NewStartChanged(val))
        })
        .padding(SPACE_2)
        .size(FONT_SIZE_BASE)
        .style(move |_, status| theme.soft_input_style(status));

    let end_input = text_input("Fim HH:MM", &state.create_draft.end_hhmm)
        .on_input({
            let map_msg = map_msg.clone();
            move |val| map_msg(CalendarMessage::NewEndChanged(val))
        })
        .padding(SPACE_2)
        .size(FONT_SIZE_BASE)
        .style(move |_, status| theme.soft_input_style(status));

    let rec_row = view_recurrence_selector(state.create_draft.recurrence, theme, &map_msg);
    let footer_actions = view_event_modal_actions(theme, &map_msg);

    let body = column![
        row![
            shaped_text(
                format!(
                    "Novo compromisso · {:02}/{:02}/{}",
                    state.cursor_date.day(),
                    state.cursor_date.month(),
                    state.cursor_date.year()
                ),
                FONT_SIZE_LG,
                FONT_SANS_SEMIBOLD,
                theme.text_primary,
            ),
            container(close_btn)
                .align_x(Alignment::End)
                .width(Length::Fill),
        ]
        .align_y(Alignment::Center),
        hairline_divider(theme),
        title_input,
        row![start_input, end_input].spacing(SPACE_2),
        rec_row,
        footer_actions,
    ]
    .spacing(SPACE_3);

    let _ = FONT_SANS;
    modal_surface(body, 460.0, theme)
}

fn view_recurrence_selector<'a, Message: Clone + 'a>(
    active_rule: Option<RecurrenceRule>,
    theme: EstudyTheme,
    map_msg: &impl Fn(CalendarMessage) -> Message,
) -> Element<'a, Message> {
    let rec_options = [
        (None, "Único"),
        (Some(RecurrenceRule::Daily), "Diário"),
        (Some(RecurrenceRule::Weekly), "Semanal"),
        (Some(RecurrenceRule::Monthly), "Mensal"),
    ];
    let mut rec_row = row![].spacing(6.0);
    for (opt, label) in rec_options {
        let active = active_rule == opt;
        rec_row = rec_row.push(
            button(shaped_text(
                label,
                FONT_SIZE_XS,
                FONT_SANS_MEDIUM,
                if active {
                    theme.on_accent
                } else {
                    theme.text_primary
                },
            ))
            .on_press(map_msg(CalendarMessage::SetNewRecurrence(opt)))
            .padding([5.0, 11.0])
            .style(move |_, status| {
                if active {
                    theme.primary_button_style(status)
                } else {
                    theme.ghost_button_style(status)
                }
            }),
        );
    }
    rec_row.into()
}

fn view_event_modal_actions<'a, Message: Clone + 'a>(
    theme: EstudyTheme,
    map_msg: &impl Fn(CalendarMessage) -> Message,
) -> Element<'a, Message> {
    container(
        row![
            button(shaped_text(
                "Cancelar",
                FONT_SIZE_BASE,
                FONT_SANS_MEDIUM,
                theme.text_primary,
            ))
            .on_press(map_msg(CalendarMessage::ToggleCreateForm))
            .padding([8.0, 14.0])
            .style(move |_, status| theme.ghost_button_style(status)),
            button(shaped_text(
                "Salvar compromisso",
                FONT_SIZE_BASE,
                FONT_SANS_SEMIBOLD,
                theme.on_accent,
            ))
            .on_press(map_msg(CalendarMessage::CreateEvent))
            .padding([8.0, 18.0])
            .style(move |_, status| theme.primary_button_style(status)),
        ]
        .spacing(SPACE_2),
    )
    .align_x(Alignment::End)
    .width(Length::Fill)
    .into()
}
