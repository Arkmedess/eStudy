//! Renderização editorial e responsiva da Home Pessoal:
//! Hero orgânico com ações rápidas, 3 cartões contrastantes (Foco, Constância e Mini-Calendário) e 3 colunas acionáveis.

use chrono::{Datelike, Duration, Months, NaiveDate};
use estudy_domain::{TaskStatus, days_in_month, expand_calendar_items};
use estudy_ui_kit::widgets::{
    card, color_dot, hairline_divider, icon_svg, illustrated_svg_themed, radio_check_circle,
    shaped_text, strikethrough_text,
};
use estudy_ui_kit::{
    CategoryColor, EstudyTheme, FONT_MONO, FONT_MONO_MEDIUM, FONT_SANS, FONT_SANS_MEDIUM,
    FONT_SANS_SEMIBOLD, FONT_SERIF, FONT_SERIF_MEDIUM, FONT_SIZE_2XL, FONT_SIZE_BASE,
    FONT_SIZE_DISPLAY, FONT_SIZE_LG, FONT_SIZE_MD, FONT_SIZE_SM, FONT_SIZE_XL, FONT_SIZE_XS, Icon,
    IllustratedIcon, RADIUS_PILL, SPACE_1, SPACE_2, SPACE_3, SPACE_4, SPACE_5, SPACE_6,
    SparklineCanvas, SparklineData, ThemeMode,
};
use iced::widget::{button, column, container, row, scrollable, svg, text};
use iced::{Alignment, Border, Color, Element, Length};
use std::collections::BTreeSet;
use std::sync::LazyLock;

use super::state::{HomeDashboardData, HomeLoadState, HomeMessage};
use crate::command_palette::AppRoute;
use crate::features::calendar::CalendarState;
use crate::features::focus::FocusState;
use crate::features::habits::{HabitsMessage, HabitsState};
use crate::features::projects::modals::month_name_pt;
use crate::features::projects::{ProjectsMessage, ProjectsState, project_color_index};
use crate::features::second_brain::{SecondBrainMessage, SecondBrainState};

static HERO_LANDSCAPE_LIGHT_SVG: LazyLock<svg::Handle> = LazyLock::new(|| {
    let art = r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 360 132"><circle cx="282" cy="40" r="30" fill="#F2C864" opacity="0.20"/><circle cx="282" cy="40" r="19" fill="#EBB94B" opacity="0.58"/><path d="M96 42 Q108 32 122 40 Q132 34 142 42" fill="none" stroke="#8C998A" stroke-width="1.4" stroke-linecap="round" opacity="0.55"/><path d="M52 132 C125 96, 200 92, 276 116 C312 127, 338 118, 360 108 L360 132 Z" fill="#9EB09A" opacity="0.28"/><path d="M136 132 C204 88, 276 82, 360 100 L360 132 Z" fill="#5C7860" opacity="0.34"/><path d="M214 132 C264 102, 314 100, 360 112 L360 132 Z" fill="#374C3B" opacity="0.28"/><g transform="translate(296, 52)"><line x1="18" y1="34" x2="18" y2="72" stroke="#4D3D30" stroke-width="2.8" stroke-linecap="round"/><ellipse cx="18" cy="32" rx="21" ry="10.5" fill="#374C3B" opacity="0.88"/><ellipse cx="18" cy="20" rx="15" ry="7.5" fill="#517056" opacity="0.92"/></g><g transform="translate(248, 68)"><line x1="12" y1="26" x2="12" y2="54" stroke="#4D3D30" stroke-width="2.2" stroke-linecap="round"/><ellipse cx="12" cy="24" rx="14" ry="7.5" fill="#3E5642" opacity="0.78"/></g></svg>"##;
    svg::Handle::from_memory(art.as_bytes().to_vec())
});

static HERO_LANDSCAPE_DARK_SVG: LazyLock<svg::Handle> = LazyLock::new(|| {
    let art = r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 360 132"><circle cx="282" cy="40" r="30" fill="#D8B36A" opacity="0.16"/><circle cx="282" cy="40" r="18" fill="#E6C77E" opacity="0.52"/><circle cx="176" cy="28" r="1.4" fill="#D8E2D4" opacity="0.55"/><circle cx="218" cy="20" r="1.1" fill="#D8E2D4" opacity="0.45"/><circle cx="326" cy="24" r="1.3" fill="#D8E2D4" opacity="0.50"/><path d="M52 132 C125 96, 200 92, 276 116 C312 127, 338 118, 360 108 L360 132 Z" fill="#415444" opacity="0.36"/><path d="M136 132 C204 88, 276 82, 360 100 L360 132 Z" fill="#2D3D30" opacity="0.48"/><path d="M214 132 C264 102, 314 100, 360 112 L360 132 Z" fill="#5E7A62" opacity="0.28"/><g transform="translate(296, 52)"><line x1="18" y1="34" x2="18" y2="72" stroke="#8F7C6A" stroke-width="2.8" stroke-linecap="round"/><ellipse cx="18" cy="32" rx="21" ry="10.5" fill="#709474" opacity="0.88"/><ellipse cx="18" cy="20" rx="15" ry="7.5" fill="#94B696" opacity="0.90"/></g><g transform="translate(248, 68)"><line x1="12" y1="26" x2="12" y2="54" stroke="#8F7C6A" stroke-width="2.2" stroke-linecap="round"/><ellipse cx="12" cy="24" rx="14" ry="7.5" fill="#688A6C" opacity="0.78"/></g></svg>"##;
    svg::Handle::from_memory(art.as_bytes().to_vec())
});

/// Renderiza a Home cobrindo Loading, Error, Empty e Populated de forma responsiva.
#[allow(clippy::too_many_arguments)]
pub fn view_home<'a, Message: Clone + 'a>(
    state: &'a HomeLoadState,
    projects: &'a ProjectsState,
    habits: &'a HabitsState,
    brain: &'a SecondBrainState,
    calendar: &'a CalendarState,
    focus: &'a FocusState,
    today: NaiveDate,
    greeting: &'a str,
    content_width: f32,
    theme: EstudyTheme,
    map_home: impl Fn(HomeMessage) -> Message + Clone + 'a,
    map_projects: impl Fn(ProjectsMessage) -> Message + Clone + 'a,
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
        HomeLoadState::Ready(data) => view_editorial_home(
            data,
            projects,
            habits,
            brain,
            calendar,
            focus,
            today,
            greeting,
            content_width,
            theme,
            map_home,
            map_projects,
            map_habits,
            map_brain,
            map_route,
        ),
    }
}

fn view_loading_state<'a, Message: 'a>(theme: EstudyTheme) -> Element<'a, Message> {
    let skeleton = card(
        column![
            shaped_text(
                "Carregando seu espaço pessoal...",
                FONT_SIZE_LG,
                FONT_SERIF_MEDIUM,
                theme.text_primary,
            ),
            shaped_text(
                "Preparando tarefas, hábitos e anotações do dia.",
                FONT_SIZE_MD,
                FONT_SANS,
                theme.text_muted,
            ),
        ]
        .spacing(SPACE_2),
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
            icon_svg(Icon::RotateCcw, 16.0, theme.on_accent),
            shaped_text(
                "Tentar novamente",
                FONT_SIZE_BASE,
                FONT_SANS_MEDIUM,
                theme.on_accent,
            ),
        ]
        .spacing(SPACE_2)
        .align_y(Alignment::Center),
    )
    .on_press(map_home(HomeMessage::RetryLoad))
    .padding([SPACE_2, SPACE_4])
    .style(move |_, status| theme.primary_button_style(status));

    let err_card = card(
        column![
            shaped_text(
                "Não foi possível carregar o Início",
                FONT_SIZE_LG,
                FONT_SERIF_MEDIUM,
                theme.text_primary,
            ),
            shaped_text(err_msg, FONT_SIZE_MD, FONT_SANS, theme.text_muted),
            retry_btn,
        ]
        .spacing(SPACE_3),
        theme,
    )
    .width(Length::Fixed(500.0));

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
            icon_svg(Icon::Plus, 16.0, theme.on_accent),
            shaped_text(
                "Preparar painel inicial",
                FONT_SIZE_BASE,
                FONT_SANS_MEDIUM,
                theme.on_accent,
            ),
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
                .size(FONT_SIZE_2XL)
                .font(FONT_SERIF)
                .color(theme.text_primary)
                .shaping(text::Shaping::Advanced),
            shaped_text(
                "Seu espaço está pronto. Crie o primeiro projeto ou use Ctrl+K para capturar uma tarefa.",
                FONT_SIZE_MD,
                FONT_SANS,
                theme.text_muted,
            ),
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
fn view_editorial_home<'a, Message: Clone + 'a>(
    data: &'a HomeDashboardData,
    projects: &'a ProjectsState,
    habits: &'a HabitsState,
    brain: &'a SecondBrainState,
    calendar: &'a CalendarState,
    focus: &'a FocusState,
    today: NaiveDate,
    greeting: &'a str,
    content_width: f32,
    theme: EstudyTheme,
    map_home: impl Fn(HomeMessage) -> Message + Clone + 'a,
    map_projects: impl Fn(ProjectsMessage) -> Message + Clone + 'a,
    map_habits: impl Fn(HabitsMessage) -> Message + Clone + 'a,
    map_brain: impl Fn(SecondBrainMessage) -> Message + Clone + 'a,
    map_route: impl Fn(AppRoute) -> Message + Clone + 'a,
) -> Element<'a, Message> {
    let open_tasks = projects
        .tasks
        .iter()
        .filter(|t| t.status != TaskStatus::Done)
        .count();
    let (done_habits, total_habits) = habits.today_progress(today);
    let pending_habits = total_habits.saturating_sub(done_habits);

    let hero = view_hero_banner(
        data,
        open_tasks,
        pending_habits,
        today,
        greeting,
        content_width,
        theme,
        map_home.clone(),
    );

    let compact = content_width < 880.0;
    let mini_cal = view_mini_calendar_card(
        projects,
        calendar,
        today,
        content_width,
        theme,
        map_home.clone(),
        map_projects.clone(),
    );
    let warm_card = view_warm_focus_card(data, focus, theme, map_home.clone());
    let pulse_card = view_dark_pulse_card(data, habits, brain, today, theme);

    let contrast_section: Element<'a, Message> = if compact {
        column![
            row![warm_card, pulse_card]
                .spacing(SPACE_4)
                .width(Length::Fill),
            mini_cal,
        ]
        .spacing(SPACE_4)
        .width(Length::Fill)
        .into()
    } else {
        row![warm_card, pulse_card, mini_cal]
            .spacing(SPACE_4)
            .width(Length::Fill)
            .into()
    };

    let bottom_section = assemble_home_bottom_section(
        projects,
        habits,
        brain,
        today,
        compact,
        theme,
        map_home,
        map_projects,
        map_habits,
        map_brain,
        map_route,
    );

    scrollable(
        container(
            container(
                column![hero, contrast_section, bottom_section]
                    .spacing(SPACE_4)
                    .width(Length::Fill),
            )
            .max_width(1420.0)
            .width(Length::Fill),
        )
        .padding([SPACE_5, SPACE_6])
        .center_x(Length::Fill)
        .width(Length::Fill),
    )
    .direction(scrollable::Direction::Vertical(
        scrollable::Scrollbar::new().width(5).scroller_width(5),
    ))
    .style(move |_, status| theme.subtle_scrollbar_style(status))
    .height(Length::Fill)
    .into()
}

#[allow(clippy::too_many_arguments)]
fn assemble_home_bottom_section<'a, Message: Clone + 'a>(
    projects: &'a ProjectsState,
    habits: &'a HabitsState,
    brain: &'a SecondBrainState,
    today: NaiveDate,
    compact: bool,
    theme: EstudyTheme,
    map_home: impl Fn(HomeMessage) -> Message + 'a,
    map_projects: impl Fn(ProjectsMessage) -> Message + 'a,
    map_habits: impl Fn(HabitsMessage) -> Message + 'a,
    map_brain: impl Fn(SecondBrainMessage) -> Message + 'a,
    map_route: impl Fn(AppRoute) -> Message + Clone + 'a,
) -> Element<'a, Message> {
    let pair_height = if compact {
        Length::Fixed(282.0)
    } else {
        Length::Fixed(308.0)
    };
    let brain_height = if compact {
        Length::Shrink
    } else {
        Length::Fixed(308.0)
    };

    let tasks_digest = view_home_tasks_digest(
        projects,
        pair_height,
        theme,
        map_home,
        map_projects,
        map_route.clone(),
    );
    let habits_digest = view_home_habits_digest(
        habits,
        today,
        pair_height,
        theme,
        map_habits,
        map_route.clone(),
    );
    let brain_digest = view_home_brain_digest(brain, brain_height, theme, map_brain, map_route);

    if compact {
        column![
            row![tasks_digest, habits_digest]
                .spacing(SPACE_4)
                .width(Length::Fill),
            brain_digest,
        ]
        .spacing(SPACE_4)
        .width(Length::Fill)
        .into()
    } else {
        row![tasks_digest, habits_digest, brain_digest]
            .spacing(SPACE_4)
            .width(Length::Fill)
            .into()
    }
}

#[allow(clippy::too_many_arguments)]
fn view_hero_banner<'a, Message: Clone + 'a>(
    data: &'a HomeDashboardData,
    open_tasks: usize,
    pending_habits: usize,
    today: NaiveDate,
    greeting: &'a str,
    content_width: f32,
    theme: EstudyTheme,
    map_home: impl Fn(HomeMessage) -> Message + 'a,
) -> Element<'a, Message> {
    let crumb = format!(
        "ESTUDY  /  INÍCIO  ·  {} DE {}",
        today.day(),
        month_name_pt(today.month()).to_uppercase()
    );

    let habits_phrase = if pending_habits == 0 {
        "todos os hábitos concluídos".to_string()
    } else if pending_habits == 1 {
        "1 hábito pendente".to_string()
    } else {
        format!("{pending_habits} hábitos pendentes")
    };
    let tasks_phrase = if open_tasks == 1 {
        "1 tarefa aberta".to_string()
    } else {
        format!("{open_tasks} tarefas abertas")
    };
    let live_summary = format!(
        "Hoje você tem {tasks_phrase}, {habits_phrase} e {} dias de constância.",
        data.profile.streak_days
    );

    let quick_actions = row![
        hero_quick_pill(
            "+ Tarefa",
            None,
            map_home(HomeMessage::QuickCreateTask),
            theme
        ),
        hero_quick_pill(
            "+ Hábito",
            None,
            map_home(HomeMessage::QuickCreateHabit),
            theme
        ),
        hero_quick_pill(
            "+ Compromisso",
            None,
            map_home(HomeMessage::QuickCreateEvent),
            theme,
        ),
        hero_quick_pill(
            "+ Nota",
            None,
            map_home(HomeMessage::QuickCreateNote),
            theme
        ),
        hero_quick_pill(
            "Timeline Hoje",
            Some(Icon::Clock),
            map_home(HomeMessage::OpenTodayTimeline),
            theme,
        ),
    ]
    .spacing(SPACE_2)
    .align_y(Alignment::Center)
    .wrap()
    .vertical_spacing(SPACE_2);

    let left_col = column![
        shaped_text(crumb, 11.5, FONT_SANS_SEMIBOLD, theme.text_muted),
        text(format!("{greeting}, {}", data.profile.display_name))
            .size(36.0)
            .font(FONT_SERIF)
            .color(theme.text_primary)
            .shaping(text::Shaping::Advanced),
        shaped_text(live_summary, FONT_SIZE_MD, FONT_SANS, theme.text_muted),
        container(quick_actions).padding([SPACE_1, 0.0]),
    ]
    .spacing(6.0)
    .width(Length::FillPortion(7));

    let inner: Element<'a, Message> = if content_width >= 760.0 {
        let landscape_handle = match theme.mode {
            ThemeMode::Light => HERO_LANDSCAPE_LIGHT_SVG.clone(),
            ThemeMode::Dark => HERO_LANDSCAPE_DARK_SVG.clone(),
        };
        let illustration = container(
            svg(landscape_handle)
                .width(Length::Fixed(310.0))
                .height(Length::Fixed(118.0)),
        )
        .align_x(Alignment::End)
        .align_y(Alignment::End)
        .width(Length::FillPortion(5));

        row![left_col, illustration]
            .align_y(Alignment::Center)
            .width(Length::Fill)
            .into()
    } else {
        left_col.width(Length::Fill).into()
    };

    let (grad_start, grad_end) = (theme.panel, theme.surface_soft);
    container(inner)
        .padding([SPACE_4, SPACE_5])
        .width(Length::Fill)
        .style(move |_| theme.gradient_card_style(grad_start, grad_end, 1.9, theme.text_primary))
        .into()
}

fn hero_quick_pill<'a, Message: Clone + 'a>(
    label: &'static str,
    icon: Option<Icon>,
    on_press: Message,
    theme: EstudyTheme,
) -> Element<'a, Message> {
    let mut content = row![].spacing(5.0).align_y(Alignment::Center);
    if let Some(ic) = icon {
        content = content.push(icon_svg(ic, 14.0, theme.accent));
    }
    content = content.push(shaped_text(
        label,
        FONT_SIZE_SM,
        FONT_SANS_MEDIUM,
        theme.text_primary,
    ));

    button(content)
        .on_press(on_press)
        .padding([6.0, 14.0])
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
                color: if matches!(status, button::Status::Hovered) {
                    theme.accent
                } else {
                    theme.border_subtle
                },
                width: 1.0,
                radius: RADIUS_PILL.into(),
            },
            ..button::Style::default()
        })
        .into()
}

fn view_warm_focus_card<'a, Message: Clone + 'a>(
    data: &'a HomeDashboardData,
    focus: &'a FocusState,
    theme: EstudyTheme,
    map_home: impl Fn(HomeMessage) -> Message + 'a,
) -> Element<'a, Message> {
    let (warm_start, warm_end) = match theme.mode {
        ThemeMode::Light => (
            Color::from_rgb8(0xC2, 0x5D, 0x33),
            Color::from_rgb8(0x9E, 0x43, 0x1F),
        ),
        ThemeMode::Dark => (
            Color::from_rgb8(0x88, 0x3F, 0x20),
            Color::from_rgb8(0x66, 0x2B, 0x14),
        ),
    };
    let cream_fg = Color::from_rgb8(0xFF, 0xFA, 0xF3);
    let muted_fg = Color {
        a: 0.85,
        ..cream_fg
    };

    let running = focus.needs_ticker();
    let rem = focus.timer.remaining_seconds();
    let first_pin_id = data
        .focus_items
        .first()
        .map_or_else(|| "foco".to_string(), |p| p.id.clone());

    let (hero_label, hero_value, hero_subtitle) = warm_focus_hero_copy(data, focus, running, rem);

    let start_pill = warm_focus_action_button(
        warm_end,
        cream_fg,
        if running {
            "Abrir foco"
        } else {
            "Iniciar foco 25m"
        },
        if running {
            map_home(HomeMessage::StartFocusWithMinutes(25))
        } else {
            map_home(HomeMessage::StartFocusForPin(first_pin_id))
        },
    );

    container(
        column![
            row![
                text("Foco & Ritmo")
                    .size(FONT_SIZE_XL)
                    .font(FONT_SERIF)
                    .color(cream_fg)
                    .shaping(text::Shaping::Advanced)
                    .width(Length::Fill),
                shaped_text(
                    format!("{}d seguidos", data.profile.streak_days),
                    FONT_SIZE_XS,
                    FONT_MONO,
                    muted_fg,
                ),
            ]
            .spacing(SPACE_2)
            .align_y(Alignment::Center),
            container(
                column![
                    shaped_text(hero_label, 11.0, FONT_SANS_SEMIBOLD, muted_fg),
                    text(hero_value)
                        .size(38.0)
                        .font(if running { FONT_MONO } else { FONT_SERIF })
                        .color(cream_fg)
                        .shaping(text::Shaping::Advanced),
                    shaped_text(hero_subtitle, FONT_SIZE_SM, FONT_SANS, muted_fg),
                ]
                .spacing(2.0),
            )
            .padding([SPACE_2, 0.0])
            .height(Length::Fill),
            row![
                start_pill,
                container(shaped_text(
                    format!("{} na fila", data.focus_items.len()),
                    FONT_SIZE_XS,
                    FONT_MONO,
                    muted_fg,
                ))
                .align_x(Alignment::End)
                .width(Length::Fill),
            ]
            .spacing(SPACE_2)
            .align_y(Alignment::Center),
        ]
        .spacing(SPACE_2)
        .height(Length::Fill),
    )
    .padding(SPACE_5)
    .height(Length::Fixed(250.0))
    .width(Length::FillPortion(4))
    .style(move |_| theme.gradient_card_style(warm_start, warm_end, 2.35, cream_fg))
    .into()
}

fn warm_focus_hero_copy(
    data: &HomeDashboardData,
    focus: &FocusState,
    running: bool,
    rem: u32,
) -> (&'static str, String, String) {
    if running {
        let active_name = focus
            .active_target_title
            .as_deref()
            .unwrap_or("Sessão em andamento");
        return (
            "EM FOCO AGORA",
            format!("{:02}:{:02}", rem / 60, rem % 60),
            active_name.to_string(),
        );
    }

    let next_title = data
        .focus_items
        .first()
        .map_or("Pronto para um bloco de 25 min", |p| p.title.as_str());
    (
        "TEMPO FOCADO HOJE",
        format!("{} min", focus.today_focused_minutes),
        next_title.to_string(),
    )
}

fn view_dark_pulse_card<'a, Message: 'a>(
    data: &'a HomeDashboardData,
    habits: &'a HabitsState,
    brain: &'a SecondBrainState,
    today: NaiveDate,
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
    let adh = habits.monthly_overall_pct(today);

    let (pulse_start, pulse_end, light_fg) = pulse_card_palette(theme);
    let sub_fg = Color {
        a: 0.80,
        ..light_fg
    };

    let stats_footer = row![
        mini_dark_metric(
            &format!("{}", data.heatmap.total_actions()),
            "NO ANO",
            light_fg,
            sub_fg
        ),
        mini_dark_metric(&format!("{adh}%"), "HÁBITOS MÊS", light_fg, sub_fg),
        mini_dark_metric(&format!("{}", brain.notes.len()), "NOTAS", light_fg, sub_fg),
    ]
    .spacing(SPACE_4)
    .align_y(Alignment::Center);

    container(
        column![
            row![
                text("Constância")
                    .size(FONT_SIZE_XL)
                    .font(FONT_SERIF)
                    .color(light_fg)
                    .shaping(text::Shaping::Advanced)
                    .width(Length::Fill),
                shaped_text("ÚLTIMOS 14 DIAS", 11.0, FONT_MONO, sub_fg),
            ]
            .align_y(Alignment::Center),
            row![
                text(format!("{recent_sum}"))
                    .size(FONT_SIZE_DISPLAY)
                    .font(FONT_SERIF)
                    .color(light_fg)
                    .shaping(text::Shaping::Advanced),
                container(shaped_text(
                    "ações concluídas",
                    FONT_SIZE_SM,
                    FONT_SANS,
                    sub_fg
                ))
                .padding(iced::Padding {
                    bottom: 6.0,
                    ..iced::Padding::ZERO
                }),
            ]
            .spacing(SPACE_2)
            .align_y(Alignment::End),
            SparklineCanvas {
                data: spark_data,
                stroke_color: light_fg,
                fill_color: Color {
                    a: 0.30,
                    ..light_fg
                },
                grid_color: Color {
                    a: 0.16,
                    ..light_fg
                },
            }
            .into_canvas()
            .width(Length::Fill)
            .height(Length::Fixed(78.0)),
            stats_footer,
        ]
        .spacing(SPACE_2)
        .height(Length::Fill),
    )
    .padding(SPACE_5)
    .height(Length::Fixed(250.0))
    .width(Length::FillPortion(5))
    .style(move |_| theme.gradient_card_style(pulse_start, pulse_end, 2.3, light_fg))
    .into()
}

fn pulse_card_palette(theme: EstudyTheme) -> (Color, Color, Color) {
    match theme.mode {
        ThemeMode::Light => {
            let darker = Color::from_rgb(
                (theme.accent.r * 0.80).clamp(0.0, 1.0),
                (theme.accent.g * 0.80).clamp(0.0, 1.0),
                (theme.accent.b * 0.80).clamp(0.0, 1.0),
            );
            (theme.accent, darker, theme.on_accent)
        }
        ThemeMode::Dark => (
            Color::from_rgb8(0x22, 0x31, 0x25),
            Color::from_rgb8(0x17, 0x22, 0x19),
            Color::from_rgb8(0xF2, 0xF7, 0xF0),
        ),
    }
}

fn mini_dark_metric<'a, Message: 'a>(
    val: &str,
    label: &'static str,
    fg: Color,
    sub: Color,
) -> Element<'a, Message> {
    column![
        shaped_text(val.to_string(), FONT_SIZE_MD, FONT_MONO_MEDIUM, fg),
        shaped_text(label, 10.5, FONT_SANS_SEMIBOLD, sub),
    ]
    .spacing(1.0)
    .into()
}

fn view_mini_calendar_card<'a, Message: Clone + 'a>(
    projects: &'a ProjectsState,
    calendar: &'a CalendarState,
    today: NaiveDate,
    content_width: f32,
    theme: EstudyTheme,
    map_home: impl Fn(HomeMessage) -> Message + 'a,
    map_projects: impl Fn(ProjectsMessage) -> Message + 'a,
) -> Element<'a, Message> {
    let year = projects.calendar_year;
    let month = projects.calendar_month;
    let total_days = days_in_month(year, month);
    let first = NaiveDate::from_ymd_opt(year, month, 1);
    let offset = first.map_or(0, |d| d.weekday().num_days_from_sunday() as usize);

    let busy_days = collect_month_busy_days(projects, calendar, year, month, total_days);
    let mut cells: Vec<Option<NaiveDate>> = vec![None; offset];
    for d in 1..=total_days {
        cells.push(NaiveDate::from_ymd_opt(year, month, d));
    }

    let rows = view_mini_calendar_rows(&cells, &busy_days, today, theme, &map_home);

    let month_label = format_mini_calendar_title(month, year, content_width);
    let bottom_nav = row![
        button(icon_svg(Icon::ChevronLeft, 14.0, theme.text_muted))
            .on_press(map_projects(ProjectsMessage::ShiftCalendarMonth(-1)))
            .padding(3.0)
            .style(move |_, status| theme.subtle_button_style(status)),
        container(shaped_text(
            month_label,
            FONT_SIZE_XS,
            FONT_SANS_SEMIBOLD,
            theme.text_primary,
        ))
        .center_x(Length::Fill)
        .width(Length::Fill),
        button(icon_svg(Icon::ChevronRight, 14.0, theme.text_muted))
            .on_press(map_projects(ProjectsMessage::ShiftCalendarMonth(1)))
            .padding(3.0)
            .style(move |_, status| theme.subtle_button_style(status)),
    ]
    .align_y(Alignment::Center);

    let card_width = if content_width < 880.0 {
        Length::Fill
    } else {
        Length::FillPortion(3)
    };
    let card_height = if content_width < 880.0 {
        Length::Shrink
    } else {
        Length::Fixed(250.0)
    };

    card(
        column![
            rows,
            container(bottom_nav)
                .align_y(Alignment::End)
                .height(Length::Fill),
        ]
        .spacing(SPACE_1)
        .height(if content_width < 880.0 {
            Length::Shrink
        } else {
            Length::Fill
        }),
        theme,
    )
    .padding(SPACE_4)
    .height(card_height)
    .width(card_width)
    .into()
}

fn view_mini_calendar_rows<'a, Message: Clone + 'a>(
    cells: &[Option<NaiveDate>],
    busy_days: &std::collections::BTreeSet<u32>,
    today: NaiveDate,
    theme: EstudyTheme,
    map_home: &impl Fn(HomeMessage) -> Message,
) -> Element<'a, Message> {
    let weekdays = ["D", "S", "T", "Q", "Q", "S", "S"];
    let mut head = row![].spacing(2.0).width(Length::Fill);
    for w in weekdays {
        head = head.push(
            container(shaped_text(w, 11.0, FONT_SANS_SEMIBOLD, theme.text_muted))
                .center_x(Length::Fill)
                .width(Length::FillPortion(1)),
        );
    }

    let mut rows = column![head].spacing(2.0);
    for week in cells.chunks(7) {
        let mut line = row![].spacing(2.0).width(Length::Fill);
        for slot in week {
            if let Some(date) = slot {
                let is_today = *date == today;
                let has_item = busy_days.contains(&date.day());
                let day_cell = column![
                    shaped_text(
                        format!("{}", date.day()),
                        FONT_SIZE_XS,
                        FONT_MONO,
                        if is_today {
                            theme.on_accent
                        } else {
                            theme.text_primary
                        },
                    ),
                    mini_day_indicator(has_item, is_today, theme),
                ]
                .spacing(1.0)
                .align_x(Alignment::Center);

                let pill_box = container(day_cell)
                    .center_x(Length::Fixed(32.0))
                    .center_y(Length::Fixed(27.0))
                    .style(move |_| {
                        if is_today {
                            container::Style {
                                background: Some(theme.accent.into()),
                                border: Border {
                                    radius: RADIUS_PILL.into(),
                                    ..Border::default()
                                },
                                ..container::Style::default()
                            }
                        } else {
                            container::Style::default()
                        }
                    });

                line = line.push(
                    button(
                        container(pill_box)
                            .center_x(Length::Fill)
                            .width(Length::Fill),
                    )
                    .on_press(map_home(HomeMessage::OpenCalendarDate(*date)))
                    .width(Length::FillPortion(1))
                    .height(Length::Fixed(27.0))
                    .padding(0.0)
                    .style(move |_, status| theme.subtle_button_style(status)),
                );
            } else {
                line = line.push(
                    container(text(""))
                        .height(Length::Fixed(27.0))
                        .width(Length::FillPortion(1)),
                );
            }
        }
        for _ in week.len()..7 {
            line = line.push(
                container(text(""))
                    .height(Length::Fixed(27.0))
                    .width(Length::FillPortion(1)),
            );
        }
        rows = rows.push(line);
    }
    rows.into()
}

fn collect_month_busy_days(
    projects: &ProjectsState,
    calendar: &CalendarState,
    year: i32,
    month: u32,
    total_days: u32,
) -> BTreeSet<u32> {
    let mut busy = BTreeSet::new();
    let Some(start) = NaiveDate::from_ymd_opt(year, month, 1) else {
        return busy;
    };
    let end = start
        .checked_add_months(Months::new(1))
        .map_or(start + Duration::days(i64::from(total_days)), |next| {
            next - Duration::days(1)
        });
    for occ in expand_calendar_items(&calendar.events, &projects.tasks, start, end) {
        if occ.date.year() == year && occ.date.month() == month {
            busy.insert(occ.date.day());
        }
    }
    busy
}

fn mini_day_indicator<'a, Message: 'a>(
    has_item: bool,
    is_today: bool,
    theme: EstudyTheme,
) -> Element<'a, Message> {
    let color = if has_item {
        if is_today {
            theme.on_accent
        } else {
            CategoryColor::Terracotta.solid(theme)
        }
    } else {
        Color::TRANSPARENT
    };
    container(text("").size(1))
        .width(Length::Fixed(3.5))
        .height(Length::Fixed(3.5))
        .style(move |_| container::Style {
            background: Some(color.into()),
            border: Border {
                radius: RADIUS_PILL.into(),
                ..Border::default()
            },
            ..container::Style::default()
        })
        .into()
}

fn format_mini_calendar_title(month: u32, year: i32, content_width: f32) -> String {
    let short = (880.0..1160.0).contains(&content_width);
    if short {
        let abbr = match month {
            1 => "Jan",
            2 => "Fev",
            3 => "Mar",
            4 => "Abr",
            5 => "Mai",
            6 => "Jun",
            7 => "Jul",
            8 => "Ago",
            9 => "Set",
            10 => "Out",
            11 => "Nov",
            _ => "Dez",
        };
        format!("{abbr} {year}")
    } else {
        let raw_m = month_name_pt(month);
        let mut chars = raw_m.chars();
        let cap_m = chars.next().map_or_else(String::new, |c| {
            format!("{}{}", c.to_uppercase(), chars.as_str())
        });
        format!("{cap_m} {year}")
    }
}

fn view_home_tasks_digest<'a, Message: Clone + 'a>(
    projects: &'a ProjectsState,
    card_height: Length,
    theme: EstudyTheme,
    map_home: impl Fn(HomeMessage) -> Message + 'a,
    map_projects: impl Fn(ProjectsMessage) -> Message + 'a,
    map_route: impl Fn(AppRoute) -> Message + 'a,
) -> Element<'a, Message> {
    let todo_c = projects
        .tasks
        .iter()
        .filter(|t| matches!(t.status, TaskStatus::Todo | TaskStatus::Backlog))
        .count();
    let doing_c = projects
        .tasks
        .iter()
        .filter(|t| t.status == TaskStatus::InProgress)
        .count();
    let done_c = projects
        .tasks
        .iter()
        .filter(|t| t.status == TaskStatus::Done)
        .count();

    let counters = row![
        mini_counter(todo_c, "A FAZER", theme),
        mini_counter(doing_c, "FAZENDO", theme),
        mini_counter(done_c, "FEITAS", theme),
    ]
    .spacing(SPACE_3);

    let mut list = column![].spacing(SPACE_1);
    for task in projects.tasks.iter().take(4) {
        list = list.push(view_home_task_digest_row(
            task,
            map_projects(ProjectsMessage::ToggleTaskDone(task.id.clone())),
            map_projects(ProjectsMessage::OpenEditTaskModal(task.id.clone())),
            theme,
        ));
    }

    let mut proj_pills = row![].spacing(6.0);
    for (idx, proj) in projects.projects.iter().take(3).enumerate() {
        let c_idx = project_color_index(&proj.color_token, idx);
        let dot = CategoryColor::from_index(c_idx).solid(theme);
        proj_pills = proj_pills.push(
            button(
                row![
                    color_dot(dot),
                    shaped_text(
                        &proj.title,
                        FONT_SIZE_XS,
                        FONT_SANS_MEDIUM,
                        theme.text_primary
                    ),
                ]
                .spacing(5.0)
                .align_y(Alignment::Center),
            )
            .on_press(map_home(HomeMessage::PreviewProject(proj.title.clone())))
            .padding([4.0, 10.0])
            .style(move |_, status| theme.ghost_button_style(status)),
        );
    }

    let column_height = if card_height == Length::Shrink {
        Length::Shrink
    } else {
        Length::Fill
    };

    card(
        column![
            digest_header(
                IllustratedIcon::Tasks,
                "Tarefas",
                "Ver todas",
                map_route(AppRoute::Projects),
                theme,
            ),
            counters,
            hairline_divider(theme),
            list,
            container(proj_pills.wrap().vertical_spacing(6.0))
                .align_y(Alignment::End)
                .height(column_height),
        ]
        .spacing(SPACE_3)
        .height(column_height),
        theme,
    )
    .height(card_height)
    .width(Length::FillPortion(1))
    .into()
}

fn mini_counter<'a, Message: 'a>(
    count: usize,
    label: &'static str,
    theme: EstudyTheme,
) -> Element<'a, Message> {
    column![
        text(format!("{count}"))
            .size(FONT_SIZE_2XL)
            .font(FONT_SERIF)
            .color(theme.text_primary)
            .shaping(text::Shaping::Advanced),
        shaped_text(label, 10.5, FONT_SANS_SEMIBOLD, theme.text_muted),
    ]
    .spacing(0.0)
    .align_x(Alignment::Center)
    .width(Length::FillPortion(1))
    .into()
}

fn view_home_habits_digest<'a, Message: Clone + 'a>(
    habits: &'a HabitsState,
    today: NaiveDate,
    card_height: Length,
    theme: EstudyTheme,
    map_habits: impl Fn(HabitsMessage) -> Message + 'a,
    map_route: impl Fn(AppRoute) -> Message + 'a,
) -> Element<'a, Message> {
    let (done, total) = habits.today_progress(today);
    let mut list = column![].spacing(SPACE_1);

    for habit in habits.habits.iter().take(4) {
        let cat = CategoryColor::from_index(habit.color_index);
        let checked = habit.is_done_on(today);
        let toggle_msg = map_habits(HabitsMessage::ToggleHabitDate {
            habit_id: habit.id.clone(),
            date: today,
        });
        list = list.push(
            row![
                radio_check_circle(checked, cat.solid(theme), 20.0, toggle_msg.clone(), theme),
                button(
                    row![
                        shaped_text(
                            habit.title.as_str(),
                            FONT_SIZE_BASE,
                            FONT_SANS_MEDIUM,
                            if checked {
                                theme.text_muted
                            } else {
                                theme.text_primary
                            },
                        )
                        .width(Length::Fill),
                        shaped_text(
                            format!("{}d", habit.current_streak(today)),
                            FONT_SIZE_XS,
                            FONT_MONO,
                            cat.solid(theme),
                        ),
                    ]
                    .spacing(SPACE_2)
                    .align_y(Alignment::Center),
                )
                .on_press(toggle_msg)
                .width(Length::Fill)
                .padding([5.0, 6.0])
                .style(move |_, status| theme.subtle_button_style(status)),
            ]
            .spacing(SPACE_2)
            .align_y(Alignment::Center),
        );
    }

    card(
        column![
            digest_header(
                IllustratedIcon::Habits,
                "Hábitos",
                "Abrir",
                map_route(AppRoute::Habits),
                theme,
            ),
            shaped_text(
                format!("{done} de {total} concluídos hoje"),
                FONT_SIZE_SM,
                FONT_SANS_MEDIUM,
                theme.text_muted,
            ),
            hairline_divider(theme),
            list,
        ]
        .spacing(SPACE_3),
        theme,
    )
    .height(card_height)
    .width(Length::FillPortion(1))
    .into()
}

fn view_home_brain_digest<'a, Message: Clone + 'a>(
    brain: &'a SecondBrainState,
    card_height: Length,
    theme: EstudyTheme,
    map_brain: impl Fn(SecondBrainMessage) -> Message + 'a,
    map_route: impl Fn(AppRoute) -> Message + 'a,
) -> Element<'a, Message> {
    let mut list = column![].spacing(SPACE_2);
    for (idx, note) in brain.notes.iter().take(3).enumerate() {
        let cat = CategoryColor::from_index(idx + 2);
        list = list.push(
            button(
                column![
                    row![
                        color_dot(cat.solid(theme)),
                        shaped_text(
                            note.title.as_str(),
                            FONT_SIZE_BASE,
                            FONT_SANS_SEMIBOLD,
                            theme.text_primary,
                        ),
                    ]
                    .spacing(6.0)
                    .align_y(Alignment::Center),
                    shaped_text(
                        note.preview_excerpt(64),
                        FONT_SIZE_XS,
                        FONT_SANS,
                        theme.text_muted,
                    ),
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
            digest_header(
                IllustratedIcon::SecondBrain,
                "Segundo cérebro",
                "Abrir",
                map_route(AppRoute::SecondBrain),
                theme,
            ),
            shaped_text(
                format!("{} ideias e referências salvas", brain.notes.len()),
                FONT_SIZE_SM,
                FONT_SANS_MEDIUM,
                theme.text_muted,
            ),
            hairline_divider(theme),
            list,
        ]
        .spacing(SPACE_3),
        theme,
    )
    .height(card_height)
    .width(Length::FillPortion(1))
    .into()
}

fn digest_header<'a, Message: Clone + 'a>(
    icon: IllustratedIcon,
    title: &'static str,
    cta_label: &'static str,
    on_cta: Message,
    theme: EstudyTheme,
) -> Element<'a, Message> {
    row![
        row![
            illustrated_svg_themed(icon, 20.0, theme),
            text(title)
                .size(FONT_SIZE_XL)
                .font(FONT_SERIF)
                .color(theme.text_primary)
                .shaping(text::Shaping::Advanced),
        ]
        .spacing(SPACE_2)
        .align_y(Alignment::Center)
        .width(Length::Fill),
        button(shaped_text(
            cta_label,
            FONT_SIZE_XS,
            FONT_SANS_SEMIBOLD,
            theme.on_accent,
        ))
        .on_press(on_cta)
        .padding([5.0, 12.0])
        .style(move |_, status| theme.pill_button_style(true, status)),
    ]
    .spacing(SPACE_2)
    .align_y(Alignment::Center)
    .width(Length::Fill)
    .into()
}

fn warm_focus_action_button<'a, Message: Clone + 'a>(
    warm_bg: Color,
    cream_fg: Color,
    label: &'static str,
    on_press: Message,
) -> Element<'a, Message> {
    button(
        row![
            icon_svg(Icon::Play, 14.0, warm_bg),
            shaped_text(label, FONT_SIZE_SM, FONT_SANS_SEMIBOLD, warm_bg),
        ]
        .spacing(6.0)
        .align_y(Alignment::Center),
    )
    .on_press(on_press)
    .padding([7.0, 14.0])
    .style(move |_, status| button::Style {
        background: Some(
            Color {
                a: if matches!(status, button::Status::Hovered) {
                    0.92
                } else {
                    1.0
                },
                ..cream_fg
            }
            .into(),
        ),
        text_color: warm_bg,
        border: Border {
            radius: RADIUS_PILL.into(),
            ..Border::default()
        },
        ..button::Style::default()
    })
    .into()
}

fn view_home_task_digest_row<'a, Message: Clone + 'a>(
    task: &'a estudy_domain::Task,
    on_toggle: Message,
    on_edit: Message,
    theme: EstudyTheme,
) -> Element<'a, Message> {
    let done = task.status == TaskStatus::Done;
    let label_el: Element<'a, Message> = if done {
        strikethrough_text(
            task.title.as_str(),
            FONT_SIZE_BASE,
            FONT_SANS_MEDIUM,
            theme.text_muted,
        )
    } else {
        shaped_text(
            task.title.as_str(),
            FONT_SIZE_BASE,
            FONT_SANS_MEDIUM,
            theme.text_primary,
        )
        .into()
    };

    row![
        radio_check_circle(
            done,
            CategoryColor::Sage.solid(theme),
            20.0,
            on_toggle,
            theme
        ),
        button(label_el)
            .on_press(on_edit)
            .width(Length::Fill)
            .padding([5.0, 6.0])
            .style(move |_, status| theme.subtle_button_style(status)),
    ]
    .spacing(SPACE_2)
    .align_y(Alignment::Center)
    .into()
}
