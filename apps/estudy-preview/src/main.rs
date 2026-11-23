//! Sandbox isolado de visualização dos componentes do `estudy-ui-kit`.

use estudy_ui_kit::widgets::{card, icon_svg, section_header, status_badge};
use estudy_ui_kit::{
    BoardFrameStyle, EstudyTheme, HeatmapData, HeatmapGrid, Icon, SPACE_2, SPACE_3, SPACE_4,
    ThemeMode, ThemePreset,
};
use iced::widget::{button, canvas, column, container, row, scrollable, text};
use iced::{Alignment, Element, Length};

#[derive(Debug, Clone, Copy)]
enum PreviewMessage {
    SelectPreset(ThemePreset),
    ToggleMode,
    SelectFrame(BoardFrameStyle),
}

struct PreviewApp {
    preset: ThemePreset,
    mode: ThemeMode,
    frame: BoardFrameStyle,
    heatmap: HeatmapData,
}

impl PreviewApp {
    fn boot() -> Self {
        let mut heatmap = HeatmapData::default();
        for (idx, cell) in heatmap.days.iter_mut().enumerate() {
            #[allow(clippy::cast_possible_truncation)]
            {
                *cell = ((idx * 5 + 2) % 11) as u16;
            }
        }
        Self {
            preset: ThemePreset::OlivaOrganico,
            mode: ThemeMode::Light,
            frame: BoardFrameStyle::Japandi,
            heatmap,
        }
    }

    fn update(&mut self, message: PreviewMessage) {
        match message {
            PreviewMessage::SelectPreset(preset) => self.preset = preset,
            PreviewMessage::ToggleMode => self.mode = self.mode.toggle(),
            PreviewMessage::SelectFrame(frame) => self.frame = frame,
        }
    }

    fn view(&self) -> Element<'_, PreviewMessage> {
        let theme = EstudyTheme::resolve(self.preset, self.mode);
        let preset_row = self.view_preset_row(theme);
        let mode_btn = button(text(self.mode.label()).size(12).color(theme.on_accent))
            .on_press(PreviewMessage::ToggleMode)
            .padding([SPACE_2, SPACE_3])
            .style(move |_, status| theme.primary_button_style(status));

        let content = column![
            row![
                section_header(Icon::Palette, "eStudy UI Kit Preview", theme),
                status_badge("WCAG 2.2 AA", theme.success, theme),
                mode_btn,
            ]
            .spacing(SPACE_3)
            .align_y(Alignment::Center),
            preset_row,
            card(
                column![
                    section_header(Icon::Flame, "Heatmap Anual (365 dias)", theme),
                    canvas(HeatmapGrid {
                        data: &self.heatmap,
                        theme,
                    })
                    .width(Length::Fill)
                    .height(Length::Fixed(106.0)),
                ]
                .spacing(SPACE_2),
                theme
            ),
            self.view_frames_card(theme),
            view_icons_card(theme),
        ]
        .spacing(SPACE_4)
        .padding(SPACE_4);

        container(scrollable(content))
            .width(Length::Fill)
            .height(Length::Fill)
            .style(move |_| container::Style {
                background: Some(theme.bg.into()),
                ..container::Style::default()
            })
            .into()
    }

    fn view_preset_row(&self, theme: EstudyTheme) -> Element<'_, PreviewMessage> {
        let mut preset_row = row![].spacing(SPACE_2);
        for preset in ThemePreset::ALL {
            let active = preset == self.preset;
            preset_row = preset_row.push(
                button(text(preset.label()).size(12).color(if active {
                    theme.on_accent
                } else {
                    theme.text_primary
                }))
                .on_press(PreviewMessage::SelectPreset(preset))
                .padding([SPACE_2, SPACE_3])
                .style(move |_, status| {
                    if active {
                        theme.primary_button_style(status)
                    } else {
                        theme.ghost_button_style(status)
                    }
                }),
            );
        }
        preset_row.into()
    }

    fn view_frames_card(&self, theme: EstudyTheme) -> Element<'_, PreviewMessage> {
        let mut frame_row = row![].spacing(SPACE_2);
        for frame in BoardFrameStyle::ALL {
            let active = frame == self.frame;
            frame_row = frame_row.push(
                button(text(frame.label()).size(11).color(if active {
                    theme.on_accent
                } else {
                    theme.text_primary
                }))
                .on_press(PreviewMessage::SelectFrame(frame))
                .padding([4.0, 8.0])
                .style(move |_, status| {
                    if active {
                        theme.primary_button_style(status)
                    } else {
                        theme.ghost_button_style(status)
                    }
                }),
            );
        }

        let active_frame = self.frame;
        let preview_frame = container(
            row![
                icon_svg(Icon::Image, 24.0, theme.accent),
                text(format!("Moldura ativa: {}", active_frame.label()))
                    .size(13)
                    .color(theme.text_primary),
            ]
            .spacing(SPACE_2)
            .align_y(Alignment::Center),
        )
        .padding(SPACE_4)
        .style(move |_| theme.board_frame_style(active_frame));

        card(
            column![
                section_header(Icon::Frame, "Molduras do Quadro", theme),
                frame_row,
                preview_frame,
            ]
            .spacing(SPACE_3),
            theme,
        )
        .into()
    }
}

fn view_icons_card<'a>(theme: EstudyTheme) -> Element<'a, PreviewMessage> {
    let mut icons_grid = column![].spacing(SPACE_2);
    for chunk in Icon::ALL.chunks(12) {
        let mut line = row![].spacing(SPACE_3);
        for icon in chunk {
            line = line.push(
                column![
                    icon_svg(*icon, 20.0, theme.text_primary),
                    text(icon.name()).size(9).color(theme.text_muted),
                ]
                .spacing(4.0)
                .align_x(Alignment::Center)
                .width(Length::Fixed(72.0)),
            );
        }
        icons_grid = icons_grid.push(line);
    }

    card(
        column![
            section_header(Icon::LayoutGrid, "Catálogo de 87 Ícones SVG", theme),
            icons_grid,
        ]
        .spacing(SPACE_3),
        theme,
    )
    .into()
}

fn main() -> iced::Result {
    iced::application(PreviewApp::boot, PreviewApp::update, PreviewApp::view)
        .title("eStudy UI Kit · Preview")
        .window_size((1180.0, 820.0))
        .centered()
        .run()
}
