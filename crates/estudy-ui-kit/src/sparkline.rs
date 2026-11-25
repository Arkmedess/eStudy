//! Gráfico de linha e área suave (*sparkline*) desenhado em `Canvas` leve.

use iced::mouse::Cursor;
use iced::widget::canvas::{self, Canvas, Frame, Geometry, Path, Stroke};
use iced::{Color, Point, Rectangle, Renderer, Theme};

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

/// Widget `Canvas` de linha + área translúcida com ponto final em destaque.
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
    type State = ();

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: Cursor,
    ) -> Vec<Geometry> {
        let mut frame = Frame::new(renderer, bounds.size());
        let width = bounds.width.max(1.0);
        let height = bounds.height.max(1.0);
        let pad_x = 4.0_f32;
        let pad_y = 6.0_f32;
        let plot_w = (width - pad_x * 2.0).max(1.0);
        let plot_h = (height - pad_y * 2.0).max(1.0);

        // Guias horizontais discretas (3 linhas).
        for step in [0.25_f32, 0.5, 0.75] {
            let y = pad_y + plot_h * step;
            let guide = Path::line(Point::new(pad_x, y), Point::new(pad_x + plot_w, y));
            frame.stroke(
                &guide,
                Stroke::default()
                    .with_color(self.grid_color)
                    .with_width(1.0),
            );
        }

        let normalized = self.data.normalized_points();
        if normalized.len() < 2 {
            return vec![frame.into_geometry()];
        }

        let points: Vec<Point> = normalized
            .iter()
            .map(|&(nx, ny)| Point::new(pad_x + nx * plot_w, pad_y + ny * plot_h))
            .collect();

        // Área preenchida sob a linha.
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

        // Linha principal.
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

        // Ponto final destacado.
        if let Some(&last) = points.last() {
            let halo = Path::circle(last, 5.0);
            frame.fill(
                &halo,
                Color {
                    a: 0.25,
                    ..self.stroke_color
                },
            );
            let dot = Path::circle(last, 2.8);
            frame.fill(&dot, self.stroke_color);
        }

        vec![frame.into_geometry()]
    }
}
