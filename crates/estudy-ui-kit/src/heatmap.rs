//! Heatmap anual de constância (365 dias) renderizado em `Canvas` vetorial.

use iced::widget::canvas;
use iced::{Color, Point, Rectangle, Renderer, Size, Theme, mouse};

use crate::theme::EstudyTheme;

/// Dados de atividade dos últimos 365 dias usados pelo heatmap.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HeatmapData {
    /// Contagem diária de ações válidas (índice `0` = mais antigo, `364` = hoje).
    pub days: [u16; 365],
}

impl Default for HeatmapData {
    fn default() -> Self {
        Self { days: [0; 365] }
    }
}

impl HeatmapData {
    /// Converte uma contagem diária em nível de intensidade (`0..=4`).
    #[must_use]
    pub const fn intensity_level(count: u16) -> u8 {
        match count {
            0 => 0,
            1..=2 => 1,
            3..=5 => 2,
            6..=9 => 3,
            _ => 4,
        }
    }

    /// Soma total de ações registradas nos 365 dias.
    #[must_use]
    pub fn total_actions(&self) -> u32 {
        self.days.iter().map(|&count| u32::from(count)).sum()
    }
}

/// Programa `Canvas` que desenha a grade anual de constância em 53 semanas × 7 dias.
#[derive(Debug, Clone)]
pub struct HeatmapGrid<'a> {
    /// Dados dos 365 dias.
    pub data: &'a HeatmapData,
    /// Tema ativo para cálculo dos tons da escala.
    pub theme: EstudyTheme,
}

impl<Message> canvas::Program<Message> for HeatmapGrid<'_> {
    type State = ();

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<canvas::Geometry> {
        let mut frame = canvas::Frame::new(renderer, bounds.size());
        let cols = 53_usize;
        let rows = 7_usize;
        let gap = 3.0_f32;

        #[allow(clippy::cast_precision_loss)]
        let cell_w = ((bounds.width - gap * (cols as f32 - 1.0)) / cols as f32).max(4.0);
        #[allow(clippy::cast_precision_loss)]
        let cell_h = ((bounds.height - gap * (rows as f32 - 1.0)) / rows as f32).max(4.0);
        let cell_size = cell_w.min(cell_h).min(12.0);

        for (day_idx, &count) in self.data.days.iter().enumerate() {
            let col = day_idx / rows;
            let row = day_idx % rows;
            #[allow(clippy::cast_precision_loss)]
            let x = col as f32 * (cell_size + gap);
            #[allow(clippy::cast_precision_loss)]
            let y = row as f32 * (cell_size + gap);

            let color = level_color(self.theme, HeatmapData::intensity_level(count));
            frame.fill_rectangle(Point::new(x, y), Size::new(cell_size, cell_size), color);
        }

        vec![frame.into_geometry()]
    }
}

fn level_color(theme: EstudyTheme, level: u8) -> Color {
    match level {
        0 => theme.border_subtle,
        1 => Color {
            a: 0.30,
            ..theme.accent
        },
        2 => Color {
            a: 0.52,
            ..theme.accent
        },
        3 => Color {
            a: 0.76,
            ..theme.accent
        },
        _ => theme.accent,
    }
}
