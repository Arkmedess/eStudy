//! Gaveta flutuante "Hoje" (faixa semanal, cápsulas proporcionais, janelas livres e linha Agora).

use chrono::{DateTime, Datelike, Duration, Utc};
use estudy_domain::{CalendarEvent, Task, TimelineSlot, TodayTimeline, build_today_timeline};
use estudy_ui_kit::widgets::{
    color_dot, floating_drawer, hairline_divider, icon_svg, section_header, soft_surface,
};
use estudy_ui_kit::{
    CategoryColor, EstudyTheme, FONT_MONO, FONT_MONO_MEDIUM, FONT_SANS_MEDIUM, FONT_SANS_SEMIBOLD,
    FONT_SERIF_MEDIUM, Icon, RADIUS_CONTROL, RADIUS_PILL, SPACE_1, SPACE_2, SPACE_3,
};
use iced::widget::{button, column, container, row, scrollable, text};
use iced::{Alignment, Border, Element, Length};

use super::state::CalendarMessage;

/// Renderiza a Gaveta Flutuante "Hoje" sobreposta à direita da tela atual.
pub fn view_today_drawer<'a, Message: Clone + 'a>(
    events: &'a [CalendarEvent],
    tasks: &'a [Task],
    now_utc: DateTime<Utc>,
    theme: EstudyTheme,
    map_msg: impl Fn(CalendarMessage) -> Message + 'a,
) -> Element<'a, Message> {
    let timeline = build_today_timeline(events, tasks, now_utc);
    let today = now_utc.date_naive();

    let header = row![
        column![
            text("Hoje")
                .size(22)
                .font(FONT_SERIF_MEDIUM)
                .color(theme.text_primary),
            text(format!(
                "{}, {:02} de {}",
                weekday_pt(today.weekday()),
                today.day(),
                month_pt(today.month())
            ))
            .size(12)
            .color(theme.text_muted),
        ]
        .spacing(2.0)
        .width(Length::Fill),
        button(icon_svg(Icon::X, 15.0, theme.text_muted))
            .on_press(map_msg(CalendarMessage::CloseTodayDrawer))
            .padding(SPACE_1)
            .style(move |_, status| theme.ghost_button_style(status)),
    ]
    .align_y(Alignment::Center);

    let week_strip = view_week_strip(today, theme);
    let unscheduled_section = view_unscheduled_section(&timeline, theme);
    let timeline_section = view_timeline_slots(&timeline, theme);

    floating_drawer(
        column![
            header,
            week_strip,
            hairline_divider(theme),
            scrollable(
                column![unscheduled_section, timeline_section]
                    .spacing(SPACE_3)
                    .width(Length::Fill),
            )
            .height(Length::Fill),
        ]
        .spacing(SPACE_3)
        .height(Length::Fill),
        440.0,
        theme,
    )
}

fn view_week_strip<'a, Message: 'a>(
    today: chrono::NaiveDate,
    theme: EstudyTheme,
) -> Element<'a, Message> {
    let sunday_offset = i64::from(today.weekday().num_days_from_sunday());
    let start_of_week = today - Duration::days(sunday_offset);
    let labels = ["D", "S", "T", "Q", "Q", "S", "S"];

    let mut strip = row![].spacing(SPACE_1).width(Length::Fill);
    for (idx, short) in labels.iter().enumerate() {
        let day = start_of_week + Duration::days(i64::try_from(idx).unwrap_or(0));
        let is_today = day == today;
        let badge = container(
            text(format!("{}", day.day()))
                .size(12)
                .font(FONT_MONO_MEDIUM)
                .color(if is_today {
                    theme.on_accent
                } else {
                    theme.text_primary
                }),
        )
        .width(Length::Fixed(28.0))
        .height(Length::Fixed(28.0))
        .center_x(Length::Fixed(28.0))
        .center_y(Length::Fixed(28.0))
        .style(move |_| container::Style {
            background: is_today.then(|| theme.accent.into()),
            border: Border {
                radius: RADIUS_PILL.into(),
                ..Border::default()
            },
            ..container::Style::default()
        });

        let dot_color = CategoryColor::from_index(idx).solid(theme);
        strip = strip.push(
            column![
                text(*short)
                    .size(10)
                    .font(FONT_SANS_MEDIUM)
                    .color(theme.text_muted),
                badge,
                row![color_dot(dot_color)].spacing(2.0),
            ]
            .spacing(4.0)
            .align_x(Alignment::Center)
            .width(Length::Fill),
        );
    }

    strip.into()
}

fn view_unscheduled_section<'a, Message: 'a>(
    timeline: &TodayTimeline,
    theme: EstudyTheme,
) -> Element<'a, Message> {
    let mut items = column![].spacing(SPACE_1);
    if timeline.unscheduled.is_empty() {
        items = items.push(
            text("Nenhum lembrete sem horário para hoje.")
                .size(12)
                .color(theme.text_muted),
        );
    } else {
        for occ in &timeline.unscheduled {
            let category = if occ.is_task_deadline {
                CategoryColor::Terracotta
            } else {
                CategoryColor::Sage
            };
            let bg = category.tint(theme);
            let border_color = category.border(theme);
            items = items.push(
                container(
                    row![
                        color_dot(category.solid(theme)),
                        text(occ.title.clone())
                            .size(12)
                            .font(FONT_SANS_MEDIUM)
                            .color(theme.text_primary),
                    ]
                    .spacing(SPACE_2)
                    .align_y(Alignment::Center),
                )
                .padding([SPACE_2, SPACE_3])
                .width(Length::Fill)
                .style(move |_| container::Style {
                    background: Some(bg.into()),
                    border: Border {
                        color: border_color,
                        width: 1.0,
                        radius: RADIUS_CONTROL.into(),
                    },
                    ..container::Style::default()
                }),
            );
        }
    }

    column![section_header(Icon::Flag, "Sem horário", theme), items,]
        .spacing(SPACE_2)
        .into()
}

fn view_timeline_slots<'a, Message: 'a>(
    timeline: &TodayTimeline,
    theme: EstudyTheme,
) -> Element<'a, Message> {
    let mut col = column![].spacing(SPACE_2);
    let mut now_inserted = false;
    let now_label = format!(
        "Agora · {:02}:{:02}",
        timeline.now_minute / 60,
        timeline.now_minute % 60
    );

    for (idx, slot) in timeline.slots.iter().enumerate() {
        let slot_start = match slot {
            TimelineSlot::Busy { start_minute, .. } | TimelineSlot::Free { start_minute, .. } => {
                *start_minute
            }
        };
        if !now_inserted && timeline.now_minute <= slot_start {
            col = col.push(view_now_rule(now_label.clone(), theme));
            now_inserted = true;
        }

        match slot {
            TimelineSlot::Busy {
                title,
                time_range,
                start_minute,
                end_minute,
                is_task,
                is_completed,
            } => {
                col = col.push(view_busy_slot_row(
                    idx,
                    title.clone(),
                    time_range,
                    *start_minute,
                    *end_minute,
                    *is_task,
                    *is_completed,
                    theme,
                ));
            }
            TimelineSlot::Free {
                duration_minutes, ..
            } => {
                col = col.push(view_free_slot_row(*duration_minutes, theme));
            }
        }
    }

    if !now_inserted {
        col = col.push(view_now_rule(now_label, theme));
    }

    if timeline.slots.is_empty() {
        col = col.push(soft_surface(
            text("Dia livre. Aproveite para focar nos seus projetos ou descansar.")
                .size(12)
                .color(theme.text_muted),
            theme,
        ));
    }

    col.into()
}

#[allow(clippy::too_many_arguments)]
fn view_busy_slot_row<'a, Message: 'a>(
    idx: usize,
    title: String,
    time_range: &str,
    start_minute: u16,
    end_minute: u16,
    is_task: bool,
    is_completed: bool,
    theme: EstudyTheme,
) -> Element<'a, Message> {
    let duration = end_minute.saturating_sub(start_minute).max(30);
    let capsule_h = ((f32::from(duration) / 30.0) * 24.0).clamp(40.0, 96.0);
    let category = if is_completed {
        CategoryColor::Sage
    } else if is_task {
        CategoryColor::Terracotta
    } else {
        CategoryColor::from_index(idx + 2)
    };
    let icon = if is_completed {
        Icon::CheckCircle2
    } else if is_task {
        Icon::Flag
    } else {
        Icon::Repeat
    };
    let hour_label = format!("{:02}:{:02}", start_minute / 60, start_minute % 60);

    let capsule = container(icon_svg(icon, 14.0, category.solid(theme)))
        .width(Length::Fixed(34.0))
        .height(Length::Fixed(capsule_h))
        .center_x(Length::Fixed(34.0))
        .center_y(Length::Fixed(capsule_h))
        .style(move |_| container::Style {
            background: Some(category.tint(theme).into()),
            border: Border {
                color: category.border(theme),
                width: 1.0,
                radius: RADIUS_PILL.into(),
            },
            ..container::Style::default()
        });

    row![
        container(
            text(hour_label)
                .size(11)
                .font(FONT_MONO)
                .color(theme.text_muted),
        )
        .width(Length::Fixed(46.0)),
        capsule,
        column![
            text(format!("{time_range} · {duration} min"))
                .size(11)
                .font(FONT_MONO)
                .color(theme.text_muted),
            text(title)
                .size(13)
                .font(FONT_SANS_SEMIBOLD)
                .color(theme.text_primary),
        ]
        .spacing(3.0)
        .width(Length::Fill),
    ]
    .spacing(SPACE_3)
    .align_y(Alignment::Center)
    .into()
}

fn view_free_slot_row<'a, Message: 'a>(
    duration_minutes: u16,
    theme: EstudyTheme,
) -> Element<'a, Message> {
    let hours = duration_minutes / 60;
    let mins = duration_minutes % 60;
    let free_label = if hours > 0 && mins > 0 {
        format!("Livre · {hours}h{mins:02}")
    } else if hours > 0 {
        format!("Livre · {hours}h")
    } else {
        format!("Livre · {mins} min")
    };

    let guide = container(text("").size(1))
        .width(Length::Fixed(2.0))
        .height(Length::Fixed(20.0))
        .style(move |_| container::Style {
            background: Some(theme.border_subtle.into()),
            ..container::Style::default()
        });

    row![
        container(text("")).width(Length::Fixed(46.0)),
        container(guide)
            .width(Length::Fixed(34.0))
            .center_x(Length::Fixed(34.0)),
        text(free_label)
            .size(11)
            .font(FONT_MONO)
            .color(theme.text_muted),
    ]
    .spacing(SPACE_3)
    .align_y(Alignment::Center)
    .into()
}

fn view_now_rule<'a, Message: 'a>(label: String, theme: EstudyTheme) -> Element<'a, Message> {
    let line_left = container(text("").size(1))
        .width(Length::Fixed(46.0))
        .height(Length::Fixed(1.5))
        .style(move |_| container::Style {
            background: Some(theme.danger.into()),
            ..container::Style::default()
        });
    let line_right = container(text("").size(1))
        .width(Length::Fill)
        .height(Length::Fixed(1.5))
        .style(move |_| container::Style {
            background: Some(theme.danger.into()),
            ..container::Style::default()
        });

    row![
        line_left,
        text(label)
            .size(11)
            .font(FONT_MONO_MEDIUM)
            .color(theme.danger),
        line_right,
    ]
    .spacing(SPACE_2)
    .align_y(Alignment::Center)
    .into()
}

const fn weekday_pt(weekday: chrono::Weekday) -> &'static str {
    match weekday {
        chrono::Weekday::Mon => "Segunda-feira",
        chrono::Weekday::Tue => "Terça-feira",
        chrono::Weekday::Wed => "Quarta-feira",
        chrono::Weekday::Thu => "Quinta-feira",
        chrono::Weekday::Fri => "Sexta-feira",
        chrono::Weekday::Sat => "Sábado",
        chrono::Weekday::Sun => "Domingo",
    }
}

const fn month_pt(month: u32) -> &'static str {
    match month {
        1 => "janeiro",
        2 => "fevereiro",
        3 => "março",
        4 => "abril",
        5 => "maio",
        6 => "junho",
        7 => "julho",
        8 => "agosto",
        9 => "setembro",
        10 => "outubro",
        11 => "novembro",
        _ => "dezembro",
    }
}
