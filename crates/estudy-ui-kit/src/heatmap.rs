//! Heatmap anual de constância (365 dias) renderizado em `Canvas` vetorial com cache.

use iced::widget::canvas;
use iced::{Color, Point, Rectangle, Renderer, Size, Theme, mouse};
use std::cell::Cell;

use crate::theme::{EstudyTheme, ThemeMode, ThemePreset};

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct HeatmapRenderKey {
    days_hash: u64,
    preset: ThemePreset,
    mode: ThemeMode,
    width: u32,
    height: u32,
}

/// Estado interno de cache de geometria do `HeatmapGrid`.
#[derive(Default)]
pub struct HeatmapCanvasState {
    cache: canvas::Cache,
    last_signature: Cell<Option<HeatmapRenderKey>>,
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
    type State = HeatmapCanvasState;

    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    fn draw(
        &self,
        state: &Self::State,
        renderer: &Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<canvas::Geometry> {
        let signature = HeatmapRenderKey {
            days_hash: hash_days(&self.data.days),
            preset: self.theme.preset,
            mode: self.theme.mode,
            width: bounds.width.round().max(0.0) as u32,
            height: bounds.height.round().max(0.0) as u32,
        };
        if state.last_signature.get() != Some(signature) {
            state.cache.clear();
            state.last_signature.set(Some(signature));
        }

        let geometry = state.cache.draw(renderer, bounds.size(), |frame| {
            let cols = 53_usize;
            let rows = 7_usize;
            let gap = 3.0_f32;

            #[allow(clippy::cast_precision_loss)]
            let cell_w = ((bounds.width - gap * (cols as f32 - 1.0)) / cols as f32).max(4.0);
            #[allow(clippy::cast_precision_loss)]
            let cell_h = ((bounds.height - gap * (rows as f32 - 1.0)) / rows as f32).max(4.0);
            let cell_size = cell_w.min(cell_h).min(16.0);

            #[allow(clippy::cast_precision_loss)]
            let total_w = cols as f32 * cell_size + (cols as f32 - 1.0) * gap;
            #[allow(clippy::cast_precision_loss)]
            let total_h = rows as f32 * cell_size + (rows as f32 - 1.0) * gap;
            let origin_x = ((bounds.width - total_w) * 0.5).max(0.0);
            let origin_y = ((bounds.height - total_h) * 0.5).max(0.0);

            // Agrupa as 365 células em apenas 5 paths retangulares (1 por nível de cor),
            // reduzindo drasticamente o custo de rasterização em CPU (tiny-skia) durante scroll.
            for level in 0_u8..=4 {
                let batch = canvas::Path::new(|builder| {
                    for (day_idx, &count) in self.data.days.iter().enumerate() {
                        if HeatmapData::intensity_level(count) != level {
                            continue;
                        }
                        let col = day_idx / rows;
                        let row = day_idx % rows;
                        #[allow(clippy::cast_precision_loss)]
                        let x = origin_x + col as f32 * (cell_size + gap);
                        #[allow(clippy::cast_precision_loss)]
                        let y = origin_y + row as f32 * (cell_size + gap);
                        builder.rectangle(Point::new(x, y), Size::new(cell_size, cell_size));
                    }
                });
                frame.fill(&batch, level_color(self.theme, level));
            }
        });

        vec![geometry]
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

fn hash_days(days: &[u16; 365]) -> u64 {
    let mut hash = 0xcbf2_9ce4_8422_2325_u64;
    for &count in days {
        hash ^= u64::from(count);
        hash = hash.wrapping_mul(0x0100_0000_01b3);
    }
    hash
}
