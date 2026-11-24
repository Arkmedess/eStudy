//! Primitivos reutilizáveis do Design System em `Iced`.

use iced::widget::{button, column, container, progress_bar, row, svg, text};
use iced::{Alignment, Border, Color, Element, Length};

use crate::icons::Icon;
use crate::theme::{
    EstudyTheme, RADIUS_CHIP, RADIUS_CONTROL, RADIUS_PILL, SPACE_1, SPACE_2, SPACE_3, SPACE_4,
    SPACE_5,
};

/// Garante a normalização de uma razão de progresso no intervalo fechado `0.0..=1.0`.
#[must_use]
pub const fn clamp_progress(ratio: f32) -> f32 {
    if ratio.is_nan() || ratio <= 0.0 {
        0.0
    } else if ratio >= 1.0 {
        1.0
    } else {
        ratio
    }
}

/// Renderiza um ícone SVG do catálogo no tamanho e cor informados usando cache estático.
#[must_use]
pub fn icon_svg<'a, Message: 'a>(icon: Icon, size: f32, color: Color) -> Element<'a, Message> {
    svg(icon.cached_handle())
        .width(Length::Fixed(size))
        .height(Length::Fixed(size))
        .style(move |_, _| svg::Style { color: Some(color) })
        .into()
}

/// Container estilizado de cartão elevado seguindo o tema ativo.
pub fn card<'a, Message: 'a>(
    content: impl Into<Element<'a, Message>>,
    theme: EstudyTheme,
) -> container::Container<'a, Message> {
    container(content)
        .padding(SPACE_5)
        .style(move |_| theme.card_style())
}

/// Cartão de destaque tonal com fundo em cor de acento e alto contraste.
pub fn tonal_card<'a, Message: 'a>(
    content: impl Into<Element<'a, Message>>,
    theme: EstudyTheme,
) -> container::Container<'a, Message> {
    container(content)
        .padding(SPACE_5)
        .style(move |_| theme.tonal_card_style())
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

/// Container de superfície macia para áreas internas sem sombra pesada.
pub fn soft_surface<'a, Message: 'a>(
    content: impl Into<Element<'a, Message>>,
    theme: EstudyTheme,
) -> container::Container<'a, Message> {
    container(content)
        .padding(SPACE_3)
        .style(move |_| theme.soft_surface_style())
}

/// Linha divisória orgânica (`1px`) para separar blocos dentro de um mesmo cartão.
#[must_use]
pub fn hairline_divider<'a, Message: 'a>(theme: EstudyTheme) -> Element<'a, Message> {
    container(text("").size(1))
        .width(Length::Fill)
        .height(Length::Fixed(1.0))
        .style(move |_| theme.divider_style())
        .into()
}

/// Ponto colorido discreto (`6px`) para tags, categorias e status.
#[must_use]
pub fn color_dot<'a, Message: 'a>(color: Color) -> Element<'a, Message> {
    container(text("").size(1))
        .width(Length::Fixed(6.0))
        .height(Length::Fixed(6.0))
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

/// Badge semântico compacto (máximo de 1 por item em cartões e linhas).
pub fn status_badge<'a, Message: 'a>(
    label: impl Into<String>,
    tint: Color,
    theme: EstudyTheme,
) -> Element<'a, Message> {
    container(text(label.into()).size(11).color(tint))
        .padding([3.0, 9.0])
        .style(move |_| container::Style {
            background: Some(Color { a: 0.13, ..tint }.into()),
            border: Border {
                color: theme.border_subtle,
                width: 1.0,
                radius: RADIUS_PILL.into(),
            },
            ..container::Style::default()
        })
        .into()
}

/// Micro-etiqueta para exibir atalhos de teclado (`Ctrl+K`, `Ctrl+B`, `Esc`).
pub fn kbd_hint<'a, Message: 'a>(
    shortcut: impl Into<String>,
    theme: EstudyTheme,
) -> Element<'a, Message> {
    container(text(shortcut.into()).size(10).color(theme.text_muted))
        .padding([2.0, 6.0])
        .style(move |_| container::Style {
            background: Some(theme.surface_soft.into()),
            border: Border {
                color: theme.border_subtle,
                width: 1.0,
                radius: RADIUS_CHIP.into(),
            },
            ..container::Style::default()
        })
        .into()
}

/// Chip interativo em formato pílula com ponto colorido ou ícone opcional.
pub fn pill_chip<'a, Message: Clone + 'a>(
    label: impl Into<String>,
    dot_color: Option<Color>,
    icon: Option<Icon>,
    active: bool,
    on_press: Message,
    theme: EstudyTheme,
) -> Element<'a, Message> {
    let text_color = if active {
        theme.on_accent
    } else {
        theme.text_primary
    };
    let mut items = row![].spacing(6.0).align_y(Alignment::Center);
    if let Some(dot) = dot_color {
        items = items.push(color_dot(if active { theme.on_accent } else { dot }));
    }
    if let Some(ic) = icon {
        items = items.push(icon_svg(ic, 13.0, text_color));
    }
    items = items.push(text(label.into()).size(12).color(text_color));

    button(items)
        .on_press(on_press)
        .padding([5.0, 12.0])
        .style(move |_, status| theme.pill_button_style(active, status))
        .into()
}

/// Seletor segmentado em bandeja cápsula (`Semana | Calendário`, `7 dias | Ano`).
pub fn segmented_switch<'a, T, Message>(
    items: &'a [(T, &'static str)],
    active: T,
    on_select: impl Fn(T) -> Message + 'a,
    theme: EstudyTheme,
) -> Element<'a, Message>
where
    T: Copy + PartialEq + 'a,
    Message: Clone + 'a,
{
    let mut row_items = row![].spacing(4.0).align_y(Alignment::Center);
    for &(value, label) in items {
        let is_active = value == active;
        let label_color = if is_active {
            theme.on_accent
        } else {
            theme.text_muted
        };
        row_items = row_items.push(
            button(text(label).size(12).color(label_color))
                .on_press_maybe((!is_active).then(|| on_select(value)))
                .padding([5.0, 12.0])
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

    container(row_items)
        .padding(3.0)
        .style(move |_| theme.pill_tray_style())
        .into()
}

/// Barra de progresso fina e arredondada seguindo as cores do tema.
#[must_use]
pub fn progress_track<'a, Message: 'a>(
    ratio: f32,
    tint: Color,
    theme: EstudyTheme,
) -> Element<'a, Message> {
    progress_bar(0.0..=1.0, clamp_progress(ratio))
        .length(Length::Fill)
        .girth(Length::Fixed(6.0))
        .style(move |_| progress_bar::Style {
            background: theme.surface_soft.into(),
            bar: tint.into(),
            border: Border {
                radius: RADIUS_PILL.into(),
                ..Border::default()
            },
        })
        .into()
}

/// Bloco compacto de métrica com valor em destaque e legenda secundária.
pub fn metric_stat<'a, Message: 'a>(
    label: impl Into<String>,
    value: impl Into<String>,
    subtitle: impl Into<String>,
    tint: Color,
    theme: EstudyTheme,
) -> container::Container<'a, Message> {
    soft_surface(
        column![
            text(label.into()).size(11).color(theme.text_muted),
            text(value.into()).size(22).color(tint),
            text(subtitle.into()).size(11).color(theme.text_muted),
        ]
        .spacing(SPACE_1),
        theme,
    )
}

/// Estado vazio acolhedor com ícone, explicação concisa e ação opcional.
pub fn empty_placeholder<'a, Message: 'a>(
    icon: Icon,
    title: impl Into<String>,
    subtitle: impl Into<String>,
    action: Option<Element<'a, Message>>,
    theme: EstudyTheme,
) -> Element<'a, Message> {
    let mut body = column![
        container(icon_svg(icon, 20.0, theme.accent))
            .padding(SPACE_2)
            .style(move |_| container::Style {
                background: Some(theme.accent_subtle.into()),
                border: Border {
                    radius: RADIUS_PILL.into(),
                    ..Border::default()
                },
                ..container::Style::default()
            }),
        text(title.into()).size(14).color(theme.text_primary),
        text(subtitle.into()).size(12).color(theme.text_muted),
    ]
    .spacing(SPACE_2)
    .align_x(Alignment::Center);

    if let Some(action_el) = action {
        body = body.push(action_el);
    }

    soft_surface(body, theme)
        .padding(SPACE_5)
        .width(Length::Fill)
        .center_x(Length::Fill)
        .into()
}

/// Faixa suave de notificação/status com botão opcional para fechar.
pub fn banner_toast<'a, Message: Clone + 'a>(
    icon: Icon,
    message: impl Into<String>,
    on_dismiss: Option<Message>,
    theme: EstudyTheme,
) -> Element<'a, Message> {
    let mut inner = row![
        icon_svg(icon, 14.0, theme.accent),
        text(message.into()).size(12).color(theme.text_primary),
    ]
    .spacing(SPACE_2)
    .align_y(Alignment::Center);

    if let Some(dismiss_msg) = on_dismiss {
        inner = inner.push(
            container(
                button(icon_svg(Icon::X, 13.0, theme.text_muted))
                    .on_press(dismiss_msg)
                    .padding(SPACE_1)
                    .style(move |_, status| theme.subtle_button_style(status)),
            )
            .align_x(Alignment::End)
            .width(Length::Fill),
        );
    }

    container(inner)
        .padding([SPACE_2, SPACE_4])
        .width(Length::Fill)
        .style(move |_| container::Style {
            background: Some(theme.surface_soft.into()),
            border: Border {
                color: theme.border_subtle,
                width: 1.0,
                radius: RADIUS_CONTROL.into(),
            },
            ..container::Style::default()
        })
        .into()
}

/// Envelope padrão de gaveta deslizante à direita sobre fundo escurecido suave.
pub fn drawer_backdrop<'a, Message: 'a>(
    content: impl Into<Element<'a, Message>>,
    width: f32,
    theme: EstudyTheme,
) -> Element<'a, Message> {
    let drawer_box = container(content)
        .width(Length::Fixed(width))
        .height(Length::Fill)
        .padding(SPACE_5)
        .style(move |_| theme.drawer_surface_style());

    container(drawer_box)
        .width(Length::Fill)
        .height(Length::Fill)
        .padding(SPACE_3)
        .align_x(Alignment::End)
        .style(|_| container::Style {
            background: Some(Color::from_rgba(0.0, 0.0, 0.0, 0.28).into()),
            ..container::Style::default()
        })
        .into()
}

/// Envelope elevado para modais e paleta de comandos (`Ctrl+K`).
pub fn modal_surface<'a, Message: 'a>(
    content: impl Into<Element<'a, Message>>,
    width: f32,
    theme: EstudyTheme,
) -> Element<'a, Message> {
    let modal_box = container(content)
        .width(Length::Fixed(width))
        .padding(SPACE_4)
        .style(move |_| theme.modal_surface_style());

    container(modal_box)
        .width(Length::Fill)
        .height(Length::Fill)
        .padding([72.0, SPACE_4])
        .align_x(Alignment::Center)
        .style(|_| container::Style {
            background: Some(Color::from_rgba(0.0, 0.0, 0.0, 0.32).into()),
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
