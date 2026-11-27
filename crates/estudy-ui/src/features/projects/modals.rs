//! Modais previsíveis da tela de Tarefas: Modal `Nova/Editar Tarefa` (multi-linha real), `Card do Dia` e `Menu de Contexto` no cursor.

use chrono::{Datelike, Duration, NaiveDate};
use estudy_domain::{Task, TaskPriority, TaskStatus, TaskTurn, classify_task_turn};
use estudy_ui_kit::widgets::{
    color_dot, hairline_divider, icon_svg, illustrated_svg, modal_surface, shaped_text,
    soft_surface,
};
use estudy_ui_kit::{
    CategoryColor, EstudyTheme, FONT_MONO, FONT_SANS, FONT_SANS_MEDIUM, FONT_SANS_SEMIBOLD,
    FONT_SERIF, FONT_SERIF_ITALIC, FONT_SERIF_MEDIUM, FONT_SIZE_2XL, FONT_SIZE_BASE, FONT_SIZE_LG,
    FONT_SIZE_MD, FONT_SIZE_SM, FONT_SIZE_XL, FONT_SIZE_XS, Icon, RADIUS_CARD, RADIUS_CONTROL,
    RADIUS_PANEL, RADIUS_PILL, SPACE_1, SPACE_2, SPACE_3, SPACE_4, SPACE_5,
};
use iced::widget::{
    button, column, container, mouse_area, row, scrollable, text, text_editor, text_input,
};
use iced::{Alignment, Border, Color, Element, Length};

use super::state::{
    ContextSubmenu, ProjectsMessage, ProjectsState, TaskContextMenu, TaskModalMode, TaskModalState,
    extract_task_date,
};
use super::view::{priority_illustrated_icon, turn_illustrated_icon, weekday_full_pt};

/// Renderiza o modal central estruturado de criação/edição de tarefa (com `text_editor` multi-linha real).
#[allow(clippy::too_many_lines)]
pub fn view_task_modal<'a, Message: Clone + 'a>(
    state: &'a ProjectsState,
    modal: &'a TaskModalState,
    today: NaiveDate,
    theme: EstudyTheme,
    map_msg: impl Fn(ProjectsMessage) -> Message + Clone + 'a,
) -> Element<'a, Message> {
    let header_title = match modal.mode {
        TaskModalMode::Create => "Nova tarefa",
        TaskModalMode::Edit(_) => "Editar tarefa",
    };

    let close_btn = button(icon_svg(Icon::X, 16.0, theme.text_muted))
        .on_press(map_msg(ProjectsMessage::CloseTaskModal))
        .padding(SPACE_1)
        .style(move |_, status| theme.subtle_button_style(status));

    let header = row![
        shaped_text(
            header_title,
            FONT_SIZE_LG,
            FONT_SANS_SEMIBOLD,
            theme.text_primary
        ),
        container(close_btn)
            .align_x(Alignment::End)
            .width(Length::Fill),
    ]
    .align_y(Alignment::Center);

    let on_title = map_msg.clone();
    let title_input = text_input("O que precisa ser feito?", &modal.title)
        .on_input(move |v| on_title(ProjectsMessage::ModalTitleChanged(v)))
        .on_submit(map_msg(ProjectsMessage::SaveTaskModal))
        .padding([SPACE_2, SPACE_3])
        .size(FONT_SIZE_XL)
        .font(FONT_SERIF_MEDIUM)
        .style(move |_, status| theme.soft_input_style(status));

    let on_notes = map_msg.clone();
    let notes_box = text_editor(&modal.notes_content)
        .placeholder("Notas: escreva à vontade (o Enter pula linha)")
        .on_action(move |action| on_notes(ProjectsMessage::ModalNotesAction(action)))
        .height(Length::Fixed(92.0))
        .padding(SPACE_3)
        .size(FONT_SIZE_BASE)
        .font(FONT_SANS)
        .style(move |_, status| editor_soft_style(theme, status));

    let checklist_block = view_modal_checklist(modal, theme, map_msg.clone());
    let properties_block = view_modal_properties(state, modal, today, theme, map_msg.clone());

    let mut body = column![
        header,
        hairline_divider(theme),
        title_input,
        notes_box,
        checklist_block,
        hairline_divider(theme),
        properties_block,
    ]
    .spacing(SPACE_3);

    if let Some(notice) = &modal.notice {
        body = body.push(shaped_text(
            notice.as_str(),
            FONT_SIZE_SM,
            FONT_SANS_MEDIUM,
            theme.danger,
        ));
    }

    let footer = row![
        container(
            row![
                button(shaped_text(
                    "Cancelar",
                    FONT_SIZE_BASE,
                    FONT_SANS_MEDIUM,
                    theme.text_primary,
                ))
                .on_press(map_msg(ProjectsMessage::CloseTaskModal))
                .padding([8.0, 16.0])
                .style(move |_, status| theme.ghost_button_style(status)),
                button(shaped_text(
                    "Salvar",
                    FONT_SIZE_BASE,
                    FONT_SANS_SEMIBOLD,
                    theme.on_accent,
                ))
                .on_press(map_msg(ProjectsMessage::SaveTaskModal))
                .padding([8.0, 20.0])
                .style(move |_, status| theme.primary_button_style(status)),
            ]
            .spacing(SPACE_2),
        )
        .align_x(Alignment::End)
        .width(Length::Fill),
    ];

    body = body.push(footer);
    modal_surface(scrollable(body).height(Length::Shrink), 520.0, theme)
}

fn view_modal_checklist<'a, Message: Clone + 'a>(
    modal: &'a TaskModalState,
    theme: EstudyTheme,
    map_msg: impl Fn(ProjectsMessage) -> Message + Clone + 'a,
) -> Element<'a, Message> {
    let done_count = modal.checklist.iter().filter(|i| i.done).count();
    let total = modal.checklist.len();

    let top = row![
        shaped_text(
            "Checklist",
            FONT_SIZE_SM,
            FONT_SANS_SEMIBOLD,
            theme.text_primary
        ),
        container(shaped_text(
            format!("{done_count} de {total}"),
            FONT_SIZE_XS,
            FONT_MONO,
            theme.text_muted,
        ))
        .align_x(Alignment::End)
        .width(Length::Fill),
    ]
    .align_y(Alignment::Center);

    let mut items = column![].spacing(4.0);
    for item in &modal.checklist {
        let icon = if item.done {
            Icon::CheckCircle2
        } else {
            Icon::CheckSquare
        };
        let fg = if item.done {
            theme.success
        } else {
            theme.text_muted
        };
        items = items.push(
            row![
                button(
                    row![
                        icon_svg(icon, 16.0, fg),
                        shaped_text(
                            item.title.as_str(),
                            FONT_SIZE_BASE,
                            FONT_SANS,
                            if item.done {
                                theme.text_muted
                            } else {
                                theme.text_primary
                            },
                        ),
                    ]
                    .spacing(SPACE_2)
                    .align_y(Alignment::Center),
                )
                .on_press(map_msg(ProjectsMessage::ModalToggleChecklistItem(
                    item.id.clone(),
                )))
                .width(Length::Fill)
                .padding([4.0, 6.0])
                .style(move |_, status| theme.subtle_button_style(status)),
                button(icon_svg(Icon::X, 14.0, theme.text_muted))
                    .on_press(map_msg(ProjectsMessage::ModalRemoveChecklistItem(
                        item.id.clone(),
                    )))
                    .padding(4.0)
                    .style(move |_, status| theme.subtle_button_style(status)),
            ]
            .align_y(Alignment::Center),
        );
    }

    let on_input = map_msg.clone();
    let add_row = row![
        text_input(
            "+ Adicionar item (Enter adiciona)",
            &modal.new_checklist_title
        )
        .on_input(move |v| on_input(ProjectsMessage::ModalChecklistInputChanged(v)))
        .on_submit(map_msg(ProjectsMessage::ModalAddChecklistItem))
        .padding([6.0, 10.0])
        .size(FONT_SIZE_SM)
        .style(move |_, status| theme.soft_input_style(status)),
    ];

    column![top, items, add_row].spacing(6.0).into()
}

#[allow(clippy::too_many_lines)]
fn view_modal_properties<'a, Message: Clone + 'a>(
    state: &'a ProjectsState,
    modal: &'a TaskModalState,
    today: NaiveDate,
    theme: EstudyTheme,
    map_msg: impl Fn(ProjectsMessage) -> Message + Clone + 'a,
) -> Element<'a, Message> {
    let statuses = [
        (TaskStatus::Todo, "Falta fazer"),
        (TaskStatus::InProgress, "Fazendo"),
        (TaskStatus::Done, "Concluído"),
    ];
    let mut status_pills = row![].spacing(6.0);
    for (st, label) in statuses {
        let active = modal.status == st;
        status_pills = status_pills.push(choice_chip(
            label,
            active,
            map_msg(ProjectsMessage::ModalSetStatus(st)),
            theme,
        ));
    }

    let priorities = [
        (TaskPriority::Low, "Baixa"),
        (TaskPriority::Medium, "Média"),
        (TaskPriority::High, "Alta"),
    ];
    let mut prio_pills = row![].spacing(6.0);
    for (p, label) in priorities {
        let active = modal.priority == p;
        prio_pills = prio_pills.push(choice_chip(
            label,
            active,
            map_msg(ProjectsMessage::ModalSetPriority(p)),
            theme,
        ));
    }

    let has_date_btn = choice_chip(
        if modal.has_date {
            "Agendada"
        } else {
            "Sem dia"
        },
        modal.has_date,
        map_msg(ProjectsMessage::ModalToggleHasDate),
        theme,
    );

    let mut rows = column![
        prop_row("Status", status_pills.into(), theme),
        prop_row("Prioridade", prio_pills.into(), theme),
        prop_row("Tem dia", has_date_btn, theme),
    ]
    .spacing(SPACE_2);

    if modal.has_date {
        let date_chips = row![
            shaped_text(
                format!(
                    "{:02}/{:02}/{}",
                    modal.date.day(),
                    modal.date.month(),
                    modal.date.year()
                ),
                FONT_SIZE_SM,
                FONT_MONO,
                theme.text_primary,
            ),
            choice_chip(
                "Hoje",
                modal.date == today,
                map_msg(ProjectsMessage::ModalSetDate(today)),
                theme,
            ),
            choice_chip(
                "Amanhã",
                modal.date == today + Duration::days(1),
                map_msg(ProjectsMessage::ModalSetDate(today + Duration::days(1))),
                theme,
            ),
            choice_chip(
                "+1 semana",
                modal.date == today + Duration::days(7),
                map_msg(ProjectsMessage::ModalSetDate(today + Duration::days(7))),
                theme,
            ),
        ]
        .spacing(6.0)
        .align_y(Alignment::Center);

        let mut turn_pills = row![].spacing(6.0);
        for turn in TaskTurn::ALL {
            turn_pills = turn_pills.push(choice_chip(
                turn.label(),
                modal.turn == turn,
                map_msg(ProjectsMessage::ModalSetTurn(turn)),
                theme,
            ));
        }

        rows = rows
            .push(prop_row("Dia", date_chips.into(), theme))
            .push(prop_row("Período", turn_pills.into(), theme));
    }

    let mut proj_pills = row![choice_chip(
        "Nenhum",
        modal.project_id.is_none(),
        map_msg(ProjectsMessage::ModalSetProject(None)),
        theme,
    )]
    .spacing(6.0);
    for proj in state.projects.iter().take(3) {
        let active = modal.project_id.as_ref() == Some(&proj.id);
        proj_pills = proj_pills.push(choice_chip(
            &proj.title,
            active,
            map_msg(ProjectsMessage::ModalSetProject(Some(proj.id.clone()))),
            theme,
        ));
    }
    rows = rows.push(prop_row("Projeto", proj_pills.into(), theme));

    rows.into()
}

fn prop_row<'a, Message: 'a>(
    label: &'static str,
    control: Element<'a, Message>,
    theme: EstudyTheme,
) -> Element<'a, Message> {
    row![
        container(shaped_text(
            label,
            FONT_SIZE_SM,
            FONT_SANS_MEDIUM,
            theme.text_muted,
        ))
        .width(Length::Fixed(86.0)),
        container(control)
            .align_x(Alignment::End)
            .width(Length::Fill),
    ]
    .align_y(Alignment::Center)
    .into()
}

fn choice_chip<'a, Message: Clone + 'a>(
    label: &'a str,
    active: bool,
    on_press: Message,
    theme: EstudyTheme,
) -> Element<'a, Message> {
    let fg = if active {
        theme.on_accent
    } else {
        theme.text_primary
    };
    button(shaped_text(label, FONT_SIZE_XS, FONT_SANS_MEDIUM, fg))
        .on_press(on_press)
        .padding([5.0, 11.0])
        .style(move |_, status| {
            if active {
                theme.primary_button_style(status)
            } else {
                theme.ghost_button_style(status)
            }
        })
        .into()
}

/// Renderiza o Card do Dia ao clicar em uma célula do modo Calendário (Vídeo 1 `00:28`).
pub fn view_day_peek_modal<'a, Message: Clone + 'a>(
    state: &'a ProjectsState,
    date: NaiveDate,
    theme: EstudyTheme,
    map_msg: impl Fn(ProjectsMessage) -> Message + Clone + 'a,
) -> Element<'a, Message> {
    let close_btn = button(icon_svg(Icon::X, 16.0, theme.text_muted))
        .on_press(map_msg(ProjectsMessage::CloseDayPeek))
        .padding(SPACE_1)
        .style(move |_, status| theme.subtle_button_style(status));

    let weekday_title = weekday_full_pt(date.weekday());
    let full_subtitle = format!(
        "{}, {} de {}",
        weekday_title.to_lowercase(),
        date.day(),
        month_name_pt(date.month())
    );

    let header = row![
        column![
            text(weekday_title)
                .size(FONT_SIZE_2XL)
                .font(FONT_SERIF)
                .color(theme.text_primary)
                .shaping(text::Shaping::Advanced),
            text(full_subtitle)
                .size(FONT_SIZE_BASE)
                .font(FONT_SERIF_ITALIC)
                .color(theme.text_muted)
                .shaping(text::Shaping::Advanced),
        ]
        .spacing(2.0),
        container(close_btn)
            .align_x(Alignment::End)
            .width(Length::Fill),
    ]
    .align_y(Alignment::Start);

    let day_tasks: Vec<&Task> = state
        .filtered_tasks()
        .into_iter()
        .filter(|t| extract_task_date(t) == Some(date))
        .collect();

    let mut groups = column![].spacing(SPACE_3);
    if day_tasks.is_empty() {
        groups = groups.push(
            soft_surface(
                shaped_text(
                    "Nenhuma tarefa marcada para este dia ainda.",
                    FONT_SIZE_BASE,
                    FONT_SANS,
                    theme.text_muted,
                ),
                theme,
            )
            .padding(SPACE_4)
            .width(Length::Fill),
        );
    } else {
        for turn in TaskTurn::ALL {
            let in_turn: Vec<&Task> = day_tasks
                .iter()
                .copied()
                .filter(|t| {
                    classify_task_turn(t.start_at.as_deref(), t.due_at.as_deref(), t.priority)
                        == turn
                })
                .collect();
            if in_turn.is_empty() {
                continue;
            }
            groups = groups.push(view_day_peek_turn_group(turn, &in_turn, theme, &map_msg));
        }
    }

    let add_btn = button(
        container(
            row![
                icon_svg(Icon::Plus, 16.0, theme.text_primary),
                shaped_text(
                    "Nova tarefa neste dia",
                    FONT_SIZE_BASE,
                    FONT_SANS_MEDIUM,
                    theme.text_primary,
                ),
            ]
            .spacing(SPACE_2)
            .align_y(Alignment::Center),
        )
        .center_x(Length::Fill)
        .width(Length::Fill),
    )
    .on_press(map_msg(ProjectsMessage::OpenCreateTaskModal {
        date: Some(date),
        turn: TaskTurn::Anytime,
    }))
    .width(Length::Fill)
    .padding([10.0, SPACE_3])
    .style(move |_, status| theme.ghost_button_style(status));

    modal_surface(
        column![header, hairline_divider(theme), groups, add_btn].spacing(SPACE_3),
        440.0,
        theme,
    )
}

fn view_day_peek_turn_group<'a, Message: Clone + 'a>(
    turn: TaskTurn,
    in_turn: &[&'a Task],
    theme: EstudyTheme,
    map_msg: &impl Fn(ProjectsMessage) -> Message,
) -> Element<'a, Message> {
    let mut turn_col = column![
        row![
            illustrated_svg(turn_illustrated_icon(turn), 16.0),
            shaped_text(
                turn.label().to_uppercase(),
                FONT_SIZE_XS,
                FONT_SANS_SEMIBOLD,
                theme.text_muted,
            ),
        ]
        .spacing(6.0)
        .align_y(Alignment::Center)
    ]
    .spacing(SPACE_2);

    for task in in_turn {
        let done = task.status == TaskStatus::Done;
        turn_col = turn_col.push(
            soft_surface(
                row![
                    button(icon_svg(
                        if done {
                            Icon::CheckCircle2
                        } else {
                            Icon::CheckSquare
                        },
                        16.0,
                        if done {
                            theme.success
                        } else {
                            theme.text_muted
                        },
                    ))
                    .on_press(map_msg(ProjectsMessage::ToggleTaskDone(task.id.clone())))
                    .padding(2.0)
                    .style(move |_, status| theme.subtle_button_style(status)),
                    shaped_text(
                        task.title.as_str(),
                        FONT_SIZE_MD,
                        FONT_SANS_MEDIUM,
                        if done {
                            theme.text_muted
                        } else {
                            theme.text_primary
                        },
                    )
                    .width(Length::Fill),
                    illustrated_svg(priority_illustrated_icon(task.priority), 16.0),
                ]
                .spacing(SPACE_2)
                .align_y(Alignment::Center),
                theme,
            )
            .padding([SPACE_2, SPACE_3]),
        );
    }
    turn_col.into()
}

/// Renderiza o Menu de Contexto flutuante exatamente na coordenada `(x, y)` do clique direito (Vídeo 1 `00:09`).
#[allow(clippy::too_many_lines)]
pub fn view_task_context_menu<'a, Message: Clone + 'a>(
    state: &'a ProjectsState,
    menu: &'a TaskContextMenu,
    theme: EstudyTheme,
    map_msg: impl Fn(ProjectsMessage) -> Message + Clone + 'a,
) -> Element<'a, Message> {
    let task_id = menu.task_id.clone();
    let is_done = state
        .tasks
        .iter()
        .find(|t| t.id == task_id)
        .is_some_and(|t| t.status == TaskStatus::Done);

    let main_menu = container(
        column![
            ctx_btn(
                "Editar",
                false,
                map_msg(ProjectsMessage::OpenEditTaskModal(task_id.clone())),
                theme,
            ),
            ctx_btn(
                "Duplicar",
                false,
                map_msg(ProjectsMessage::DuplicateTask(task_id.clone())),
                theme,
            ),
            ctx_btn(
                if is_done { "Reabrir" } else { "Concluir" },
                false,
                map_msg(ProjectsMessage::ToggleTaskDone(task_id.clone())),
                theme,
            ),
            ctx_btn(
                "Iniciar foco (25m)",
                false,
                map_msg(ProjectsMessage::StartFocusForTask(task_id.clone())),
                theme,
            ),
            hairline_divider(theme),
            ctx_submenu_trigger(
                "Status",
                menu.submenu == Some(ContextSubmenu::Status),
                map_msg(ProjectsMessage::SetContextSubmenu(Some(
                    ContextSubmenu::Status,
                ))),
                theme,
            ),
            ctx_submenu_trigger(
                "Prioridade",
                menu.submenu == Some(ContextSubmenu::Priority),
                map_msg(ProjectsMessage::SetContextSubmenu(Some(
                    ContextSubmenu::Priority,
                ))),
                theme,
            ),
            ctx_submenu_trigger(
                "Período",
                menu.submenu == Some(ContextSubmenu::Turn),
                map_msg(ProjectsMessage::SetContextSubmenu(Some(
                    ContextSubmenu::Turn,
                ))),
                theme,
            ),
            ctx_submenu_trigger(
                "Dia",
                menu.submenu == Some(ContextSubmenu::Day),
                map_msg(ProjectsMessage::SetContextSubmenu(Some(
                    ContextSubmenu::Day,
                ))),
                theme,
            ),
            hairline_divider(theme),
            ctx_btn(
                "Excluir",
                true,
                map_msg(ProjectsMessage::DeleteTask(task_id.clone())),
                theme,
            ),
        ]
        .spacing(2.0),
    )
    .width(Length::Fixed(186.0))
    .padding(6.0)
    .style(move |_| theme.floating_menu_style());

    let mut menu_cluster = row![main_menu].spacing(6.0).align_y(Alignment::Start);

    if let Some(sub) = menu.submenu {
        let sub_box = view_context_submenu_box(sub, &task_id, theme, &map_msg);
        menu_cluster = menu_cluster.push(sub_box);
    }

    let safe_x = menu.x.clamp(16.0, 960.0);
    let safe_y = menu.y.clamp(16.0, 520.0);

    mouse_area(
        container(menu_cluster)
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(iced::Padding {
                top: safe_y,
                right: 0.0,
                bottom: 0.0,
                left: safe_x,
            }),
    )
    .on_press(map_msg(ProjectsMessage::CloseContextMenu))
    .on_right_press(map_msg(ProjectsMessage::CloseContextMenu))
    .into()
}

fn view_context_submenu_box<'a, Message: Clone + 'a>(
    sub: ContextSubmenu,
    task_id: &estudy_domain::EntityId,
    theme: EstudyTheme,
    map_msg: &impl Fn(ProjectsMessage) -> Message,
) -> Element<'a, Message> {
    let entries: Vec<(&'static str, ProjectsMessage)> = match sub {
        ContextSubmenu::Status => vec![
            (
                "Falta fazer",
                ProjectsMessage::MoveTaskStatus(task_id.clone(), TaskStatus::Todo),
            ),
            (
                "Fazendo",
                ProjectsMessage::MoveTaskStatus(task_id.clone(), TaskStatus::InProgress),
            ),
            (
                "Concluído",
                ProjectsMessage::MoveTaskStatus(task_id.clone(), TaskStatus::Done),
            ),
        ],
        ContextSubmenu::Priority => vec![
            (
                "Baixa",
                ProjectsMessage::ContextSetPriority(task_id.clone(), TaskPriority::Low),
            ),
            (
                "Média",
                ProjectsMessage::ContextSetPriority(task_id.clone(), TaskPriority::Medium),
            ),
            (
                "Alta",
                ProjectsMessage::ContextSetPriority(task_id.clone(), TaskPriority::High),
            ),
        ],
        ContextSubmenu::Turn => TaskTurn::ALL
            .into_iter()
            .map(|turn| {
                (
                    turn.label(),
                    ProjectsMessage::ContextSetTurn(task_id.clone(), turn),
                )
            })
            .collect(),
        ContextSubmenu::Day => vec![
            (
                "Hoje",
                ProjectsMessage::ContextSetDayOffset(task_id.clone(), Some(0)),
            ),
            (
                "Amanhã",
                ProjectsMessage::ContextSetDayOffset(task_id.clone(), Some(1)),
            ),
            (
                "+1 semana",
                ProjectsMessage::ContextSetDayOffset(task_id.clone(), Some(7)),
            ),
            (
                "Sem dia",
                ProjectsMessage::ContextSetDayOffset(task_id.clone(), None),
            ),
        ],
    };

    let mut col = column![].spacing(2.0);
    for (label, msg) in entries {
        col = col.push(ctx_btn(label, false, map_msg(msg), theme));
    }

    container(col)
        .width(Length::Fixed(152.0))
        .padding(6.0)
        .style(move |_| theme.floating_menu_style())
        .into()
}

fn ctx_btn<'a, Message: Clone + 'a>(
    label: &'static str,
    destructive: bool,
    on_press: Message,
    theme: EstudyTheme,
) -> Element<'a, Message> {
    button(shaped_text(
        label,
        FONT_SIZE_BASE,
        FONT_SANS_MEDIUM,
        if destructive {
            theme.danger
        } else {
            theme.text_primary
        },
    ))
    .on_press(on_press)
    .width(Length::Fill)
    .padding([6.0, 10.0])
    .style(move |_, status| {
        if matches!(status, button::Status::Hovered | button::Status::Pressed) {
            button::Style {
                background: Some(
                    if destructive {
                        Color {
                            a: 0.14,
                            ..theme.danger
                        }
                    } else {
                        theme.accent_subtle
                    }
                    .into(),
                ),
                text_color: if destructive {
                    theme.danger
                } else {
                    theme.text_primary
                },
                border: Border {
                    radius: 8.0.into(),
                    ..Border::default()
                },
                ..button::Style::default()
            }
        } else {
            theme.subtle_button_style(status)
        }
    })
    .into()
}

fn ctx_submenu_trigger<'a, Message: Clone + 'a>(
    label: &'static str,
    open: bool,
    on_enter: Message,
    theme: EstudyTheme,
) -> Element<'a, Message> {
    mouse_area(
        button(
            row![
                shaped_text(label, FONT_SIZE_BASE, FONT_SANS_MEDIUM, theme.text_primary)
                    .width(Length::Fill),
                icon_svg(Icon::ChevronRight, 14.0, theme.text_muted),
            ]
            .align_y(Alignment::Center),
        )
        .on_press(on_enter.clone())
        .width(Length::Fill)
        .padding([6.0, 10.0])
        .style(move |_, status| {
            if open || matches!(status, button::Status::Hovered) {
                button::Style {
                    background: Some(theme.accent_subtle.into()),
                    text_color: theme.text_primary,
                    border: Border {
                        radius: 8.0.into(),
                        ..Border::default()
                    },
                    ..button::Style::default()
                }
            } else {
                theme.subtle_button_style(status)
            }
        }),
    )
    .on_enter(on_enter)
    .into()
}

/// Estilo limpo para `text_editor` multi-linha alinhado aos tokens do tema.
#[must_use]
pub fn editor_soft_style(theme: EstudyTheme, status: text_editor::Status) -> text_editor::Style {
    let border_color = match status {
        text_editor::Status::Focused { .. } => theme.accent,
        text_editor::Status::Hovered => Color {
            a: 0.35,
            ..theme.text_muted
        },
        text_editor::Status::Active | text_editor::Status::Disabled => theme.border_subtle,
    };
    text_editor::Style {
        background: theme.surface_soft.into(),
        border: Border {
            color: border_color,
            width: 1.0,
            radius: RADIUS_CONTROL.into(),
        },
        placeholder: theme.text_muted,
        value: theme.text_primary,
        selection: theme.accent_subtle,
    }
}

/// Nome do mês em português.
#[must_use]
pub const fn month_name_pt(month: u32) -> &'static str {
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

/// Indicador colorido de status de tarefa estilo Centralpinha.
#[must_use]
pub fn task_status_visual(status: TaskStatus, theme: EstudyTheme) -> (&'static str, Color) {
    match status {
        TaskStatus::Backlog | TaskStatus::Todo => {
            ("FALTA FAZER", CategoryColor::Terracotta.solid(theme))
        }
        TaskStatus::InProgress => ("FAZENDO", CategoryColor::Lavender.solid(theme)),
        TaskStatus::Done => ("CONCLUÍDO", CategoryColor::Sage.solid(theme)),
    }
}

/// Auxiliar para exibir ponto colorido + nome do projeto num cartão.
pub fn project_tag_inline<'a, Message: 'a>(
    state: &'a ProjectsState,
    task: &'a Task,
    theme: EstudyTheme,
) -> Option<Element<'a, Message>> {
    let (idx, proj) = state.task_project_info(task)?;
    let dot_color = CategoryColor::from_index(idx).solid(theme);
    Some(
        row![
            color_dot(dot_color),
            shaped_text(
                proj.title.as_str(),
                FONT_SIZE_XS,
                FONT_SANS_MEDIUM,
                theme.text_muted,
            ),
        ]
        .spacing(5.0)
        .align_y(Alignment::Center)
        .into(),
    )
}

/// Superfície de cartão para fantasma de Drag & Drop flutuando sob o cursor.
pub fn view_drag_ghost<'a, Message: 'a>(
    state: &'a ProjectsState,
    theme: EstudyTheme,
) -> Option<Element<'a, Message>> {
    let drag_id = state.dragging_task_id.as_ref()?;
    let task = state.tasks.iter().find(|t| &t.id == drag_id)?;
    let (x, y) = state.cursor_pos;

    let ghost_card = container(
        row![
            illustrated_svg(priority_illustrated_icon(task.priority), 16.0),
            shaped_text(
                task.title.as_str(),
                FONT_SIZE_BASE,
                FONT_SANS_SEMIBOLD,
                theme.text_primary,
            ),
        ]
        .spacing(SPACE_2)
        .align_y(Alignment::Center),
    )
    .padding([SPACE_2, SPACE_3])
    .max_width(260.0)
    .style(move |_| container::Style {
        background: Some(
            Color {
                a: 0.95,
                ..theme.card
            }
            .into(),
        ),
        border: Border {
            color: theme.accent,
            width: 1.5,
            radius: RADIUS_CARD.into(),
        },
        shadow: theme.floating_menu_style().shadow,
        snap: true,
        ..container::Style::default()
    });

    Some(
        container(ghost_card)
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(iced::Padding {
                top: (y + 12.0).clamp(8.0, 720.0),
                right: 0.0,
                bottom: 0.0,
                left: (x + 12.0).clamp(8.0, 1080.0),
            })
            .into(),
    )
}

#[allow(dead_code)]
const _: f32 = RADIUS_PANEL + RADIUS_PILL + SPACE_5;
