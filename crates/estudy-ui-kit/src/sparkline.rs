//! Gráfico de linha e área suave (*sparkline*) desenhado em `Canvas` leve com cache.

use iced::mouse::Cursor;
use iced::widget::canvas::{self, Canvas, Geometry, Path, Stroke};
use iced::{Color, Point, Rectangle, Renderer, Theme};
use std::cell::Cell;

/// Série numérica imutável para renderização em [`SparklineCanvas`].
#[derive(Debug, Clone, PartialEq)]
pub struct SparklineData {
    /// Valores ordenados cronologicamente.
    pub values: Vec<f32>,
}

impl SparklineData {
    /// Cria uma nova série numérica.
    #[must_use]
    pub const fn new(values: Vec<f32>) -> Self {
        Self { values }
    }

    /// Normaliza os pontos para o espaço `(x, y)` no intervalo `0.0..=1.0`,
    /// onde `y = 0.0` representa o topo visual e `y = 1.0` a base.
    #[must_use]
    pub fn normalized_points(&self) -> Vec<(f32, f32)> {
        if self.values.is_empty() {
            return Vec::new();
        }
        if self.values.len() == 1 {
            return vec![(0.0, 0.5), (1.0, 0.5)];
        }

        let mut min_val = f32::INFINITY;
        let mut max_val = f32::NEG_INFINITY;
        for &val in &self.values {
            if val.is_finite() {
                min_val = min_val.min(val);
                max_val = max_val.max(val);
            }
        }

        if !min_val.is_finite() || !max_val.is_finite() {
            return Vec::new();
        }

        let span = (max_val - min_val).max(f32::EPSILON);
        #[allow(clippy::cast_precision_loss)]
        let denom = (self.values.len() - 1) as f32;

        self.values
            .iter()
            .enumerate()
            .map(|(idx, &val)| {
                #[allow(clippy::cast_precision_loss)]
                let x = (idx as f32) / denom;
                let y = if (max_val - min_val).abs() <= f32::EPSILON {
                    0.5
                } else {
                    1.0 - ((val - min_val) / span).clamp(0.0, 1.0)
                };
                (x, y)
            })
            .collect()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct SparklineRenderKey {
    values_hash: u64,
    stroke_rgba: u32,
    width: u32,
    height: u32,
}

/// Estado interno de cache de geometria do [`SparklineCanvas`].
#[derive(Default)]
pub struct SparklineCanvasState {
    cache: canvas::Cache,
    last_signature: Cell<Option<SparklineRenderKey>>,
}

/// Widget `Canvas` de linha + área translúcida com ponto final em destaque e cache de geometria.
#[derive(Debug, Clone)]
pub struct SparklineCanvas {
    /// Dados da série temporal.
    pub data: SparklineData,
    /// Cor da linha principal e do ponto final.
    pub stroke_color: Color,
    /// Cor de preenchimento translúcido sob a curva.
    pub fill_color: Color,
    /// Cor sutil das guias horizontais de fundo.
    pub grid_color: Color,
}

impl SparklineCanvas {
    /// Constrói um `Canvas` configurado pronto para entrar na árvore de widgets.
    #[must_use]
    pub fn into_canvas<Message>(self) -> Canvas<Self, Message> {
        Canvas::new(self)
    }
}

impl<Message> canvas::Program<Message> for SparklineCanvas {
    type State = SparklineCanvasState;

    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    fn draw(
        &self,
        state: &Self::State,
        renderer: &Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: Cursor,
    ) -> Vec<Geometry> {
        let signature = SparklineRenderKey {
            values_hash: hash_values(&self.data.values),
            stroke_rgba: pack_color(self.stroke_color),
            width: bounds.width.round().max(0.0) as u32,
            height: bounds.height.round().max(0.0) as u32,
        };
        if state.last_signature.get() != Some(signature) {
            state.cache.clear();
            state.last_signature.set(Some(signature));
        }

        let geometry = state.cache.draw(renderer, bounds.size(), |frame| {
            let width = bounds.width.max(1.0);
            let height = bounds.height.max(1.0);
            let pad_x = 4.0_f32;
            let pad_y = 6.0_f32;
            let plot_w = (width - pad_x * 2.0).max(1.0);
            let plot_h = (height - pad_y * 2.0).max(1.0);

            // Guias horizontais agrupadas em um único path.
            let guides = Path::new(|builder| {
                for step in [0.25_f32, 0.5, 0.75] {
                    let y = pad_y + plot_h * step;
                    builder.move_to(Point::new(pad_x, y));
                    builder.line_to(Point::new(pad_x + plot_w, y));
                }
            });
            frame.stroke(
                &guides,
                Stroke::default()
                    .with_color(self.grid_color)
                    .with_width(1.0),
            );

            let normalized = self.data.normalized_points();
            if normalized.len() < 2 {
                return;
            }

            let points: Vec<Point> = normalized
                .iter()
                .map(|&(nx, ny)| Point::new(pad_x + nx * plot_w, pad_y + ny * plot_h))
                .collect();

            let base_y = pad_y + plot_h;
            let area = Path::new(|builder| {
                builder.move_to(Point::new(points[0].x, base_y));
                for &pt in &points {
                    builder.line_to(pt);
                }
                if let Some(last) = points.last() {
                    builder.line_to(Point::new(last.x, base_y));
                }
                builder.close();
            });
            frame.fill(&area, self.fill_color);

            let line = Path::new(|builder| {
                builder.move_to(points[0]);
                for &pt in &points[1..] {
                    builder.line_to(pt);
                }
            });
            frame.stroke(
                &line,
                Stroke::default()
                    .with_color(self.stroke_color)
                    .with_width(2.0),
            );

            if let Some(&last) = points.last() {
                frame.fill(
                    &Path::circle(last, 5.0),
                    Color {
                        a: 0.25,
                        ..self.stroke_color
                    },
                );
                frame.fill(&Path::circle(last, 2.8), self.stroke_color);
            }
        });

        vec![geometry]
    }
}

fn hash_values(values: &[f32]) -> u64 {
    let mut hash = 0xcbf2_9ce4_8422_2325_u64;
    for &v in values {
        hash ^= u64::from(v.to_bits());
        hash = hash.wrapping_mul(0x0100_0000_01b3);
    }
    hash
}

#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
fn pack_color(color: Color) -> u32 {
    let red = (color.r.clamp(0.0, 1.0) * 255.0) as u32;
    let green = (color.g.clamp(0.0, 1.0) * 255.0) as u32;
    let blue = (color.b.clamp(0.0, 1.0) * 255.0) as u32;
    let alpha = (color.a.clamp(0.0, 1.0) * 255.0) as u32;
    (red << 24) | (green << 16) | (blue << 8) | alpha
}
