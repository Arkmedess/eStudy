//! Renderização da Home Pessoal no ritmo editorial do Vídeo 2 (`00:00–00:03`):
//! Hero acolhedor com `+ações`, 3 cartões contrastantes no meio (Terracota, Musgo Escuro e Mini-Calendário) e 3 colunas acionáveis embaixo.

use chrono::{Datelike, NaiveDate};
use estudy_domain::{TaskStatus, days_in_month};
use estudy_ui_kit::widgets::{
    card, color_dot, hairline_divider, icon_svg, illustrated_svg, shaped_text, tonal_card,
};
use estudy_ui_kit::{
    CategoryColor, EstudyTheme, FONT_MONO, FONT_MONO_MEDIUM, FONT_SANS, FONT_SANS_MEDIUM,
    FONT_SANS_SEMIBOLD, FONT_SERIF, FONT_SERIF_MEDIUM, FONT_SIZE_2XL, FONT_SIZE_BASE,
    FONT_SIZE_DISPLAY, FONT_SIZE_LG, FONT_SIZE_MD, FONT_SIZE_SM, FONT_SIZE_XL, FONT_SIZE_XS, Icon,
    IllustratedIcon, RADIUS_CARD, RADIUS_PILL, SPACE_1, SPACE_2, SPACE_3, SPACE_4, SPACE_5,
    SPACE_6, SparklineCanvas, SparklineData, ThemeMode,
};
use iced::widget::{button, column, container, row, scrollable, svg, text};
use iced::{Alignment, Border, Color, Element, Length};
use std::sync::LazyLock;

use super::state::{HomeDashboardData, HomeLoadState, HomeMessage};
use crate::command_palette::AppRoute;
use crate::features::habits::{HabitsMessage, HabitsState};
use crate::features::projects::modals::month_name_pt;
use crate::features::projects::{ProjectsMessage, ProjectsState};
use crate::features::second_brain::{SecondBrainMessage, SecondBrainState};

static HERO_LANDSCAPE_SVG: LazyLock<svg::Handle> = LazyLock::new(|| {
    let art = r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 360 140"><defs><linearGradient id="sky" x1="0" y1="0" x2="1" y2="1"><stop offset="0%" stop-color="#E6DFCE" stop-opacity="0.0"/><stop offset="100%" stop-color="#D4C7AD" stop-opacity="0.55"/></linearGradient></defs><rect width="360" height="140" fill="url(#sky)"/><circle cx="286" cy="42" r="24" fill="#F2C864" opacity="0.55"/><path d="M40 140 C110 92, 185 88, 260 118 C300 132, 332 122, 360 108 L360 140 Z" fill="#8BA186" opacity="0.34"/><path d="M120 140 C190 84, 265 76, 360 98 L360 140 Z" fill="#4E6B52" opacity="0.38"/><g transform="translate(298, 52)"><line x1="18" y1="36" x2="18" y2="74" stroke="#3B3026" stroke-width="3" stroke-linecap="round"/><ellipse cx="18" cy="34" rx="22" ry="11" fill="#374C3B" opacity="0.88"/><ellipse cx="18" cy="22" rx="16" ry="8" fill="#4E6B52" opacity="0.92"/></g><g transform="translate(246, 68)"><line x1="12" y1="28" x2="12" y2="56" stroke="#3B3026" stroke-width="2.4" stroke-linecap="round"/><ellipse cx="12" cy="26" rx="15" ry="8" fill="#374C3B" opacity="0.78"/></g></svg>"##;
    svg::Handle::from_memory(art.as_bytes().to_vec())
});

/// Renderiza a Home cobrindo Loading, Error, Empty e Populated.
#[allow(clippy::too_many_arguments)]
pub fn view_home<'a, Message: Clone + 'a>(
    state: &'a HomeLoadState,
    projects: &'a ProjectsState,
    habits: &'a HabitsState,
    brain: &'a SecondBrainState,
    today_focused_minutes: u32,
    today: NaiveDate,
    greeting: &'a str,
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
            today_focused_minutes,
            today,
            greeting,
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
    today_focused_minutes: u32,
    today: NaiveDate,
    greeting: &'a str,
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
        theme,
        map_home.clone(),
    );

    let contrast_row = row![
        view_warm_focus_card(data, today_focused_minutes, theme, map_home.clone()),
        view_dark_pulse_card(data, habits, brain, today, theme),
        view_mini_calendar_card(
            projects,
            today,
            theme,
            map_projects.clone(),
            map_route.clone()
        ),
    ]
    .spacing(SPACE_4)
    .width(Length::Fill);

    let bottom_row = row![
        view_home_tasks_digest(projects, theme, map_home, map_projects, map_route.clone()),
        view_home_habits_digest(habits, today, theme, map_habits, map_route.clone()),
        view_home_brain_digest(brain, theme, map_brain, map_route),
    ]
    .spacing(SPACE_4)
    .width(Length::Fill);

    scrollable(
        container(
            container(
                column![hero, contrast_row, bottom_row]
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
    .height(Length::Fill)
    .into()
}

fn view_hero_banner<'a, Message: Clone + 'a>(
    data: &'a HomeDashboardData,
    open_tasks: usize,
    pending_habits: usize,
    today: NaiveDate,
    greeting: &'a str,
    theme: EstudyTheme,
    map_home: impl Fn(HomeMessage) -> Message + 'a,
) -> Element<'a, Message> {
    let crumb = format!(
        "ESTUDY  /  INÍCIO     ·     {} DE {}",
        today.day(),
        month_name_pt(today.month()).to_uppercase()
    );
    let live_summary = format!(
        "HOJE VOCÊ TEM {open_tasks} TAREFAS EM ABERTO, {pending_habits} HÁBITOS PENDENTES E {} DIAS DE CONSTÂNCIA.",
        data.profile.streak_days
    );

    let quick_actions = row![
        hero_quick_pill("+ Tarefa", map_home(HomeMessage::QuickCreateTask), theme),
        hero_quick_pill("+ Hábito", map_home(HomeMessage::QuickCreateHabit), theme),
        hero_quick_pill(
            "+ Compromisso",
            map_home(HomeMessage::QuickCreateEvent),
            theme,
        ),
        hero_quick_pill("+ Nota", map_home(HomeMessage::QuickCreateNote), theme),
    ]
    .spacing(SPACE_2)
    .align_y(Alignment::Center);

    let left_col = column![
        shaped_text(crumb, 11.0, FONT_SANS_SEMIBOLD, theme.text_muted),
        text(format!("{greeting}, {}", data.profile.display_name))
            .size(38.0)
            .font(FONT_SERIF)
            .color(theme.text_primary)
            .shaping(text::Shaping::Advanced),
        shaped_text(live_summary, 11.5, FONT_SANS_SEMIBOLD, theme.text_muted),
        container(quick_actions).padding([SPACE_1, 0.0]),
    ]
    .spacing(6.0)
    .width(Length::FillPortion(7));

    let illustration = container(
        svg(HERO_LANDSCAPE_SVG.clone())
            .width(Length::Fixed(320.0))
            .height(Length::Fixed(128.0)),
    )
    .align_x(Alignment::End)
    .align_y(Alignment::End)
    .width(Length::FillPortion(5));

    container(
        row![left_col, illustration]
            .align_y(Alignment::Center)
            .width(Length::Fill),
    )
    .padding([SPACE_4, SPACE_5])
    .width(Length::Fill)
    .style(move |_| container::Style {
        background: Some(theme.panel.into()),
        border: Border {
            color: theme.border_subtle,
            width: 1.0,
            radius: RADIUS_CARD.into(),
        },
        shadow: theme.card_style().shadow,
        snap: true,
        ..container::Style::default()
    })
    .into()
}

fn hero_quick_pill<'a, Message: Clone + 'a>(
    label: &'static str,
    on_press: Message,
    theme: EstudyTheme,
) -> Element<'a, Message> {
    button(shaped_text(
        label,
        FONT_SIZE_SM,
        FONT_SANS_MEDIUM,
        theme.text_primary,
    ))
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
            color: theme.border_subtle,
            width: 1.0,
            radius: RADIUS_PILL.into(),
        },
        ..button::Style::default()
    })
    .into()
}

fn view_warm_focus_card<'a, Message: Clone + 'a>(
    data: &'a HomeDashboardData,
    today_focused_minutes: u32,
    theme: EstudyTheme,
    map_home: impl Fn(HomeMessage) -> Message + 'a,
) -> Element<'a, Message> {
    let warm_bg = match theme.mode {
        ThemeMode::Light => Color::from_rgb8(0xB8, 0x56, 0x2F),
        ThemeMode::Dark => Color::from_rgb8(0x7D, 0x39, 0x1E),
    };
    let cream_fg = Color::from_rgb8(0xFF, 0xFA, 0xF3);
    let muted_fg = Color {
        a: 0.84,
        ..cream_fg
    };

    let first_pin_id = data
        .focus_items
        .first()
        .map_or_else(|| "foco".to_string(), |p| p.id.clone());
    let next_focus_title = data
        .focus_items
        .first()
        .map_or("Pronto para um bloco calmo de 25 minutos", |p| {
            p.title.as_str()
        });

    let start_pill = warm_focus_start_button(
        warm_bg,
        cream_fg,
        map_home(HomeMessage::StartFocusForPin(first_pin_id)),
    );

    container(
        column![
            row![
                text("Foco & Ritmo")
                    .size(FONT_SIZE_XL)
                    .font(FONT_SERIF)
                    .color(cream_fg)
                    .shaping(text::Shaping::Advanced),
                container(shaped_text(
                    format!("{}D SEGUIDOS", data.profile.streak_days),
                    11.0,
                    FONT_MONO,
                    muted_fg,
                ))
                .align_x(Alignment::End)
                .width(Length::Fill),
            ]
            .align_y(Alignment::Center),
            container(
                column![
                    shaped_text("TEMPO FOCADO HOJE", 11.0, FONT_SANS_SEMIBOLD, muted_fg),
                    text(format!("{today_focused_minutes} min"))
                        .size(40.0)
                        .font(FONT_SERIF)
                        .color(cream_fg)
                        .shaping(text::Shaping::Advanced),
                    shaped_text(next_focus_title, FONT_SIZE_SM, FONT_SANS, muted_fg),
                ]
                .spacing(2.0),
            )
            .padding([SPACE_3, 0.0])
            .height(Length::Fill),
            row![
                start_pill,
                container(shaped_text(
                    format!("{} em foco", data.focus_items.len()),
                    FONT_SIZE_XS,
                    FONT_MONO,
                    muted_fg,
                ))
                .align_x(Alignment::End)
                .width(Length::Fill),
            ]
            .align_y(Alignment::Center),
        ]
        .spacing(SPACE_2)
        .height(Length::Fill),
    )
    .padding(SPACE_5)
    .height(Length::Fixed(242.0))
    .width(Length::FillPortion(4))
    .style(move |_| container::Style {
        background: Some(warm_bg.into()),
        border: Border {
            radius: RADIUS_CARD.into(),
            ..Border::default()
        },
        shadow: theme.card_style().shadow,
        snap: true,
        ..container::Style::default()
    })
    .into()
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

    let (dark_bg, light_fg) = match theme.mode {
        ThemeMode::Light => (theme.accent, theme.on_accent),
        ThemeMode::Dark => (
            Color::from_rgb8(0x1E, 0x2B, 0x20),
            Color::from_rgb8(0xF2, 0xF7, 0xF0),
        ),
    };
    let sub_fg = Color {
        a: 0.78,
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

    tonal_card(
        column![
            row![
                text("Constância")
                    .size(FONT_SIZE_XL)
                    .font(FONT_SERIF)
                    .color(light_fg)
                    .shaping(text::Shaping::Advanced),
                container(shaped_text("ÚLTIMOS 14 DIAS", 11.0, FONT_MONO, sub_fg))
                    .align_x(Alignment::End)
                    .width(Length::Fill),
            ]
            .align_y(Alignment::Center),
            row![
                text(format!("{recent_sum}"))
                    .size(FONT_SIZE_DISPLAY)
                    .font(FONT_SERIF)
                    .color(light_fg)
                    .shaping(text::Shaping::Advanced),
                shaped_text("ações concluídas", FONT_SIZE_SM, FONT_SANS, sub_fg),
            ]
            .spacing(SPACE_2)
            .align_y(Alignment::End),
            SparklineCanvas {
                data: spark_data,
                stroke_color: light_fg,
                fill_color: Color {
                    a: 0.18,
                    ..light_fg
                },
                grid_color: Color {
                    a: 0.14,
                    ..light_fg
                },
            }
            .into_canvas()
            .width(Length::Fill)
            .height(Length::Fixed(74.0)),
            stats_footer,
        ]
        .spacing(SPACE_2)
        .height(Length::Fill),
        theme,
    )
    .style(move |_| container::Style {
        background: Some(dark_bg.into()),
        border: Border {
            radius: RADIUS_CARD.into(),
            ..Border::default()
        },
        shadow: theme.card_style().shadow,
        snap: true,
        ..container::Style::default()
    })
    .height(Length::Fixed(242.0))
    .width(Length::FillPortion(5))
    .into()
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
    today: NaiveDate,
    theme: EstudyTheme,
    map_projects: impl Fn(ProjectsMessage) -> Message + 'a,
    map_route: impl Fn(AppRoute) -> Message + 'a,
) -> Element<'a, Message> {
    let year = projects.calendar_year;
    let month = projects.calendar_month;
    let total_days = days_in_month(year, month);
    let first = NaiveDate::from_ymd_opt(year, month, 1);
    let offset = first.map_or(0, |d| d.weekday().num_days_from_sunday() as usize);

    let mut cells: Vec<Option<NaiveDate>> = vec![None; offset];
    for d in 1..=total_days {
        cells.push(NaiveDate::from_ymd_opt(year, month, d));
    }

    let weekdays = ["D", "S", "T", "Q", "Q", "S", "S"];
    let mut head = row![].spacing(2.0).width(Length::Fill);
    for w in weekdays {
        head = head.push(
            container(shaped_text(w, 11.0, FONT_SANS_SEMIBOLD, theme.text_muted))
                .center_x(Length::Fill)
                .width(Length::FillPortion(1)),
        );
    }

    let mut rows = column![head].spacing(3.0);
    for week in cells.chunks(7) {
        let mut line = row![].spacing(2.0).width(Length::Fill);
        for slot in week {
            if let Some(date) = slot {
                let is_today = *date == today;
                line = line.push(
                    button(
                        container(shaped_text(
                            format!("{}", date.day()),
                            FONT_SIZE_XS,
                            FONT_MONO,
                            if is_today {
                                theme.on_accent
                            } else {
                                theme.text_primary
                            },
                        ))
                        .center_x(Length::Fill)
                        .width(Length::Fill),
                    )
                    .on_press(map_route(AppRoute::Calendar))
                    .width(Length::FillPortion(1))
                    .padding([3.0, 0.0])
                    .style(move |_, status| {
                        if is_today {
                            theme.pill_button_style(true, status)
                        } else {
                            theme.subtle_button_style(status)
                        }
                    }),
                );
            } else {
                line = line.push(container(text("")).width(Length::FillPortion(1)));
            }
        }
        for _ in week.len()..7 {
            line = line.push(container(text("")).width(Length::FillPortion(1)));
        }
        rows = rows.push(line);
    }

    let raw_m = month_name_pt(month);
    let mut chars = raw_m.chars();
    let cap_m = chars.next().map_or_else(String::new, |c| {
        format!("{}{}", c.to_uppercase(), chars.as_str())
    });

    let bottom_nav = row![
        button(icon_svg(Icon::ChevronLeft, 14.0, theme.text_muted))
            .on_press(map_projects(ProjectsMessage::ShiftCalendarMonth(-1)))
            .padding(2.0)
            .style(move |_, status| theme.subtle_button_style(status)),
        container(shaped_text(
            format!("{cap_m} de {year}"),
            FONT_SIZE_XS,
            FONT_SANS_SEMIBOLD,
            theme.text_primary,
        ))
        .center_x(Length::Fill)
        .width(Length::Fill),
        button(icon_svg(Icon::ChevronRight, 14.0, theme.text_muted))
            .on_press(map_projects(ProjectsMessage::ShiftCalendarMonth(1)))
            .padding(2.0)
            .style(move |_, status| theme.subtle_button_style(status)),
    ]
    .align_y(Alignment::Center);

    card(
        column![
            rows,
            container(bottom_nav)
                .align_y(Alignment::End)
                .height(Length::Fill),
        ]
        .spacing(SPACE_2)
        .height(Length::Fill),
        theme,
    )
    .padding(SPACE_4)
    .height(Length::Fixed(242.0))
    .width(Length::FillPortion(3))
    .into()
}

fn view_home_tasks_digest<'a, Message: Clone + 'a>(
    projects: &'a ProjectsState,
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
    .spacing(SPACE_4);

    let mut list = column![].spacing(SPACE_2);
    for task in projects.tasks.iter().take(3) {
        list = list.push(view_home_task_digest_row(
            task,
            map_projects(ProjectsMessage::ToggleTaskDone(task.id.clone())),
            theme,
        ));
    }

    let mut proj_pills = row![].spacing(6.0);
    for (idx, proj) in projects.projects.iter().take(2).enumerate() {
        let dot = CategoryColor::from_index(idx).solid(theme);
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

    card(
        column![
            digest_header(
                IllustratedIcon::Tasks,
                "Tarefas",
                "VER TODAS",
                map_route(AppRoute::Projects),
                theme,
            ),
            counters,
            hairline_divider(theme),
            list,
            proj_pills,
        ]
        .spacing(SPACE_3),
        theme,
    )
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
    theme: EstudyTheme,
    map_habits: impl Fn(HabitsMessage) -> Message + 'a,
    map_route: impl Fn(AppRoute) -> Message + 'a,
) -> Element<'a, Message> {
    let (done, total) = habits.today_progress(today);
    let mut list = column![].spacing(SPACE_2);

    for habit in habits.habits.iter().take(4) {
        let cat = CategoryColor::from_index(habit.color_index);
        let checked = habit.is_done_on(today);
        list = list.push(
            button(
                row![
                    icon_svg(
                        if checked {
                            Icon::CheckCircle2
                        } else {
                            Icon::CheckSquare
                        },
                        16.0,
                        if checked {
                            cat.solid(theme)
                        } else {
                            theme.text_muted
                        },
                    ),
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
            .on_press(map_habits(HabitsMessage::ToggleHabitDate {
                habit_id: habit.id.clone(),
                date: today,
            }))
            .width(Length::Fill)
            .padding([7.0, 8.0])
            .style(move |_, status| theme.subtle_button_style(status)),
        );
    }

    card(
        column![
            digest_header(
                IllustratedIcon::Habits,
                "Hábitos",
                "ABRIR",
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
    .width(Length::FillPortion(1))
    .into()
}

fn view_home_brain_digest<'a, Message: Clone + 'a>(
    brain: &'a SecondBrainState,
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
                        note.preview_excerpt(68),
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
                "ABRIR",
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
            illustrated_svg(icon, 20.0),
            text(title)
                .size(FONT_SIZE_XL)
                .font(FONT_SERIF)
                .color(theme.text_primary)
                .shaping(text::Shaping::Advanced),
        ]
        .spacing(SPACE_2)
        .align_y(Alignment::Center),
        container(
            button(shaped_text(
                cta_label,
                11.0,
                FONT_SANS_SEMIBOLD,
                theme.on_accent,
            ))
            .on_press(on_cta)
            .padding([4.0, 11.0])
            .style(move |_, status| theme.pill_button_style(true, status)),
        )
        .align_x(Alignment::End)
        .width(Length::Fill),
    ]
    .align_y(Alignment::Center)
    .into()
}

fn warm_focus_start_button<'a, Message: Clone + 'a>(
    warm_bg: Color,
    cream_fg: Color,
    on_press: Message,
) -> Element<'a, Message> {
    button(
        row![
            icon_svg(Icon::Play, 14.0, warm_bg),
            shaped_text(
                "Iniciar foco 25m",
                FONT_SIZE_SM,
                FONT_SANS_SEMIBOLD,
                warm_bg
            ),
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
    theme: EstudyTheme,
) -> Element<'a, Message> {
    let done = task.status == TaskStatus::Done;
    button(
        row![
            icon_svg(
                if done {
                    Icon::CheckCircle2
                } else {
                    Icon::CheckSquare
                },
                16.0,
                if done {
                    CategoryColor::Sage.solid(theme)
                } else {
                    theme.text_muted
                },
            ),
            shaped_text(
                task.title.as_str(),
                FONT_SIZE_BASE,
                FONT_SANS_MEDIUM,
                if done {
                    theme.text_muted
                } else {
                    theme.text_primary
                },
            )
            .width(Length::Fill),
        ]
        .spacing(SPACE_2)
        .align_y(Alignment::Center),
    )
    .on_press(on_toggle)
    .width(Length::Fill)
    .padding([6.0, 8.0])
    .style(move |_, status| theme.subtle_button_style(status))
    .into()
}
