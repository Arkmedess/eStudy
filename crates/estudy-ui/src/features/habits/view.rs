//! Renderização do módulo de Hábitos (Vídeo 2 `00:06`: cabeçalho editorial, matriz semanal com `+ Novo hábito` integrado na base e resumo mensal).

use chrono::{Datelike, Duration, NaiveDate};
use estudy_domain::days_in_month;
use estudy_ui_kit::widgets::{
    card, color_dot, editorial_page_header, empty_placeholder, hairline_divider, icon_svg,
    progress_track, shaped_text, soft_surface,
};
use estudy_ui_kit::{
    CategoryColor, EstudyTheme, FONT_MONO, FONT_MONO_MEDIUM, FONT_SANS, FONT_SANS_MEDIUM,
    FONT_SANS_SEMIBOLD, FONT_SERIF, FONT_SIZE_2XL, FONT_SIZE_BASE, FONT_SIZE_LG, FONT_SIZE_MD,
    FONT_SIZE_SM, FONT_SIZE_XS, Icon, RADIUS_CONTROL, RADIUS_PILL, SPACE_1, SPACE_2, SPACE_3,
    SPACE_4, SPACE_5, SPACE_6,
};
use iced::widget::{button, column, container, row, scrollable, text, text_input};
use iced::{Alignment, Border, Element, Length};

use super::state::{HabitsMessage, HabitsState};
use crate::features::projects::modals::month_name_pt;

/// Renderiza a tela completa de Hábitos no padrão editorial limpo.
pub fn view_habits<'a, Message: Clone + 'a>(
    state: &'a HabitsState,
    today: NaiveDate,
    theme: EstudyTheme,
    map_msg: impl Fn(HabitsMessage) -> Message + Clone + 'a,
) -> Element<'a, Message> {
    let (done_today, total_habits) = state.today_progress(today);
    let summary_line = column![
        shaped_text(
            format!("Hoje, {} de {}", today.day(), month_name_pt(today.month())),
            FONT_SIZE_XS,
            FONT_SANS_MEDIUM,
            theme.text_muted,
        ),
        text(format!("{done_today} de {total_habits} feitos hoje"))
            .size(FONT_SIZE_LG)
            .font(FONT_SERIF)
            .color(theme.text_primary)
            .shaping(text::Shaping::Advanced),
    ]
    .spacing(2.0);

    let header = editorial_page_header(
        "ESTUDY  /  HÁBITOS",
        "Hábitos",
        "O que você repete",
        None,
        theme,
    );

    let weekly_card = view_weekly_grid_card(state, today, theme, map_msg.clone());
    let monthly_row = row![
        view_monthly_bars_card(state, today, theme),
        view_monthly_calendar_card(state, today, theme, &map_msg),
    ]
    .spacing(SPACE_4)
    .width(Length::Fill);

    scrollable(
        container(
            container(
                column![header, summary_line, weekly_card, monthly_row]
                    .spacing(SPACE_4)
                    .width(Length::Fill),
            )
            .max_width(1360.0)
            .width(Length::Fill),
        )
        .padding([SPACE_5, SPACE_6])
        .center_x(Length::Fill)
        .width(Length::Fill),
    )
    .height(Length::Fill)
    .into()
}

fn view_weekly_grid_card<'a, Message: Clone + 'a>(
    state: &'a HabitsState,
    today: NaiveDate,
    theme: EstudyTheme,
    map_msg: impl Fn(HabitsMessage) -> Message + Clone + 'a,
) -> Element<'a, Message> {
    let week_end = state.week_start + Duration::days(6);
    let range_label = format!(
        "{} a {} de {}",
        state.week_start.day(),
        week_end.day(),
        month_name_pt(week_end.month())
    );

    let header = row![
        text("Semana")
            .size(FONT_SIZE_2XL)
            .font(FONT_SERIF)
            .color(theme.text_primary)
            .shaping(text::Shaping::Advanced),
        container(
            row![
                button(icon_svg(Icon::ChevronLeft, 16.0, theme.text_primary))
                    .on_press(map_msg(HabitsMessage::ShiftWeek(-1)))
                    .padding(5.0)
                    .style(move |_, status| theme.subtle_button_style(status)),
                shaped_text(
                    range_label,
                    FONT_SIZE_BASE,
                    FONT_SANS_MEDIUM,
                    theme.text_muted
                ),
                button(icon_svg(Icon::ChevronRight, 16.0, theme.text_primary))
                    .on_press(map_msg(HabitsMessage::ShiftWeek(1)))
                    .padding(5.0)
                    .style(move |_, status| theme.subtle_button_style(status)),
            ]
            .spacing(SPACE_2)
            .align_y(Alignment::Center),
        )
        .align_x(Alignment::End)
        .width(Length::Fill),
    ]
    .align_y(Alignment::Center);

    let create_footer = view_inline_create_footer(state, theme, map_msg.clone());

    if state.habits.is_empty() {
        return card(
            column![
                header,
                hairline_divider(theme),
                empty_placeholder(
                    Icon::Flame,
                    "Nenhum hábito cadastrado ainda",
                    "Adicione seu primeiro hábito abaixo para acompanhar sua constância semanal.",
                    None,
                    theme,
                ),
                create_footer,
            ]
            .spacing(SPACE_3),
            theme,
        )
        .into();
    }

    let days_header = view_weekly_days_header(state.week_start, today, theme);
    let mut rows = column![days_header, hairline_divider(theme)].spacing(SPACE_2);
    for habit in &state.habits {
        rows = rows.push(view_weekly_habit_row(
            habit,
            state.week_start,
            today,
            theme,
            &map_msg,
        ));
    }

    card(
        column![header, rows, hairline_divider(theme), create_footer].spacing(SPACE_3),
        theme,
    )
    .into()
}

fn view_weekly_days_header<'a, Message: 'a>(
    week_start: NaiveDate,
    today: NaiveDate,
    theme: EstudyTheme,
) -> Element<'a, Message> {
    let day_names = ["SEG", "TER", "QUA", "QUI", "SEX", "SÁB", "DOM"];
    let mut days_header = row![container(text("")).width(Length::FillPortion(4))]
        .spacing(SPACE_2)
        .align_y(Alignment::Center);

    for (idx, short) in day_names.iter().enumerate() {
        let d = week_start + Duration::days(i64::try_from(idx).unwrap_or(0));
        let is_today = d == today;
        let col_color = if is_today {
            CategoryColor::Terracotta.solid(theme)
        } else {
            theme.text_muted
        };
        days_header = days_header.push(
            column![
                shaped_text(*short, 11.0, FONT_SANS_SEMIBOLD, col_color),
                shaped_text(
                    format!("{:02}", d.day()),
                    FONT_SIZE_SM,
                    FONT_MONO,
                    if is_today {
                        col_color
                    } else {
                        theme.text_primary
                    },
                ),
            ]
            .spacing(1.0)
            .align_x(Alignment::Center)
            .width(Length::FillPortion(1)),
        );
    }
    days_header.into()
}

fn view_inline_create_footer<'a, Message: Clone + 'a>(
    state: &'a HabitsState,
    theme: EstudyTheme,
    map_msg: impl Fn(HabitsMessage) -> Message + Clone + 'a,
) -> Element<'a, Message> {
    let on_title = map_msg.clone();
    let input = text_input(
        "+ Novo hábito (ex.: Leitura 20 min, Estudar Inglês)...",
        &state.new_title,
    )
    .on_input(move |val| on_title(HabitsMessage::NewTitleChanged(val)))
    .on_submit(map_msg(HabitsMessage::CreateHabit))
    .padding([8.0, 14.0])
    .size(FONT_SIZE_BASE)
    .width(Length::Fill)
    .style(move |_, status| theme.soft_input_style(status));

    let btn = button(
        row![
            icon_svg(Icon::Plus, 16.0, theme.text_primary),
            shaped_text(
                "Novo hábito",
                FONT_SIZE_BASE,
                FONT_SANS_MEDIUM,
                theme.text_primary
            ),
        ]
        .spacing(6.0)
        .align_y(Alignment::Center),
    )
    .on_press(map_msg(HabitsMessage::CreateHabit))
    .padding([8.0, 16.0])
    .style(move |_, status| theme.ghost_button_style(status));

    row![input, btn]
        .spacing(SPACE_2)
        .align_y(Alignment::Center)
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
            shaped_text(
                habit.title.as_str(),
                FONT_SIZE_MD,
                FONT_SANS_SEMIBOLD,
                theme.text_primary,
            ),
            shaped_text(
                format!("{} · {}d seguidos", habit.subtitle, streak),
                FONT_SIZE_XS,
                FONT_SANS,
                theme.text_muted,
            ),
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
                        shaped_text(
                            habit.title.as_str(),
                            FONT_SIZE_MD,
                            FONT_SANS_SEMIBOLD,
                            theme.text_primary,
                        ),
                    ]
                    .spacing(SPACE_2)
                    .align_y(Alignment::Center),
                    container(shaped_text(
                        format!("{pct}%"),
                        FONT_SIZE_BASE,
                        FONT_MONO_MEDIUM,
                        cat.solid(theme),
                    ))
                    .align_x(Alignment::End)
                    .width(Length::Fill),
                ]
                .align_y(Alignment::Center),
                progress_track(ratio, cat.solid(theme), theme),
                shaped_text(
                    format!("{done_count} dias concluídos no mês"),
                    FONT_SIZE_XS,
                    FONT_SANS,
                    theme.text_muted,
                ),
            ]
            .spacing(SPACE_1),
        );
    }

    card(
        column![
            row![
                text("O mês")
                    .size(FONT_SIZE_2XL)
                    .font(FONT_SERIF)
                    .color(theme.text_primary)
                    .shaping(text::Shaping::Advanced),
                container(
                    text(format!("{overall}%"))
                        .size(30.0)
                        .font(FONT_SERIF)
                        .color(theme.text_primary)
                        .shaping(text::Shaping::Advanced),
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
    let raw_m = month_name_pt(state.view_month);
    let mut chars = raw_m.chars();
    let cap_m = chars.next().map_or_else(String::new, |c| {
        format!("{}{}", c.to_uppercase(), chars.as_str())
    });

    let header = row![
        button(icon_svg(Icon::ChevronLeft, 16.0, theme.text_primary))
            .on_press(map_msg(HabitsMessage::ShiftMonth(-1)))
            .padding(4.0)
            .style(move |_, status| theme.subtle_button_style(status)),
        text(format!("{cap_m} de {}", state.view_year))
            .size(FONT_SIZE_2XL)
            .font(FONT_SERIF)
            .color(theme.text_primary)
            .shaping(text::Shaping::Advanced),
        button(icon_svg(Icon::ChevronRight, 16.0, theme.text_primary))
            .on_press(map_msg(HabitsMessage::ShiftMonth(1)))
            .padding(4.0)
            .style(move |_, status| theme.subtle_button_style(status)),
    ]
    .spacing(SPACE_2)
    .align_y(Alignment::Center);

    let total_days = days_in_month(state.view_year, state.view_month);
    let first = NaiveDate::from_ymd_opt(state.view_year, state.view_month, 1);
    let offset = first.map_or(0, |d| d.weekday().num_days_from_sunday() as usize);
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
                    .height(Length::Fixed(52.0))
                    .width(Length::FillPortion(1)),
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
            .height(Length::Fixed(52.0))
            .width(Length::FillPortion(1))
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
            shaped_text(
                format!("{}", date.day()),
                FONT_SIZE_XS,
                FONT_MONO,
                if is_today {
                    CategoryColor::Terracotta.solid(theme)
                } else {
                    theme.text_muted
                },
            ),
            dots,
        ]
        .spacing(4.0),
        theme,
    )
    .padding(SPACE_2)
    .height(Length::Fixed(52.0))
    .width(Length::FillPortion(1))
    .style(move |_| container::Style {
        background: Some(theme.surface_soft.into()),
        border: Border {
            color: if is_today {
                CategoryColor::Terracotta.solid(theme)
            } else {
                theme.border_subtle
            },
            width: if is_today { 1.5 } else { 1.0 },
            radius: RADIUS_CONTROL.into(),
        },
        ..container::Style::default()
    })
    .into()
}
