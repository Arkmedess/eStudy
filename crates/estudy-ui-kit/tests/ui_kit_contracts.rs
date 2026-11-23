//! Testes de contrato para presets WCAG 2.2 AA, catálogo de 87 SVGs e Heatmap 365d.

#![allow(clippy::expect_used)]

use estudy_ui_kit::{
    BoardFrameStyle, CosmeticLoadout, EstudyTheme, HeatmapData, Icon, ThemeMode, ThemePreset,
    contrast_ratio, meets_wcag_aa,
};
use iced::Color;
use std::collections::HashSet;

#[test]
fn should_pass_wcag_aa_contrast_across_all_10_theme_variants() {
    for preset in ThemePreset::ALL {
        for mode in ThemeMode::ALL {
            let theme = EstudyTheme::resolve(preset, mode);
            let accent_panel = contrast_ratio(theme.accent, theme.panel);
            assert!(
                accent_panel >= 4.5,
                "accent/panel abaixo de 4.5:1 em {preset:?}/{mode:?}: {accent_panel:.2}"
            );
            assert!(
                meets_wcag_aa(theme.on_accent, theme.accent),
                "on_accent/accent falhou WCAG AA em {preset:?}/{mode:?}"
            );
            assert!(
                meets_wcag_aa(theme.text_primary, theme.bg),
                "text_primary/bg falhou WCAG AA em {preset:?}/{mode:?}"
            );
            assert!(
                meets_wcag_aa(theme.text_primary, theme.card),
                "text_primary/card falhou WCAG AA em {preset:?}/{mode:?}"
            );
            assert!(
                meets_wcag_aa(theme.text_muted, theme.panel),
                "text_muted/panel falhou WCAG AA em {preset:?}/{mode:?}"
            );
        }
    }
}

#[test]
fn should_provide_87_unique_valid_svg_icons_without_emojis() {
    assert_eq!(Icon::ALL.len(), 87);

    let mut names = HashSet::new();
    for icon in Icon::ALL {
        let name = icon.name();
        assert!(!name.is_empty());
        assert!(names.insert(name), "nome de ícone duplicado: {name}");

        let svg = icon.to_svg_string(Color::WHITE);
        assert!(
            svg.starts_with("<svg ") && svg.ends_with("</svg>"),
            "SVG inválido no ícone {name}: {svg}"
        );
        assert!(
            svg.contains("viewBox=\"0 0 24 24\""),
            "viewBox ausente no ícone {name}"
        );
    }
}

#[test]
fn should_expose_all_board_frames_and_default_cosmetic_loadout() {
    assert_eq!(BoardFrameStyle::ALL.len(), 7);
    let loadout = CosmeticLoadout::default();
    assert_eq!(loadout.theme_preset, ThemePreset::OlivaOrganico);
    assert_eq!(loadout.theme_mode, ThemeMode::Light);
}

#[test]
fn should_map_heatmap_counts_to_five_intensity_levels() {
    assert_eq!(HeatmapData::intensity_level(0), 0);
    assert_eq!(HeatmapData::intensity_level(1), 1);
    assert_eq!(HeatmapData::intensity_level(3), 2);
    assert_eq!(HeatmapData::intensity_level(6), 3);
    assert_eq!(HeatmapData::intensity_level(12), 4);
}
