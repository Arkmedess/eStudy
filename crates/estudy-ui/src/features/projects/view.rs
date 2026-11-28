//! Renderização fluida da tela `Tarefas` (`Semana | Calendário | Kanban`), bandeja `SEM DIA` e cartões táteis.

use chrono::{Datelike, Duration, NaiveDate};
use estudy_domain::{Task, TaskPriority, TaskStatus, TaskTurn, classify_task_turn, days_in_month};
use estudy_ui_kit::widgets::{
    color_dot, dashed_frame, editorial_page_header, hairline_divider, icon_svg,
    illustrated_svg_themed, pill_chip, radio_check_circle, segmented_switch, shaped_text,
    soft_surface, strikethrough_text,
};
use estudy_ui_kit::{
    CategoryColor, EstudyTheme, FONT_MONO, FONT_SANS, FONT_SANS_MEDIUM, FONT_SANS_SEMIBOLD,
    FONT_SIZE_BASE, FONT_SIZE_MD, FONT_SIZE_SM, FONT_SIZE_XS, Icon, IllustratedIcon, RADIUS_CARD,
    RADIUS_CONTROL, RADIUS_PANEL, RADIUS_PILL, SPACE_1, SPACE_2, SPACE_3, SPACE_4, SPACE_5,
    SPACE_6,
};
use iced::widget::{button, column, container, mouse_area, row, scrollable, stack, text};
use iced::{Alignment, Border, Color, Element, Length};

use super::modals::{
    month_name_pt, project_tag_inline, task_status_visual, view_create_project_modal,
    view_day_peek_modal, view_drag_ghost, view_task_context_menu, view_task_modal,
};
use super::state::{
    DragTarget, ProjectsMessage, ProjectsState, TasksViewMode, extract_task_date,
    project_color_index,
};
use crate::features::home::TodayItem;

/// Largura mantida como constante pública de compatibilidade.
pub const CONTEXT_SIDEBAR_WIDTH: f32 = 264.0;

/// Renderiza a tela completa de Tarefas sem segunda sidebar, com 7 dias em scroll fluido, Drag & Drop, Menu de Contexto e Modais.
pub fn view_projects<'a, Message: Clone + 'a>(
    state: &'a ProjectsState,
    today: NaiveDate,
    _today_summary: &'a [TodayItem],
    window_size: (f32, f32),
    theme: EstudyTheme,
    map_msg: impl Fn(ProjectsMessage) -> Message + Clone + 'a,
) -> Element<'a, Message> {
    let new_task_btn = button(
        row![
            icon_svg(Icon::Plus, 16.0, theme.on_accent),
            shaped_text(
                "Nova tarefa",
                FONT_SIZE_BASE,
                FONT_SANS_SEMIBOLD,
                theme.on_accent,
            ),
        ]
        .spacing(6.0)
        .align_y(Alignment::Center),
    )
    .on_press(map_msg(ProjectsMessage::OpenCreateTaskModal {
        date: Some(today),
        turn: TaskTurn::Anytime,
    }))
    .padding([8.0, 16.0])
    .style(move |_, status| theme.pill_button_style(true, status));

    let page_header = editorial_page_header(
        "ESTUDY  /  TAREFAS",
        "Tarefas",
        "A semana e o calendário",
        Some(new_task_btn.into()),
        theme,
    );

    let controls_bar = view_controls_bar(state, today, window_size.0, theme, map_msg.clone());

    let main_body: Element<'a, Message> = match state.view_mode {
        TasksViewMode::Week => view_week_workspace(state, today, theme, &map_msg),
        TasksViewMode::Calendar => view_tasks_calendar_grid(state, today, theme, &map_msg),
        TasksViewMode::Kanban => view_kanban_board(state, today, window_size.0, theme, &map_msg),
    };

    let page_shell = container(
        column![page_header, controls_bar, main_body]
            .spacing(SPACE_4)
            .width(Length::Fill)
            .height(Length::Fill),
    )
    .padding([SPACE_5, SPACE_6])
    .width(Length::Fill)
    .height(Length::Fill);

    let base: Element<'a, Message> = if state.dragging_task_id.is_some() {
        mouse_area(page_shell)
            .on_release(map_msg(ProjectsMessage::DropTask))
            .into()
    } else {
        page_shell.into()
    };

    let mut layers: Vec<Element<'a, Message>> = vec![base];

    if let Some(ghost) = view_drag_ghost(state, window_size, theme) {
        layers.push(ghost);
    }
    if let Some(peek_date) = state.day_peek_date {
        layers.push(view_day_peek_modal(
            state,
            peek_date,
            theme,
            map_msg.clone(),
        ));
    }
    if let Some(modal) = &state.task_modal {
        layers.push(view_task_modal(state, modal, today, theme, map_msg.clone()));
    }
    if state.create_project_modal_open {
        layers.push(view_create_project_modal(state, theme, map_msg.clone()));
    }
    if let Some(menu) = &state.context_menu {
        layers.push(view_task_context_menu(
            state,
            menu,
            window_size,
            theme,
            map_msg,
        ));
    }

    stack(layers)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

fn view_controls_bar<'a, Message: Clone + 'a>(
    state: &'a ProjectsState,
    today: NaiveDate,
    window_width: f32,
    theme: EstudyTheme,
    map_msg: impl Fn(ProjectsMessage) -> Message + Clone + 'a,
) -> Element<'a, Message> {
    let on_mode = map_msg.clone();
    let mode_switch = segmented_switch(
        &TasksViewMode::ALL,
        state.view_mode,
        move |mode| on_mode(ProjectsMessage::SetTasksViewMode(mode)),
        theme,
    );

    let period_nav = view_period_navigation(state, today, theme, &map_msg);

    let mut project_filters = row![pill_chip(
        "Todos",
        None,
        None,
        state.selected_project_id.is_none(),
        map_msg(ProjectsMessage::SelectProject(None)),
        theme,
    )]
    .spacing(SPACE_1)
    .align_y(Alignment::Center);

    for (idx, proj) in state.projects.iter().enumerate() {
        let active = state.selected_project_id.as_ref() == Some(&proj.id);
        let color_idx = project_color_index(&proj.color_token, idx);
        let dot = CategoryColor::from_index(color_idx).solid(theme);
        project_filters = project_filters.push(pill_chip(
            &proj.title,
            Some(dot),
            None,
            active,
            map_msg(ProjectsMessage::SelectProject(Some(proj.id.clone()))),
            theme,
        ));
    }

    project_filters = project_filters.push(
        button(
            row![
                icon_svg(Icon::Plus, 13.0, theme.text_muted),
                shaped_text("Projeto", FONT_SIZE_XS, FONT_SANS_MEDIUM, theme.text_muted),
            ]
            .spacing(4.0)
            .align_y(Alignment::Center),
        )
        .on_press(map_msg(ProjectsMessage::ToggleCreateProjectModal))
        .padding([6.0, 11.0])
        .style(move |_, status| theme.ghost_button_style(status)),
    );

    let left_cluster = row![mode_switch, period_nav]
        .spacing(SPACE_3)
        .align_y(Alignment::Center);

    if window_width >= 1240.0 {
        row![
            left_cluster,
            container(project_filters.wrap().vertical_spacing(SPACE_1))
                .align_x(Alignment::End)
                .width(Length::Fill),
        ]
        .spacing(SPACE_3)
        .align_y(Alignment::Center)
        .width(Length::Fill)
        .into()
    } else {
        column![
            left_cluster,
            project_filters.wrap().vertical_spacing(SPACE_1)
        ]
        .spacing(SPACE_2)
        .width(Length::Fill)
        .into()
    }
}

fn view_period_navigation<'a, Message: Clone + 'a>(
    state: &'a ProjectsState,
    today: NaiveDate,
    theme: EstudyTheme,
    map_msg: &impl Fn(ProjectsMessage) -> Message,
) -> Element<'a, Message> {
    match state.view_mode {
        TasksViewMode::Week => {
            let end = state.week_start + Duration::days(6);
            let label = format!(
                "{} a {} de {}",
                state.week_start.day(),
                end.day(),
                month_name_pt(end.month())
            );
            let away_from_current = today < state.week_start || today > end;
            let mut items = row![
                nav_arrow_btn(
                    Icon::ChevronLeft,
                    map_msg(ProjectsMessage::ShiftWeek(-1)),
                    theme,
                ),
                shaped_text(label, FONT_SIZE_MD, FONT_SANS_MEDIUM, theme.text_primary),
                nav_arrow_btn(
                    Icon::ChevronRight,
                    map_msg(ProjectsMessage::ShiftWeek(1)),
                    theme,
                ),
            ]
            .spacing(SPACE_2)
            .align_y(Alignment::Center);

            if away_from_current {
                items = items.push(
                    button(shaped_text(
                        "Esta semana",
                        FONT_SIZE_XS,
                        FONT_SANS_SEMIBOLD,
                        theme.accent,
                    ))
                    .on_press(map_msg(ProjectsMessage::JumpToWeekOf(today)))
                    .padding([4.0, 10.0])
                    .style(move |_, status| theme.ghost_button_style(status)),
                );
            }
            items.into()
        }
        TasksViewMode::Calendar => {
            let m_name = month_name_pt(state.calendar_month);
            let mut chars = m_name.chars();
            let cap = chars.next().map_or_else(String::new, |first| {
                format!("{}{}", first.to_uppercase(), chars.as_str())
            });
            let label = format!("{cap} de {}", state.calendar_year);
            let away_from_month =
                today.year() != state.calendar_year || today.month() != state.calendar_month;

            let mut items = row![
                nav_arrow_btn(
                    Icon::ChevronLeft,
                    map_msg(ProjectsMessage::ShiftCalendarMonth(-1)),
                    theme,
                ),
                shaped_text(label, FONT_SIZE_MD, FONT_SANS_MEDIUM, theme.text_primary),
                nav_arrow_btn(
                    Icon::ChevronRight,
                    map_msg(ProjectsMessage::ShiftCalendarMonth(1)),
                    theme,
                ),
            ]
            .spacing(SPACE_2)
            .align_y(Alignment::Center);

            if away_from_month {
                items = items.push(
                    button(shaped_text(
                        "Hoje",
                        FONT_SIZE_XS,
                        FONT_SANS_SEMIBOLD,
                        theme.accent,
                    ))
                    .on_press(map_msg(ProjectsMessage::JumpToWeekOf(today)))
                    .padding([4.0, 10.0])
                    .style(move |_, status| theme.ghost_button_style(status)),
                );
            }
            items.into()
        }
        TasksViewMode::Kanban => shaped_text(
            "Quadro por etapa",
            FONT_SIZE_BASE,
            FONT_SANS_MEDIUM,
            theme.text_muted,
        )
        .into(),
    }
}

fn nav_arrow_btn<'a, Message: Clone + 'a>(
    icon: Icon,
    on_press: Message,
    theme: EstudyTheme,
) -> Element<'a, Message> {
    button(icon_svg(icon, 16.0, theme.text_primary))
        .on_press(on_press)
        .padding(6.0)
        .style(move |_, status| theme.subtle_button_style(status))
        .into()
}

fn view_week_workspace<'a, Message: Clone + 'a>(
    state: &'a ProjectsState,
    today: NaiveDate,
    theme: EstudyTheme,
    map_msg: &impl Fn(ProjectsMessage) -> Message,
) -> Element<'a, Message> {
    let tasks = state.filtered_tasks();
    let unscheduled: Vec<&Task> = tasks
        .iter()
        .copied()
        .filter(|t| extract_task_date(t).is_none())
        .collect();

    let unscheduled_tray = view_unscheduled_tray(state, today, &unscheduled, theme, map_msg);

    let mut days_row = row![].spacing(SPACE_3).height(Length::Fill);

    for offset in 0..7_i64 {
        let day = state.week_start + Duration::days(offset);
        let day_tasks: Vec<&Task> = tasks
            .iter()
            .copied()
            .filter(|t| extract_task_date(t) == Some(day))
            .collect();

        days_row = days_row.push(view_day_column(
            state, day, today, &day_tasks, theme, map_msg,
        ));
    }

    let days_scroll = scrollable(days_row)
        .direction(scrollable::Direction::Horizontal(
            scrollable::Scrollbar::new().width(6).scroller_width(6),
        ))
        .style(move |_, status| theme.subtle_scrollbar_style(status))
        .width(Length::Fill)
        .height(Length::Fill);

    column![unscheduled_tray, days_scroll]
        .spacing(SPACE_3)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

fn view_unscheduled_tray<'a, Message: Clone + 'a>(
    state: &'a ProjectsState,
    today: NaiveDate,
    unscheduled: &[&'a Task],
    theme: EstudyTheme,
    map_msg: &impl Fn(ProjectsMessage) -> Message,
) -> Element<'a, Message> {
    let is_drop_hovered = state.dragging_task_id.is_some()
        && state.hovered_drop_target == Some(DragTarget::Unscheduled);

    let add_unscheduled = button(icon_svg(Icon::Plus, 16.0, theme.text_muted))
        .on_press(map_msg(ProjectsMessage::OpenCreateTaskModal {
            date: None,
            turn: TaskTurn::Anytime,
        }))
        .padding(6.0)
        .style(move |_, status| theme.subtle_button_style(status));

    let body_el: Element<'a, Message> = if unscheduled.is_empty() {
        let hint = if state.dragging_task_id.is_some() {
            "Solte aqui para deixar sem data fixa"
        } else {
            "Nenhuma tarefa sem data — arraste para cá quando quiser pausar uma data"
        };
        dashed_frame(
            container(shaped_text(
                hint,
                FONT_SIZE_SM,
                FONT_SANS,
                if is_drop_hovered {
                    theme.accent
                } else {
                    theme.text_muted
                },
            ))
            .padding([SPACE_2, SPACE_3])
            .width(Length::Fill),
            if is_drop_hovered {
                theme.accent
            } else {
                theme.border_subtle
            },
            is_drop_hovered.then_some(theme.accent_subtle),
            RADIUS_CONTROL,
        )
    } else {
        let mut cards_row = row![].spacing(SPACE_2);
        for task in unscheduled {
            cards_row = cards_row.push(
                container(view_task_card(state, task, today, theme, map_msg))
                    .width(Length::Fixed(240.0)),
            );
        }
        cards_row.wrap().vertical_spacing(SPACE_2).into()
    };

    let tray_inner = row![
        shaped_text(
            "SEM DIA",
            FONT_SIZE_XS,
            FONT_SANS_SEMIBOLD,
            theme.text_muted
        ),
        container(body_el).width(Length::Fill),
        add_unscheduled,
    ]
    .spacing(SPACE_3)
    .align_y(Alignment::Center)
    .width(Length::Fill);

    let tray = container(tray_inner)
        .padding([SPACE_2, SPACE_4])
        .width(Length::Fill)
        .style(move |_| container::Style {
            background: Some(
                if is_drop_hovered {
                    theme.accent_subtle
                } else {
                    theme.surface_soft
                }
                .into(),
            ),
            border: Border {
                color: if is_drop_hovered {
                    theme.accent
                } else {
                    theme.border_subtle
                },
                width: if is_drop_hovered { 1.5 } else { 1.0 },
                radius: RADIUS_CARD.into(),
            },
            ..container::Style::default()
        });

    mouse_area(tray)
        .on_enter(map_msg(ProjectsMessage::HoverDropTarget(Some(
            DragTarget::Unscheduled,
        ))))
        .on_release(map_msg(ProjectsMessage::DropTask))
        .into()
}

fn view_day_column<'a, Message: Clone + 'a>(
    state: &'a ProjectsState,
    day: NaiveDate,
    today: NaiveDate,
    day_tasks: &[&'a Task],
    theme: EstudyTheme,
    map_msg: &impl Fn(ProjectsMessage) -> Message,
) -> Element<'a, Message> {
    let is_today = day == today;
    let open_count = day_tasks
        .iter()
        .filter(|t| t.status != TaskStatus::Done)
        .count();
    let col_header = view_day_column_header(day, is_today, open_count, theme, map_msg);

    let mut periods_col = column![].spacing(SPACE_3);
    for turn in TaskTurn::ALL {
        periods_col = periods_col.push(view_period_drop_slot(
            state, day, today, turn, day_tasks, theme, map_msg,
        ));
    }

    let border_c = if is_today {
        CategoryColor::Terracotta.border(theme)
    } else {
        theme.border_subtle
    };

    let day_scroll = scrollable(periods_col)
        .direction(scrollable::Direction::Vertical(
            scrollable::Scrollbar::new().width(5).scroller_width(5),
        ))
        .style(move |_, status| theme.subtle_scrollbar_style(status))
        .height(Length::Fill);

    container(
        column![col_header, hairline_divider(theme), day_scroll]
            .spacing(SPACE_2)
            .height(Length::Fill),
    )
    .padding(SPACE_3)
    .width(Length::Fixed(248.0))
    .height(Length::Fill)
    .style(move |_| container::Style {
        background: Some(
            Color {
                a: if theme.mode == estudy_ui_kit::ThemeMode::Light {
                    0.55
                } else {
                    0.65
                },
                ..theme.surface_soft
            }
            .into(),
        ),
        border: Border {
            color: border_c,
            width: if is_today { 1.5 } else { 1.0 },
            radius: RADIUS_CARD.into(),
        },
        ..container::Style::default()
    })
    .into()
}

fn view_day_column_header<'a, Message: Clone + 'a>(
    day: NaiveDate,
    is_today: bool,
    open_count: usize,
    theme: EstudyTheme,
    map_msg: &impl Fn(ProjectsMessage) -> Message,
) -> Element<'a, Message> {
    let title_color = if is_today {
        CategoryColor::Terracotta.solid(theme)
    } else {
        theme.text_primary
    };

    let mut header_left = row![
        shaped_text(
            weekday_full_pt(day.weekday()),
            FONT_SIZE_MD,
            FONT_SANS_SEMIBOLD,
            title_color,
        ),
        shaped_text(
            format!("{:02}/{:02}", day.day(), day.month()),
            FONT_SIZE_SM,
            FONT_MONO,
            theme.text_muted,
        ),
    ]
    .spacing(6.0)
    .align_y(Alignment::Center);

    if open_count > 0 {
        header_left = header_left.push(
            container(shaped_text(
                format!("{open_count}"),
                FONT_SIZE_XS,
                FONT_MONO,
                theme.text_muted,
            ))
            .padding([1.0, 6.0])
            .style(move |_| container::Style {
                background: Some(theme.surface_soft.into()),
                border: Border {
                    radius: RADIUS_PILL.into(),
                    ..Border::default()
                },
                ..container::Style::default()
            }),
        );
    }

    let add_day_btn = button(icon_svg(Icon::Plus, 16.0, theme.text_muted))
        .on_press(map_msg(ProjectsMessage::OpenCreateTaskModal {
            date: Some(day),
            turn: TaskTurn::Anytime,
        }))
        .padding(4.0)
        .style(move |_, status| theme.subtle_button_style(status));

    row![
        header_left,
        container(add_day_btn)
            .align_x(Alignment::End)
            .width(Length::Fill),
    ]
    .align_y(Alignment::Center)
    .into()
}

fn view_period_drop_slot<'a, Message: Clone + 'a>(
    state: &'a ProjectsState,
    day: NaiveDate,
    today: NaiveDate,
    turn: TaskTurn,
    day_tasks: &[&'a Task],
    theme: EstudyTheme,
    map_msg: &impl Fn(ProjectsMessage) -> Message,
) -> Element<'a, Message> {
    let mut matching: Vec<&Task> = day_tasks
        .iter()
        .copied()
        .filter(|t| {
            classify_task_turn(t.start_at.as_deref(), t.due_at.as_deref(), t.priority) == turn
        })
        .collect();
    matching.sort_by_key(|t| t.status == TaskStatus::Done);

    let target = DragTarget::DayTurn { date: day, turn };
    let is_hovered = state.dragging_task_id.is_some() && state.hovered_drop_target == Some(target);

    let period_header = row![
        illustrated_svg_themed(turn_illustrated_icon(turn), 16.0, theme),
        shaped_text(
            turn.label(),
            FONT_SIZE_XS,
            FONT_SANS_SEMIBOLD,
            turn_label_color(turn, theme),
        ),
        container(
            button(icon_svg(Icon::Plus, 14.0, theme.text_muted))
                .on_press(map_msg(ProjectsMessage::OpenCreateTaskModal {
                    date: Some(day),
                    turn,
                }))
                .padding(2.0)
                .style(move |_, status| theme.subtle_button_style(status)),
        )
        .align_x(Alignment::End)
        .width(Length::Fill),
    ]
    .spacing(5.0)
    .align_y(Alignment::Center);

    let mut slot_col = column![period_header].spacing(SPACE_2);

    if matching.is_empty() {
        let empty_zone = dashed_frame(
            container(shaped_text(
                "Arraste para cá",
                FONT_SIZE_XS,
                FONT_SANS,
                Color {
                    a: if is_hovered { 0.98 } else { 0.65 },
                    ..if is_hovered {
                        theme.accent
                    } else {
                        theme.text_muted
                    }
                },
            ))
            .padding([10.0, SPACE_2])
            .center_x(Length::Fill)
            .width(Length::Fill),
            if is_hovered {
                theme.accent
            } else {
                Color {
                    a: 0.45,
                    ..theme.border_subtle
                }
            },
            is_hovered.then_some(theme.accent_subtle),
            RADIUS_PANEL,
        );
        slot_col = slot_col.push(empty_zone);
    } else {
        for task in matching {
            slot_col = slot_col.push(view_task_card(state, task, today, theme, map_msg));
        }
    }

    let slot_box = container(slot_col)
        .padding([SPACE_1, 2.0])
        .width(Length::Fill)
        .style(move |_| container::Style {
            background: is_hovered.then_some(theme.accent_subtle.into()),
            border: Border {
                color: if is_hovered {
                    theme.accent
                } else {
                    Color::TRANSPARENT
                },
                width: if is_hovered { 1.5 } else { 0.0 },
                radius: RADIUS_PANEL.into(),
            },
            ..container::Style::default()
        });

    mouse_area(slot_box)
        .on_enter(map_msg(ProjectsMessage::HoverDropTarget(Some(target))))
        .on_release(map_msg(ProjectsMessage::DropTask))
        .into()
}

/// Cartão tátil de tarefa com check circular `○ -> ✓`, riscado ao concluir, indicador de atraso, clique direito e Drag & Drop.
pub fn view_task_card<'a, Message: Clone + 'a>(
    state: &'a ProjectsState,
    task: &'a Task,
    today: NaiveDate,
    theme: EstudyTheme,
    map_msg: &impl Fn(ProjectsMessage) -> Message,
) -> Element<'a, Message> {
    let is_done = task.status == TaskStatus::Done;
    let is_being_dragged = state.dragging_task_id.as_ref() == Some(&task.id);

    let radio_circle = radio_check_circle(
        is_done,
        CategoryColor::Sage.solid(theme),
        18.0,
        map_msg(ProjectsMessage::ToggleTaskDone(task.id.clone())),
        theme,
    );

    let title_el: Element<'a, Message> = if is_done {
        strikethrough_text(
            task.title.as_str(),
            FONT_SIZE_BASE,
            FONT_SANS_SEMIBOLD,
            theme.text_muted,
        )
    } else {
        shaped_text(
            task.title.as_str(),
            FONT_SIZE_BASE,
            FONT_SANS_SEMIBOLD,
            theme.text_primary,
        )
        .into()
    };

    let title_btn = button(title_el)
        .on_press(map_msg(ProjectsMessage::OpenEditTaskModal(task.id.clone())))
        .padding(0.0)
        .width(Length::Fill)
        .style(|_, _| button::Style::default());

    let top_row = row![radio_circle, title_btn]
        .spacing(SPACE_2)
        .align_y(Alignment::Start);

    let status_pill = task_status_pill(
        task.status,
        map_msg(ProjectsMessage::CycleTaskStatus(task.id.clone())),
        theme,
    );

    let mut pills_row = row![status_pill].spacing(5.0).align_y(Alignment::Center);

    if !is_done
        && let Some(task_date) = extract_task_date(task)
        && task_date < today
    {
        let label = if task_date == today - Duration::days(1) {
            "ONTEM"
        } else {
            "ATRASADA"
        };
        pills_row = pills_row.push(overdue_pill(label, theme));
    }

    let mut card_col = column![top_row, pills_row.wrap().vertical_spacing(4.0)].spacing(6.0);
    card_col = card_col.push(view_task_card_footer(state, task, theme));

    let card_surface = container(card_col)
        .padding([10.0, 12.0])
        .width(Length::Fill)
        .style(move |_| container::Style {
            background: Some(
                Color {
                    a: if is_being_dragged {
                        0.45
                    } else if is_done {
                        0.72
                    } else {
                        1.0
                    },
                    ..theme.card
                }
                .into(),
            ),
            border: Border {
                color: if is_being_dragged {
                    theme.accent
                } else {
                    theme.border_subtle
                },
                width: 1.0,
                radius: RADIUS_CONTROL.into(),
            },
            shadow: theme.card_style().shadow,
            snap: true,
            ..container::Style::default()
        });

    mouse_area(card_surface)
        .on_press(map_msg(ProjectsMessage::PointerDownTask(task.id.clone())))
        .on_release(map_msg(ProjectsMessage::DropTask))
        .on_right_press(map_msg(ProjectsMessage::OpenContextMenu(task.id.clone())))
        .into()
}

fn view_task_card_footer<'a, Message: 'a>(
    state: &'a ProjectsState,
    task: &'a Task,
    theme: EstudyTheme,
) -> Element<'a, Message> {
    let (done_chk, total_chk) = task.checklist_progress();
    let mut footer_row = row![].spacing(SPACE_2).align_y(Alignment::Center);
    if total_chk > 0 {
        footer_row = footer_row.push(
            row![
                icon_svg(Icon::CheckSquare, 12.0, theme.text_muted),
                shaped_text(
                    format!("{done_chk} de {total_chk}"),
                    FONT_SIZE_XS,
                    FONT_MONO,
                    theme.text_muted,
                ),
            ]
            .spacing(4.0)
            .align_y(Alignment::Center),
        );
    }
    if let Some(proj_el) = project_tag_inline(state, task, theme) {
        footer_row = footer_row.push(proj_el);
    }
    footer_row = footer_row.push(
        container(illustrated_svg_themed(
            priority_illustrated_icon(task.priority),
            16.0,
            theme,
        ))
        .align_x(Alignment::End)
        .width(Length::Fill),
    );
    footer_row.into()
}

fn overdue_pill<'a, Message: 'a>(label: &'static str, theme: EstudyTheme) -> Element<'a, Message> {
    let tint = CategoryColor::Terracotta.solid(theme);
    container(
        row![
            icon_svg(Icon::Clock, 11.0, tint),
            shaped_text(label, 10.5, FONT_SANS_SEMIBOLD, tint),
        ]
        .spacing(4.0)
        .align_y(Alignment::Center),
    )
    .padding([3.0, 7.0])
    .style(move |_| container::Style {
        background: Some(Color { a: 0.14, ..tint }.into()),
        border: Border {
            color: Color { a: 0.30, ..tint },
            width: 1.0,
            radius: RADIUS_PILL.into(),
        },
        ..container::Style::default()
    })
    .into()
}

fn task_status_pill<'a, Message: Clone + 'a>(
    status: TaskStatus,
    on_press: Message,
    theme: EstudyTheme,
) -> Element<'a, Message> {
    let (status_label, status_tint) = task_status_visual(status, theme);
    button(
        row![
            color_dot(status_tint),
            shaped_text(status_label, 11.0, FONT_SANS_SEMIBOLD, status_tint),
        ]
        .spacing(5.0)
        .align_y(Alignment::Center),
    )
    .on_press(on_press)
    .padding([3.0, 9.0])
    .style(move |_, btn_status| button::Style {
        background: Some(
            Color {
                a: if matches!(btn_status, button::Status::Hovered) {
                    0.22
                } else {
                    0.13
                },
                ..status_tint
            }
            .into(),
        ),
        text_color: status_tint,
        border: Border {
            radius: RADIUS_PILL.into(),
            ..Border::default()
        },
        ..button::Style::default()
    })
    .into()
}

fn view_tasks_calendar_grid<'a, Message: Clone + 'a>(
    state: &'a ProjectsState,
    today: NaiveDate,
    theme: EstudyTheme,
    map_msg: &impl Fn(ProjectsMessage) -> Message,
) -> Element<'a, Message> {
    let total_days = days_in_month(state.calendar_year, state.calendar_month);
    let first = NaiveDate::from_ymd_opt(state.calendar_year, state.calendar_month, 1);
    let offset = first.map_or(0, |d| d.weekday().num_days_from_sunday() as usize);

    let mut cells: Vec<Option<NaiveDate>> = vec![None; offset];
    for d in 1..=total_days {
        cells.push(NaiveDate::from_ymd_opt(
            state.calendar_year,
            state.calendar_month,
            d,
        ));
    }

    let weekdays = ["DOM", "SEG", "TER", "QUA", "QUI", "SEX", "SÁB"];
    let mut header = row![].spacing(SPACE_2).width(Length::Fill);
    for w in weekdays {
        header = header.push(
            container(shaped_text(
                w,
                FONT_SIZE_XS,
                FONT_SANS_SEMIBOLD,
                theme.text_muted,
            ))
            .center_x(Length::Fill)
            .width(Length::FillPortion(1)),
        );
    }

    let tasks = state.filtered_tasks();
    let mut grid = column![header].spacing(SPACE_2);

    for week in cells.chunks(7) {
        let mut line = row![].spacing(SPACE_2).width(Length::Fill);
        for slot in week {
            line = line.push(view_calendar_task_cell(
                *slot, &tasks, today, theme, map_msg,
            ));
        }
        for _ in week.len()..7 {
            line = line.push(
                container(text(""))
                    .height(Length::Fixed(108.0))
                    .width(Length::FillPortion(1)),
            );
        }
        grid = grid.push(line);
    }

    scrollable(grid).height(Length::Fill).into()
}

fn view_calendar_task_cell<'a, Message: Clone + 'a>(
    slot: Option<NaiveDate>,
    tasks: &[&'a Task],
    today: NaiveDate,
    theme: EstudyTheme,
    map_msg: &impl Fn(ProjectsMessage) -> Message,
) -> Element<'a, Message> {
    let Some(date) = slot else {
        return empty_calendar_task_slot(theme);
    };

    let day_tasks: Vec<&Task> = tasks
        .iter()
        .copied()
        .filter(|t| extract_task_date(t) == Some(date))
        .collect();
    let is_today = date == today;
    let top = calendar_cell_top_bar(date, is_today, day_tasks.len(), theme);

    let mut list = column![top].spacing(4.0);
    for t in day_tasks.iter().take(3) {
        let (_, tint) = task_status_visual(t.status, theme);
        list = list.push(
            row![
                color_dot(tint),
                shaped_text(
                    truncate_str(&t.title, 20),
                    FONT_SIZE_XS,
                    FONT_SANS_MEDIUM,
                    if t.status == TaskStatus::Done {
                        theme.text_muted
                    } else {
                        theme.text_primary
                    },
                ),
            ]
            .spacing(4.0)
            .align_y(Alignment::Center),
        );
    }

    button(list)
        .on_press(map_msg(ProjectsMessage::OpenDayPeek(date)))
        .width(Length::FillPortion(1))
        .height(Length::Fixed(108.0))
        .padding(SPACE_2)
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
                color: if is_today {
                    CategoryColor::Terracotta.solid(theme)
                } else {
                    theme.border_subtle
                },
                width: if is_today { 1.5 } else { 1.0 },
                radius: RADIUS_CONTROL.into(),
            },
            ..button::Style::default()
        })
        .into()
}

fn empty_calendar_task_slot<'a, Message: 'a>(theme: EstudyTheme) -> Element<'a, Message> {
    container(text(""))
        .height(Length::Fixed(108.0))
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
        })
        .into()
}

fn calendar_cell_top_bar<'a, Message: 'a>(
    date: NaiveDate,
    is_today: bool,
    count: usize,
    theme: EstudyTheme,
) -> Element<'a, Message> {
    let day_num: Element<'a, Message> = if is_today {
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

    let mut top = row![day_num].align_y(Alignment::Center);
    if count > 0 {
        top = top.push(
            container(
                container(shaped_text(
                    format!("{count}"),
                    11.0,
                    FONT_MONO,
                    theme.on_accent,
                ))
                .padding([1.0, 6.0])
                .style(move |_| container::Style {
                    background: Some(theme.text_primary.into()),
                    border: Border {
                        radius: RADIUS_PILL.into(),
                        ..Border::default()
                    },
                    ..container::Style::default()
                }),
            )
            .align_x(Alignment::End)
            .width(Length::Fill),
        );
    }
    top.into()
}

fn view_kanban_board<'a, Message: Clone + 'a>(
    state: &'a ProjectsState,
    today: NaiveDate,
    window_width: f32,
    theme: EstudyTheme,
    map_msg: &impl Fn(ProjectsMessage) -> Message,
) -> Element<'a, Message> {
    let tasks = state.filtered_tasks();
    let use_horizontal_scroll = window_width < 1040.0;
    let mut columns_row = row![].spacing(SPACE_3).height(Length::Fill);
    if !use_horizontal_scroll {
        columns_row = columns_row.width(Length::Fill);
    }

    for status in TaskStatus::ALL {
        let col_tasks: Vec<&Task> = tasks
            .iter()
            .copied()
            .filter(|t| t.status == status)
            .collect();
        let target = DragTarget::KanbanStatus(status);
        let is_hovered =
            state.dragging_task_id.is_some() && state.hovered_drop_target == Some(target);

        let mut cards_col = column![].spacing(SPACE_2);
        for task in &col_tasks {
            cards_col = cards_col.push(view_task_card(state, task, today, theme, map_msg));
        }

        let col_box = soft_surface(
            column![
                row![
                    shaped_text(
                        status.label(),
                        FONT_SIZE_MD,
                        FONT_SANS_SEMIBOLD,
                        theme.text_primary,
                    ),
                    container(shaped_text(
                        format!("{}", col_tasks.len()),
                        FONT_SIZE_XS,
                        FONT_MONO,
                        theme.text_muted,
                    ))
                    .align_x(Alignment::End)
                    .width(Length::Fill),
                ]
                .align_y(Alignment::Center),
                hairline_divider(theme),
                scrollable(cards_col).height(Length::Fill),
            ]
            .spacing(SPACE_2),
            theme,
        )
        .padding(SPACE_3)
        .width(if use_horizontal_scroll {
            Length::Fixed(256.0)
        } else {
            Length::FillPortion(1)
        })
        .height(Length::Fill)
        .style(move |_| container::Style {
            background: Some(
                if is_hovered {
                    theme.accent_subtle
                } else {
                    theme.surface_soft
                }
                .into(),
            ),
            border: Border {
                color: if is_hovered {
                    theme.accent
                } else {
                    theme.border_subtle
                },
                width: if is_hovered { 1.5 } else { 1.0 },
                radius: RADIUS_CARD.into(),
            },
            ..container::Style::default()
        });

        columns_row = columns_row.push(
            mouse_area(col_box)
                .on_enter(map_msg(ProjectsMessage::HoverDropTarget(Some(target))))
                .on_release(map_msg(ProjectsMessage::DropTask)),
        );
    }

    if use_horizontal_scroll {
        scrollable(columns_row)
            .direction(scrollable::Direction::Horizontal(
                scrollable::Scrollbar::new().width(6).scroller_width(6),
            ))
            .style(move |_, status| theme.subtle_scrollbar_style(status))
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    } else {
        columns_row.into()
    }
}

/// Ícone ilustrado colorido correspondente ao período do dia.
#[must_use]
pub const fn turn_illustrated_icon(turn: TaskTurn) -> IllustratedIcon {
    match turn {
        TaskTurn::Anytime => IllustratedIcon::TurnAnytime,
        TaskTurn::Morning => IllustratedIcon::TurnMorning,
        TaskTurn::Afternoon => IllustratedIcon::TurnAfternoon,
        TaskTurn::Evening => IllustratedIcon::TurnEvening,
    }
}

/// Ícone ilustrado colorido correspondente à prioridade da tarefa.
#[must_use]
pub const fn priority_illustrated_icon(priority: TaskPriority) -> IllustratedIcon {
    match priority {
        TaskPriority::Low => IllustratedIcon::PriorityLow,
        TaskPriority::Medium => IllustratedIcon::PriorityMedium,
        TaskPriority::High => IllustratedIcon::PriorityHigh,
        TaskPriority::Urgent => IllustratedIcon::PriorityUrgent,
    }
}

fn turn_label_color(turn: TaskTurn, theme: EstudyTheme) -> Color {
    match turn {
        TaskTurn::Anytime => theme.text_muted,
        TaskTurn::Morning => CategoryColor::Ochre.solid(theme),
        TaskTurn::Afternoon => CategoryColor::Terracotta.solid(theme),
        TaskTurn::Evening => CategoryColor::Lavender.solid(theme),
    }
}

/// Nome do dia da semana em português.
#[must_use]
pub const fn weekday_full_pt(weekday: chrono::Weekday) -> &'static str {
    match weekday {
        chrono::Weekday::Mon => "Segunda",
        chrono::Weekday::Tue => "Terça",
        chrono::Weekday::Wed => "Quarta",
        chrono::Weekday::Thu => "Quinta",
        chrono::Weekday::Fri => "Sexta",
        chrono::Weekday::Sat => "Sábado",
        chrono::Weekday::Sun => "Domingo",
    }
}

fn truncate_str(raw: &str, max_chars: usize) -> String {
    if raw.chars().count() <= max_chars {
        raw.to_string()
    } else {
        let head: String = raw.chars().take(max_chars.saturating_sub(1)).collect();
        format!("{head}…")
    }
}

#[allow(dead_code)]
const _: f32 = SPACE_1;
