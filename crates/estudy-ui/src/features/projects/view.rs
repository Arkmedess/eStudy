//! Renderização da fatia Projetos: barra contextual (240px), Kanban, frameworks e Drawer lateral.

use chrono::{Datelike, NaiveDate};
use estudy_domain::{
    EisenhowerQuadrant, MAX_TASK_CHECKLIST_ITEMS, ProjectFramework, Task, TaskPriority, TaskStatus,
};
use estudy_ui_kit::widgets::{card, icon_svg, panel, section_header, status_badge};
use estudy_ui_kit::{EstudyTheme, Icon, RADIUS_CONTROL, SPACE_1, SPACE_2, SPACE_3, SPACE_4};
use iced::widget::{
    button, column, container, progress_bar, row, scrollable, stack, text, text_input,
};
use iced::{Alignment, Border, Color, Element, Length};
use std::collections::BTreeMap;

use super::state::{DescriptionTab, ProjectsMessage, ProjectsState, TaskDrawerState};
use crate::features::home::TodayItem;

/// Largura fixa da barra lateral esquerda contextual (`240px`).
pub const CONTEXT_SIDEBAR_WIDTH: f32 = 240.0;

/// Renderiza a visão completa de Projetos com barra contextual opcional e Drawer sobreposto.
pub fn view_projects<'a, Message: Clone + 'a>(
    state: &'a ProjectsState,
    today: NaiveDate,
    today_summary: &'a [TodayItem],
    theme: EstudyTheme,
    map_msg: impl Fn(ProjectsMessage) -> Message + Clone + 'a,
) -> Element<'a, Message> {
    let main_area = view_main_workspace(state, theme, map_msg.clone());

    let content: Element<'a, Message> = if state.sidebar_visible {
        let sidebar = view_context_sidebar(state, today, today_summary, theme, map_msg.clone());
        row![sidebar, main_area]
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    } else {
        main_area
    };

    if let (Some(drawer), Some(task)) = (&state.drawer, state.drawer_task()) {
        let overlay = view_task_drawer(state, drawer, task, theme, map_msg);
        stack![content, overlay].into()
    } else {
        content
    }
}

fn view_context_sidebar<'a, Message: Clone + 'a>(
    state: &'a ProjectsState,
    today: NaiveDate,
    today_summary: &'a [TodayItem],
    theme: EstudyTheme,
    map_msg: impl Fn(ProjectsMessage) -> Message + 'a,
) -> Element<'a, Message> {
    let mini_cal = view_mini_calendar(state, today, theme, &map_msg);

    let mut today_col = column![].spacing(SPACE_1);
    if today_summary.is_empty() {
        today_col = today_col.push(
            text("Sem blocos para hoje.")
                .size(11)
                .color(theme.text_muted),
        );
    } else {
        for item in today_summary.iter().take(4) {
            today_col = today_col.push(
                row![
                    text(&item.time_label).size(11).color(theme.text_muted),
                    text(&item.title).size(11).color(theme.text_primary),
                ]
                .spacing(SPACE_1),
            );
        }
    }

    let mut proj_list = column![].spacing(SPACE_1);
    let all_active = state.selected_project_id.is_none();
    proj_list = proj_list.push(sidebar_project_btn(
        "Todos os projetos",
        all_active,
        map_msg(ProjectsMessage::SelectProject(None)),
        theme,
    ));

    for proj in &state.projects {
        let active = state.selected_project_id.as_ref() == Some(&proj.id);
        proj_list = proj_list.push(sidebar_project_btn(
            &proj.title,
            active,
            map_msg(ProjectsMessage::SelectProject(Some(proj.id.clone()))),
            theme,
        ));
    }

    container(
        scrollable(
            column![
                mini_cal,
                panel(
                    column![section_header(Icon::Clock, "Hoje", theme), today_col].spacing(SPACE_2),
                    theme
                ),
                panel(
                    column![
                        section_header(Icon::FolderKanban, "Projetos", theme),
                        proj_list
                    ]
                    .spacing(SPACE_2),
                    theme
                ),
            ]
            .spacing(SPACE_3),
        )
        .height(Length::Fill),
    )
    .width(Length::Fixed(CONTEXT_SIDEBAR_WIDTH))
    .height(Length::Fill)
    .padding(SPACE_3)
    .style(move |_| container::Style {
        background: Some(theme.panel.into()),
        border: Border {
            color: theme.border_subtle,
            width: 1.0,
            radius: 0.0.into(),
        },
        ..container::Style::default()
    })
    .into()
}

fn view_mini_calendar<'a, Message: Clone + 'a>(
    state: &'a ProjectsState,
    today: NaiveDate,
    theme: EstudyTheme,
    map_msg: &impl Fn(ProjectsMessage) -> Message,
) -> Element<'a, Message> {
    let header = row![
        button(icon_svg(Icon::ChevronLeft, 13.0, theme.text_primary))
            .on_press(map_msg(ProjectsMessage::ShiftCalendarMonth(-1)))
            .padding(4.0)
            .style(move |_, status| theme.ghost_button_style(status)),
        container(
            text(format!(
                "{:02}/{}",
                state.calendar_month, state.calendar_year
            ))
            .size(12)
            .color(theme.text_primary),
        )
        .width(Length::Fill)
        .align_x(Alignment::Center),
        button(icon_svg(Icon::ChevronRight, 13.0, theme.text_primary))
            .on_press(map_msg(ProjectsMessage::ShiftCalendarMonth(1)))
            .padding(4.0)
            .style(move |_, status| theme.ghost_button_style(status)),
    ]
    .align_y(Alignment::Center);

    let first_day = NaiveDate::from_ymd_opt(state.calendar_year, state.calendar_month, 1);
    let mut grid = column![].spacing(2.0);
    if let Some(start) = first_day {
        let offset = start.weekday().num_days_from_monday();
        let days_in_month = count_days_in_month(state.calendar_year, state.calendar_month);
        let mut cells: Vec<Option<NaiveDate>> = vec![None; offset as usize];
        for day in 1..=days_in_month {
            cells.push(NaiveDate::from_ymd_opt(
                state.calendar_year,
                state.calendar_month,
                day,
            ));
        }

        for week in cells.chunks(7) {
            let mut week_row = row![].spacing(2.0);
            for cell in week {
                if let Some(date) = cell {
                    let is_selected = *date == state.selected_day;
                    let is_today = *date == today;
                    let label_color = if is_selected {
                        theme.on_accent
                    } else if is_today {
                        theme.accent
                    } else {
                        theme.text_primary
                    };
                    week_row = week_row.push(
                        button(
                            text(format!("{:02}", date.day()))
                                .size(10)
                                .color(label_color),
                        )
                        .on_press(map_msg(ProjectsMessage::SelectCalendarDay(*date)))
                        .width(Length::Fixed(26.0))
                        .padding([3.0, 2.0])
                        .style(move |_, status| {
                            if is_selected {
                                theme.primary_button_style(status)
                            } else {
                                theme.ghost_button_style(status)
                            }
                        }),
                    );
                } else {
                    week_row = week_row.push(container(text("")).width(Length::Fixed(26.0)));
                }
            }
            grid = grid.push(week_row);
        }
    }

    panel(column![header, grid].spacing(SPACE_2), theme).into()
}

fn view_main_workspace<'a, Message: Clone + 'a>(
    state: &'a ProjectsState,
    theme: EstudyTheme,
    map_msg: impl Fn(ProjectsMessage) -> Message + Clone + 'a,
) -> Element<'a, Message> {
    let toolbar = view_top_toolbar(state, theme, &map_msg);
    let events_and_tags = view_events_and_tags_strip(state, theme, &map_msg);

    let quick_add = row![
        text_input("Nova tarefa rápida...", &state.quick_task_title)
            .on_input({
                let map_msg = map_msg.clone();
                move |val| map_msg(ProjectsMessage::QuickTaskTitleChanged(val))
            })
            .on_submit(map_msg(ProjectsMessage::CreateQuickTask))
            .padding(SPACE_2)
            .size(13),
        button(
            row![
                icon_svg(Icon::Plus, 14.0, theme.on_accent),
                text("Adicionar").size(12).color(theme.on_accent),
            ]
            .spacing(SPACE_1)
            .align_y(Alignment::Center),
        )
        .on_press(map_msg(ProjectsMessage::CreateQuickTask))
        .padding([SPACE_2, SPACE_3])
        .style(move |_, status| theme.primary_button_style(status)),
    ]
    .spacing(SPACE_2)
    .align_y(Alignment::Center);

    let board = match state.active_framework() {
        ProjectFramework::Kanban => view_kanban_board(state, theme, &map_msg),
        ProjectFramework::Milestones => view_milestones_board(state, theme, &map_msg),
        ProjectFramework::Eisenhower => view_eisenhower_matrix(state, theme, &map_msg),
        ProjectFramework::WeeklyFocus => view_weekly_focus_board(state, theme, &map_msg),
    };

    container(
        column![toolbar, events_and_tags, quick_add, board]
            .spacing(SPACE_3)
            .width(Length::Fill)
            .height(Length::Fill),
    )
    .padding(SPACE_4)
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}

fn view_top_toolbar<'a, Message: Clone + 'a>(
    state: &'a ProjectsState,
    theme: EstudyTheme,
    map_msg: &impl Fn(ProjectsMessage) -> Message,
) -> Element<'a, Message> {
    let title = state
        .active_project()
        .map_or("Todos os Projetos", |proj| proj.title.as_str());
    let active_fw = state.active_framework();

    let mut fw_chips = row![].spacing(SPACE_1);
    for fw in ProjectFramework::ALL {
        let active = fw == active_fw;
        fw_chips = fw_chips.push(
            button(text(fw.label()).size(11).color(if active {
                theme.on_accent
            } else {
                theme.text_primary
            }))
            .on_press(map_msg(ProjectsMessage::SwitchFramework(fw)))
            .padding([SPACE_1, SPACE_2])
            .style(move |_, status| {
                if active {
                    theme.primary_button_style(status)
                } else {
                    theme.ghost_button_style(status)
                }
            }),
        );
    }

    row![
        button(icon_svg(Icon::PanelLeft, 15.0, theme.text_primary))
            .on_press(map_msg(ProjectsMessage::ToggleSidebar))
            .padding(SPACE_2)
            .style(move |_, status| theme.ghost_button_style(status)),
        text(title).size(18).color(theme.text_primary),
        container(fw_chips)
            .align_x(Alignment::End)
            .width(Length::Fill),
    ]
    .spacing(SPACE_3)
    .align_y(Alignment::Center)
    .into()
}

fn view_events_and_tags_strip<'a, Message: Clone + 'a>(
    state: &'a ProjectsState,
    theme: EstudyTheme,
    map_msg: &impl Fn(ProjectsMessage) -> Message,
) -> Element<'a, Message> {
    let mut events_row =
        row![section_header(Icon::Calendar, "Próximos eventos:", theme)].spacing(SPACE_2);
    for ev in state.upcoming_events.iter().take(3) {
        events_row = events_row.push(status_badge(
            format!("{} · {}", ev.when_label, ev.title),
            theme.accent,
            theme,
        ));
    }

    let mut tags_row = row![icon_svg(Icon::Tag, 13.0, theme.text_muted)].spacing(SPACE_1);
    let all_active = state.active_tag_filter.is_none();
    tags_row = tags_row.push(
        button(text("Todas").size(11).color(if all_active {
            theme.on_accent
        } else {
            theme.text_primary
        }))
        .on_press(map_msg(ProjectsMessage::ToggleTagFilter(None)))
        .padding([3.0, 8.0])
        .style(move |_, status| {
            if all_active {
                theme.primary_button_style(status)
            } else {
                theme.ghost_button_style(status)
            }
        }),
    );

    for tag in state.available_tags() {
        let active = state.active_tag_filter.as_deref() == Some(tag.as_str());
        tags_row = tags_row.push(
            button(text(format!("#{tag}")).size(11).color(if active {
                theme.on_accent
            } else {
                theme.text_primary
            }))
            .on_press(map_msg(ProjectsMessage::ToggleTagFilter(Some(tag))))
            .padding([3.0, 8.0])
            .style(move |_, status| {
                if active {
                    theme.primary_button_style(status)
                } else {
                    theme.ghost_button_style(status)
                }
            }),
        );
    }

    panel(column![events_row, tags_row].spacing(SPACE_2), theme).into()
}

fn view_kanban_board<'a, Message: Clone + 'a>(
    state: &'a ProjectsState,
    theme: EstudyTheme,
    map_msg: &impl Fn(ProjectsMessage) -> Message,
) -> Element<'a, Message> {
    let tasks = state.filtered_tasks();
    let mut columns_row = row![].spacing(SPACE_3).width(Length::Fill);

    for status in TaskStatus::ALL {
        let col_tasks: Vec<&Task> = tasks
            .iter()
            .copied()
            .filter(|task| task.status == status)
            .collect();

        let mut cards_col = column![].spacing(SPACE_2);
        for task in &col_tasks {
            cards_col = cards_col.push(view_task_card(task, theme, map_msg));
        }

        let col_panel = panel(
            column![
                row![
                    text(status.label()).size(13).color(theme.text_primary),
                    container(
                        text(format!("{}", col_tasks.len()))
                            .size(12)
                            .color(theme.text_muted)
                    )
                    .align_x(Alignment::End)
                    .width(Length::Fill),
                ]
                .align_y(Alignment::Center),
                scrollable(cards_col).height(Length::Fill),
            ]
            .spacing(SPACE_2),
            theme,
        )
        .width(Length::Fill)
        .height(Length::Fill);

        columns_row = columns_row.push(col_panel);
    }

    columns_row.into()
}

fn view_milestones_board<'a, Message: Clone + 'a>(
    state: &'a ProjectsState,
    theme: EstudyTheme,
    map_msg: &impl Fn(ProjectsMessage) -> Message,
) -> Element<'a, Message> {
    let tasks = state.filtered_tasks();
    let active_proj_id = state.selected_project_id.as_ref();
    let mut grouped: BTreeMap<String, Vec<&Task>> = BTreeMap::new();

    for task in tasks {
        let milestone = task
            .project_links
            .iter()
            .find(|link| active_proj_id.is_none_or(|id| *id == link.project_id))
            .and_then(|link| link.milestone_name.clone())
            .unwrap_or_else(|| "Sem marco".to_string());
        grouped.entry(milestone).or_default().push(task);
    }

    let mut list = column![].spacing(SPACE_3);
    for (milestone, group) in grouped {
        let done = group
            .iter()
            .filter(|task| task.status == TaskStatus::Done)
            .count();
        let total = group.len();
        #[allow(clippy::cast_precision_loss)]
        let ratio = if total == 0 {
            0.0
        } else {
            done as f32 / total as f32
        };

        let mut cards = column![].spacing(SPACE_2);
        for task in group {
            cards = cards.push(view_task_card(task, theme, map_msg));
        }

        list = list.push(card(
            column![
                row![
                    section_header(Icon::Layers, milestone, theme),
                    container(
                        text(format!("{done}/{total} concluídas"))
                            .size(12)
                            .color(theme.text_muted)
                    )
                    .align_x(Alignment::End)
                    .width(Length::Fill),
                ],
                progress_bar(0.0..=1.0, ratio),
                cards,
            ]
            .spacing(SPACE_2),
            theme,
        ));
    }

    scrollable(list).height(Length::Fill).into()
}

fn view_eisenhower_matrix<'a, Message: Clone + 'a>(
    state: &'a ProjectsState,
    theme: EstudyTheme,
    map_msg: &impl Fn(ProjectsMessage) -> Message,
) -> Element<'a, Message> {
    let tasks = state.filtered_tasks();
    let mut rows = column![].spacing(SPACE_3).height(Length::Fill);

    for pair in EisenhowerQuadrant::ALL.chunks(2) {
        let mut row_line = row![].spacing(SPACE_3).height(Length::Fill);
        for quadrant in pair {
            let matching: Vec<&Task> = tasks
                .iter()
                .copied()
                .filter(|task| {
                    task.eisenhower_quadrant
                        .unwrap_or(EisenhowerQuadrant::Schedule)
                        == *quadrant
                })
                .collect();

            let mut col_cards = column![].spacing(SPACE_2);
            for task in matching {
                col_cards = col_cards.push(view_task_card(task, theme, map_msg));
            }

            row_line = row_line.push(
                panel(
                    column![
                        section_header(Icon::LayoutGrid, quadrant.label(), theme),
                        scrollable(col_cards).height(Length::Fill),
                    ]
                    .spacing(SPACE_2),
                    theme,
                )
                .width(Length::Fill)
                .height(Length::Fill),
            );
        }
        rows = rows.push(row_line);
    }

    rows.into()
}

fn view_weekly_focus_board<'a, Message: Clone + 'a>(
    state: &'a ProjectsState,
    theme: EstudyTheme,
    map_msg: &impl Fn(ProjectsMessage) -> Message,
) -> Element<'a, Message> {
    let mut list = column![].spacing(SPACE_2);
    for task in state.filtered_tasks() {
        let selected = state.weekly_focus_ids.contains(&task.id);
        let toggle_btn = button(
            text(if selected { "Na semana" } else { "Selecionar" })
                .size(11)
                .color(if selected {
                    theme.on_accent
                } else {
                    theme.text_primary
                }),
        )
        .on_press(map_msg(ProjectsMessage::ToggleWeeklyFocus(task.id.clone())))
        .padding([SPACE_1, SPACE_2])
        .style(move |_, status| {
            if selected {
                theme.primary_button_style(status)
            } else {
                theme.ghost_button_style(status)
            }
        });

        list = list.push(card(
            row![
                toggle_btn,
                button(text(&task.title).size(13).color(theme.text_primary))
                    .on_press(map_msg(ProjectsMessage::OpenTaskDrawer(task.id.clone())))
                    .width(Length::Fill)
                    .style(move |_, status| theme.ghost_button_style(status)),
                status_badge(task.status.label(), theme.accent, theme),
            ]
            .spacing(SPACE_2)
            .align_y(Alignment::Center),
            theme,
        ));
    }
    scrollable(list).height(Length::Fill).into()
}

fn view_task_card<'a, Message: Clone + 'a>(
    task: &'a Task,
    theme: EstudyTheme,
    map_msg: &impl Fn(ProjectsMessage) -> Message,
) -> Element<'a, Message> {
    let (done, total) = task.checklist_progress();
    let mut meta = row![status_badge(task.priority.label(), theme.accent, theme)].spacing(SPACE_1);
    if total > 0 {
        meta = meta.push(
            text(format!("{done}/{total}"))
                .size(11)
                .color(theme.text_muted),
        );
    }
    if task.pinned_focus {
        meta = meta.push(icon_svg(Icon::Pin, 12.0, theme.warning));
    }

    let next_status = match task.status {
        TaskStatus::Backlog => Some(TaskStatus::Todo),
        TaskStatus::Todo => Some(TaskStatus::InProgress),
        TaskStatus::InProgress => Some(TaskStatus::Done),
        TaskStatus::Done => None,
    };

    let open_btn = button(
        column![text(&task.title).size(13).color(theme.text_primary), meta,].spacing(SPACE_1),
    )
    .on_press(map_msg(ProjectsMessage::OpenTaskDrawer(task.id.clone())))
    .width(Length::Fill)
    .padding(SPACE_2)
    .style(move |_, status| theme.ghost_button_style(status));

    let mut card_col = column![open_btn].spacing(SPACE_1);
    if let Some(target_status) = next_status {
        card_col = card_col.push(
            button(
                row![
                    icon_svg(Icon::ChevronRight, 12.0, theme.text_muted),
                    text(format!("Mover para {}", target_status.label()))
                        .size(11)
                        .color(theme.text_muted),
                ]
                .spacing(4.0)
                .align_y(Alignment::Center),
            )
            .on_press(map_msg(ProjectsMessage::MoveTaskStatus(
                task.id.clone(),
                target_status,
            )))
            .padding([3.0, 6.0])
            .style(move |_, status| theme.ghost_button_style(status)),
        );
    }

    card(card_col, theme).into()
}

fn view_task_drawer<'a, Message: Clone + 'a>(
    state: &'a ProjectsState,
    drawer: &'a TaskDrawerState,
    task: &'a Task,
    theme: EstudyTheme,
    map_msg: impl Fn(ProjectsMessage) -> Message + Clone + 'a,
) -> Element<'a, Message> {
    let header = view_drawer_header(task, theme, map_msg.clone());
    let schedule = view_drawer_schedule(state, task, theme, &map_msg);
    let checklist = view_drawer_checklist(drawer, task, theme, map_msg.clone());
    let markdown = view_drawer_markdown(drawer, task, theme, map_msg.clone());
    let tags = view_drawer_tags(drawer, task, theme, map_msg);

    let mut body = column![header, schedule, checklist, markdown, tags].spacing(SPACE_3);
    if let Some(notice) = &drawer.notice {
        body = body.push(status_badge(notice, theme.warning, theme));
    }

    let drawer_box = container(scrollable(body).height(Length::Fill))
        .width(Length::Fixed(420.0))
        .height(Length::Fill)
        .padding(SPACE_4)
        .style(move |_| container::Style {
            background: Some(theme.card.into()),
            text_color: Some(theme.text_primary),
            border: Border {
                color: theme.border_subtle,
                width: 1.0,
                radius: RADIUS_CONTROL.into(),
            },
            ..container::Style::default()
        });

    container(drawer_box)
        .width(Length::Fill)
        .height(Length::Fill)
        .align_x(Alignment::End)
        .style(|_| container::Style {
            background: Some(Color::from_rgba(0.0, 0.0, 0.0, 0.35).into()),
            ..container::Style::default()
        })
        .into()
}

fn view_drawer_header<'a, Message: Clone + 'a>(
    task: &'a Task,
    theme: EstudyTheme,
    map_msg: impl Fn(ProjectsMessage) -> Message + Clone + 'a,
) -> Element<'a, Message> {
    let pin_btn = button(
        row![
            icon_svg(
                Icon::Pin,
                14.0,
                if task.pinned_focus {
                    theme.on_accent
                } else {
                    theme.text_primary
                },
            ),
            text(if task.pinned_focus {
                "Em foco"
            } else {
                "Fixar"
            })
            .size(12)
            .color(if task.pinned_focus {
                theme.on_accent
            } else {
                theme.text_primary
            }),
        ]
        .spacing(SPACE_1)
        .align_y(Alignment::Center),
    )
    .on_press(map_msg(ProjectsMessage::DrawerToggleFocusPin))
    .padding([SPACE_1, SPACE_2])
    .style(move |_, status| {
        if task.pinned_focus {
            theme.primary_button_style(status)
        } else {
            theme.ghost_button_style(status)
        }
    });

    let close_btn = button(icon_svg(Icon::X, 15.0, theme.text_muted))
        .on_press(map_msg(ProjectsMessage::CloseTaskDrawer))
        .padding(4.0)
        .style(move |_, status| theme.ghost_button_style(status));

    let title_input = text_input("Título da tarefa", &task.title)
        .on_input(move |val| map_msg(ProjectsMessage::DrawerTitleChanged(val)))
        .padding(SPACE_1)
        .size(15);

    column![
        row![
            status_badge(task.status.label(), theme.accent, theme),
            container(row![pin_btn, close_btn].spacing(SPACE_1))
                .align_x(Alignment::End)
                .width(Length::Fill),
        ]
        .align_y(Alignment::Center),
        title_input,
    ]
    .spacing(SPACE_2)
    .into()
}

fn view_drawer_schedule<'a, Message: Clone + 'a>(
    state: &'a ProjectsState,
    task: &'a Task,
    theme: EstudyTheme,
    map_msg: &impl Fn(ProjectsMessage) -> Message,
) -> Element<'a, Message> {
    let mut proj_row = row![].spacing(SPACE_1);
    for proj in &state.projects {
        let linked = task
            .project_links
            .iter()
            .any(|link| link.project_id == proj.id);
        proj_row = proj_row.push(
            button(text(&proj.title).size(11).color(if linked {
                theme.on_accent
            } else {
                theme.text_primary
            }))
            .on_press(map_msg(ProjectsMessage::DrawerToggleProjectLink(
                proj.id.clone(),
            )))
            .padding([3.0, 8.0])
            .style(move |_, status| {
                if linked {
                    theme.primary_button_style(status)
                } else {
                    theme.ghost_button_style(status)
                }
            }),
        );
    }

    let mut prio_row = row![].spacing(SPACE_1);
    for prio in TaskPriority::ALL {
        let active = task.priority == prio;
        prio_row = prio_row.push(
            button(text(prio.label()).size(11).color(if active {
                theme.on_accent
            } else {
                theme.text_primary
            }))
            .on_press(map_msg(ProjectsMessage::DrawerPriorityChanged(prio)))
            .padding([3.0, 8.0])
            .style(move |_, status| {
                if active {
                    theme.primary_button_style(status)
                } else {
                    theme.ghost_button_style(status)
                }
            }),
        );
    }

    let dates = text(format!(
        "Início: {} · Prazo: {} · Recorrência: {}",
        task.start_at.as_deref().unwrap_or("—"),
        task.due_at.as_deref().unwrap_or("—"),
        task.recurrence.as_deref().unwrap_or("única"),
    ))
    .size(11)
    .color(theme.text_muted);

    panel(
        column![
            text("Projetos vinculados:")
                .size(11)
                .color(theme.text_muted),
            proj_row,
            text("Prioridade:").size(11).color(theme.text_muted),
            prio_row,
            dates,
        ]
        .spacing(SPACE_2),
        theme,
    )
    .into()
}

fn view_drawer_checklist<'a, Message: Clone + 'a>(
    drawer: &'a TaskDrawerState,
    task: &'a Task,
    theme: EstudyTheme,
    map_msg: impl Fn(ProjectsMessage) -> Message + Clone + 'a,
) -> Element<'a, Message> {
    let (done, total) = task.checklist_progress();
    let header = section_header(
        Icon::CheckSquare,
        format!("Checklist ({done}/{total} · máx {MAX_TASK_CHECKLIST_ITEMS})"),
        theme,
    );

    let mut items_col = column![].spacing(SPACE_1);
    for item in &task.checklist {
        let icon = if item.done {
            Icon::CheckCircle2
        } else {
            Icon::CheckSquare
        };
        let tint = if item.done {
            theme.success
        } else {
            theme.text_muted
        };
        items_col = items_col.push(
            row![
                button(
                    row![
                        icon_svg(icon, 14.0, tint),
                        text(&item.title).size(12).color(theme.text_primary),
                    ]
                    .spacing(SPACE_2)
                    .align_y(Alignment::Center),
                )
                .on_press(map_msg(ProjectsMessage::DrawerToggleChecklistItem(
                    item.id.clone()
                )))
                .width(Length::Fill)
                .style(move |_, status| theme.ghost_button_style(status)),
                button(icon_svg(Icon::Trash2, 13.0, theme.text_muted))
                    .on_press(map_msg(ProjectsMessage::DrawerRemoveChecklistItem(
                        item.id.clone()
                    )))
                    .padding(4.0)
                    .style(move |_, status| theme.ghost_button_style(status)),
            ]
            .spacing(SPACE_1)
            .align_y(Alignment::Center),
        );
    }

    let input_row = row![
        text_input("Novo item da checklist...", &drawer.new_checklist_title)
            .on_input({
                let map_msg = map_msg.clone();
                move |val| map_msg(ProjectsMessage::DrawerChecklistInputChanged(val))
            })
            .on_submit(map_msg(ProjectsMessage::DrawerAddChecklistItem))
            .padding(SPACE_1)
            .size(12),
        button(icon_svg(Icon::Plus, 14.0, theme.on_accent))
            .on_press(map_msg(ProjectsMessage::DrawerAddChecklistItem))
            .padding(SPACE_1)
            .style(move |_, status| theme.primary_button_style(status)),
    ]
    .spacing(SPACE_1);

    panel(
        column![header, items_col, input_row].spacing(SPACE_2),
        theme,
    )
    .into()
}

fn view_drawer_markdown<'a, Message: Clone + 'a>(
    drawer: &'a TaskDrawerState,
    task: &'a Task,
    theme: EstudyTheme,
    map_msg: impl Fn(ProjectsMessage) -> Message + Clone + 'a,
) -> Element<'a, Message> {
    let tabs = row![
        tab_button(
            "Escrever",
            drawer.description_tab == DescriptionTab::Write,
            map_msg(ProjectsMessage::SetDescriptionTab(DescriptionTab::Write)),
            theme
        ),
        tab_button(
            "Visualizar",
            drawer.description_tab == DescriptionTab::Preview,
            map_msg(ProjectsMessage::SetDescriptionTab(DescriptionTab::Preview)),
            theme
        ),
    ]
    .spacing(SPACE_1);

    let body: Element<'a, Message> = match drawer.description_tab {
        DescriptionTab::Write => text_input("Descrição em Markdown...", &task.description_md)
            .on_input(move |val| map_msg(ProjectsMessage::DrawerDescriptionChanged(val)))
            .padding(SPACE_2)
            .size(12)
            .into(),
        DescriptionTab::Preview => render_markdown_preview(&task.description_md, theme),
    };

    panel(
        column![
            row![
                section_header(Icon::Code2, "Descrição", theme),
                container(tabs).align_x(Alignment::End).width(Length::Fill)
            ],
            body,
        ]
        .spacing(SPACE_2),
        theme,
    )
    .into()
}

fn view_drawer_tags<'a, Message: Clone + 'a>(
    drawer: &'a TaskDrawerState,
    task: &'a Task,
    theme: EstudyTheme,
    map_msg: impl Fn(ProjectsMessage) -> Message + Clone + 'a,
) -> Element<'a, Message> {
    let mut tags_row = row![].spacing(SPACE_1);
    for tag in &task.tags {
        tags_row = tags_row.push(
            button(text(format!("#{tag} ×")).size(11).color(theme.accent))
                .on_press(map_msg(ProjectsMessage::DrawerRemoveTag(tag.clone())))
                .padding([3.0, 8.0])
                .style(move |_, status| theme.ghost_button_style(status)),
        );
    }

    let add_row = row![
        text_input("Adicionar tag...", &drawer.new_tag_name)
            .on_input({
                let map_msg = map_msg.clone();
                move |val| map_msg(ProjectsMessage::DrawerTagInputChanged(val))
            })
            .on_submit(map_msg(ProjectsMessage::DrawerAddTag))
            .padding(SPACE_1)
            .size(12),
        button(icon_svg(Icon::Plus, 13.0, theme.on_accent))
            .on_press(map_msg(ProjectsMessage::DrawerAddTag))
            .padding(SPACE_1)
            .style(move |_, status| theme.primary_button_style(status)),
    ]
    .spacing(SPACE_1);

    panel(
        column![section_header(Icon::Tag, "Tags", theme), tags_row, add_row,].spacing(SPACE_2),
        theme,
    )
    .into()
}

fn render_markdown_preview<'a, Message: 'a>(
    source: &'a str,
    theme: EstudyTheme,
) -> Element<'a, Message> {
    if source.trim().is_empty() {
        return text("Sem descrição adicionada.")
            .size(12)
            .color(theme.text_muted)
            .into();
    }

    let mut col = column![].spacing(SPACE_1);
    for raw_line in source.lines() {
        let line = raw_line.trim();
        if let Some(heading) = line.strip_prefix("# ") {
            col = col.push(text(heading).size(15).color(theme.text_primary));
        } else if let Some(subheading) = line.strip_prefix("## ") {
            col = col.push(text(subheading).size(14).color(theme.accent));
        } else if let Some(bullet) = line.strip_prefix("- ") {
            col = col.push(
                text(format!("• {bullet}"))
                    .size(12)
                    .color(theme.text_primary),
            );
        } else if !line.is_empty() {
            col = col.push(text(line).size(12).color(theme.text_primary));
        }
    }
    col.into()
}

fn sidebar_project_btn<'a, Message: Clone + 'a>(
    label: &'a str,
    active: bool,
    on_press: Message,
    theme: EstudyTheme,
) -> Element<'a, Message> {
    button(text(label).size(12).color(if active {
        theme.on_accent
    } else {
        theme.text_primary
    }))
    .on_press(on_press)
    .width(Length::Fill)
    .padding([SPACE_1, SPACE_2])
    .style(move |_, status| {
        if active {
            theme.primary_button_style(status)
        } else {
            theme.ghost_button_style(status)
        }
    })
    .into()
}

fn tab_button<'a, Message: Clone + 'a>(
    label: &'a str,
    active: bool,
    on_press: Message,
    theme: EstudyTheme,
) -> Element<'a, Message> {
    button(text(label).size(11).color(if active {
        theme.on_accent
    } else {
        theme.text_primary
    }))
    .on_press(on_press)
    .padding([3.0, 8.0])
    .style(move |_, status| {
        if active {
            theme.primary_button_style(status)
        } else {
            theme.ghost_button_style(status)
        }
    })
    .into()
}

fn count_days_in_month(year: i32, month: u32) -> u32 {
    let (next_year, next_month) = if month == 12 {
        (year + 1, 1)
    } else {
        (year, month + 1)
    };
    let current = NaiveDate::from_ymd_opt(year, month, 1);
    let next = NaiveDate::from_ymd_opt(next_year, next_month, 1);
    match (current, next) {
        (Some(start), Some(end)) => u32::try_from((end - start).num_days()).unwrap_or(30),
        _ => 30,
    }
}
