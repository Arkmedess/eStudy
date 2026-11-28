//! Primitivos reutilizáveis do Design System em `Iced`.

use iced::border;
use iced::widget::canvas::{self, Canvas, Geometry, LineCap, LineDash, Path, Stroke};
use iced::widget::text::{LineHeight, Shaping};
use iced::widget::{
    Stack, button, column, container, progress_bar, rich_text, row, span, svg, text,
};
use iced::{
    Alignment, Border, Color, Element, Font, Length, Point, Rectangle, Renderer, Size, Theme,
};

use crate::fonts::{
    FONT_MONO, FONT_SANS, FONT_SANS_MEDIUM, FONT_SANS_SEMIBOLD, FONT_SERIF, FONT_SERIF_ITALIC,
    FONT_SERIF_MEDIUM, FONT_SIZE_2XL, FONT_SIZE_BASE, FONT_SIZE_DISPLAY, FONT_SIZE_LG,
    FONT_SIZE_MD, FONT_SIZE_SM, FONT_SIZE_XL, FONT_SIZE_XS,
};
use crate::icons::{Icon, IllustratedIcon};
use crate::theme::{
    CategoryColor, EstudyTheme, RADIUS_CHIP, RADIUS_CONTROL, RADIUS_PILL, SPACE_1, SPACE_2,
    SPACE_3, SPACE_4, SPACE_5,
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

/// Cria um elemento de texto com kerning/posicionamento OpenType completo (`Shaping::Advanced`).
pub fn shaped_text<'a>(
    content: impl text::IntoFragment<'a>,
    size: f32,
    font: Font,
    color: Color,
) -> text::Text<'a> {
    text(content)
        .size(size)
        .font(font)
        .color(color)
        .shaping(Shaping::Advanced)
        .line_height(LineHeight::Relative(1.38))
}

/// Renderiza um ícone SVG monoline (`1.75px`) no tamanho inteiro próximo e cor informada.
#[must_use]
pub fn icon_svg<'a, Message: 'a>(icon: Icon, size: f32, color: Color) -> Element<'a, Message> {
    let snapped = size.round().max(12.0);
    svg(icon.cached_handle())
        .width(Length::Fixed(snapped))
        .height(Length::Fixed(snapped))
        .style(move |_, _| svg::Style { color: Some(color) })
        .into()
}

/// Renderiza um ícone SVG ilustrado multi-cor preservando todas as cores e luzes originais.
#[must_use]
pub fn illustrated_svg<'a, Message: 'a>(icon: IllustratedIcon, size: f32) -> Element<'a, Message> {
    let snapped = size.round().max(16.0);
    svg(icon.cached_handle())
        .width(Length::Fixed(snapped))
        .height(Length::Fixed(snapped))
        .into()
}

/// Renderiza um ícone SVG ilustrado adaptado automaticamente ao tema Claro/Escuro ativo.
#[must_use]
pub fn illustrated_svg_themed<'a, Message: 'a>(
    icon: IllustratedIcon,
    size: f32,
    theme: EstudyTheme,
) -> Element<'a, Message> {
    let snapped = size.round().max(16.0);
    svg(icon.cached_handle_for_mode(theme.mode))
        .width(Length::Fixed(snapped))
        .height(Length::Fixed(snapped))
        .into()
}

/// Texto com traço horizontal real (`strikethrough`) e kerning OpenType para itens concluídos.
pub fn strikethrough_text<'a, Message: 'a>(
    content: impl Into<String>,
    size: f32,
    font: Font,
    color: Color,
) -> Element<'a, Message> {
    rich_text::<(), Message, Theme, Renderer>([span(content.into())
        .font(font)
        .size(size)
        .color(color)
        .strikethrough(true)])
    .line_height(LineHeight::Relative(1.38))
    .into()
}

/// Botão circular tátil (`○ -> ✓`) sem anel duplo: vazio em repouso e preenchido com `Icon::Check` quando marcado.
pub fn radio_check_circle<'a, Message: Clone + 'a>(
    checked: bool,
    tint: Color,
    size: f32,
    on_press: Message,
    theme: EstudyTheme,
) -> Element<'a, Message> {
    let snapped = size.round().max(18.0);
    let check_size = (snapped * 0.56).round().max(11.0);
    let inner: Element<'a, Message> = if checked {
        container(icon_svg(Icon::Check, check_size, theme.on_accent))
            .width(Length::Fixed(snapped))
            .height(Length::Fixed(snapped))
            .center_x(Length::Fixed(snapped))
            .center_y(Length::Fixed(snapped))
            .into()
    } else {
        container(text("").size(1))
            .width(Length::Fixed(snapped))
            .height(Length::Fixed(snapped))
            .into()
    };

    button(inner)
        .on_press(on_press)
        .width(Length::Fixed(snapped))
        .height(Length::Fixed(snapped))
        .padding(0.0)
        .style(move |_, status| {
            let hovered = matches!(status, button::Status::Hovered | button::Status::Pressed);
            let bg = if checked {
                Some(
                    Color {
                        a: if hovered { 0.88 } else { 1.0 },
                        ..tint
                    }
                    .into(),
                )
            } else if hovered {
                Some(Color { a: 0.14, ..tint }.into())
            } else {
                Some(Color::TRANSPARENT.into())
            };
            let border_color = if checked || hovered {
                tint
            } else {
                Color {
                    a: 0.45,
                    ..theme.text_muted
                }
            };
            button::Style {
                background: bg,
                text_color: theme.on_accent,
                border: Border {
                    color: border_color,
                    width: 1.5,
                    radius: RADIUS_PILL.into(),
                },
                ..button::Style::default()
            }
        })
        .into()
}

struct DashedFrameProgram {
    stroke_color: Color,
    fill_color: Option<Color>,
    radius: f32,
}

impl<Message> canvas::Program<Message> for DashedFrameProgram {
    type State = ();

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: iced::mouse::Cursor,
    ) -> Vec<Geometry> {
        let mut frame = canvas::Frame::new(renderer, bounds.size());
        let rect = Path::rounded_rectangle(
            Point::new(1.0, 1.0),
            Size::new(
                (bounds.width - 2.0).max(1.0),
                (bounds.height - 2.0).max(1.0),
            ),
            border::Radius::from(self.radius),
        );
        if let Some(fill) = self.fill_color {
            frame.fill(&rect, fill);
        }
        frame.stroke(
            &rect,
            Stroke {
                line_dash: LineDash {
                    segments: &[5.0, 4.0],
                    offset: 0,
                },
                line_cap: LineCap::Round,
                ..Stroke::default()
                    .with_color(self.stroke_color)
                    .with_width(1.25)
            },
        );
        vec![frame.into_geometry()]
    }
}

/// Envolve o conteúdo com uma moldura tracejada desenhada em `Canvas` leve sem distorcer o tamanho intrínseco.
pub fn dashed_frame<'a, Message: 'a>(
    content: impl Into<Element<'a, Message>>,
    stroke_color: Color,
    fill_color: Option<Color>,
    radius: f32,
) -> Element<'a, Message> {
    Stack::new()
        .push(content.into())
        .push_under(
            Canvas::new(DashedFrameProgram {
                stroke_color,
                fill_color,
                radius,
            })
            .width(Length::Fill)
            .height(Length::Fill),
        )
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
    container(shaped_text(
        label.into(),
        FONT_SIZE_XS,
        FONT_SANS_SEMIBOLD,
        tint,
    ))
    .padding([4.0, 10.0])
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
    container(shaped_text(
        shortcut.into(),
        FONT_SIZE_XS,
        FONT_MONO,
        theme.text_muted,
    ))
    .padding([3.0, 7.0])
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
        items = items.push(icon_svg(ic, 16.0, text_color));
    }
    items = items.push(shaped_text(
        label.into(),
        FONT_SIZE_BASE,
        FONT_SANS_MEDIUM,
        text_color,
    ));

    button(items)
        .on_press(on_press)
        .padding([6.0, 14.0])
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
        let label_font = if is_active {
            FONT_SANS_SEMIBOLD
        } else {
            FONT_SANS_MEDIUM
        };
        row_items = row_items.push(
            button(shaped_text(label, FONT_SIZE_BASE, label_font, label_color))
                .on_press_maybe((!is_active).then(|| on_select(value)))
                .padding([6.0, 15.0])
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
            shaped_text(
                label.into(),
                FONT_SIZE_SM,
                FONT_SANS_MEDIUM,
                theme.text_muted
            ),
            shaped_text(value.into(), FONT_SIZE_2XL, FONT_SERIF_MEDIUM, tint),
            shaped_text(subtitle.into(), FONT_SIZE_SM, FONT_SANS, theme.text_muted),
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
        container(icon_svg(icon, 24.0, theme.accent))
            .padding(SPACE_2)
            .style(move |_| container::Style {
                background: Some(theme.accent_subtle.into()),
                border: Border {
                    radius: RADIUS_PILL.into(),
                    ..Border::default()
                },
                ..container::Style::default()
            }),
        shaped_text(
            title.into(),
            FONT_SIZE_LG,
            FONT_SERIF_MEDIUM,
            theme.text_primary
        ),
        shaped_text(subtitle.into(), FONT_SIZE_BASE, FONT_SANS, theme.text_muted),
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
        icon_svg(icon, 16.0, theme.accent),
        shaped_text(
            message.into(),
            FONT_SIZE_BASE,
            FONT_SANS,
            theme.text_primary
        ),
    ]
    .spacing(SPACE_2)
    .align_y(Alignment::Center);

    if let Some(dismiss_msg) = on_dismiss {
        inner = inner.push(
            container(
                button(icon_svg(Icon::X, 15.0, theme.text_muted))
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

/// Cabeçalho enxuto de seção com ícone SVG e título em peso semibold.
pub fn section_header<'a, Message: 'a>(
    icon: Icon,
    title: impl Into<String>,
    theme: EstudyTheme,
) -> Element<'a, Message> {
    row![
        icon_svg(icon, 16.0, theme.accent),
        shaped_text(
            title.into(),
            FONT_SIZE_LG,
            FONT_SANS_SEMIBOLD,
            theme.text_primary
        ),
    ]
    .spacing(SPACE_2)
    .align_y(Alignment::Center)
    .into()
}

/// Cabeçalho editorial unificado: micro-breadcrumb + Título Serifa + Subtítulo Itálico na mesma base + ação direita opcional.
pub fn editorial_page_header<'a, Message: 'a>(
    breadcrumb: impl Into<String>,
    title: impl Into<String>,
    italic_subtitle: impl Into<String>,
    right_action: Option<Element<'a, Message>>,
    theme: EstudyTheme,
) -> Element<'a, Message> {
    let crumb = text(breadcrumb.into())
        .size(FONT_SIZE_XS)
        .font(FONT_SANS_MEDIUM)
        .color(theme.text_muted)
        .shaping(Shaping::Advanced);

    let title_row = row![
        text(title.into())
            .size(FONT_SIZE_DISPLAY)
            .font(FONT_SERIF)
            .color(theme.text_primary)
            .shaping(Shaping::Advanced)
            .line_height(LineHeight::Relative(1.12)),
        container(
            text(italic_subtitle.into())
                .size(FONT_SIZE_XL)
                .font(FONT_SERIF_ITALIC)
                .color(theme.text_muted)
                .shaping(Shaping::Advanced)
                .line_height(LineHeight::Relative(1.2)),
        )
        .padding([0.0, 0.0]),
    ]
    .spacing(SPACE_3)
    .align_y(Alignment::End)
    .wrap();

    let left = column![crumb, title_row].spacing(4.0).width(Length::Fill);

    if let Some(action) = right_action {
        row![
            left,
            container(action)
                .align_x(Alignment::End)
                .width(Length::Shrink),
        ]
        .spacing(SPACE_3)
        .align_y(Alignment::End)
        .width(Length::Fill)
        .into()
    } else {
        left.into()
    }
}

/// Cabeçalho editorial compacto com título (`Newsreader`) e subtítulo em itálico real na mesma linha de base.
pub fn editorial_heading<'a, Message: 'a>(
    title: impl Into<String>,
    subtitle: impl Into<String>,
    theme: EstudyTheme,
) -> Element<'a, Message> {
    let sub = subtitle.into();
    if sub.chars().count() <= 48 {
        row![
            text(title.into())
                .size(FONT_SIZE_DISPLAY)
                .font(FONT_SERIF_MEDIUM)
                .color(theme.text_primary)
                .shaping(Shaping::Advanced)
                .line_height(LineHeight::Relative(1.12)),
            text(sub)
                .size(FONT_SIZE_XL)
                .font(FONT_SERIF_ITALIC)
                .color(theme.text_muted)
                .shaping(Shaping::Advanced),
        ]
        .spacing(SPACE_3)
        .align_y(Alignment::End)
        .into()
    } else {
        column![
            text(title.into())
                .size(FONT_SIZE_DISPLAY)
                .font(FONT_SERIF_MEDIUM)
                .color(theme.text_primary)
                .shaping(Shaping::Advanced)
                .line_height(LineHeight::Relative(1.15)),
            shaped_text(sub, FONT_SIZE_MD, FONT_SANS, theme.text_muted),
        ]
        .spacing(SPACE_1)
        .into()
    }
}

/// Quadrado arredondado com ícone tintado pela cor semântica da categoria/projeto.
#[must_use]
pub fn category_tile<'a, Message: 'a>(
    icon: Icon,
    category: CategoryColor,
    size: f32,
    theme: EstudyTheme,
) -> Element<'a, Message> {
    let icon_size = (size * 0.52).clamp(12.0, 24.0);
    let fg = category.solid(theme);
    let bg = category.tint(theme);
    container(icon_svg(icon, icon_size, fg))
        .width(Length::Fixed(size))
        .height(Length::Fixed(size))
        .center_x(Length::Fixed(size))
        .center_y(Length::Fixed(size))
        .style(move |_| container::Style {
            background: Some(bg.into()),
            border: Border {
                color: category.border(theme),
                width: 1.0,
                radius: RADIUS_CONTROL.into(),
            },
            ..container::Style::default()
        })
        .into()
}

/// Pílula compacta de tag com ponto colorido semântico.
pub fn category_pill<'a, Message: 'a>(
    label: impl Into<String>,
    category: CategoryColor,
    theme: EstudyTheme,
) -> Element<'a, Message> {
    let fg = category.solid(theme);
    let bg = category.tint(theme);
    container(
        row![
            color_dot(fg),
            shaped_text(
                label.into(),
                FONT_SIZE_XS,
                FONT_SANS_MEDIUM,
                theme.text_primary
            ),
        ]
        .spacing(6.0)
        .align_y(Alignment::Center),
    )
    .padding([4.0, 10.0])
    .style(move |_| container::Style {
        background: Some(bg.into()),
        border: Border {
            color: category.border(theme),
            width: 1.0,
            radius: RADIUS_PILL.into(),
        },
        ..container::Style::default()
    })
    .into()
}

/// Gaveta flutuante arredondada à direita com margem externa (`20px` de raio).
pub fn floating_drawer<'a, Message: 'a>(
    content: impl Into<Element<'a, Message>>,
    width: f32,
    theme: EstudyTheme,
) -> Element<'a, Message> {
    let drawer_box = container(content)
        .width(Length::Fixed(width))
        .height(Length::Fill)
        .padding(SPACE_5)
        .style(move |_| theme.floating_drawer_style());

    container(drawer_box)
        .width(Length::Fill)
        .height(Length::Fill)
        .padding(SPACE_4)
        .align_x(Alignment::End)
        .style(|_| container::Style {
            background: Some(Color::from_rgba(0.0, 0.0, 0.0, 0.24).into()),
            ..container::Style::default()
        })
        .into()
}
