//! Drawer deslizante "Hoje" (Timeline Vertical com janelas livres e linha Agora).

use chrono::{DateTime, Utc};
use estudy_domain::{CalendarEvent, Task, TimelineSlot, TodayTimeline, build_today_timeline};
use estudy_ui_kit::widgets::{
    card, drawer_backdrop, hairline_divider, icon_svg, panel, section_header, soft_surface,
    status_badge,
};
use estudy_ui_kit::{EstudyTheme, Icon, SPACE_1, SPACE_2, SPACE_3};
use iced::widget::{button, column, container, row, scrollable, text};
use iced::{Alignment, Element, Length};

use super::state::CalendarMessage;

/// Renderiza o Drawer "Hoje" sobreposto à direita da tela atual.
pub fn view_today_drawer<'a, Message: Clone + 'a>(
    events: &'a [CalendarEvent],
    tasks: &'a [Task],
    now_utc: DateTime<Utc>,
    theme: EstudyTheme,
    map_msg: impl Fn(CalendarMessage) -> Message + 'a,
) -> Element<'a, Message> {
    let timeline = build_today_timeline(events, tasks, now_utc);
    let header = row![
        section_header(Icon::Clock, "Timeline · Hoje", theme),
        container(
            button(icon_svg(Icon::X, 15.0, theme.text_muted))
                .on_press(map_msg(CalendarMessage::CloseTodayDrawer))
                .padding(4.0)
                .style(move |_, status| theme.subtle_button_style(status)),
        )
        .align_x(Alignment::End)
        .width(Length::Fill),
    ]
    .align_y(Alignment::Center);

    let unscheduled_section = view_unscheduled_section(&timeline, theme);
    let timeline_section = view_timeline_slots(&timeline, theme);

    drawer_backdrop(
        scrollable(
            column![
                header,
                hairline_divider(theme),
                unscheduled_section,
                timeline_section,
            ]
            .spacing(SPACE_3),
        )
        .height(Length::Fill),
        420.0,
        theme,
    )
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
            let icon = if occ.is_task_deadline {
                Icon::Flag
            } else {
                Icon::Calendar
            };
            items = items.push(
                row![
                    icon_svg(icon, 13.0, theme.accent),
                    text(occ.title.clone()).size(12).color(theme.text_primary),
                ]
                .spacing(SPACE_2)
                .align_y(Alignment::Center),
            );
        }
    }

    soft_surface(
        column![text("Sem horário").size(12).color(theme.text_muted), items].spacing(SPACE_2),
        theme,
    )
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

    for slot in &timeline.slots {
        let slot_start = match slot {
            TimelineSlot::Busy { start_minute, .. } | TimelineSlot::Free { start_minute, .. } => {
                *start_minute
            }
        };
        if !now_inserted && timeline.now_minute <= slot_start {
            col = col.push(status_badge(now_label.clone(), theme.danger, theme));
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
                let duration = end_minute.saturating_sub(*start_minute).max(30);
                let height = ((f32::from(duration) / 30.0) * 28.0).clamp(42.0, 140.0);
                let icon = if *is_task { Icon::Flag } else { Icon::Clock };
                let tint = if *is_completed {
                    theme.success
                } else {
                    theme.accent
                };

                col = col.push(
                    card(
                        column![
                            row![
                                icon_svg(icon, 13.0, tint),
                                text(time_range.clone()).size(11).color(theme.text_muted),
                            ]
                            .spacing(SPACE_1)
                            .align_y(Alignment::Center),
                            text(title.clone()).size(13).color(theme.text_primary),
                        ]
                        .spacing(4.0),
                        theme,
                    )
                    .height(Length::Fixed(height))
                    .width(Length::Fill),
                );
            }
            TimelineSlot::Free { label, .. } => {
                col = col.push(
                    panel(
                        row![
                            icon_svg(Icon::Sprout, 12.0, theme.success),
                            text(label.clone()).size(12).color(theme.text_muted),
                        ]
                        .spacing(SPACE_2)
                        .align_y(Alignment::Center),
                        theme,
                    )
                    .width(Length::Fill),
                );
            }
        }
    }

    if !now_inserted {
        col = col.push(status_badge(now_label, theme.danger, theme));
    }

    col.into()
}
