//! Renderização da Home Pessoal em 3 colunas (Iced).

use estudy_ui_kit::widgets::{
    card, color_dot, hairline_divider, icon_svg, metric_stat, pill_chip, progress_track,
    section_header, segmented_switch, soft_surface, status_badge, tonal_card,
};
use estudy_ui_kit::{
    BoardFrameStyle, EstudyTheme, HeatmapGrid, Icon, RADIUS_PILL, SPACE_1, SPACE_2, SPACE_3,
    SPACE_4, SPACE_5, SPACE_6,
};
use iced::widget::{button, canvas, column, container, row, scrollable, text};
use iced::{Alignment, Border, Element, Length};

use super::state::{
    HomeDashboardData, HomeLoadState, HomeMessage, MAX_FOCUS_PINS, MAX_PINNED_MEDALS, MoodLevel,
    MoodViewMode, ProjectCardSummary,
};
use crate::command_palette::AppRoute;

/// Renderiza o estado atual da Home cobrindo Loading, Error, Empty e Populated.
pub fn view_home<'a, Message: Clone + 'a>(
    state: &'a HomeLoadState,
    greeting: &'a str,
    theme: EstudyTheme,
    map_home: impl Fn(HomeMessage) -> Message + Clone + 'a,
    map_route: impl Fn(AppRoute) -> Message + Clone + 'a,
) -> Element<'a, Message> {
    match state {
        HomeLoadState::Loading => view_loading_state(theme),
        HomeLoadState::Error(err_msg) => view_error_state(err_msg, theme, map_home),
        HomeLoadState::Ready(data) if data.is_empty_workspace() => {
            view_empty_state(data, greeting, theme, map_home)
        }
        HomeLoadState::Ready(data) => {
            view_three_columns(data, greeting, theme, map_home, map_route)
        }
    }
}

fn view_loading_state<'a, Message: 'a>(theme: EstudyTheme) -> Element<'a, Message> {
    let skeleton = card(
        column![
            section_header(Icon::Clock, "Carregando seu painel pessoal...", theme),
            text("Preparando resumo do dia, constância anual e projetos.")
                .size(13)
                .color(theme.text_muted),
        ]
        .spacing(SPACE_3),
        theme,
    )
    .width(Length::Fixed(460.0));

    container(skeleton)
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .into()
}

fn view_error_state<'a, Message: Clone + 'a>(
    err_msg: &'a str,
    theme: EstudyTheme,
    map_home: impl Fn(HomeMessage) -> Message + 'a,
) -> Element<'a, Message> {
    let retry_btn = button(
        row![
            icon_svg(Icon::RotateCcw, 15.0, theme.on_accent),
            text("Tentar novamente").size(13).color(theme.on_accent),
        ]
        .spacing(SPACE_2)
        .align_y(Alignment::Center),
    )
    .on_press(map_home(HomeMessage::RetryLoad))
    .padding([SPACE_2, SPACE_4])
    .style(move |_, status| theme.primary_button_style(status));

    let err_card = card(
        column![
            section_header(
                Icon::AlertTriangle,
                "Não foi possível carregar a Home",
                theme
            ),
            text(err_msg).size(13).color(theme.text_muted),
            retry_btn,
        ]
        .spacing(SPACE_3),
        theme,
    )
    .width(Length::Fixed(480.0));

    container(err_card)
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .into()
}

fn view_empty_state<'a, Message: Clone + 'a>(
    data: &'a HomeDashboardData,
    greeting: &'a str,
    theme: EstudyTheme,
    map_home: impl Fn(HomeMessage) -> Message + 'a,
) -> Element<'a, Message> {
    let start_btn = button(
        row![
            icon_svg(Icon::Plus, 15.0, theme.on_accent),
            text("Criar painel inicial").size(13).color(theme.on_accent),
        ]
        .spacing(SPACE_2)
        .align_y(Alignment::Center),
    )
    .on_press(map_home(HomeMessage::SeedStarterWorkspace))
    .padding([SPACE_2, SPACE_4])
    .style(move |_, status| theme.primary_button_style(status));

    let empty_card = card(
        column![
            text(format!("{greeting}, {}", data.profile.display_name))
                .size(20)
                .color(theme.text_primary),
            text("Seu espaço está pronto. Crie o primeiro projeto ou use Ctrl+K para adicionar uma tarefa.")
                .size(13)
                .color(theme.text_muted),
            start_btn,
        ]
        .spacing(SPACE_3),
        theme,
    )
    .width(Length::Fixed(520.0));

    container(empty_card)
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .into()
}

fn view_three_columns<'a, Message: Clone + 'a>(
    data: &'a HomeDashboardData,
    greeting: &'a str,
    theme: EstudyTheme,
    map_home: impl Fn(HomeMessage) -> Message + Clone + 'a,
    map_route: impl Fn(AppRoute) -> Message + Clone + 'a,
) -> Element<'a, Message> {
    let hero = view_hero_banner(data, greeting, theme, map_home.clone(), map_route.clone());
    let left = view_left_column(data, theme, map_home.clone());
    let center = view_center_column(data, theme);
    let right = view_right_column(data, theme, map_home);

    let columns = row![left, center, right]
        .spacing(SPACE_6)
        .width(Length::Fill);

    scrollable(
        container(
            container(column![hero, columns].spacing(SPACE_5).width(Length::Fill))
                .max_width(1360.0)
                .width(Length::Fill),
        )
        .padding(SPACE_6)
        .center_x(Length::Fill)
        .width(Length::Fill),
    )
    .height(Length::Fill)
    .into()
}

fn view_hero_banner<'a, Message: Clone + 'a>(
    data: &'a HomeDashboardData,
    greeting: &'a str,
    theme: EstudyTheme,
    map_home: impl Fn(HomeMessage) -> Message + 'a,
    map_route: impl Fn(AppRoute) -> Message + 'a,
) -> Element<'a, Message> {
    let pending_today = data.today_items.iter().filter(|item| !item.done).count();
    let subtitle = format!(
        "{} pendentes hoje · {} em foco · {} dias seguidos",
        pending_today,
        data.focus_items.len(),
        data.profile.streak_days
    );

    let quick_pills = row![
        pill_chip(
            "Timeline",
            None,
            Some(Icon::Clock),
            true,
            map_home(HomeMessage::OpenTodayTimeline),
            theme,
        ),
        pill_chip(
            "Semana",
            None,
            Some(Icon::Calendar),
            false,
            map_home(HomeMessage::OpenCalendarWeek),
            theme,
        ),
        pill_chip(
            "Projetos",
            None,
            Some(Icon::FolderKanban),
            false,
            map_route(AppRoute::Projects),
            theme,
        ),
        pill_chip(
            "Calendário",
            None,
            Some(Icon::Calendar),
            false,
            map_route(AppRoute::Calendar),
            theme,
        ),
    ]
    .spacing(SPACE_2)
    .align_y(Alignment::Center);

    card(
        column![
            row![
                column![
                    text(format!("{greeting}, {}", data.profile.display_name))
                        .size(24)
                        .color(theme.text_primary),
                    text(subtitle).size(13).color(theme.text_muted),
                ]
                .spacing(SPACE_1)
                .width(Length::Fill),
                row![
                    icon_svg(Icon::PawPrint, 15.0, theme.accent),
                    text(&data.pet_badge_label).size(12).color(theme.text_muted),
                ]
                .spacing(SPACE_1)
                .align_y(Alignment::Center),
            ]
            .align_y(Alignment::Center),
            hairline_divider(theme),
            quick_pills,
        ]
        .spacing(SPACE_3),
        theme,
    )
    .into()
}

fn view_left_column<'a, Message: Clone + 'a>(
    data: &'a HomeDashboardData,
    theme: EstudyTheme,
    map_home: impl Fn(HomeMessage) -> Message + 'a,
) -> Element<'a, Message> {
    column![
        view_profile_card(data, theme),
        view_today_card(data, theme, map_home),
    ]
    .spacing(SPACE_4)
    .width(Length::FillPortion(4))
    .into()
}

fn view_profile_card<'a, Message: 'a>(
    data: &'a HomeDashboardData,
    theme: EstudyTheme,
) -> Element<'a, Message> {
    let initials: String = data
        .profile
        .display_name
        .chars()
        .next()
        .map_or_else(|| "E".to_string(), |ch| ch.to_uppercase().to_string());

    let avatar = container(text(initials).size(18).color(theme.on_accent))
        .width(Length::Fixed(44.0))
        .height(Length::Fixed(44.0))
        .center_x(Length::Fixed(44.0))
        .center_y(Length::Fixed(44.0))
        .style(move |_| iced::widget::container::Style {
            background: Some(theme.accent.into()),
            border: iced::Border {
                radius: 22.0.into(),
                ..iced::Border::default()
            },
            ..iced::widget::container::Style::default()
        });

    let mut medals_row = row![].spacing(SPACE_1);
    for medal in data.profile.pinned_medals.iter().take(MAX_PINNED_MEDALS) {
        medals_row = medals_row.push(status_badge(&medal.title, theme.accent, theme));
    }

    card(
        column![
            row![
                avatar,
                column![
                    text(&data.profile.display_name)
                        .size(16)
                        .color(theme.text_primary),
                    row![
                        icon_svg(Icon::Flame, 14.0, theme.warning),
                        text(format!("{}d seguidos", data.profile.streak_days))
                            .size(12)
                            .color(theme.text_muted),
                        icon_svg(Icon::Coins, 14.0, theme.accent),
                        text(format!("{} moedas", data.profile.coins))
                            .size(12)
                            .color(theme.text_muted),
                    ]
                    .spacing(SPACE_2)
                    .align_y(Alignment::Center),
                ]
                .spacing(SPACE_1),
            ]
            .spacing(SPACE_3)
            .align_y(Alignment::Center),
            hairline_divider(theme),
            medals_row,
        ]
        .spacing(SPACE_3),
        theme,
    )
    .into()
}

fn view_today_card<'a, Message: Clone + 'a>(
    data: &'a HomeDashboardData,
    theme: EstudyTheme,
    map_home: impl Fn(HomeMessage) -> Message + 'a,
) -> Element<'a, Message> {
    let mut today_list = column![].spacing(SPACE_1);
    for (idx, item) in data.today_items.iter().enumerate() {
        let icon = if item.done {
            Icon::CheckCircle2
        } else {
            Icon::Clock
        };
        let tint = if item.done {
            theme.success
        } else {
            theme.accent
        };

        let item_btn = button(
            row![
                icon_svg(icon, 15.0, tint),
                text(&item.time_label).size(12).color(theme.text_muted),
                text(&item.title).size(13).color(if item.done {
                    theme.text_muted
                } else {
                    theme.text_primary
                }),
            ]
            .spacing(SPACE_2)
            .align_y(Alignment::Center),
        )
        .on_press(map_home(HomeMessage::ToggleTodayItem(idx)))
        .width(Length::Fill)
        .padding([SPACE_2, SPACE_2])
        .style(move |_, status| theme.subtle_button_style(status));

        today_list = today_list.push(item_btn);
        if idx + 1 < data.today_items.len() {
            today_list = today_list.push(hairline_divider(theme));
        }
    }

    let timeline_action = pill_chip(
        "Abrir timeline completa",
        None,
        Some(Icon::Clock),
        false,
        map_home(HomeMessage::OpenTodayTimeline),
        theme,
    );

    card(
        column![
            section_header(Icon::Calendar, "Hoje", theme),
            hairline_divider(theme),
            today_list,
            timeline_action,
        ]
        .spacing(SPACE_3),
        theme,
    )
    .into()
}

fn view_center_column<'a, Message: 'a>(
    data: &'a HomeDashboardData,
    theme: EstudyTheme,
) -> Element<'a, Message> {
    let heatmap_header = row![
        section_header(Icon::Flame, "Constância anual (365 dias)", theme),
        container(
            row![
                color_dot(theme.success),
                text(format!("{} registros", data.heatmap.total_actions()))
                    .size(12)
                    .color(theme.text_muted),
            ]
            .spacing(SPACE_1)
            .align_y(Alignment::Center)
        )
        .align_x(Alignment::End)
        .width(Length::Fill),
    ]
    .align_y(Alignment::Center);

    let heatmap_canvas = canvas(HeatmapGrid {
        data: &data.heatmap,
        theme,
    })
    .width(Length::Fill)
    .height(Length::Fixed(124.0));

    let heatmap_card = card(
        column![
            heatmap_header,
            hairline_divider(theme),
            heatmap_canvas,
            row![
                metric_stat(
                    "Sequência",
                    format!("{}d", data.profile.streak_days),
                    "dias ativos seguidos",
                    theme.accent,
                    theme,
                )
                .width(Length::Fill),
                tonal_card(
                    column![
                        text("Foco anual").size(11).color(theme.on_accent),
                        text(format!("{}", data.heatmap.total_actions()))
                            .size(22)
                            .color(theme.on_accent),
                        text("ações registradas").size(11).color(theme.on_accent),
                    ]
                    .spacing(SPACE_1),
                    theme,
                )
                .padding(SPACE_3)
                .width(Length::Fill),
            ]
            .spacing(SPACE_3),
        ]
        .spacing(SPACE_3),
        theme,
    );

    let projects_grid = view_projects_two_columns(&data.projects, theme);
    let projects_card = card(
        column![
            section_header(Icon::FolderKanban, "Projetos", theme),
            hairline_divider(theme),
            projects_grid,
        ]
        .spacing(SPACE_3),
        theme,
    );

    column![heatmap_card, projects_card]
        .spacing(SPACE_4)
        .width(Length::FillPortion(5))
        .into()
}

fn view_projects_two_columns<'a, Message: 'a>(
    projects: &'a [ProjectCardSummary],
    theme: EstudyTheme,
) -> Element<'a, Message> {
    let mut rows = column![].spacing(SPACE_3);
    for pair in projects.chunks(2) {
        let mut line = row![].spacing(SPACE_3).width(Length::Fill);
        for project in pair {
            let total = project.todo_count + project.in_progress_count + project.done_count;
            let ratio = if total == 0 {
                0.0
            } else {
                f32::from(project.done_count) / f32::from(total)
            };
            let item = soft_surface(
                column![
                    row![
                        text(&project.title).size(14).color(theme.text_primary),
                        container(status_badge(&project.status_label, theme.accent, theme))
                            .align_x(Alignment::End)
                            .width(Length::Fill),
                    ]
                    .align_y(Alignment::Center),
                    progress_track(ratio, theme.accent, theme),
                    text(format!(
                        "{} a fazer · {} em curso · {} feitos",
                        project.todo_count, project.in_progress_count, project.done_count
                    ))
                    .size(11)
                    .color(theme.text_muted),
                ]
                .spacing(SPACE_2),
                theme,
            )
            .width(Length::Fill);
            line = line.push(item);
        }
        if pair.len() == 1 {
            line = line.push(container(text("")).width(Length::Fill));
        }
        rows = rows.push(line);
    }
    rows.into()
}

fn view_right_column<'a, Message: Clone + 'a>(
    data: &'a HomeDashboardData,
    theme: EstudyTheme,
    map_home: impl Fn(HomeMessage) -> Message + Clone + 'a,
) -> Element<'a, Message> {
    column![
        view_focus_card(data, theme, map_home.clone()),
        view_mood_card(data, theme, map_home.clone()),
        view_pinboard_card(data, theme, map_home),
    ]
    .spacing(SPACE_4)
    .width(Length::FillPortion(4))
    .into()
}

fn view_focus_card<'a, Message: Clone + 'a>(
    data: &'a HomeDashboardData,
    theme: EstudyTheme,
    map_home: impl Fn(HomeMessage) -> Message + 'a,
) -> Element<'a, Message> {
    let focus_title = format!("Em foco ({}/{MAX_FOCUS_PINS})", data.focus_items.len());
    let mut focus_list = column![].spacing(SPACE_2);
    for pin in &data.focus_items {
        let start_btn = button(icon_svg(Icon::Play, 13.0, theme.on_accent))
            .on_press(map_home(HomeMessage::StartFocusForPin(pin.id.clone())))
            .padding(SPACE_2)
            .style(move |_, status| theme.pill_button_style(true, status));

        let item = soft_surface(
            row![
                column![
                    text(&pin.title).size(13).color(theme.text_primary),
                    text(format!(
                        "{} · {}/{} pomos",
                        pin.context_label, pin.completed_pomos, pin.estimated_pomos
                    ))
                    .size(11)
                    .color(theme.text_muted),
                ]
                .spacing(2.0)
                .width(Length::Fill),
                start_btn,
            ]
            .spacing(SPACE_2)
            .align_y(Alignment::Center),
            theme,
        );
        focus_list = focus_list.push(item);
    }

    card(
        column![
            section_header(Icon::Pin, focus_title, theme),
            hairline_divider(theme),
            focus_list,
        ]
        .spacing(SPACE_3),
        theme,
    )
    .into()
}

fn view_mood_card<'a, Message: Clone + 'a>(
    data: &'a HomeDashboardData,
    theme: EstudyTheme,
    map_home: impl Fn(HomeMessage) -> Message + Clone + 'a,
) -> Element<'a, Message> {
    const MOOD_MODES: [(MoodViewMode, &str); 2] =
        [(MoodViewMode::Week, "7 dias"), (MoodViewMode::Year, "Ano")];

    let mode_switch = segmented_switch(
        &MOOD_MODES,
        data.mood_view,
        {
            let map_home = map_home.clone();
            move |_| map_home(HomeMessage::ToggleMoodViewMode)
        },
        theme,
    );

    let header = row![
        section_header(Icon::Smile, "Diário de Humor", theme),
        container(mode_switch)
            .align_x(Alignment::End)
            .width(Length::Fill),
    ]
    .align_y(Alignment::Center);

    let body: Element<'a, Message> = match data.mood_view {
        MoodViewMode::Week => {
            let mut week_row = row![].spacing(SPACE_1).width(Length::Fill);
            for day in &data.mood_week {
                let icon = day.level.map_or(Icon::HelpCircle, MoodLevel::icon);
                let color = if day.level.is_some() {
                    theme.accent
                } else {
                    theme.text_muted
                };
                week_row = week_row.push(
                    column![
                        container(icon_svg(icon, 15.0, color))
                            .padding(6.0)
                            .style(move |_| iced::widget::container::Style {
                                background: Some(
                                    if day.level.is_some() {
                                        theme.accent_subtle
                                    } else {
                                        theme.surface_soft
                                    }
                                    .into()
                                ),
                                border: Border {
                                    radius: RADIUS_PILL.into(),
                                    ..Border::default()
                                },
                                ..iced::widget::container::Style::default()
                            }),
                        text(&day.day_label).size(10).color(theme.text_muted),
                    ]
                    .spacing(SPACE_1)
                    .align_x(Alignment::Center)
                    .width(Length::Fill),
                );
            }
            week_row.into()
        }
        MoodViewMode::Year => {
            let mut year_row = row![].spacing(SPACE_2).width(Length::Fill);
            for (idx, mood) in MoodLevel::ALL.iter().enumerate() {
                let count = data.mood_year_counts.get(idx).copied().unwrap_or(0);
                year_row = year_row.push(
                    soft_surface(
                        column![
                            icon_svg(mood.icon(), 15.0, theme.accent),
                            text(format!("{count}d")).size(12).color(theme.text_primary),
                            text(mood.label()).size(10).color(theme.text_muted),
                        ]
                        .spacing(2.0)
                        .align_x(Alignment::Center),
                        theme,
                    )
                    .width(Length::Fill),
                );
            }
            year_row.into()
        }
    };

    let today_mood = data.mood_week.last().and_then(|day| day.level);
    let mut mood_picker = row![].spacing(SPACE_1);
    for mood in MoodLevel::ALL {
        let is_selected = today_mood == Some(mood);
        mood_picker = mood_picker.push(pill_chip(
            mood.label(),
            None,
            Some(mood.icon()),
            is_selected,
            map_home(HomeMessage::SelectTodayMood(mood)),
            theme,
        ));
    }

    card(
        column![header, hairline_divider(theme), body, mood_picker].spacing(SPACE_3),
        theme,
    )
    .into()
}

fn view_pinboard_card<'a, Message: Clone + 'a>(
    data: &'a HomeDashboardData,
    theme: EstudyTheme,
    map_home: impl Fn(HomeMessage) -> Message + 'a,
) -> Element<'a, Message> {
    let active_frame = data.pinboard.frame;
    let frame_preview = container(
        column![
            icon_svg(Icon::Image, 28.0, theme.accent),
            text(&data.pinboard.caption)
                .size(13)
                .color(theme.text_primary),
            text(format!("Moldura: {}", active_frame.label()))
                .size(11)
                .color(theme.text_muted),
        ]
        .spacing(SPACE_2)
        .align_x(Alignment::Center),
    )
    .padding(SPACE_4)
    .width(Length::Fill)
    .align_x(Alignment::Center)
    .style(move |_| theme.board_frame_style(active_frame));

    let mut frame_rows = column![].spacing(SPACE_1);
    for chunk in BoardFrameStyle::ALL.chunks(4) {
        let mut frame_chips = row![].spacing(SPACE_1);
        for frame in chunk {
            let is_active = *frame == active_frame;
            frame_chips = frame_chips.push(pill_chip(
                frame.label(),
                None,
                None,
                is_active,
                map_home(HomeMessage::SelectBoardFrame(*frame)),
                theme,
            ));
        }
        frame_rows = frame_rows.push(frame_chips);
    }

    card(
        column![
            section_header(Icon::Frame, "Quadro Personalizável", theme),
            hairline_divider(theme),
            frame_preview,
            frame_rows,
        ]
        .spacing(SPACE_3),
        theme,
    )
    .into()
}
