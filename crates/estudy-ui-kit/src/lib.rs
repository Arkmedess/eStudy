//! Design System oficial do **eStudy** em `Iced`.

pub mod fonts;
pub mod heatmap;
pub mod icons;
pub mod sparkline;
pub mod theme;
pub mod widgets;

pub use fonts::{
    FONT_MONO, FONT_MONO_MEDIUM, FONT_SANS, FONT_SANS_MEDIUM, FONT_SANS_SEMIBOLD, FONT_SERIF,
    FONT_SERIF_MEDIUM, embedded_fonts,
};
pub use heatmap::{HeatmapData, HeatmapGrid};
pub use icons::Icon;
pub use sparkline::{SparklineCanvas, SparklineData};
pub use theme::{
    BoardFrameStyle, CategoryColor, CosmeticLoadout, EstudyTheme, RADIUS_CARD, RADIUS_CHIP,
    RADIUS_CONTROL, RADIUS_DRAWER, RADIUS_PANEL, RADIUS_PILL, SIDEBAR_WIDTH, SPACE_1, SPACE_2,
    SPACE_3, SPACE_4, SPACE_5, SPACE_6, SPACE_8, TOPBAR_HEIGHT, ThemeMode, ThemePreset,
    contrast_ratio, meets_wcag_aa,
};
