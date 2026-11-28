//! Renderização do Segundo Cérebro: grade adaptativa (2/3/4 colunas), tags removíveis, URL em Links e preview formatado.

use estudy_domain::{KnowledgeNote, NoteKind};
use estudy_ui_kit::widgets::{
    category_pill, color_dot, dashed_frame, editorial_page_header, hairline_divider, icon_svg,
    modal_surface, pill_chip, segmented_switch, shaped_text, soft_surface,
};
use estudy_ui_kit::{
    CategoryColor, EstudyTheme, FONT_MONO, FONT_SANS, FONT_SANS_MEDIUM, FONT_SANS_SEMIBOLD,
    FONT_SERIF, FONT_SERIF_MEDIUM, FONT_SIZE_2XL, FONT_SIZE_BASE, FONT_SIZE_LG, FONT_SIZE_MD,
    FONT_SIZE_SM, FONT_SIZE_XL, FONT_SIZE_XS, Icon, RADIUS_CARD, RADIUS_PILL, SPACE_1, SPACE_2,
    SPACE_3, SPACE_4, SPACE_5, SPACE_6,
};
use iced::widget::{
    button, column, container, row, scrollable, stack, text, text_editor, text_input,
};
use iced::{Alignment, Border, Color, Element, Length};

use super::state::{BrainEditorTab, SecondBrainMessage, SecondBrainState};
use crate::features::projects::modals::editor_soft_style;

/// Renderiza a tela completa do Segundo Cérebro com grade adaptativa e modal multi-linha real.
pub fn view_second_brain<'a, Message: Clone + 'a>(
    state: &'a SecondBrainState,
    content_width: f32,
    theme: EstudyTheme,
    map_msg: impl Fn(SecondBrainMessage) -> Message + Clone + 'a,
) -> Element<'a, Message> {
    let on_search = map_msg.clone();
    let search_input = text_input("Buscar notas ou tags...", &state.search_query)
        .on_input(move |q| on_search(SecondBrainMessage::SearchChanged(q)))
        .padding([8.0, 14.0])
        .size(FONT_SIZE_BASE)
        .width(Length::Fixed(if content_width < 780.0 {
            210.0
        } else {
            280.0
        }))
        .style(move |_, status| theme.soft_input_style(status));

    let mut search_bar = row![search_input]
        .spacing(SPACE_1)
        .align_y(Alignment::Center);
    if !state.search_query.is_empty() {
        search_bar = search_bar.push(
            button(icon_svg(Icon::X, 15.0, theme.text_muted))
                .on_press(map_msg(SecondBrainMessage::SearchChanged(String::new())))
                .padding(6.0)
                .style(move |_, status| theme.subtle_button_style(status)),
        );
    }

    let header = editorial_page_header(
        "ESTUDY  /  SEGUNDO CÉREBRO",
        "Segundo cérebro",
        "Ideias e referências",
        Some(search_bar.into()),
        theme,
    );

    let filters = view_filters_strip(state, theme, map_msg.clone());
    let grid = view_wide_notes_grid(state, content_width, theme, &map_msg);

    let page_scroll = scrollable(
        container(
            container(
                column![header, filters, grid]
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
    .height(Length::Fill);

    if state.note_modal_open {
        stack![page_scroll, view_note_editor_modal(state, theme, map_msg)].into()
    } else {
        page_scroll.into()
    }
}

fn note_kind_color(kind: NoteKind) -> CategoryColor {
    match kind {
        NoteKind::Note => CategoryColor::Slate,
        NoteKind::Idea => CategoryColor::Ochre,
        NoteKind::Reference => CategoryColor::Lavender,
        NoteKind::Link => CategoryColor::Sage,
    }
}

fn view_filters_strip<'a, Message: Clone + 'a>(
    state: &'a SecondBrainState,
    theme: EstudyTheme,
    map_msg: impl Fn(SecondBrainMessage) -> Message + Clone + 'a,
) -> Element<'a, Message> {
    const KINDS: [(Option<NoteKind>, &str); 5] = [
        (None, "Tudo"),
        (Some(NoteKind::Note), "Notas"),
        (Some(NoteKind::Idea), "Ideias"),
        (Some(NoteKind::Reference), "Referências"),
        (Some(NoteKind::Link), "Links"),
    ];

    let mut kind_row = row![].spacing(4.0);
    for &(kind, base_label) in &KINDS {
        let count = match kind {
            None => state.notes.len(),
            Some(target) => state.notes.iter().filter(|n| n.kind == target).count(),
        };
        let is_active = state.kind_filter == kind;
        let fg = if is_active {
            theme.on_accent
        } else {
            theme.text_muted
        };
        let font = if is_active {
            FONT_SANS_SEMIBOLD
        } else {
            FONT_SANS_MEDIUM
        };
        kind_row = kind_row.push(
            button(shaped_text(
                format!("{base_label} ({count})"),
                FONT_SIZE_SM,
                font,
                fg,
            ))
            .on_press_maybe((!is_active).then(|| map_msg(SecondBrainMessage::SetKindFilter(kind))))
            .padding([6.0, 13.0])
            .style(move |_, status| {
                if is_active {
                    theme.pill_button_style(true, status)
                } else {
                    button::Style {
                        background: match status {
                            button::Status::Hovered => Some(theme.card_hover.into()),
                            _ => None,
                        },
                        text_color: theme.text_muted,
                        border: Border {
                            radius: RADIUS_PILL.into(),
                            ..Border::default()
                        },
                        ..button::Style::default()
                    }
                }
            }),
        );
    }

    let kind_tray = container(kind_row.wrap().vertical_spacing(4.0))
        .padding(3.0)
        .style(move |_| theme.pill_tray_style());

    let mut tags_row = row![pill_chip(
        "Todas",
        None,
        None,
        state.tag_filter.is_none(),
        map_msg(SecondBrainMessage::ToggleTagFilter(None)),
        theme,
    )]
    .spacing(SPACE_1)
    .align_y(Alignment::Center);

    for (idx, tag) in state.available_tags().into_iter().take(10).enumerate() {
        let active = state.tag_filter.as_deref() == Some(tag.as_str());
        let cat = CategoryColor::from_index(idx);
        let next = if active { None } else { Some(tag.clone()) };
        tags_row = tags_row.push(pill_chip(
            format!("#{tag}"),
            Some(cat.solid(theme)),
            None,
            active,
            map_msg(SecondBrainMessage::ToggleTagFilter(next)),
            theme,
        ));
    }

    column![kind_tray, tags_row.wrap().vertical_spacing(SPACE_1)]
        .spacing(SPACE_2)
        .into()
}

enum GridCell<'a> {
    CreateNew,
    Note(&'a KnowledgeNote),
}

fn view_wide_notes_grid<'a, Message: Clone + 'a>(
    state: &'a SecondBrainState,
    content_width: f32,
    theme: EstudyTheme,
    map_msg: &impl Fn(SecondBrainMessage) -> Message,
) -> Element<'a, Message> {
    let mut cells: Vec<GridCell<'a>> = vec![GridCell::CreateNew];
    for note in state.filtered_notes() {
        cells.push(GridCell::Note(note));
    }

    let col_count = if content_width >= 1080.0 {
        4
    } else if content_width >= 760.0 {
        3
    } else {
        2
    };

    let mut rows = column![].spacing(SPACE_3);
    for chunk in cells.chunks(col_count) {
        let mut line = row![].spacing(SPACE_3).width(Length::Fill);
        for cell in chunk {
            match cell {
                GridCell::CreateNew => {
                    line = line.push(view_new_note_card(theme, map_msg));
                }
                GridCell::Note(note) => {
                    line = line.push(view_note_card(note, theme, map_msg));
                }
            }
        }
        for _ in chunk.len()..col_count {
            line = line.push(container(text("")).width(Length::FillPortion(1)));
        }
        rows = rows.push(line);
    }

    rows.into()
}

fn view_new_note_card<'a, Message: Clone + 'a>(
    theme: EstudyTheme,
    map_msg: &impl Fn(SecondBrainMessage) -> Message,
) -> Element<'a, Message> {
    let plus_badge = container(icon_svg(Icon::Plus, 20.0, theme.text_primary))
        .width(Length::Fixed(42.0))
        .height(Length::Fixed(42.0))
        .center_x(Length::Fixed(42.0))
        .center_y(Length::Fixed(42.0))
        .style(move |_| container::Style {
            background: Some(theme.surface_soft.into()),
            border: Border {
                color: theme.border_subtle,
                width: 1.0,
                radius: RADIUS_PILL.into(),
            },
            ..container::Style::default()
        });

    let btn = button(
        container(
            column![
                plus_badge,
                shaped_text(
                    "Nova nota",
                    FONT_SIZE_MD,
                    FONT_SANS_SEMIBOLD,
                    theme.text_primary
                ),
            ]
            .spacing(SPACE_2)
            .align_x(Alignment::Center),
        )
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .width(Length::Fill)
        .height(Length::Fill),
    )
    .on_press(map_msg(SecondBrainMessage::CreateNote))
    .width(Length::Fill)
    .height(Length::Fixed(218.0))
    .padding(SPACE_4)
    .style(move |_, status| button::Style {
        background: matches!(status, button::Status::Hovered).then_some(theme.card_hover.into()),
        text_color: theme.text_primary,
        border: Border {
            radius: RADIUS_CARD.into(),
            ..Border::default()
        },
        ..button::Style::default()
    });

    container(dashed_frame(
        btn,
        Color {
            a: 0.30,
            ..theme.text_muted
        },
        Some(Color {
            a: 0.35,
            ..theme.surface_soft
        }),
        RADIUS_CARD,
    ))
    .width(Length::FillPortion(1))
    .height(Length::Fixed(218.0))
    .into()
}

fn view_note_card<'a, Message: Clone + 'a>(
    note: &'a KnowledgeNote,
    theme: EstudyTheme,
    map_msg: &impl Fn(SecondBrainMessage) -> Message,
) -> Element<'a, Message> {
    let cat = note_kind_color(note.kind);
    let mut header = row![category_pill(note.kind.label(), cat, theme)]
        .spacing(SPACE_2)
        .align_y(Alignment::Center);
    if note.pinned {
        header = header.push(icon_svg(Icon::Pin, 14.0, CategoryColor::Ochre.solid(theme)));
    }
    header = header.push(
        container(shaped_text(
            note.updated_at.format("%d/%m").to_string(),
            FONT_SIZE_XS,
            FONT_MONO,
            theme.text_muted,
        ))
        .align_x(Alignment::End)
        .width(Length::Fill),
    );

    let excerpt = if note.body_md.trim().is_empty() {
        "Clique para escrever ideias ou colar referências...".to_string()
    } else {
        note.preview_excerpt(116)
    };

    let mut tags_line = row![].spacing(4.0);
    for tag in note.tags.iter().take(3) {
        let tag_ink = cat.solid(theme);
        let tag_fill = cat.tint(theme);
        tags_line = tags_line.push(
            container(shaped_text(format!("#{tag}"), 11.0, FONT_MONO, tag_ink))
                .padding([2.0, 7.0])
                .style(move |_| container::Style {
                    background: Some(tag_fill.into()),
                    border: Border {
                        radius: RADIUS_PILL.into(),
                        ..Border::default()
                    },
                    ..container::Style::default()
                }),
        );
    }

    button(
        column![
            header,
            shaped_text(
                note.title.as_str(),
                FONT_SIZE_LG,
                FONT_SANS_SEMIBOLD,
                theme.text_primary,
            ),
            container(shaped_text(
                excerpt,
                FONT_SIZE_BASE,
                FONT_SANS,
                theme.text_muted,
            ))
            .height(Length::Fill),
            tags_line.wrap().vertical_spacing(4.0),
        ]
        .spacing(SPACE_2)
        .width(Length::Fill)
        .height(Length::Fill),
    )
    .on_press(map_msg(SecondBrainMessage::SelectNote(note.id.clone())))
    .padding(SPACE_4)
    .width(Length::FillPortion(1))
    .height(Length::Fixed(218.0))
    .style(move |_, status| theme.interactive_card_style(status))
    .into()
}

/// Modal amplo de leitura e escrita multi-linha (`text_editor`) de uma nota (usado também a partir da Home).
pub fn view_note_editor_modal<'a, Message: Clone + 'a>(
    state: &'a SecondBrainState,
    theme: EstudyTheme,
    map_msg: impl Fn(SecondBrainMessage) -> Message + Clone + 'a,
) -> Element<'a, Message> {
    let Some(note) = state.selected_note() else {
        return container(text("")).into();
    };

    let top_bar = view_note_modal_top_bar(state.editor_tab, note, theme, map_msg.clone());

    let mut kind_row = row![].spacing(SPACE_1);
    for kind in NoteKind::ALL {
        let active = note.kind == kind;
        let cat = note_kind_color(kind);
        kind_row = kind_row.push(pill_chip(
            kind.label(),
            Some(cat.solid(theme)),
            None,
            active,
            map_msg(SecondBrainMessage::KindChanged(kind)),
            theme,
        ));
    }

    let removable_tags = view_note_removable_tags(note, theme, &map_msg);
    let body = view_note_modal_body(state, note, theme, map_msg.clone());

    let tag_mapper = map_msg.clone();
    let tag_footer = row![
        text_input("+ tag (ex.: rust, revisão)", &state.new_tag_input)
            .on_input(move |v| tag_mapper(SecondBrainMessage::NewTagInputChanged(v)))
            .on_submit(map_msg(SecondBrainMessage::AddTag))
            .padding([7.0, 12.0])
            .size(FONT_SIZE_BASE)
            .width(Length::Fixed(210.0))
            .style(move |_, status| theme.soft_input_style(status)),
        button(shaped_text(
            "Adicionar tag",
            FONT_SIZE_BASE,
            FONT_SANS_MEDIUM,
            theme.text_primary
        ))
        .on_press(map_msg(SecondBrainMessage::AddTag))
        .padding([7.0, 14.0])
        .style(move |_, status| theme.ghost_button_style(status)),
        container(
            button(shaped_text(
                "Concluir",
                FONT_SIZE_BASE,
                FONT_SANS_SEMIBOLD,
                theme.on_accent,
            ))
            .on_press(map_msg(SecondBrainMessage::CloseNoteModal))
            .padding([8.0, 20.0])
            .style(move |_, status| theme.primary_button_style(status)),
        )
        .align_x(Alignment::End)
        .width(Length::Fill),
    ]
    .spacing(SPACE_2)
    .align_y(Alignment::Center);

    let mut modal_col = column![
        top_bar,
        kind_row.wrap().vertical_spacing(SPACE_1),
        hairline_divider(theme),
        body
    ]
    .spacing(SPACE_3);
    if let Some(tags_el) = removable_tags {
        modal_col = modal_col.push(tags_el);
    }
    modal_col = modal_col.push(tag_footer);

    modal_surface(modal_col, 650.0, theme)
}

fn view_note_removable_tags<'a, Message: Clone + 'a>(
    note: &'a KnowledgeNote,
    theme: EstudyTheme,
    map_msg: &impl Fn(SecondBrainMessage) -> Message,
) -> Option<Element<'a, Message>> {
    if note.tags.is_empty() {
        return None;
    }
    let cat = note_kind_color(note.kind);
    let fg = cat.solid(theme);
    let bg = cat.tint(theme);
    let border_c = cat.border(theme);

    let mut tags_row = row![].spacing(6.0);
    for tag in &note.tags {
        tags_row = tags_row.push(
            button(
                row![
                    shaped_text(format!("#{tag}"), FONT_SIZE_XS, FONT_MONO, fg),
                    icon_svg(Icon::X, 12.0, fg),
                ]
                .spacing(4.0)
                .align_y(Alignment::Center),
            )
            .on_press(map_msg(SecondBrainMessage::RemoveTag(tag.clone())))
            .padding([3.0, 9.0])
            .style(move |_, _| button::Style {
                background: Some(bg.into()),
                text_color: fg,
                border: Border {
                    color: border_c,
                    width: 1.0,
                    radius: RADIUS_PILL.into(),
                },
                ..button::Style::default()
            }),
        );
    }
    Some(tags_row.wrap().vertical_spacing(6.0).into())
}

fn view_note_modal_top_bar<'a, Message: Clone + 'a>(
    active_tab: BrainEditorTab,
    note: &'a KnowledgeNote,
    theme: EstudyTheme,
    map_msg: impl Fn(SecondBrainMessage) -> Message + Clone + 'a,
) -> Element<'a, Message> {
    const TABS: [(BrainEditorTab, &str); 2] = [
        (BrainEditorTab::Write, "Escrever"),
        (BrainEditorTab::Preview, "Visualizar"),
    ];
    let tab_mapper = map_msg.clone();
    let tab_switch = segmented_switch(
        &TABS,
        active_tab,
        move |tab| tab_mapper(SecondBrainMessage::SetEditorTab(tab)),
        theme,
    );

    row![
        tab_switch,
        container(
            row![
                button(icon_svg(
                    Icon::Pin,
                    16.0,
                    if note.pinned {
                        CategoryColor::Ochre.solid(theme)
                    } else {
                        theme.text_muted
                    },
                ))
                .on_press(map_msg(SecondBrainMessage::TogglePin(note.id.clone())))
                .padding(6.0)
                .style(move |_, status| theme.subtle_button_style(status)),
                button(icon_svg(Icon::Trash2, 16.0, theme.danger))
                    .on_press(map_msg(SecondBrainMessage::DeleteNote(note.id.clone())))
                    .padding(6.0)
                    .style(move |_, status| theme.subtle_button_style(status)),
                button(icon_svg(Icon::X, 16.0, theme.text_muted))
                    .on_press(map_msg(SecondBrainMessage::CloseNoteModal))
                    .padding(6.0)
                    .style(move |_, status| theme.subtle_button_style(status)),
            ]
            .spacing(SPACE_1)
            .align_y(Alignment::Center),
        )
        .align_x(Alignment::End)
        .width(Length::Fill),
    ]
    .align_y(Alignment::Center)
    .into()
}

fn view_note_modal_body<'a, Message: Clone + 'a>(
    state: &'a SecondBrainState,
    note: &'a KnowledgeNote,
    theme: EstudyTheme,
    map_msg: impl Fn(SecondBrainMessage) -> Message + Clone + 'a,
) -> Element<'a, Message> {
    match state.editor_tab {
        BrainEditorTab::Write => {
            let on_title = map_msg.clone();
            let on_editor = map_msg.clone();
            let on_url = map_msg;
            let mut col = column![
                text_input("Título da nota...", &note.title)
                    .on_input(move |v| on_title(SecondBrainMessage::TitleChanged(v)))
                    .padding([SPACE_2, SPACE_3])
                    .size(FONT_SIZE_XL)
                    .font(FONT_SERIF_MEDIUM)
                    .style(move |_, status| theme.soft_input_style(status)),
            ]
            .spacing(SPACE_2);

            if note.kind == NoteKind::Link {
                col = col.push(
                    text_input(
                        "URL de referência (https://...)...",
                        note.source_url.as_deref().unwrap_or(""),
                    )
                    .on_input(move |v| on_url(SecondBrainMessage::SourceUrlChanged(v)))
                    .padding([7.0, 12.0])
                    .size(FONT_SIZE_SM)
                    .font(FONT_MONO)
                    .style(move |_, status| theme.soft_input_style(status)),
                );
            }

            col = col.push(
                text_editor(&state.body_editor)
                    .placeholder("Escreva à vontade — Enter cria novos parágrafos...")
                    .on_action(move |action| {
                        on_editor(SecondBrainMessage::BodyEditorAction(action))
                    })
                    .height(Length::Fixed(230.0))
                    .padding(SPACE_3)
                    .size(FONT_SIZE_MD)
                    .font(FONT_SANS)
                    .style(move |_, status| editor_soft_style(theme, status)),
            );
            col.into()
        }
        BrainEditorTab::Preview => view_note_markdown_preview(note, theme),
    }
}

fn view_note_markdown_preview<'a, Message: 'a>(
    note: &'a KnowledgeNote,
    theme: EstudyTheme,
) -> Element<'a, Message> {
    let mut blocks = column![
        text(&note.title)
            .size(FONT_SIZE_2XL)
            .font(FONT_SERIF)
            .color(theme.text_primary)
            .shaping(text::Shaping::Advanced),
    ]
    .spacing(SPACE_2);

    if let Some(url) = note.source_url.as_deref().filter(|u| !u.trim().is_empty()) {
        blocks = blocks.push(shaped_text(
            url,
            FONT_SIZE_SM,
            FONT_MONO,
            CategoryColor::Sage.solid(theme),
        ));
    }

    if note.body_md.trim().is_empty() {
        blocks = blocks.push(shaped_text(
            "Nota em branco. Alterne para Escrever para capturar ideias.",
            FONT_SIZE_MD,
            FONT_SANS,
            theme.text_muted,
        ));
    } else {
        for raw_line in note.body_md.lines() {
            let trimmed = raw_line.trim();
            if trimmed.is_empty() {
                blocks = blocks.push(container(text("").size(1)).height(Length::Fixed(4.0)));
            } else if let Some(h1) = trimmed.strip_prefix("# ") {
                blocks = blocks.push(
                    text(h1)
                        .size(FONT_SIZE_XL)
                        .font(FONT_SERIF_MEDIUM)
                        .color(theme.text_primary)
                        .shaping(text::Shaping::Advanced),
                );
            } else if let Some(h2) = trimmed.strip_prefix("## ") {
                blocks = blocks.push(shaped_text(
                    h2,
                    FONT_SIZE_LG,
                    FONT_SANS_SEMIBOLD,
                    theme.text_primary,
                ));
            } else if let Some(bullet) = trimmed
                .strip_prefix("- ")
                .or_else(|| trimmed.strip_prefix("* "))
            {
                blocks = blocks.push(
                    row![
                        color_dot(theme.accent),
                        shaped_text(bullet, FONT_SIZE_MD, FONT_SANS, theme.text_primary),
                    ]
                    .spacing(SPACE_2)
                    .align_y(Alignment::Center),
                );
            } else {
                blocks = blocks.push(shaped_text(
                    trimmed,
                    FONT_SIZE_MD,
                    FONT_SANS,
                    theme.text_primary,
                ));
            }
        }
    }

    soft_surface(scrollable(blocks).height(Length::Fixed(270.0)), theme)
        .padding(SPACE_4)
        .width(Length::Fill)
        .into()
}
