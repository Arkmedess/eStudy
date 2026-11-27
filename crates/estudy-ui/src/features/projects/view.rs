//! Renderização fluida da tela `Tarefas` (`Semana | Calendário | Kanban`), bandeja `SEM DIA` e cartões táteis.

use chrono::{Datelike, Duration, NaiveDate};
use estudy_domain::{Task, TaskPriority, TaskStatus, TaskTurn, classify_task_turn, days_in_month};
use estudy_ui_kit::widgets::{
    color_dot, editorial_page_header, hairline_divider, icon_svg, illustrated_svg, pill_chip,
    segmented_switch, shaped_text, soft_surface,
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
    month_name_pt, project_tag_inline, task_status_visual, view_day_peek_modal, view_drag_ghost,
    view_task_context_menu, view_task_modal,
};
use super::state::{DragTarget, ProjectsMessage, ProjectsState, TasksViewMode, extract_task_date};
use crate::features::home::TodayItem;

/// Largura mantida como constante pública de compatibilidade.
pub const CONTEXT_SIDEBAR_WIDTH: f32 = 264.0;

/// Renderiza a tela completa de Tarefas sem segunda sidebar, com Drag & Drop, Menu de Contexto e Modais.
pub fn view_projects<'a, Message: Clone + 'a>(
    state: &'a ProjectsState,
    today: NaiveDate,
    _today_summary: &'a [TodayItem],
    theme: EstudyTheme,
    map_msg: impl Fn(ProjectsMessage) -> Message + Clone + 'a,
) -> Element<'a, Message> {
    let new_task_btn = button(
        row![
            icon_svg(Icon::Plus, 16.0, theme.text_primary),
            shaped_text(
                "Nova tarefa",
                FONT_SIZE_BASE,
                FONT_SANS_MEDIUM,
                theme.text_primary
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
        shadow: theme.card_style().shadow,
        snap: true,
    });

    let page_header = editorial_page_header(
        "ESTUDY  /  TAREFAS",
        "Tarefas",
        "A semana e o calendário",
        Some(new_task_btn.into()),
        theme,
    );

    let controls_bar = view_controls_bar(state, theme, map_msg.clone());

    let main_body: Element<'a, Message> = match state.view_mode {
        TasksViewMode::Week => view_week_workspace(state, today, theme, &map_msg),
        TasksViewMode::Calendar => view_tasks_calendar_grid(state, today, theme, &map_msg),
        TasksViewMode::Kanban => view_kanban_board(state, theme, &map_msg),
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

    if let Some(ghost) = view_drag_ghost(state, theme) {
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
    if let Some(menu) = &state.context_menu {
        layers.push(view_task_context_menu(state, menu, theme, map_msg));
    }

    stack(layers)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

fn view_controls_bar<'a, Message: Clone + 'a>(
    state: &'a ProjectsState,
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

    let period_nav: Element<'a, Message> = match state.view_mode {
        TasksViewMode::Week => {
            let end = state.week_start + Duration::days(6);
            let label = format!(
                "{} a {} de {}",
                state.week_start.day(),
                end.day(),
                month_name_pt(end.month())
            );
            row![
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
            .align_y(Alignment::Center)
            .into()
        }
        TasksViewMode::Calendar => {
            let m_name = month_name_pt(state.calendar_month);
            let mut chars = m_name.chars();
            let cap = chars.next().map_or_else(String::new, |first| {
                format!("{}{}", first.to_uppercase(), chars.as_str())
            });
            let label = format!("{cap} de {}", state.calendar_year);
            row![
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
            .align_y(Alignment::Center)
            .into()
        }
        TasksViewMode::Kanban => shaped_text(
            "Quadro por etapa",
            FONT_SIZE_BASE,
            FONT_SANS_MEDIUM,
            theme.text_muted,
        )
        .into(),
    };

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
        let dot = CategoryColor::from_index(idx).solid(theme);
        project_filters = project_filters.push(pill_chip(
            &proj.title,
            Some(dot),
            None,
            active,
            map_msg(ProjectsMessage::SelectProject(Some(proj.id.clone()))),
            theme,
        ));
    }

    row![
        row![mode_switch, period_nav]
            .spacing(SPACE_4)
            .align_y(Alignment::Center),
        container(project_filters)
            .align_x(Alignment::End)
            .width(Length::Fill),
    ]
    .align_y(Alignment::Center)
    .width(Length::Fill)
    .into()
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

    let unscheduled_tray = view_unscheduled_tray(state, &unscheduled, theme, map_msg);

    let mut days_row = row![]
        .spacing(SPACE_3)
        .width(Length::Fill)
        .height(Length::Fill);

    for offset in 0..6_i64 {
        let day = state.week_start + Duration::days(offset);
        let day_tasks: Vec<&Task> = tasks
            .iter()
            .copied()
            .filter(|t| extract_task_date(t) == Some(day))
            .collect();

        days_row = days_row.push(view_day_column(
            state,
            day,
            day == today,
            &day_tasks,
            theme,
            map_msg,
        ));
    }

    column![unscheduled_tray, days_row]
        .spacing(SPACE_3)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

fn view_unscheduled_tray<'a, Message: Clone + 'a>(
    state: &'a ProjectsState,
    unscheduled: &[&'a Task],
    theme: EstudyTheme,
    map_msg: &impl Fn(ProjectsMessage) -> Message,
) -> Element<'a, Message> {
    let is_drop_hovered = state.dragging_task_id.is_some()
        && state.hovered_drop_target == Some(DragTarget::Unscheduled);

    let mut items_row = row![shaped_text(
        "SEM DIA",
        FONT_SIZE_XS,
        FONT_SANS_SEMIBOLD,
        theme.text_muted
    ),]
    .spacing(SPACE_3)
    .align_y(Alignment::Center);

    if unscheduled.is_empty() {
        items_row = items_row.push(shaped_text(
            if state.dragging_task_id.is_some() {
                "Solte aqui para deixar sem data fixa"
            } else {
                "Nenhuma tarefa sem data — tudo organizado na semana"
            },
            FONT_SIZE_SM,
            FONT_SANS,
            theme.text_muted,
        ));
    } else {
        for task in unscheduled.iter().take(4) {
            items_row = items_row
                .push(container(view_task_card(state, task, theme, map_msg)).max_width(270.0));
        }
    }

    let add_unscheduled = button(icon_svg(Icon::Plus, 16.0, theme.text_muted))
        .on_press(map_msg(ProjectsMessage::OpenCreateTaskModal {
            date: None,
            turn: TaskTurn::Anytime,
        }))
        .padding(6.0)
        .style(move |_, status| theme.subtle_button_style(status));

    items_row = items_row.push(
        container(add_unscheduled)
            .align_x(Alignment::End)
            .width(Length::Fill),
    );

    let tray = container(items_row)
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
    is_today: bool,
    day_tasks: &[&'a Task],
    theme: EstudyTheme,
    map_msg: &impl Fn(ProjectsMessage) -> Message,
) -> Element<'a, Message> {
    let open_count = day_tasks
        .iter()
        .filter(|t| t.status != TaskStatus::Done)
        .count();
    let col_header = view_day_column_header(day, is_today, open_count, theme, map_msg);

    let mut periods_col = column![].spacing(SPACE_2);
    for turn in TaskTurn::ALL {
        periods_col = periods_col.push(view_period_drop_slot(
            state, day, turn, day_tasks, theme, map_msg,
        ));
    }

    let border_c = if is_today {
        CategoryColor::Terracotta.border(theme)
    } else {
        theme.border_subtle
    };

    container(
        column![
            col_header,
            hairline_divider(theme),
            scrollable(periods_col).height(Length::Fill),
        ]
        .spacing(SPACE_2)
        .height(Length::Fill),
    )
    .padding(SPACE_3)
    .width(Length::FillPortion(1))
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
        illustrated_svg(turn_illustrated_icon(turn), 16.0),
        shaped_text(
            turn.label().to_uppercase(),
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
        slot_col = slot_col.push(
            container(shaped_text(
                "Arraste para cá",
                FONT_SIZE_XS,
                FONT_SANS,
                Color {
                    a: if is_hovered { 0.95 } else { 0.55 },
                    ..theme.text_muted
                },
            ))
            .padding([SPACE_3, SPACE_2])
            .center_x(Length::Fill)
            .width(Length::Fill),
        );
    } else {
        for task in matching {
            slot_col = slot_col.push(view_task_card(state, task, theme, map_msg));
        }
    }

    let slot_box = container(slot_col)
        .padding(SPACE_2)
        .width(Length::Fill)
        .style(move |_| container::Style {
            background: if is_hovered {
                Some(theme.accent_subtle.into())
            } else {
                None
            },
            border: Border {
                color: if is_hovered {
                    theme.accent
                } else {
                    Color {
                        a: 0.55,
                        ..theme.border_subtle
                    }
                },
                width: if is_hovered { 1.5 } else { 1.0 },
                radius: RADIUS_PANEL.into(),
            },
            ..container::Style::default()
        });

    mouse_area(slot_box)
        .on_enter(map_msg(ProjectsMessage::HoverDropTarget(Some(target))))
        .on_release(map_msg(ProjectsMessage::DropTask))
        .into()
}

/// Cartão tátil de tarefa com check circular `○ -> ✓`, pílula de status clicável, clique direito e Drag & Drop.
pub fn view_task_card<'a, Message: Clone + 'a>(
    state: &'a ProjectsState,
    task: &'a Task,
    theme: EstudyTheme,
    map_msg: &impl Fn(ProjectsMessage) -> Message,
) -> Element<'a, Message> {
    let is_done = task.status == TaskStatus::Done;
    let is_being_dragged = state.dragging_task_id.as_ref() == Some(&task.id);

    let radio_circle = task_radio_toggle(
        is_done,
        map_msg(ProjectsMessage::ToggleTaskDone(task.id.clone())),
        theme,
    );

    let title_btn = button(shaped_text(
        task.title.as_str(),
        FONT_SIZE_BASE,
        FONT_SANS_SEMIBOLD,
        if is_done {
            theme.text_muted
        } else {
            theme.text_primary
        },
    ))
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

    let mut meta_row = row![status_pill]
        .spacing(SPACE_2)
        .align_y(Alignment::Center);
    if let Some(proj_el) = project_tag_inline(state, task, theme) {
        meta_row = meta_row.push(proj_el);
    }
    meta_row = meta_row.push(
        container(illustrated_svg(
            priority_illustrated_icon(task.priority),
            16.0,
        ))
        .align_x(Alignment::End)
        .width(Length::Fill),
    );

    let mut card_col = column![top_row].spacing(6.0);

    let (done_chk, total_chk) = task.checklist_progress();
    if total_chk > 0 {
        card_col = card_col.push(
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
    card_col = card_col.push(meta_row);

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
                color: theme.border_subtle,
                width: 1.0,
                radius: RADIUS_CONTROL.into(),
            },
            shadow: theme.card_style().shadow,
            snap: true,
            ..container::Style::default()
        });

    mouse_area(card_surface)
        .on_press(map_msg(ProjectsMessage::BeginDragTask(task.id.clone())))
        .on_release(map_msg(ProjectsMessage::DropTask))
        .on_right_press(map_msg(ProjectsMessage::OpenContextMenu(task.id.clone())))
        .into()
}

fn task_radio_toggle<'a, Message: Clone + 'a>(
    is_done: bool,
    on_press: Message,
    theme: EstudyTheme,
) -> Element<'a, Message> {
    button(if is_done {
        container(icon_svg(
            Icon::CheckCircle2,
            16.0,
            CategoryColor::Sage.solid(theme),
        ))
        .center_x(Length::Fixed(18.0))
        .center_y(Length::Fixed(18.0))
    } else {
        container(text("").size(1))
            .width(Length::Fixed(16.0))
            .height(Length::Fixed(16.0))
            .style(move |_| container::Style {
                border: Border {
                    color: Color {
                        a: 0.45,
                        ..theme.text_muted
                    },
                    width: 1.5,
                    radius: RADIUS_PILL.into(),
                },
                ..container::Style::default()
            })
    })
    .on_press(on_press)
    .padding(0.0)
    .width(Length::Fixed(20.0))
    .height(Length::Fixed(20.0))
    .style(|_, _| button::Style::default())
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
    theme: EstudyTheme,
    map_msg: &impl Fn(ProjectsMessage) -> Message,
) -> Element<'a, Message> {
    let tasks = state.filtered_tasks();
    let mut columns_row = row![]
        .spacing(SPACE_3)
        .width(Length::Fill)
        .height(Length::Fill);

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
            cards_col = cards_col.push(view_task_card(state, task, theme, map_msg));
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
        .width(Length::FillPortion(1))
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

    columns_row.into()
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
