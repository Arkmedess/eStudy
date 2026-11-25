//! Renderização do Segundo Cérebro (grade de cartões + painel mestre-detalhe na mesma tela).

use estudy_domain::{KnowledgeNote, NoteKind};
use estudy_ui_kit::widgets::{
    card, category_pill, editorial_heading, empty_placeholder, hairline_divider, icon_svg,
    pill_chip, section_header, segmented_switch, soft_surface,
};
use estudy_ui_kit::{
    CategoryColor, EstudyTheme, FONT_MONO, FONT_SANS_MEDIUM, FONT_SANS_SEMIBOLD, FONT_SERIF_MEDIUM,
    Icon, RADIUS_CONTROL, SPACE_1, SPACE_2, SPACE_3, SPACE_4, SPACE_5, SPACE_6,
};
use iced::widget::{button, column, container, row, scrollable, text, text_input};
use iced::{Alignment, Border, Element, Length};

use super::state::{BrainEditorTab, SecondBrainMessage, SecondBrainState};

/// Renderiza a tela completa do Segundo Cérebro.
pub fn view_second_brain<'a, Message: Clone + 'a>(
    state: &'a SecondBrainState,
    theme: EstudyTheme,
    map_msg: impl Fn(SecondBrainMessage) -> Message + Clone + 'a,
) -> Element<'a, Message> {
    let header = view_header(state, theme, map_msg.clone());
    let filters = view_filters_strip(state, theme, map_msg.clone());
    let grid = view_notes_grid(state, theme, &map_msg);
    let editor = view_inline_editor(state, theme, map_msg);

    let workspace = row![grid, editor]
        .spacing(SPACE_4)
        .width(Length::Fill)
        .height(Length::Fill);

    container(
        container(
            column![header, filters, workspace]
                .spacing(SPACE_4)
                .width(Length::Fill)
                .height(Length::Fill),
        )
        .max_width(1360.0)
        .width(Length::Fill)
        .height(Length::Fill),
    )
    .padding([SPACE_5, SPACE_6])
    .center_x(Length::Fill)
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}

fn note_kind_color(kind: NoteKind) -> CategoryColor {
    match kind {
        NoteKind::Note => CategoryColor::Slate,
        NoteKind::Idea => CategoryColor::Ochre,
        NoteKind::Reference => CategoryColor::Lavender,
        NoteKind::Link => CategoryColor::Sage,
    }
}

fn view_header<'a, Message: Clone + 'a>(
    state: &'a SecondBrainState,
    theme: EstudyTheme,
    map_msg: impl Fn(SecondBrainMessage) -> Message + Clone + 'a,
) -> Element<'a, Message> {
    let subtitle = format!(
        "{} registros organizados · ideias, referências e anotações pesquisáveis",
        state.notes.len()
    );

    let on_search = map_msg.clone();
    let search = text_input("Buscar notas ou tags...", &state.search_query)
        .on_input(move |q| on_search(SecondBrainMessage::SearchChanged(q)))
        .padding([6.0, 12.0])
        .size(12)
        .width(Length::Fixed(220.0))
        .style(move |_, status| theme.soft_input_style(status));

    let new_btn = button(
        row![
            icon_svg(Icon::Plus, 13.0, theme.on_accent),
            text("Nova nota")
                .size(12)
                .font(FONT_SANS_MEDIUM)
                .color(theme.on_accent),
        ]
        .spacing(4.0)
        .align_y(Alignment::Center),
    )
    .on_press(map_msg(SecondBrainMessage::CreateNote))
    .padding([6.0, 14.0])
    .style(move |_, status| theme.pill_button_style(true, status));

    row![
        editorial_heading("Segundo Cérebro", subtitle, theme),
        container(
            row![search, new_btn]
                .spacing(SPACE_2)
                .align_y(Alignment::Center),
        )
        .align_x(Alignment::End)
        .width(Length::Fill),
    ]
    .spacing(SPACE_4)
    .align_y(Alignment::Center)
    .into()
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

    let mut tags_row = row![icon_svg(Icon::Tag, 13.0, theme.text_muted)]
        .spacing(SPACE_1)
        .align_y(Alignment::Center);

    for (idx, tag) in state.available_tags().into_iter().take(6).enumerate() {
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

    row![kind_switch, tags_row]
        .spacing(SPACE_3)
        .align_y(Alignment::Center)
        .into()
}

fn view_notes_grid<'a, Message: Clone + 'a>(
    state: &'a SecondBrainState,
    theme: EstudyTheme,
    map_msg: &impl Fn(SecondBrainMessage) -> Message,
) -> Element<'a, Message> {
    let notes = state.filtered_notes();
    if notes.is_empty() {
        return container(empty_placeholder(
            Icon::Brain,
            "Nenhuma nota encontrada",
            "Ajuste o filtro acima ou clique em Nova nota para capturar uma ideia.",
            None,
            theme,
        ))
        .width(Length::FillPortion(7))
        .into();
    }

    let mut rows = column![].spacing(SPACE_3);
    for pair in notes.chunks(2) {
        let mut line = row![].spacing(SPACE_3).width(Length::Fill);
        for note in pair {
            line = line.push(view_note_card(
                note,
                state.selected_note_id.as_ref() == Some(&note.id),
                theme,
                map_msg,
            ));
        }
        if pair.len() == 1 {
            line = line.push(container(text("")).width(Length::Fill));
        }
        rows = rows.push(line);
    }

    scrollable(rows)
        .width(Length::FillPortion(7))
        .height(Length::Fill)
        .into()
}

fn view_note_card<'a, Message: Clone + 'a>(
    note: &'a KnowledgeNote,
    selected: bool,
    theme: EstudyTheme,
    map_msg: &impl Fn(SecondBrainMessage) -> Message,
) -> Element<'a, Message> {
    let cat = note_kind_color(note.kind);
    let mut header = row![category_pill(note.kind.label(), cat, theme)]
        .spacing(SPACE_2)
        .align_y(Alignment::Center);
    if note.pinned {
        header = header.push(icon_svg(Icon::Pin, 12.0, theme.warning));
    }

    let excerpt = if note.body_md.trim().is_empty() {
        "Sem conteúdo ainda — clique para escrever.".to_string()
    } else {
        note.preview_excerpt(115)
    };

    let mut tags_line = row![].spacing(6.0);
    for tag in note.tags.iter().take(3) {
        tags_line = tags_line.push(
            text(format!("#{tag}"))
                .size(10)
                .font(FONT_MONO)
                .color(theme.text_muted),
        );
    }

    let border_color = if selected {
        cat.solid(theme)
    } else {
        theme.border_subtle
    };
    let bg = if selected {
        cat.tint(theme)
    } else {
        theme.card
    };

    button(
        column![
            header,
            text(&note.title)
                .size(14)
                .font(FONT_SANS_SEMIBOLD)
                .color(theme.text_primary),
            text(excerpt).size(12).color(theme.text_muted),
            tags_line,
        ]
        .spacing(SPACE_2)
        .width(Length::Fill),
    )
    .on_press(map_msg(SecondBrainMessage::SelectNote(note.id.clone())))
    .padding(SPACE_4)
    .width(Length::Fill)
    .style(move |_, status| button::Style {
        background: if matches!(status, button::Status::Hovered) && !selected {
            Some(theme.card_hover.into())
        } else {
            Some(bg.into())
        },
        border: Border {
            color: border_color,
            width: if selected { 1.5 } else { 1.0 },
            radius: RADIUS_CONTROL.into(),
        },
        ..button::Style::default()
    })
    .into()
}

fn view_inline_editor<'a, Message: Clone + 'a>(
    state: &'a SecondBrainState,
    theme: EstudyTheme,
    map_msg: impl Fn(SecondBrainMessage) -> Message + Clone + 'a,
) -> Element<'a, Message> {
    const TABS: [(BrainEditorTab, &str); 2] = [
        (BrainEditorTab::Preview, "Visualizar"),
        (BrainEditorTab::Write, "Escrever"),
    ];

    let Some(note) = state.selected_note() else {
        return card(
            empty_placeholder(
                Icon::Brain,
                "Selecione uma nota",
                "Clique em qualquer cartão à esquerda para ler ou editar aqui mesmo.",
                None,
                theme,
            ),
            theme,
        )
        .width(Length::FillPortion(5))
        .height(Length::Fill)
        .into();
    };

    let on_tab = map_msg.clone();
    let tab_switch = segmented_switch(
        &TABS,
        state.editor_tab,
        move |tab| on_tab(SecondBrainMessage::SetEditorTab(tab)),
        theme,
    );

    let top_actions = row![
        tab_switch,
        container(
            row![
                button(icon_svg(
                    Icon::Pin,
                    14.0,
                    if note.pinned {
                        theme.warning
                    } else {
                        theme.text_muted
                    },
                ))
                .on_press(map_msg(SecondBrainMessage::TogglePin(note.id.clone())))
                .padding(SPACE_1)
                .style(move |_, status| theme.ghost_button_style(status)),
                button(icon_svg(Icon::Trash2, 14.0, theme.danger))
                    .on_press(map_msg(SecondBrainMessage::DeleteNote(note.id.clone())))
                    .padding(SPACE_1)
                    .style(move |_, status| theme.ghost_button_style(status)),
            ]
            .spacing(SPACE_1),
        )
        .align_x(Alignment::End)
        .width(Length::Fill),
    ]
    .align_y(Alignment::Center);

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

    let body_section = view_editor_body(note, state.editor_tab, theme, map_msg.clone());
    let on_tag_input = map_msg.clone();
    let tag_input = row![
        text_input("+ tag (ex.: rust)", &state.new_tag_input)
            .on_input(move |v| on_tag_input(SecondBrainMessage::NewTagInputChanged(v)))
            .on_submit(map_msg(SecondBrainMessage::AddTag))
            .padding([5.0, 10.0])
            .size(12)
            .style(move |_, status| theme.soft_input_style(status)),
        button(icon_svg(Icon::Plus, 13.0, theme.on_accent))
            .on_press(map_msg(SecondBrainMessage::AddTag))
            .padding([5.0, 10.0])
            .style(move |_, status| theme.pill_button_style(true, status)),
    ]
    .spacing(SPACE_2)
    .align_y(Alignment::Center);

    card(
        scrollable(
            column![
                section_header(Icon::Brain, "Leitura & Edição Rápida", theme),
                top_actions,
                kind_row,
                hairline_divider(theme),
                body_section,
                tag_input,
            ]
            .spacing(SPACE_3),
        )
        .height(Length::Fill),
        theme,
    )
    .width(Length::FillPortion(5))
    .height(Length::Fill)
    .into()
}

fn view_editor_body<'a, Message: Clone + 'a>(
    note: &'a KnowledgeNote,
    tab: BrainEditorTab,
    theme: EstudyTheme,
    map_msg: impl Fn(SecondBrainMessage) -> Message + Clone + 'a,
) -> Element<'a, Message> {
    match tab {
        BrainEditorTab::Preview => soft_surface(
            column![
                text(&note.title)
                    .size(20)
                    .font(FONT_SERIF_MEDIUM)
                    .color(theme.text_primary),
                text(if note.body_md.trim().is_empty() {
                    "Nota em branco. Troque para Escrever para adicionar ideias."
                } else {
                    note.body_md.as_str()
                })
                .size(13)
                .color(theme.text_primary),
            ]
            .spacing(SPACE_3),
            theme,
        )
        .padding(SPACE_4)
        .width(Length::Fill)
        .into(),
        BrainEditorTab::Write => {
            let on_title = map_msg.clone();
            let on_body = map_msg;
            column![
                text_input("Título da nota...", &note.title)
                    .on_input(move |v| on_title(SecondBrainMessage::TitleChanged(v)))
                    .padding(SPACE_2)
                    .size(15)
                    .style(move |_, status| theme.soft_input_style(status)),
                text_input("Escreva sua nota em Markdown...", &note.body_md)
                    .on_input(move |v| on_body(SecondBrainMessage::BodyChanged(v)))
                    .padding(SPACE_3)
                    .size(13)
                    .style(move |_, status| theme.soft_input_style(status)),
            ]
            .spacing(SPACE_2)
            .into()
        }
    }
}
