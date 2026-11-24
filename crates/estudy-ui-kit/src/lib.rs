//! Design System oficial do **eStudy** em `Iced`.

pub mod heatmap;
pub mod icons;
pub mod theme;
pub mod widgets;

pub use heatmap::{HeatmapData, HeatmapGrid};
pub use icons::Icon;
pub use theme::{
    BoardFrameStyle, CosmeticLoadout, EstudyTheme, RADIUS_CARD, RADIUS_CHIP, RADIUS_CONTROL,
    RADIUS_PANEL, RADIUS_PILL, SPACE_1, SPACE_2, SPACE_3, SPACE_4, SPACE_5, SPACE_6, SPACE_8,
    TOPBAR_HEIGHT, ThemeMode, ThemePreset, contrast_ratio, meets_wcag_aa,
};
