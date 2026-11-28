//! Renderização limpa e interativa da `Agenda` (`Calendário | Lista`, Card do Dia, editar/excluir e filtros por categoria).

use chrono::{Datelike, Duration, Months, NaiveDate};
use estudy_domain::{CalendarOccurrence, CalendarViewMode, Project, RecurrenceRule, Task};
use estudy_ui_kit::widgets::{
    card, color_dot, editorial_page_header, empty_placeholder, hairline_divider, icon_svg,
    modal_surface, pill_chip, segmented_switch, shaped_text, soft_surface, status_badge,
};
use estudy_ui_kit::{
    CategoryColor, EstudyTheme, FONT_MONO, FONT_SANS, FONT_SANS_MEDIUM, FONT_SANS_SEMIBOLD,
    FONT_SERIF, FONT_SERIF_ITALIC, FONT_SIZE_2XL, FONT_SIZE_BASE, FONT_SIZE_LG, FONT_SIZE_MD,
    FONT_SIZE_SM, FONT_SIZE_XS, Icon, RADIUS_CONTROL, RADIUS_PILL, SPACE_1, SPACE_2, SPACE_3,
    SPACE_4, SPACE_5, SPACE_6,
};
use iced::widget::{button, column, container, row, scrollable, stack, text, text_input};
use iced::{Alignment, Border, Color, Element, Length};

use super::state::{AGENDA_CATEGORIES, CalendarMessage, CalendarState};
use crate::features::projects::modals::{month_name_pt, weekday_full_pt_from_date};
use crate::features::projects::project_color_index;

/// Renderiza a tela completa da Agenda com Card do Dia clicável, lista mensal e categorias interativas.
pub fn view_calendar<'a, Message: Clone + 'a>(
    state: &'a CalendarState,
    projects: &'a [Project],
    tasks: &'a [Task],
    today: NaiveDate,
    content_width: f32,
    theme: EstudyTheme,
    map_msg: impl Fn(CalendarMessage) -> Message + Clone + 'a,
) -> Element<'a, Message> {
    let new_event_btn = button(
        row![
            icon_svg(Icon::Plus, 16.0, theme.on_accent),
            shaped_text(
                "Novo compromisso",
                FONT_SIZE_BASE,
                FONT_SANS_SEMIBOLD,
                theme.on_accent,
            ),
        ]
        .spacing(6.0)
        .align_y(Alignment::Center),
    )
    .on_press(map_msg(CalendarMessage::ToggleCreateForm))
    .padding([8.0, 16.0])
    .style(move |_, status| theme.pill_button_style(true, status));

    let header = editorial_page_header(
        "ESTUDY  /  AGENDA",
        "Agenda",
        "Compromissos e prazos",
        Some(new_event_btn.into()),
        theme,
    );

    let toolbar = view_agenda_toolbar(
        state,
        projects,
        today,
        content_width,
        theme,
        map_msg.clone(),
    );
    let occurrences = state.visible_occurrences(tasks);

    let body: Element<'a, Message> = match state.view_mode {
        CalendarViewMode::Day => {
            let month_items = state.month_occurrences(tasks);
            view_agenda_list(state, &month_items, theme, &map_msg)
        }
        CalendarViewMode::Year | CalendarViewMode::Month | CalendarViewMode::Week => {
            view_month_grid(state, &occurrences, today, theme, &map_msg)
        }
    };

    let footer = view_agenda_footer_legend(state, content_width, theme, &map_msg);

    let page_shell = container(
        column![header, toolbar, body, footer]
            .spacing(SPACE_4)
            .width(Length::Fill)
            .height(Length::Fill),
    )
    .padding([SPACE_5, SPACE_6])
    .width(Length::Fill)
    .height(Length::Fill);

    let mut layers: Vec<Element<'a, Message>> = vec![page_shell.into()];
    if state.day_peek_open {
        let day_occurrences = state.month_occurrences(tasks);
        layers.push(view_agenda_day_peek_modal(
            state,
            &day_occurrences,
            theme,
            map_msg.clone(),
        ));
    }
    if state.create_draft.open {
        layers.push(view_create_event_modal(state, theme, map_msg));
    }

    stack(layers)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

fn view_agenda_toolbar<'a, Message: Clone + 'a>(
    state: &'a CalendarState,
    projects: &'a [Project],
    today: NaiveDate,
    content_width: f32,
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
    let away_from_month =
        state.cursor_date.year() != today.year() || state.cursor_date.month() != today.month();

    let mut period_nav = row![
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

    if away_from_month {
        period_nav = period_nav.push(
            button(shaped_text(
                "Hoje",
                FONT_SIZE_XS,
                FONT_SANS_SEMIBOLD,
                theme.accent,
            ))
            .on_press(map_msg(CalendarMessage::JumpToToday(today)))
            .padding([4.0, 10.0])
            .style(move |_, status| theme.ghost_button_style(status)),
        );
    }

    let mut proj_filters = row![].spacing(SPACE_1).align_y(Alignment::Center);
    for (idx, proj) in projects.iter().enumerate() {
        let checked = state.selected_project_ids.contains(&proj.id);
        let color_idx = project_color_index(&proj.color_token, idx);
        proj_filters = proj_filters.push(pill_chip(
            &proj.title,
            Some(CategoryColor::from_index(color_idx).solid(theme)),
            None,
            checked,
            map_msg(CalendarMessage::ToggleProjectFilter(proj.id.clone())),
            theme,
        ));
    }

    let left_cluster = row![mode_switch, period_nav]
        .spacing(SPACE_4)
        .align_y(Alignment::Center);

    if content_width >= 1020.0 {
        row![
            left_cluster,
            container(proj_filters.wrap().vertical_spacing(SPACE_1))
                .align_x(Alignment::End)
                .width(Length::Fill),
        ]
        .spacing(SPACE_3)
        .align_y(Alignment::Center)
        .width(Length::Fill)
        .into()
    } else {
        column![left_cluster, proj_filters.wrap().vertical_spacing(SPACE_1)]
            .spacing(SPACE_2)
            .width(Length::Fill)
            .into()
    }
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

    scrollable(grid)
        .direction(scrollable::Direction::Vertical(
            scrollable::Scrollbar::new().width(5).scroller_width(5),
        ))
        .style(move |_, status| theme.subtle_scrollbar_style(status))
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
    let day_all: Vec<&CalendarOccurrence> =
        occurrences.iter().filter(|occ| occ.date == date).collect();
    let total_items = day_all.len();
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

    let mut top_bar = row![day_badge].align_y(Alignment::Center);
    if total_items > 2 {
        top_bar = top_bar.push(
            container(shaped_text(
                format!("+{}", total_items - 2),
                11.0,
                FONT_MONO,
                theme.text_muted,
            ))
            .align_x(Alignment::End)
            .width(Length::Fill),
        );
    }

    let mut list = column![top_bar].spacing(4.0);
    for occ in day_all.into_iter().take(2) {
        let cat = state.occurrence_color(occ);
        list = list.push(
            row![
                color_dot(cat.solid(theme)),
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
        .on_press(map_msg(CalendarMessage::OpenDayPeek(date)))
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

fn view_agenda_list<'a, Message: Clone + 'a>(
    state: &'a CalendarState,
    occurrences: &[CalendarOccurrence],
    theme: EstudyTheme,
    map_msg: &impl Fn(CalendarMessage) -> Message,
) -> Element<'a, Message> {
    if occurrences.is_empty() {
        return empty_placeholder(
            Icon::Calendar,
            "Agenda livre neste mês",
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
        let cat = state.occurrence_color(occ);
        let tint = cat.solid(theme);
        let cat_label = AGENDA_CATEGORIES[state.occurrence_category_idx(occ)].0;

        let mut row_items = row![
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
            status_badge(cat_label, tint, theme),
        ]
        .spacing(SPACE_3)
        .align_y(Alignment::Center);

        if !occ.is_task_deadline {
            row_items = row_items
                .push(
                    button(icon_svg(Icon::Pencil, 15.0, theme.text_muted))
                        .on_press(map_msg(CalendarMessage::EditEvent(occ.source_id.clone())))
                        .padding(6.0)
                        .style(move |_, status| theme.subtle_button_style(status)),
                )
                .push(
                    button(icon_svg(Icon::Trash2, 15.0, theme.danger))
                        .on_press(map_msg(CalendarMessage::DeleteEvent(occ.source_id.clone())))
                        .padding(6.0)
                        .style(move |_, status| theme.subtle_button_style(status)),
                );
        }

        list = list.push(card(row_items, theme));
    }

    scrollable(list)
        .direction(scrollable::Direction::Vertical(
            scrollable::Scrollbar::new().width(5).scroller_width(5),
        ))
        .style(move |_, status| theme.subtle_scrollbar_style(status))
        .height(Length::Fill)
        .into()
}

fn view_agenda_footer_legend<'a, Message: Clone + 'a>(
    state: &'a CalendarState,
    content_width: f32,
    theme: EstudyTheme,
    map_msg: &impl Fn(CalendarMessage) -> Message,
) -> Element<'a, Message> {
    let mut left = row![].spacing(SPACE_2);
    for (idx, (name, _, cat)) in AGENDA_CATEGORIES.iter().enumerate() {
        let active = state.active_category_filter == Some(idx);
        let c = cat.solid(theme);
        left = left.push(
            button(
                row![
                    color_dot(if active { theme.on_accent } else { c }),
                    shaped_text(
                        *name,
                        FONT_SIZE_XS,
                        FONT_SANS_MEDIUM,
                        if active {
                            theme.on_accent
                        } else {
                            theme.text_muted
                        },
                    ),
                ]
                .spacing(5.0)
                .align_y(Alignment::Center),
            )
            .on_press(map_msg(CalendarMessage::ToggleCategoryFilter(Some(idx))))
            .padding([4.0, 9.0])
            .style(move |_, status| {
                if active {
                    theme.pill_button_style(true, status)
                } else {
                    theme.subtle_button_style(status)
                }
            }),
        );
    }

    if content_width >= 820.0 {
        row![
            left.wrap().vertical_spacing(SPACE_1),
            container(
                text("Clique num dia para ver, editar ou criar um compromisso")
                    .size(FONT_SIZE_SM)
                    .font(FONT_SERIF_ITALIC)
                    .color(theme.text_muted)
                    .shaping(text::Shaping::Advanced),
            )
            .align_x(Alignment::End)
            .width(Length::Fill),
        ]
        .spacing(SPACE_3)
        .align_y(Alignment::Center)
        .width(Length::Fill)
        .into()
    } else {
        left.wrap().vertical_spacing(SPACE_1).into()
    }
}

fn view_agenda_day_peek_modal<'a, Message: Clone + 'a>(
    state: &'a CalendarState,
    occurrences: &[CalendarOccurrence],
    theme: EstudyTheme,
    map_msg: impl Fn(CalendarMessage) -> Message + Clone + 'a,
) -> Element<'a, Message> {
    let date = state.cursor_date;
    let weekday_title = weekday_full_pt_from_date(date);
    let subtitle = format!(
        "{}, {} de {}",
        weekday_title.to_lowercase(),
        date.day(),
        month_name_pt(date.month())
    );

    let close_btn = button(icon_svg(Icon::X, 16.0, theme.text_muted))
        .on_press(map_msg(CalendarMessage::CloseDayPeek))
        .padding(SPACE_1)
        .style(move |_, status| theme.subtle_button_style(status));

    let header = row![
        column![
            text(weekday_title)
                .size(FONT_SIZE_2XL)
                .font(FONT_SERIF)
                .color(theme.text_primary)
                .shaping(text::Shaping::Advanced),
            text(subtitle)
                .size(FONT_SIZE_BASE)
                .font(FONT_SERIF_ITALIC)
                .color(theme.text_muted)
                .shaping(text::Shaping::Advanced),
        ]
        .spacing(2.0),
        container(close_btn)
            .align_x(Alignment::End)
            .width(Length::Fill),
    ]
    .align_y(Alignment::Start);

    let day_items: Vec<&CalendarOccurrence> =
        occurrences.iter().filter(|o| o.date == date).collect();

    let mut list = column![].spacing(SPACE_2);
    if day_items.is_empty() {
        list = list.push(
            soft_surface(
                shaped_text(
                    "Nenhum compromisso ou prazo marcado para este dia.",
                    FONT_SIZE_BASE,
                    FONT_SANS,
                    theme.text_muted,
                ),
                theme,
            )
            .padding(SPACE_4)
            .width(Length::Fill),
        );
    } else {
        for occ in day_items {
            list = list.push(view_agenda_day_peek_item(state, occ, theme, &map_msg));
        }
    }

    let add_btn = button(
        container(
            row![
                icon_svg(Icon::Plus, 16.0, theme.on_accent),
                shaped_text(
                    "Novo compromisso neste dia",
                    FONT_SIZE_BASE,
                    FONT_SANS_SEMIBOLD,
                    theme.on_accent,
                ),
            ]
            .spacing(SPACE_2)
            .align_y(Alignment::Center),
        )
        .center_x(Length::Fill)
        .width(Length::Fill),
    )
    .on_press(map_msg(CalendarMessage::ToggleCreateForm))
    .width(Length::Fill)
    .padding([10.0, SPACE_3])
    .style(move |_, status| theme.primary_button_style(status));

    modal_surface(
        column![
            header,
            hairline_divider(theme),
            scrollable(list).height(Length::Shrink),
            add_btn,
        ]
        .spacing(SPACE_3),
        460.0,
        theme,
    )
}

fn view_agenda_day_peek_item<'a, Message: Clone + 'a>(
    state: &'a CalendarState,
    occ: &CalendarOccurrence,
    theme: EstudyTheme,
    map_msg: &impl Fn(CalendarMessage) -> Message,
) -> Element<'a, Message> {
    let cat = state.occurrence_color(occ);
    let time_label = if occ.all_day {
        "Dia todo".to_string()
    } else {
        format!(
            "{:02}:{:02} – {:02}:{:02}",
            occ.start_minute / 60,
            occ.start_minute % 60,
            occ.end_minute / 60,
            occ.end_minute % 60
        )
    };
    let mut row_el = row![
        color_dot(cat.solid(theme)),
        column![
            shaped_text(
                occ.title.clone(),
                FONT_SIZE_MD,
                FONT_SANS_SEMIBOLD,
                theme.text_primary,
            ),
            shaped_text(time_label, FONT_SIZE_XS, FONT_MONO, theme.text_muted),
        ]
        .spacing(2.0)
        .width(Length::Fill),
    ]
    .spacing(SPACE_2)
    .align_y(Alignment::Center);

    if !occ.is_task_deadline {
        row_el = row_el
            .push(
                button(icon_svg(Icon::Pencil, 14.0, theme.text_muted))
                    .on_press(map_msg(CalendarMessage::EditEvent(occ.source_id.clone())))
                    .padding(5.0)
                    .style(move |_, status| theme.subtle_button_style(status)),
            )
            .push(
                button(icon_svg(Icon::Trash2, 14.0, theme.danger))
                    .on_press(map_msg(CalendarMessage::DeleteEvent(occ.source_id.clone())))
                    .padding(5.0)
                    .style(move |_, status| theme.subtle_button_style(status)),
            );
    }

    soft_surface(row_el, theme)
        .padding([SPACE_2, SPACE_3])
        .into()
}

fn view_create_event_modal<'a, Message: Clone + 'a>(
    state: &'a CalendarState,
    theme: EstudyTheme,
    map_msg: impl Fn(CalendarMessage) -> Message + Clone + 'a,
) -> Element<'a, Message> {
    let is_editing = state.create_draft.editing_event_id.is_some();
    let modal_title = if is_editing {
        "Editar compromisso"
    } else {
        "Novo compromisso"
    };

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

    let all_day_btn = pill_chip(
        "Dia todo",
        None,
        Some(Icon::Clock),
        state.create_draft.all_day,
        map_msg(CalendarMessage::ToggleNewAllDay),
        theme,
    );

    let time_row: Element<'a, Message> = if state.create_draft.all_day {
        row![all_day_btn].into()
    } else {
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

        row![start_input, end_input, all_day_btn]
            .spacing(SPACE_2)
            .align_y(Alignment::Center)
            .into()
    };

    let mut cat_row = row![].spacing(SPACE_1);
    for (idx, (name, _, cat)) in AGENDA_CATEGORIES.iter().enumerate() {
        cat_row = cat_row.push(pill_chip(
            *name,
            Some(cat.solid(theme)),
            None,
            state.create_draft.category_idx == idx,
            map_msg(CalendarMessage::SetNewCategory(idx)),
            theme,
        ));
    }

    let rec_row = view_recurrence_selector(state.create_draft.recurrence, theme, &map_msg);
    let footer_actions = view_event_modal_actions(theme, &map_msg);

    let body = column![
        row![
            shaped_text(
                format!(
                    "{modal_title} · {:02}/{:02}/{}",
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
        time_row,
        cat_row.wrap().vertical_spacing(SPACE_1),
        rec_row,
        footer_actions,
    ]
    .spacing(SPACE_3);

    modal_surface(body, 480.0, theme)
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
