//! Sandbox isolado de visualização dos componentes do `estudy-ui-kit`.

use estudy_ui_kit::widgets::{
    banner_toast, card, empty_placeholder, hairline_divider, icon_svg, kbd_hint, metric_stat,
    pill_chip, progress_track, section_header, segmented_switch, status_badge, tonal_card,
};
use estudy_ui_kit::{
    BoardFrameStyle, EstudyTheme, HeatmapData, HeatmapGrid, Icon, SPACE_2, SPACE_3, SPACE_4,
    ThemeMode, ThemePreset,
};
use iced::widget::{button, canvas, column, container, row, scrollable, text};
use iced::{Alignment, Element, Length};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PreviewFilter {
    All,
    Goals,
    Ideas,
}

#[derive(Debug, Clone, Copy)]
enum PreviewMessage {
    SelectPreset(ThemePreset),
    ToggleMode,
    SelectFrame(BoardFrameStyle),
    SelectFilter(PreviewFilter),
}

struct PreviewApp {
    preset: ThemePreset,
    mode: ThemeMode,
    frame: BoardFrameStyle,
    filter: PreviewFilter,
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
            filter: PreviewFilter::All,
            heatmap,
        }
    }

    fn update(&mut self, message: PreviewMessage) {
        match message {
            PreviewMessage::SelectPreset(preset) => self.preset = preset,
            PreviewMessage::ToggleMode => self.mode = self.mode.toggle(),
            PreviewMessage::SelectFrame(frame) => self.frame = frame,
            PreviewMessage::SelectFilter(filter) => self.filter = filter,
        }
    }

    fn view(&self) -> Element<'_, PreviewMessage> {
        let theme = EstudyTheme::resolve(self.preset, self.mode);
        let preset_row = self.view_preset_row(theme);
        let mode_btn = pill_chip(
            self.mode.label(),
            None,
            Some(match self.mode {
                ThemeMode::Light => Icon::Moon,
                ThemeMode::Dark => Icon::Sun,
            }),
            true,
            PreviewMessage::ToggleMode,
            theme,
        );

        let content = column![
            row![
                section_header(Icon::Palette, "eStudy UI Kit Preview", theme),
                status_badge("WCAG 2.2 AA", theme.success, theme),
                kbd_hint("Ctrl+K", theme),
                mode_btn,
            ]
            .spacing(SPACE_3)
            .align_y(Alignment::Center),
            preset_row,
            banner_toast(
                Icon::Info,
                "Cartões com cantos concêntricos, hairline dividers e componentes em pílula.",
                None,
                theme,
            ),
            self.view_primitives_showcase(theme),
            card(
                column![
                    section_header(Icon::Flame, "Heatmap Anual (365 dias)", theme),
                    hairline_divider(theme),
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
            preset_row = preset_row.push(pill_chip(
                preset.label(),
                None,
                None,
                preset == self.preset,
                PreviewMessage::SelectPreset(preset),
                theme,
            ));
        }
        preset_row.into()
    }

    fn view_primitives_showcase(&self, theme: EstudyTheme) -> Element<'_, PreviewMessage> {
        const FILTERS: [(PreviewFilter, &str); 3] = [
            (PreviewFilter::All, "Todas"),
            (PreviewFilter::Goals, "Metas"),
            (PreviewFilter::Ideas, "Ideias"),
        ];
        let switch = segmented_switch(&FILTERS, self.filter, PreviewMessage::SelectFilter, theme);

        let stats = row![
            metric_stat("Sessões", "18", "nos últimos 7 dias", theme.accent, theme)
                .width(Length::Fill),
            metric_stat(
                "RAM Alvo",
                "33 MiB",
                "tiny-skia padrão",
                theme.success,
                theme
            )
            .width(Length::Fill),
            tonal_card(
                column![
                    text("Foco ativo").size(11).color(theme.on_accent),
                    text("25:00").size(22).color(theme.on_accent),
                    text("Bloco pronto").size(11).color(theme.on_accent),
                ]
                .spacing(4.0),
                theme,
            )
            .padding(SPACE_3)
            .width(Length::Fill),
        ]
        .spacing(SPACE_3);

        card(
            column![
                row![
                    section_header(Icon::Layers, "Componentes Base", theme),
                    container(switch)
                        .align_x(Alignment::End)
                        .width(Length::Fill),
                ]
                .align_y(Alignment::Center),
                hairline_divider(theme),
                stats,
                progress_track(0.68, theme.accent, theme),
                empty_placeholder(
                    Icon::FolderKanban,
                    "Estado vazio acolhedor",
                    "Orientação clara sem ruído visual excessivo.",
                    Some(
                        button(text("Criar item").size(12).color(theme.on_accent))
                            .on_press(PreviewMessage::SelectFilter(PreviewFilter::All))
                            .padding([6.0, 14.0])
                            .style(move |_, status| theme.pill_button_style(true, status))
                            .into(),
                    ),
                    theme,
                ),
            ]
            .spacing(SPACE_3),
            theme,
        )
        .into()
    }

    fn view_frames_card(&self, theme: EstudyTheme) -> Element<'_, PreviewMessage> {
        let mut frame_row = row![].spacing(SPACE_2);
        for frame in BoardFrameStyle::ALL {
            frame_row = frame_row.push(pill_chip(
                frame.label(),
                None,
                None,
                frame == self.frame,
                PreviewMessage::SelectFrame(frame),
                theme,
            ));
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
        .window_size((1180.0, 860.0))
        .centered()
        .run()
}
