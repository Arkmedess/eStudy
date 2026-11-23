//! Primitivos reutilizáveis do Design System em `Iced`.

use iced::widget::{container, row, svg, text};
use iced::{Alignment, Border, Color, Element, Length};

use crate::icons::Icon;
use crate::theme::{EstudyTheme, RADIUS_CHIP, SPACE_2, SPACE_3, SPACE_4};

/// Renderiza um ícone SVG do catálogo no tamanho e cor informados.
#[must_use]
pub fn icon_svg<'a, Message: 'a>(icon: Icon, size: f32, color: Color) -> Element<'a, Message> {
    let bytes = icon.to_svg_string(color).into_bytes();
    svg(svg::Handle::from_memory(bytes))
        .width(Length::Fixed(size))
        .height(Length::Fixed(size))
        .into()
}

/// Container estilizado de cartão elevado seguindo o tema ativo.
pub fn card<'a, Message: 'a>(
    content: impl Into<Element<'a, Message>>,
    theme: EstudyTheme,
) -> container::Container<'a, Message> {
    container(content)
        .padding(SPACE_4)
        .style(move |_| theme.card_style())
}

/// Container estilizado de painel lateral ou faixa de suporte.
pub fn panel<'a, Message: 'a>(
    content: impl Into<Element<'a, Message>>,
    theme: EstudyTheme,
) -> container::Container<'a, Message> {
    container(content)
        .padding(SPACE_3)
        .style(move |_| theme.panel_style())
}

/// Badge semântico compacto (máximo de 1 por item em cartões e linhas).
pub fn status_badge<'a, Message: 'a>(
    label: impl Into<String>,
    tint: Color,
    theme: EstudyTheme,
) -> Element<'a, Message> {
    container(text(label.into()).size(11).color(tint))
        .padding([3.0, 8.0])
        .style(move |_| container::Style {
            background: Some(Color { a: 0.14, ..tint }.into()),
            border: Border {
                color: theme.border_subtle,
                width: 1.0,
                radius: RADIUS_CHIP.into(),
            },
            ..container::Style::default()
        })
        .into()
}

/// Cabeçalho enxuto de seção com ícone SVG e título.
pub fn section_header<'a, Message: 'a>(
    icon: Icon,
    title: impl Into<String>,
    theme: EstudyTheme,
) -> Element<'a, Message> {
    row![
        icon_svg(icon, 16.0, theme.accent),
        text(title.into()).size(14).color(theme.text_primary),
    ]
    .spacing(SPACE_2)
    .align_y(Alignment::Center)
    .into()
}
