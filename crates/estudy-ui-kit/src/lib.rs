//! Design System oficial do **eStudy** em `Iced`.

pub mod fonts;
pub mod heatmap;
pub mod icons;
pub mod sparkline;
pub mod theme;
pub mod widgets;

pub use fonts::{
    FONT_MONO, FONT_MONO_MEDIUM, FONT_SANS, FONT_SANS_BOLD, FONT_SANS_MEDIUM, FONT_SANS_SEMIBOLD,
    FONT_SERIF, FONT_SERIF_ITALIC, FONT_SERIF_MEDIUM, FONT_SIZE_2XL, FONT_SIZE_BASE,
    FONT_SIZE_DISPLAY, FONT_SIZE_HERO_TIMER, FONT_SIZE_LG, FONT_SIZE_MD, FONT_SIZE_SM,
    FONT_SIZE_TIMER, FONT_SIZE_XL, FONT_SIZE_XS, embedded_fonts,
};
pub use heatmap::{HeatmapData, HeatmapGrid};
pub use icons::{Icon, IllustratedIcon};
pub use sparkline::{SparklineCanvas, SparklineData};
pub use theme::{
    BoardFrameStyle, CategoryColor, CosmeticLoadout, EstudyTheme, RADIUS_CARD, RADIUS_CHIP,
    RADIUS_CONTROL, RADIUS_DRAWER, RADIUS_PANEL, RADIUS_PILL, SIDEBAR_COMPACT_WIDTH, SIDEBAR_WIDTH,
    SPACE_1, SPACE_2, SPACE_3, SPACE_4, SPACE_5, SPACE_6, SPACE_8, TOPBAR_HEIGHT, ThemeMode,
    ThemePreset, contrast_ratio, meets_wcag_aa,
};
