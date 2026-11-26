//! Renderização da Home Pessoal em painel Bento editorial (Iced).

use chrono::NaiveDate;
use estudy_ui_kit::widgets::{
    card, category_tile, color_dot, editorial_heading, hairline_divider, icon_svg, pill_chip,
    progress_track, section_header, segmented_switch, soft_surface, tonal_card,
};
use estudy_ui_kit::{
    CategoryColor, EstudyTheme, FONT_MONO, FONT_MONO_MEDIUM, FONT_SANS_MEDIUM, FONT_SANS_SEMIBOLD,
    HeatmapGrid, Icon, RADIUS_PILL, SPACE_1, SPACE_2, SPACE_3, SPACE_4, SPACE_5, SPACE_6,
    SparklineCanvas, SparklineData,
};
use iced::widget::{button, canvas, column, container, row, scrollable, text};
use iced::{Alignment, Border, Color, Element, Length};

use super::state::{
    HomeDashboardData, HomeLoadState, HomeMessage, MAX_FOCUS_PINS, MoodLevel, MoodViewMode,
    ProjectCardSummary,
};
use crate::command_palette::AppRoute;
use crate::features::habits::{HabitsMessage, HabitsState};
use crate::features::second_brain::{SecondBrainMessage, SecondBrainState};

/// Renderiza o estado atual da Home cobrindo Loading, Error, Empty e Populated.
#[allow(clippy::too_many_arguments)]
pub fn view_home<'a, Message: Clone + 'a>(
    state: &'a HomeLoadState,
    habits: &'a HabitsState,
    brain: &'a SecondBrainState,
    today: NaiveDate,
    greeting: &'a str,
    theme: EstudyTheme,
    map_home: impl Fn(HomeMessage) -> Message + Clone + 'a,
    map_habits: impl Fn(HabitsMessage) -> Message + Clone + 'a,
    map_brain: impl Fn(SecondBrainMessage) -> Message + Clone + 'a,
    map_route: impl Fn(AppRoute) -> Message + Clone + 'a,
) -> Element<'a, Message> {
    match state {
        HomeLoadState::Loading => view_loading_state(theme),
        HomeLoadState::Error(err_msg) => view_error_state(err_msg, theme, map_home),
        HomeLoadState::Ready(data) if data.is_empty_workspace() => {
            view_empty_state(data, greeting, theme, map_home)
        }
        HomeLoadState::Ready(data) => view_bento_workspace(
            data, habits, brain, today, greeting, theme, map_home, map_habits, map_brain, map_route,
        ),
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

#[allow(clippy::too_many_arguments)]
fn view_bento_workspace<'a, Message: Clone + 'a>(
    data: &'a HomeDashboardData,
    habits: &'a HabitsState,
    brain: &'a SecondBrainState,
    today: NaiveDate,
    greeting: &'a str,
    theme: EstudyTheme,
    map_home: impl Fn(HomeMessage) -> Message + Clone + 'a,
    map_habits: impl Fn(HabitsMessage) -> Message + Clone + 'a,
    map_brain: impl Fn(SecondBrainMessage) -> Message + Clone + 'a,
    map_route: impl Fn(AppRoute) -> Message + Clone + 'a,
) -> Element<'a, Message> {
    let header = view_editorial_header(data, greeting, theme, map_home.clone(), map_route.clone());
    let bento_top = view_bento_hero_row(data, theme);

    let main_left = column![
        view_projects_section(&data.projects, theme, map_home.clone(), map_route.clone()),
        view_inline_brain_card(brain, theme, map_brain, map_route.clone()),
    ]
    .spacing(SPACE_5)
    .width(Length::FillPortion(8));

    let main_right = column![
        view_focus_card(data, theme, map_home.clone()),
        view_inline_habits_card(habits, today, theme, map_habits, map_route),
        view_mood_card(data, theme, map_home),
    ]
    .spacing(SPACE_4)
    .width(Length::FillPortion(4));

    let main_grid = row![main_left, main_right]
        .spacing(SPACE_5)
        .width(Length::Fill);

    scrollable(
        container(
            container(
                column![header, bento_top, main_grid]
                    .spacing(SPACE_5)
                    .width(Length::Fill),
            )
            .max_width(1420.0)
            .width(Length::Fill),
        )
        .padding([SPACE_5, SPACE_6])
        .center_x(Length::Fill)
        .width(Length::Fill),
    )
    .height(Length::Fill)
    .into()
}

fn view_inline_habits_card<'a, Message: Clone + 'a>(
    habits: &'a HabitsState,
    today: NaiveDate,
    theme: EstudyTheme,
    map_habits: impl Fn(HabitsMessage) -> Message + 'a,
    map_route: impl Fn(AppRoute) -> Message + 'a,
) -> Element<'a, Message> {
    let (done, total) = habits.today_progress(today);
    let mut list = column![].spacing(SPACE_2);
    for habit in habits.habits.iter().take(4) {
        let cat = CategoryColor::from_index(habit.color_index);
        let checked = habit.is_done_on(today);
        let icon = if checked {
            Icon::CheckCircle2
        } else {
            Icon::Flame
        };
        let fg = if checked {
            cat.solid(theme)
        } else {
            theme.text_muted
        };

        list = list.push(
            button(
                row![
                    icon_svg(icon, 15.0, fg),
                    text(&habit.title)
                        .size(13)
                        .font(FONT_SANS_MEDIUM)
                        .color(if checked {
                            theme.text_muted
                        } else {
                            theme.text_primary
                        }),
                    container(
                        text(format!("{}d", habit.current_streak(today)))
                            .size(11)
                            .font(FONT_MONO)
                            .color(cat.solid(theme)),
                    )
                    .align_x(Alignment::End)
                    .width(Length::Fill),
                ]
                .spacing(SPACE_2)
                .align_y(Alignment::Center),
            )
            .on_press(map_habits(HabitsMessage::ToggleHabitDate {
                habit_id: habit.id.clone(),
                date: today,
            }))
            .width(Length::Fill)
            .padding([6.0, 8.0])
            .style(move |_, status| theme.subtle_button_style(status)),
        );
    }

    card(
        column![
            row![
                section_header(Icon::Flame, "Hábitos de Hoje", theme),
                container(
                    button(
                        text(format!("{done}/{total} · ver tudo"))
                            .size(11)
                            .font(FONT_MONO)
                            .color(theme.accent),
                    )
                    .on_press(map_route(AppRoute::Habits))
                    .padding([2.0, 6.0])
                    .style(move |_, status| theme.ghost_button_style(status)),
                )
                .align_x(Alignment::End)
                .width(Length::Fill),
            ]
            .align_y(Alignment::Center),
            hairline_divider(theme),
            list,
        ]
        .spacing(SPACE_3),
        theme,
    )
    .width(Length::FillPortion(1))
    .into()
}

fn view_inline_brain_card<'a, Message: Clone + 'a>(
    brain: &'a SecondBrainState,
    theme: EstudyTheme,
    map_brain: impl Fn(SecondBrainMessage) -> Message + 'a,
    map_route: impl Fn(AppRoute) -> Message + 'a,
) -> Element<'a, Message> {
    let mut list = column![].spacing(SPACE_2);
    for (idx, note) in brain.notes.iter().take(3).enumerate() {
        let cat = CategoryColor::from_index(idx + 3);
        list = list.push(
            button(
                column![
                    row![
                        color_dot(cat.solid(theme)),
                        text(&note.title)
                            .size(13)
                            .font(FONT_SANS_SEMIBOLD)
                            .color(theme.text_primary),
                    ]
                    .spacing(6.0)
                    .align_y(Alignment::Center),
                    text(note.preview_excerpt(72))
                        .size(11)
                        .color(theme.text_muted),
                ]
                .spacing(2.0)
                .width(Length::Fill),
            )
            .on_press(map_brain(SecondBrainMessage::SelectNote(note.id.clone())))
            .width(Length::Fill)
            .padding([6.0, 8.0])
            .style(move |_, status| theme.subtle_button_style(status)),
        );
    }

    card(
        column![
            row![
                section_header(Icon::Brain, "Segundo Cérebro", theme),
                container(
                    button(
                        text("Abrir notas")
                            .size(11)
                            .font(FONT_SANS_MEDIUM)
                            .color(theme.accent),
                    )
                    .on_press(map_route(AppRoute::SecondBrain))
                    .padding([2.0, 6.0])
                    .style(move |_, status| theme.ghost_button_style(status)),
                )
                .align_x(Alignment::End)
                .width(Length::Fill),
            ]
            .align_y(Alignment::Center),
            hairline_divider(theme),
            list,
        ]
        .spacing(SPACE_3),
        theme,
    )
    .width(Length::FillPortion(1))
    .into()
}

fn view_editorial_header<'a, Message: Clone + 'a>(
    data: &'a HomeDashboardData,
    greeting: &'a str,
    theme: EstudyTheme,
    map_home: impl Fn(HomeMessage) -> Message + 'a,
    map_route: impl Fn(AppRoute) -> Message + 'a,
) -> Element<'a, Message> {
    let pending_today = data.today_items.iter().filter(|item| !item.done).count();
    let subtitle = format!(
        "Hoje você tem {} blocos pendentes, {} itens em foco e {} dias seguidos de constância.",
        pending_today,
        data.focus_items.len(),
        data.profile.streak_days
    );

    let quick_pills = row![
        pill_chip(
            "Timeline Hoje",
            None,
            Some(Icon::Clock),
            true,
            map_home(HomeMessage::OpenTodayTimeline),
            theme,
        ),
        pill_chip(
            "Hábitos",
            Some(CategoryColor::Sage.solid(theme)),
            Some(Icon::Flame),
            false,
            map_route(AppRoute::Habits),
            theme,
        ),
        pill_chip(
            "Segundo Cérebro",
            Some(CategoryColor::Lavender.solid(theme)),
            Some(Icon::Brain),
            false,
            map_route(AppRoute::SecondBrain),
            theme,
        ),
        pill_chip(
            "Semana",
            Some(CategoryColor::Slate.solid(theme)),
            Some(Icon::Calendar),
            false,
            map_home(HomeMessage::OpenCalendarWeek),
            theme,
        ),
    ]
    .spacing(SPACE_2)
    .align_y(Alignment::Center);

    column![
        editorial_heading(
            format!("{greeting}, {}", data.profile.display_name),
            subtitle,
            theme,
        ),
        quick_pills,
    ]
    .spacing(SPACE_3)
    .into()
}

fn view_bento_hero_row<'a, Message: 'a>(
    data: &'a HomeDashboardData,
    theme: EstudyTheme,
) -> Element<'a, Message> {
    let pulse_card = view_pulse_sparkline_card(data, theme);
    let heatmap_card = view_annual_heatmap_card(data, theme);

    row![pulse_card, heatmap_card]
        .spacing(SPACE_4)
        .width(Length::Fill)
        .into()
}

fn view_pulse_sparkline_card<'a, Message: 'a>(
    data: &'a HomeDashboardData,
    theme: EstudyTheme,
) -> Element<'a, Message> {
    let recent_days: Vec<f32> = data
        .heatmap
        .days
        .iter()
        .rev()
        .take(14)
        .rev()
        .map(|&count| f32::from(count))
        .collect();
    let recent_sum: u32 = data
        .heatmap
        .days
        .iter()
        .rev()
        .take(14)
        .map(|&count| u32::from(count))
        .sum();
    let spark_data = SparklineData::new(recent_days);

    tonal_card(
        column![
            row![
                text("Pulso de Foco · 14 dias")
                    .size(12)
                    .font(FONT_SANS_MEDIUM)
                    .color(theme.on_accent),
                container(
                    text(format!("{}d sequência", data.profile.streak_days))
                        .size(11)
                        .font(FONT_MONO)
                        .color(theme.on_accent),
                )
                .align_x(Alignment::End)
                .width(Length::Fill),
            ]
            .align_y(Alignment::Center),
            row![
                text(format!("{recent_sum}"))
                    .size(30)
                    .font(FONT_MONO_MEDIUM)
                    .color(theme.on_accent),
                text("ações concluídas nas últimas 2 semanas")
                    .size(12)
                    .color(Color {
                        a: 0.85,
                        ..theme.on_accent
                    }),
            ]
            .spacing(SPACE_2)
            .align_y(Alignment::End),
            SparklineCanvas {
                data: spark_data,
                stroke_color: theme.on_accent,
                fill_color: Color {
                    a: 0.16,
                    ..theme.on_accent
                },
                grid_color: Color {
                    a: 0.12,
                    ..theme.on_accent
                },
            }
            .into_canvas()
            .width(Length::Fill)
            .height(Length::Fixed(86.0)),
        ]
        .spacing(SPACE_2),
        theme,
    )
    .height(Length::Fixed(198.0))
    .width(Length::FillPortion(5))
    .into()
}

fn view_annual_heatmap_card<'a, Message: 'a>(
    data: &'a HomeDashboardData,
    theme: EstudyTheme,
) -> Element<'a, Message> {
    let heatmap_legend = row![
        text("Menos").size(10).color(theme.text_muted),
        color_dot(theme.surface_soft),
        color_dot(Color {
            a: 0.30,
            ..theme.accent
        }),
        color_dot(Color {
            a: 0.55,
            ..theme.accent
        }),
        color_dot(Color {
            a: 0.80,
            ..theme.accent
        }),
        color_dot(theme.accent),
        text("Mais").size(10).color(theme.text_muted),
    ]
    .spacing(4.0)
    .align_y(Alignment::Center);

    card(
        column![
            row![
                section_header(Icon::Flame, "Ritmo anual (365 dias)", theme),
                container(
                    text(format!("{} no último ano", data.heatmap.total_actions()))
                        .size(12)
                        .font(FONT_MONO)
                        .color(theme.text_muted),
                )
                .align_x(Alignment::End)
                .width(Length::Fill),
            ]
            .align_y(Alignment::Center),
            canvas(HeatmapGrid {
                data: &data.heatmap,
                theme,
            })
            .width(Length::Fill)
            .height(Length::Fixed(112.0)),
            container(heatmap_legend)
                .align_x(Alignment::End)
                .width(Length::Fill),
        ]
        .spacing(SPACE_2),
        theme,
    )
    .height(Length::Fixed(198.0))
    .width(Length::FillPortion(7))
    .into()
}

fn view_projects_section<'a, Message: Clone + 'a>(
    projects: &'a [ProjectCardSummary],
    theme: EstudyTheme,
    map_home: impl Fn(HomeMessage) -> Message + Clone + 'a,
    map_route: impl Fn(AppRoute) -> Message + 'a,
) -> Element<'a, Message> {
    let header = row![
        row![
            section_header(Icon::FolderKanban, "Projetos", theme),
            container(
                text(format!("{}", projects.len()))
                    .size(11)
                    .font(FONT_MONO)
                    .color(theme.text_muted),
            )
            .padding([2.0, 8.0])
            .style(move |_| container::Style {
                background: Some(theme.surface_soft.into()),
                border: Border {
                    radius: RADIUS_PILL.into(),
                    ..Border::default()
                },
                ..container::Style::default()
            }),
        ]
        .spacing(SPACE_2)
        .align_y(Alignment::Center),
        container(
            button(
                row![
                    icon_svg(Icon::Plus, 13.0, theme.on_accent),
                    text("Abrir projetos")
                        .size(12)
                        .font(FONT_SANS_MEDIUM)
                        .color(theme.on_accent),
                ]
                .spacing(SPACE_1)
                .align_y(Alignment::Center),
            )
            .on_press(map_route(AppRoute::Projects))
            .padding([5.0, 12.0])
            .style(move |_, status| theme.pill_button_style(true, status)),
        )
        .align_x(Alignment::End)
        .width(Length::Fill),
    ]
    .align_y(Alignment::Center);

    let mut rows = column![].spacing(SPACE_3);
    for (pair_idx, pair) in projects.chunks(2).enumerate() {
        let mut line = row![].spacing(SPACE_3).width(Length::Fill);
        for (offset, project) in pair.iter().enumerate() {
            line = line.push(view_project_tile_card(
                pair_idx * 2 + offset,
                project,
                theme,
                map_home.clone(),
            ));
        }
        if pair.len() == 1 {
            line = line.push(container(text("")).width(Length::Fill));
        }
        rows = rows.push(line);
    }

    card(
        column![header, hairline_divider(theme), rows].spacing(SPACE_3),
        theme,
    )
    .into()
}

fn view_project_tile_card<'a, Message: Clone + 'a>(
    idx: usize,
    project: &'a ProjectCardSummary,
    theme: EstudyTheme,
    map_home: impl Fn(HomeMessage) -> Message + 'a,
) -> Element<'a, Message> {
    let icons = [
        Icon::Heart,
        Icon::GraduationCap,
        Icon::BookOpen,
        Icon::FolderKanban,
        Icon::Target,
        Icon::Layers,
    ];
    let category = CategoryColor::from_index(idx);
    let tile_icon = icons[idx % icons.len()];
    let total = project.todo_count + project.in_progress_count + project.done_count;
    let ratio = if total == 0 {
        0.0
    } else {
        f32::from(project.done_count) / f32::from(total)
    };

    let counts_row = row![
        color_dot(CategoryColor::Terracotta.solid(theme)),
        text(format!("{} a fazer", project.todo_count))
            .size(11)
            .font(FONT_MONO)
            .color(theme.text_muted),
        color_dot(CategoryColor::Ochre.solid(theme)),
        text(format!("{} em curso", project.in_progress_count))
            .size(11)
            .font(FONT_MONO)
            .color(theme.text_muted),
        color_dot(CategoryColor::Sage.solid(theme)),
        text(format!("{} feitos", project.done_count))
            .size(11)
            .font(FONT_MONO)
            .color(theme.text_muted),
    ]
    .spacing(6.0)
    .align_y(Alignment::Center);

    button(
        soft_surface(
            column![
                row![
                    category_tile(tile_icon, category, 32.0, theme),
                    column![
                        text(&project.title)
                            .size(14)
                            .font(FONT_SANS_SEMIBOLD)
                            .color(theme.text_primary),
                        text(&project.status_label)
                            .size(11)
                            .color(category.solid(theme)),
                    ]
                    .spacing(1.0)
                    .width(Length::Fill),
                ]
                .spacing(SPACE_2)
                .align_y(Alignment::Center),
                progress_track(ratio, category.solid(theme), theme),
                counts_row,
            ]
            .spacing(SPACE_2),
            theme,
        )
        .padding(SPACE_4)
        .width(Length::Fill),
    )
    .on_press(map_home(HomeMessage::PreviewProject(project.title.clone())))
    .padding(0.0)
    .width(Length::Fill)
    .style(move |_, status| theme.ghost_button_style(status))
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
