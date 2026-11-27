//! Renderização do Segundo Cérebro (Vídeo 2 `00:08`: grade ampla em 4 colunas com cartão `+ Nova nota` e modal multi-linha `text_editor`).

use estudy_domain::{KnowledgeNote, NoteKind};
use estudy_ui_kit::widgets::{
    category_pill, editorial_page_header, hairline_divider, icon_svg, modal_surface, pill_chip,
    segmented_switch, shaped_text, soft_surface,
};
use estudy_ui_kit::{
    CategoryColor, EstudyTheme, FONT_MONO, FONT_SANS, FONT_SANS_MEDIUM, FONT_SANS_SEMIBOLD,
    FONT_SERIF, FONT_SERIF_MEDIUM, FONT_SIZE_2XL, FONT_SIZE_BASE, FONT_SIZE_LG, FONT_SIZE_MD,
    FONT_SIZE_XL, FONT_SIZE_XS, Icon, RADIUS_CARD, RADIUS_PILL, SPACE_1, SPACE_2, SPACE_3, SPACE_4,
    SPACE_5, SPACE_6,
};
use iced::widget::{
    button, column, container, row, scrollable, stack, text, text_editor, text_input,
};
use iced::{Alignment, Border, Color, Element, Length};

use super::state::{BrainEditorTab, SecondBrainMessage, SecondBrainState};
use crate::features::projects::modals::editor_soft_style;

/// Renderiza a tela completa do Segundo Cérebro com grade ampla e modal multi-linha real.
pub fn view_second_brain<'a, Message: Clone + 'a>(
    state: &'a SecondBrainState,
    theme: EstudyTheme,
    map_msg: impl Fn(SecondBrainMessage) -> Message + Clone + 'a,
) -> Element<'a, Message> {
    let on_search = map_msg.clone();
    let search_input = text_input("Buscar notas, ideias ou tags...", &state.search_query)
        .on_input(move |q| on_search(SecondBrainMessage::SearchChanged(q)))
        .padding([8.0, 14.0])
        .size(FONT_SIZE_BASE)
        .width(Length::Fixed(280.0))
        .style(move |_, status| theme.soft_input_style(status));

    let header = editorial_page_header(
        "ESTUDY  /  SEGUNDO CÉREBRO",
        "Segundo cérebro",
        "Ideias e referências",
        Some(search_input.into()),
        theme,
    );

    let filters = view_filters_strip(state, theme, map_msg.clone());
    let grid = view_wide_notes_grid(state, theme, &map_msg);

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

    let on_kind = map_msg.clone();
    let kind_switch = segmented_switch(
        &KINDS,
        state.kind_filter,
        move |kind| on_kind(SecondBrainMessage::SetKindFilter(kind)),
        theme,
    );

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

    for (idx, tag) in state.available_tags().into_iter().take(7).enumerate() {
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

    column![kind_switch, tags_row].spacing(SPACE_2).into()
}

enum GridCell<'a> {
    CreateNew,
    Note(&'a KnowledgeNote),
}

fn view_wide_notes_grid<'a, Message: Clone + 'a>(
    state: &'a SecondBrainState,
    theme: EstudyTheme,
    map_msg: &impl Fn(SecondBrainMessage) -> Message,
) -> Element<'a, Message> {
    let mut cells: Vec<GridCell<'a>> = vec![GridCell::CreateNew];
    for note in state.filtered_notes() {
        cells.push(GridCell::Note(note));
    }

    let mut rows = column![].spacing(SPACE_3);
    for chunk in cells.chunks(4) {
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
        for _ in chunk.len()..4 {
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

    button(
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
    .width(Length::FillPortion(1))
    .height(Length::Fixed(210.0))
    .padding(SPACE_4)
    .style(move |_, status| button::Style {
        background: Some(
            if matches!(status, button::Status::Hovered) {
                theme.card_hover
            } else {
                Color {
                    a: 0.45,
                    ..theme.surface_soft
                }
            }
            .into(),
        ),
        text_color: theme.text_primary,
        border: Border {
            color: Color {
                a: 0.25,
                ..theme.text_muted
            },
            width: 1.2,
            radius: RADIUS_CARD.into(),
        },
        ..button::Style::default()
    })
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

    let excerpt = if note.body_md.trim().is_empty() {
        "Clique para escrever ideias ou colar referências...".to_string()
    } else {
        note.preview_excerpt(120)
    };

    let mut tags_line = row![].spacing(6.0);
    for tag in note.tags.iter().take(3) {
        tags_line = tags_line.push(shaped_text(
            format!("#{tag}"),
            FONT_SIZE_XS,
            FONT_MONO,
            cat.solid(theme),
        ));
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
            tags_line,
        ]
        .spacing(SPACE_2)
        .width(Length::Fill)
        .height(Length::Fill),
    )
    .on_press(map_msg(SecondBrainMessage::SelectNote(note.id.clone())))
    .padding(SPACE_4)
    .width(Length::FillPortion(1))
    .height(Length::Fixed(210.0))
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
            radius: RADIUS_CARD.into(),
        },
        shadow: theme.card_style().shadow,
        snap: true,
    })
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

    let body = view_note_modal_body(state, note, theme, map_msg.clone());

    let tag_mapper = map_msg.clone();
    let tag_footer = row![
        text_input("+ tag (ex.: rust, revisão)", &state.new_tag_input)
            .on_input(move |v| tag_mapper(SecondBrainMessage::NewTagInputChanged(v)))
            .on_submit(map_msg(SecondBrainMessage::AddTag))
            .padding([7.0, 12.0])
            .size(FONT_SIZE_BASE)
            .width(Length::Fixed(220.0))
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

    modal_surface(
        column![top_bar, kind_row, hairline_divider(theme), body, tag_footer].spacing(SPACE_3),
        640.0,
        theme,
    )
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
            let on_editor = map_msg;
            column![
                text_input("Título da nota...", &note.title)
                    .on_input(move |v| on_title(SecondBrainMessage::TitleChanged(v)))
                    .padding([SPACE_2, SPACE_3])
                    .size(FONT_SIZE_XL)
                    .font(FONT_SERIF_MEDIUM)
                    .style(move |_, status| theme.soft_input_style(status)),
                text_editor(&state.body_editor)
                    .placeholder("Escreva à vontade — Enter cria novos parágrafos...")
                    .on_action(
                        move |action| on_editor(SecondBrainMessage::BodyEditorAction(action))
                    )
                    .height(Length::Fixed(240.0))
                    .padding(SPACE_3)
                    .size(FONT_SIZE_MD)
                    .font(FONT_SANS)
                    .style(move |_, status| editor_soft_style(theme, status)),
            ]
            .spacing(SPACE_3)
            .into()
        }
        BrainEditorTab::Preview => soft_surface(
            scrollable(
                column![
                    text(&note.title)
                        .size(FONT_SIZE_2XL)
                        .font(FONT_SERIF)
                        .color(theme.text_primary)
                        .shaping(text::Shaping::Advanced),
                    shaped_text(
                        if note.body_md.trim().is_empty() {
                            "Nota em branco. Alterne para Escrever para capturar ideias."
                        } else {
                            note.body_md.as_str()
                        },
                        FONT_SIZE_MD,
                        FONT_SANS,
                        theme.text_primary,
                    ),
                ]
                .spacing(SPACE_3),
            )
            .height(Length::Fixed(270.0)),
            theme,
        )
        .padding(SPACE_4)
        .width(Length::Fill)
        .into(),
    }
}
