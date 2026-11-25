//! Renderização do módulo de Hábitos (grade semanal + barras mensais + calendário de constância).

use chrono::{Datelike, Duration, NaiveDate};
use estudy_domain::days_in_month;
use estudy_ui_kit::widgets::{
    card, color_dot, editorial_heading, empty_placeholder, hairline_divider, icon_svg,
    progress_track, section_header, soft_surface,
};
use estudy_ui_kit::{
    CategoryColor, EstudyTheme, FONT_MONO, FONT_MONO_MEDIUM, FONT_SANS_MEDIUM, FONT_SANS_SEMIBOLD,
    Icon, RADIUS_CONTROL, RADIUS_PILL, SPACE_1, SPACE_2, SPACE_3, SPACE_4, SPACE_5, SPACE_6,
};
use iced::widget::{button, column, container, row, scrollable, text, text_input};
use iced::{Alignment, Border, Element, Length};

use super::state::{HabitsMessage, HabitsState};

/// Renderiza a tela completa de Hábitos.
pub fn view_habits<'a, Message: Clone + 'a>(
    state: &'a HabitsState,
    today: NaiveDate,
    theme: EstudyTheme,
    map_msg: impl Fn(HabitsMessage) -> Message + Clone + 'a,
) -> Element<'a, Message> {
    let (done_today, total_habits) = state.today_progress(today);
    let subtitle = format!("{done_today} de {total_habits} feitos hoje · consistência sem pressa");

    let header = row![
        editorial_heading("Hábitos", subtitle, theme),
        container(view_quick_create_bar(state, theme, map_msg.clone()))
            .align_x(Alignment::End)
            .width(Length::Fill),
    ]
    .spacing(SPACE_4)
    .align_y(Alignment::Center);

    let weekly_card = view_weekly_grid_card(state, today, theme, &map_msg);
    let monthly_row = row![
        view_monthly_bars_card(state, today, theme),
        view_monthly_calendar_card(state, today, theme, &map_msg),
    ]
    .spacing(SPACE_4)
    .width(Length::Fill);

    scrollable(
        container(
            container(
                column![header, weekly_card, monthly_row]
                    .spacing(SPACE_5)
                    .width(Length::Fill),
            )
            .max_width(1320.0)
            .width(Length::Fill),
        )
        .padding([SPACE_5, SPACE_6])
        .center_x(Length::Fill)
        .width(Length::Fill),
    )
    .height(Length::Fill)
    .into()
}

fn view_quick_create_bar<'a, Message: Clone + 'a>(
    state: &'a HabitsState,
    theme: EstudyTheme,
    map_msg: impl Fn(HabitsMessage) -> Message + Clone + 'a,
) -> Element<'a, Message> {
    let on_title = map_msg.clone();
    let title_input = text_input("Novo hábito (ex.: Meditar 10m)...", &state.new_title)
        .on_input(move |val| on_title(HabitsMessage::NewTitleChanged(val)))
        .on_submit(map_msg(HabitsMessage::CreateHabit))
        .padding([6.0, 12.0])
        .size(12)
        .width(Length::Fixed(230.0))
        .style(move |_, status| theme.soft_input_style(status));

    let add_btn = button(
        row![
            icon_svg(Icon::Plus, 13.0, theme.on_accent),
            text("Adicionar")
                .size(12)
                .font(FONT_SANS_MEDIUM)
                .color(theme.on_accent),
        ]
        .spacing(4.0)
        .align_y(Alignment::Center),
    )
    .on_press(map_msg(HabitsMessage::CreateHabit))
    .padding([6.0, 14.0])
    .style(move |_, status| theme.pill_button_style(true, status));

    row![title_input, add_btn]
        .spacing(SPACE_2)
        .align_y(Alignment::Center)
        .into()
}

fn view_weekly_grid_card<'a, Message: Clone + 'a>(
    state: &'a HabitsState,
    today: NaiveDate,
    theme: EstudyTheme,
    map_msg: &impl Fn(HabitsMessage) -> Message,
) -> Element<'a, Message> {
    let week_end = state.week_start + Duration::days(6);
    let range_label = format!(
        "{:02}/{:02} a {:02}/{:02}",
        state.week_start.day(),
        state.week_start.month(),
        week_end.day(),
        week_end.month()
    );

    let header = row![
        section_header(Icon::Flame, "Semana", theme),
        container(
            row![
                button(icon_svg(Icon::ChevronLeft, 13.0, theme.text_primary))
                    .on_press(map_msg(HabitsMessage::ShiftWeek(-1)))
                    .padding(4.0)
                    .style(move |_, status| theme.ghost_button_style(status)),
                text(range_label)
                    .size(12)
                    .font(FONT_MONO)
                    .color(theme.text_muted),
                button(icon_svg(Icon::ChevronRight, 13.0, theme.text_primary))
                    .on_press(map_msg(HabitsMessage::ShiftWeek(1)))
                    .padding(4.0)
                    .style(move |_, status| theme.ghost_button_style(status)),
            ]
            .spacing(SPACE_2)
            .align_y(Alignment::Center),
        )
        .align_x(Alignment::End)
        .width(Length::Fill),
    ]
    .align_y(Alignment::Center);

    if state.habits.is_empty() {
        return card(
            column![
                header,
                hairline_divider(theme),
                empty_placeholder(
                    Icon::Flame,
                    "Nenhum hábito cadastrado ainda",
                    "Adicione seu primeiro hábito acima para acompanhar sua constância semanal.",
                    None,
                    theme,
                ),
            ]
            .spacing(SPACE_3),
            theme,
        )
        .into();
    }

    let day_names = ["SEG", "TER", "QUA", "QUI", "SEX", "SÁB", "DOM"];
    let mut days_header = row![
        container(text("Hábito").size(11).color(theme.text_muted)).width(Length::FillPortion(4))
    ]
    .spacing(SPACE_2)
    .align_y(Alignment::Center);

    for (idx, short) in day_names.iter().enumerate() {
        let d = state.week_start + Duration::days(i64::try_from(idx).unwrap_or(0));
        let is_today = d == today;
        days_header = days_header.push(
            column![
                text(*short)
                    .size(10)
                    .font(FONT_SANS_SEMIBOLD)
                    .color(if is_today {
                        theme.accent
                    } else {
                        theme.text_muted
                    }),
                text(format!("{:02}", d.day()))
                    .size(11)
                    .font(FONT_MONO)
                    .color(if is_today {
                        theme.accent
                    } else {
                        theme.text_primary
                    }),
            ]
            .spacing(1.0)
            .align_x(Alignment::Center)
            .width(Length::FillPortion(1)),
        );
    }

    let mut rows = column![days_header, hairline_divider(theme)].spacing(SPACE_2);
    for habit in &state.habits {
        rows = rows.push(view_weekly_habit_row(
            habit,
            state.week_start,
            today,
            theme,
            map_msg,
        ));
    }

    card(
        column![header, hairline_divider(theme), rows].spacing(SPACE_3),
        theme,
    )
    .into()
}

fn view_weekly_habit_row<'a, Message: Clone + 'a>(
    habit: &'a estudy_domain::Habit,
    week_start: NaiveDate,
    today: NaiveDate,
    theme: EstudyTheme,
    map_msg: &impl Fn(HabitsMessage) -> Message,
) -> Element<'a, Message> {
    let cat = CategoryColor::from_index(habit.color_index);
    let streak = habit.current_streak(today);

    let label_col = row![
        color_dot(cat.solid(theme)),
        column![
            text(&habit.title)
                .size(13)
                .font(FONT_SANS_SEMIBOLD)
                .color(theme.text_primary),
            text(format!("{} · {}d seguidos", habit.subtitle, streak))
                .size(11)
                .color(theme.text_muted),
        ]
        .spacing(1.0),
    ]
    .spacing(SPACE_2)
    .align_y(Alignment::Center)
    .width(Length::FillPortion(4));

    let mut line = row![label_col].spacing(SPACE_2).align_y(Alignment::Center);

    for offset in 0..7_i64 {
        let d = week_start + Duration::days(offset);
        let done = habit.is_done_on(d);
        let solid = cat.solid(theme);
        let border_c = cat.border(theme);
        let check_btn = button(if done {
            icon_svg(Icon::CheckCircle2, 16.0, theme.on_accent)
        } else {
            container(text("").size(1))
                .width(Length::Fixed(16.0))
                .height(Length::Fixed(16.0))
                .into()
        })
        .on_press(map_msg(HabitsMessage::ToggleHabitDate {
            habit_id: habit.id.clone(),
            date: d,
        }))
        .width(Length::Fixed(30.0))
        .height(Length::Fixed(30.0))
        .padding(0.0)
        .style(move |_, status| button::Style {
            background: if done {
                Some(solid.into())
            } else if matches!(status, button::Status::Hovered) {
                Some(theme.card_hover.into())
            } else {
                Some(theme.surface_soft.into())
            },
            border: Border {
                color: if done { solid } else { border_c },
                width: 1.0,
                radius: RADIUS_PILL.into(),
            },
            ..button::Style::default()
        });

        line = line.push(
            container(check_btn)
                .width(Length::FillPortion(1))
                .center_x(Length::Fill),
        );
    }

    line.into()
}

fn view_monthly_bars_card<'a, Message: 'a>(
    state: &'a HabitsState,
    today: NaiveDate,
    theme: EstudyTheme,
) -> Element<'a, Message> {
    let overall = state.monthly_overall_pct(today);
    let mut bars = column![].spacing(SPACE_3);

    for habit in &state.habits {
        let cat = CategoryColor::from_index(habit.color_index);
        let pct = habit.monthly_adherence_pct(state.view_year, state.view_month, today);
        let done_count = habit.completions_in_month(state.view_year, state.view_month);
        let ratio = f32::from(pct) / 100.0;

        bars = bars.push(
            column![
                row![
                    row![
                        color_dot(cat.solid(theme)),
                        text(&habit.title)
                            .size(13)
                            .font(FONT_SANS_SEMIBOLD)
                            .color(theme.text_primary),
                    ]
                    .spacing(SPACE_2)
                    .align_y(Alignment::Center),
                    container(
                        text(format!("{pct}%"))
                            .size(12)
                            .font(FONT_MONO_MEDIUM)
                            .color(cat.solid(theme)),
                    )
                    .align_x(Alignment::End)
                    .width(Length::Fill),
                ]
                .align_y(Alignment::Center),
                progress_track(ratio, cat.solid(theme), theme),
                text(format!("{done_count} dias concluídos neste mês"))
                    .size(11)
                    .color(theme.text_muted),
            ]
            .spacing(SPACE_1),
        );
    }

    card(
        column![
            row![
                section_header(Icon::TrendingUp, "O mês", theme),
                container(
                    text(format!("{overall}%"))
                        .size(22)
                        .font(FONT_MONO_MEDIUM)
                        .color(theme.accent),
                )
                .align_x(Alignment::End)
                .width(Length::Fill),
            ]
            .align_y(Alignment::Center),
            hairline_divider(theme),
            bars,
        ]
        .spacing(SPACE_3),
        theme,
    )
    .width(Length::FillPortion(5))
    .into()
}

fn view_monthly_calendar_card<'a, Message: Clone + 'a>(
    state: &'a HabitsState,
    today: NaiveDate,
    theme: EstudyTheme,
    map_msg: &impl Fn(HabitsMessage) -> Message,
) -> Element<'a, Message> {
    let header = row![
        section_header(
            Icon::Calendar,
            format!("{:02}/{}", state.view_month, state.view_year),
            theme,
        ),
        container(
            row![
                button(icon_svg(Icon::ChevronLeft, 13.0, theme.text_primary))
                    .on_press(map_msg(HabitsMessage::ShiftMonth(-1)))
                    .padding(4.0)
                    .style(move |_, status| theme.ghost_button_style(status)),
                button(icon_svg(Icon::ChevronRight, 13.0, theme.text_primary))
                    .on_press(map_msg(HabitsMessage::ShiftMonth(1)))
                    .padding(4.0)
                    .style(move |_, status| theme.ghost_button_style(status)),
            ]
            .spacing(SPACE_1),
        )
        .align_x(Alignment::End)
        .width(Length::Fill),
    ]
    .align_y(Alignment::Center);

    let total_days = days_in_month(state.view_year, state.view_month);
    let first = NaiveDate::from_ymd_opt(state.view_year, state.view_month, 1);
    let offset = first.map_or(0, |d| d.weekday().num_days_from_monday() as usize);
    let mut cells: Vec<Option<NaiveDate>> = vec![None; offset];
    for day in 1..=total_days {
        cells.push(NaiveDate::from_ymd_opt(
            state.view_year,
            state.view_month,
            day,
        ));
    }

    let mut grid = column![].spacing(SPACE_1);
    for week in cells.chunks(7) {
        let mut line = row![].spacing(SPACE_1).width(Length::Fill);
        for slot in week {
            line = line.push(view_month_day_cell(*slot, &state.habits, today, theme));
        }
        for _ in week.len()..7 {
            line = line.push(
                container(text(""))
                    .height(Length::Fixed(48.0))
                    .width(Length::Fill),
            );
        }
        grid = grid.push(line);
    }

    card(
        column![header, hairline_divider(theme), grid].spacing(SPACE_3),
        theme,
    )
    .width(Length::FillPortion(7))
    .into()
}

fn view_month_day_cell<'a, Message: 'a>(
    slot: Option<NaiveDate>,
    habits: &'a [estudy_domain::Habit],
    today: NaiveDate,
    theme: EstudyTheme,
) -> Element<'a, Message> {
    let Some(date) = slot else {
        return container(text(""))
            .height(Length::Fixed(48.0))
            .width(Length::Fill)
            .into();
    };
    let mut dots = row![].spacing(3.0).align_y(Alignment::Center);
    for habit in habits {
        if habit.is_done_on(date) {
            let c = CategoryColor::from_index(habit.color_index).solid(theme);
            dots = dots.push(color_dot(c));
        }
    }
    let is_today = date == today;
    soft_surface(
        column![
            text(format!("{}", date.day()))
                .size(11)
                .font(FONT_MONO)
                .color(if is_today {
                    theme.accent
                } else {
                    theme.text_muted
                }),
            dots,
        ]
        .spacing(4.0),
        theme,
    )
    .padding(SPACE_2)
    .height(Length::Fixed(48.0))
    .width(Length::Fill)
    .style(move |_| container::Style {
        background: Some(theme.surface_soft.into()),
        border: Border {
            color: if is_today {
                theme.accent
            } else {
                theme.border_subtle
            },
            width: 1.0,
            radius: RADIUS_CONTROL.into(),
        },
        ..container::Style::default()
    })
    .into()
}
