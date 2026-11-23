//! Renderização da Home Pessoal em 3 colunas (Iced).

use estudy_ui_kit::widgets::{card, icon_svg, panel, section_header, status_badge};
use estudy_ui_kit::{
    BoardFrameStyle, EstudyTheme, HeatmapGrid, Icon, SPACE_1, SPACE_2, SPACE_3, SPACE_4, SPACE_6,
};
use iced::widget::{button, canvas, column, container, row, scrollable, text};
use iced::{Alignment, Element, Length};

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
    let left = view_left_column(data, greeting, theme, map_home.clone(), map_route);
    let center = view_center_column(data, theme);
    let right = view_right_column(data, theme, map_home);

    let columns = row![left, center, right]
        .spacing(SPACE_6)
        .width(Length::Fill);

    scrollable(container(columns).padding(SPACE_6).width(Length::Fill))
        .height(Length::Fill)
        .into()
}

fn view_left_column<'a, Message: Clone + 'a>(
    data: &'a HomeDashboardData,
    greeting: &'a str,
    theme: EstudyTheme,
    map_home: impl Fn(HomeMessage) -> Message + 'a,
    map_route: impl Fn(AppRoute) -> Message + 'a,
) -> Element<'a, Message> {
    column![
        view_profile_card(data, greeting, theme),
        view_today_card(data, theme, map_home, map_route),
    ]
    .spacing(SPACE_4)
    .width(Length::FillPortion(3))
    .into()
}

fn view_profile_card<'a, Message: 'a>(
    data: &'a HomeDashboardData,
    greeting: &'a str,
    theme: EstudyTheme,
) -> Element<'a, Message> {
    let initials: String = data
        .profile
        .display_name
        .chars()
        .next()
        .map_or_else(|| "E".to_string(), |ch| ch.to_uppercase().to_string());

    let avatar = container(text(initials).size(18).color(theme.on_accent))
        .width(Length::Fixed(42.0))
        .height(Length::Fixed(42.0))
        .center_x(Length::Fixed(42.0))
        .center_y(Length::Fixed(42.0))
        .style(move |_| iced::widget::container::Style {
            background: Some(theme.accent.into()),
            border: iced::Border {
                radius: 21.0.into(),
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
                    text(format!("{greeting}, {}", data.profile.display_name))
                        .size(16)
                        .color(theme.text_primary),
                    row![
                        icon_svg(Icon::Flame, 14.0, theme.warning),
                        text(format!("{}d", data.profile.streak_days))
                            .size(12)
                            .color(theme.text_muted),
                        icon_svg(Icon::Coins, 14.0, theme.accent),
                        text(format!("{}", data.profile.coins))
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
    map_route: impl Fn(AppRoute) -> Message + 'a,
) -> Element<'a, Message> {
    let mut today_list = column![].spacing(SPACE_2);
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
                text(&item.title).size(13).color(theme.text_primary),
            ]
            .spacing(SPACE_2)
            .align_y(Alignment::Center),
        )
        .on_press(map_home(HomeMessage::ToggleTodayItem(idx)))
        .width(Length::Fill)
        .padding(SPACE_2)
        .style(move |_, status| theme.ghost_button_style(status));

        today_list = today_list.push(item_btn);
    }

    let shortcuts = row![
        button(text("Semana").size(12).color(theme.text_primary))
            .on_press(map_route(AppRoute::Calendar))
            .padding([SPACE_1, SPACE_3])
            .style(move |_, status| theme.ghost_button_style(status)),
        button(text("Calendário").size(12).color(theme.text_primary))
            .on_press(map_route(AppRoute::Calendar))
            .padding([SPACE_1, SPACE_3])
            .style(move |_, status| theme.ghost_button_style(status)),
    ]
    .spacing(SPACE_2);

    card(
        column![
            section_header(Icon::Calendar, "Hoje", theme),
            today_list,
            shortcuts,
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
                icon_svg(Icon::PawPrint, 14.0, theme.accent),
                text(&data.pet_badge_label).size(12).color(theme.text_muted),
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
    .height(Length::Fixed(106.0));

    let heatmap_card = card(
        column![
            heatmap_header,
            heatmap_canvas,
            text(format!(
                "{} registros acumulados no ano",
                data.heatmap.total_actions()
            ))
            .size(12)
            .color(theme.text_muted),
        ]
        .spacing(SPACE_3),
        theme,
    );

    let projects_grid = view_projects_two_columns(&data.projects, theme);
    let projects_card = card(
        column![
            section_header(Icon::FolderKanban, "Projetos", theme),
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
            let item = panel(
                column![
                    row![
                        text(&project.title).size(14).color(theme.text_primary),
                        container(status_badge(&project.status_label, theme.accent, theme))
                            .align_x(Alignment::End)
                            .width(Length::Fill),
                    ]
                    .align_y(Alignment::Center),
                    text(format!(
                        "{} a fazer · {} em curso · {} feitos",
                        project.todo_count, project.in_progress_count, project.done_count
                    ))
                    .size(12)
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
    .width(Length::FillPortion(3))
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
        let start_btn = button(icon_svg(Icon::Play, 13.0, theme.accent))
            .on_press(map_home(HomeMessage::StartFocusForPin(pin.id.clone())))
            .padding(SPACE_1)
            .style(move |_, status| theme.ghost_button_style(status));

        let item = panel(
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
        column![section_header(Icon::Pin, focus_title, theme), focus_list].spacing(SPACE_3),
        theme,
    )
    .into()
}

fn view_mood_card<'a, Message: Clone + 'a>(
    data: &'a HomeDashboardData,
    theme: EstudyTheme,
    map_home: impl Fn(HomeMessage) -> Message + 'a,
) -> Element<'a, Message> {
    let toggle_label = match data.mood_view {
        MoodViewMode::Week => "Ver Ano",
        MoodViewMode::Year => "Ver 7d",
    };

    let header = row![
        section_header(Icon::Smile, "Diário de Humor", theme),
        container(
            button(text(toggle_label).size(11).color(theme.text_primary))
                .on_press(map_home(HomeMessage::ToggleMoodViewMode))
                .padding([SPACE_1, SPACE_2])
                .style(move |_, status| theme.ghost_button_style(status)),
        )
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
                        icon_svg(icon, 16.0, color),
                        text(&day.day_label).size(11).color(theme.text_muted),
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
                    panel(
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

    let mut mood_picker = row![].spacing(SPACE_1);
    for mood in MoodLevel::ALL {
        let btn = button(
            row![
                icon_svg(mood.icon(), 13.0, theme.text_primary),
                text(mood.label()).size(11).color(theme.text_primary),
            ]
            .spacing(SPACE_1)
            .align_y(Alignment::Center),
        )
        .on_press(map_home(HomeMessage::SelectTodayMood(mood)))
        .padding([SPACE_1, SPACE_2])
        .style(move |_, status| theme.ghost_button_style(status));
        mood_picker = mood_picker.push(btn);
    }

    card(column![header, body, mood_picker].spacing(SPACE_3), theme).into()
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
            let btn = button(text(frame.label()).size(11).color(if is_active {
                theme.on_accent
            } else {
                theme.text_primary
            }))
            .on_press(map_home(HomeMessage::SelectBoardFrame(*frame)))
            .padding([SPACE_1, SPACE_2])
            .style(move |_, status| {
                if is_active {
                    theme.primary_button_style(status)
                } else {
                    theme.ghost_button_style(status)
                }
            });
            frame_chips = frame_chips.push(btn);
        }
        frame_rows = frame_rows.push(frame_chips);
    }

    card(
        column![
            section_header(Icon::Frame, "Quadro Personalizável", theme),
            frame_preview,
            frame_rows,
        ]
        .spacing(SPACE_3),
        theme,
    )
    .into()
}
