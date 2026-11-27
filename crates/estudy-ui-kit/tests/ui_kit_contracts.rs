//! Testes de contrato para presets WCAG 2.2 AA, catálogo de 87 SVGs e Heatmap 365d.

#![allow(clippy::expect_used)]

use estudy_ui_kit::widgets::clamp_progress;
use estudy_ui_kit::{
    BoardFrameStyle, CategoryColor, CosmeticLoadout, EstudyTheme, FONT_MONO, FONT_MONO_MEDIUM,
    FONT_SANS, FONT_SANS_BOLD, FONT_SANS_MEDIUM, FONT_SANS_SEMIBOLD, FONT_SERIF, FONT_SERIF_ITALIC,
    FONT_SERIF_MEDIUM, FONT_SIZE_2XL, FONT_SIZE_BASE, FONT_SIZE_DISPLAY, FONT_SIZE_LG,
    FONT_SIZE_MD, FONT_SIZE_SM, FONT_SIZE_XL, FONT_SIZE_XS, HeatmapData, Icon, IllustratedIcon,
    RADIUS_CARD, RADIUS_CHIP, RADIUS_CONTROL, RADIUS_DRAWER, RADIUS_PANEL, RADIUS_PILL,
    SIDEBAR_COMPACT_WIDTH, SIDEBAR_WIDTH, SPACE_1, SPACE_2, SPACE_3, SPACE_4, SPACE_5, SPACE_6,
    SPACE_8, SparklineData, ThemeMode, ThemePreset, contrast_ratio, embedded_fonts, meets_wcag_aa,
};
use iced::Color;
use iced::font::{Style, Weight};
use std::collections::HashSet;

fn composite_over(fg: Color, bg: Color, alpha: f32) -> Color {
    Color {
        r: fg.r * alpha + bg.r * (1.0 - alpha),
        g: fg.g * alpha + bg.g * (1.0 - alpha),
        b: fg.b * alpha + bg.b * (1.0 - alpha),
        a: 1.0,
    }
}

#[test]
fn should_pass_wcag_aa_contrast_across_all_10_theme_variants() {
    for preset in ThemePreset::ALL {
        for mode in ThemeMode::ALL {
            let theme = EstudyTheme::resolve(preset, mode);
            assert!(
                (0.08..=0.16).contains(&theme.border_subtle.a),
                "border_subtle fora da faixa hairline delicada (0.08..=0.16) em {preset:?}/{mode:?}: {:.2}",
                theme.border_subtle.a
            );
            assert!(
                theme.card_style().shadow.blur_radius > 0.0,
                "card_style deve ter elevação suave por sombra em {preset:?}/{mode:?}"
            );
            let accent_panel = contrast_ratio(theme.accent, theme.panel);
            assert!(
                accent_panel >= 4.5,
                "accent/panel abaixo de 4.5:1 em {preset:?}/{mode:?}: {accent_panel:.2}"
            );
            assert!(
                meets_wcag_aa(theme.on_accent, theme.accent),
                "on_accent/accent falhou WCAG AA em {preset:?}/{mode:?}"
            );
            for (surface_name, surface) in [
                ("bg", theme.bg),
                ("surface_soft", theme.surface_soft),
                ("panel", theme.panel),
                ("card", theme.card),
                ("card_hover", theme.card_hover),
            ] {
                let primary_ratio = contrast_ratio(theme.text_primary, surface);
                assert!(
                    primary_ratio >= 10.0,
                    "text_primary/{surface_name} abaixo de 10:1 em {preset:?}/{mode:?}: {primary_ratio:.2}"
                );
                let muted_ratio = contrast_ratio(theme.text_muted, surface);
                assert!(
                    muted_ratio >= 6.0,
                    "text_muted/{surface_name} abaixo de 6.0:1 em {preset:?}/{mode:?}: {muted_ratio:.2}"
                );
            }

            for (sem_name, sem_color) in [
                ("accent", theme.accent),
                ("success", theme.success),
                ("warning", theme.warning),
                ("danger", theme.danger),
            ] {
                for (bg_name, bg) in [
                    ("panel", theme.panel),
                    ("card", theme.card),
                    ("surface_soft", theme.surface_soft),
                ] {
                    assert!(
                        meets_wcag_aa(sem_color, bg),
                        "{sem_name}/{bg_name} falhou WCAG AA em {preset:?}/{mode:?}"
                    );
                    let badge_bg = composite_over(sem_color, bg, 0.13);
                    assert!(
                        meets_wcag_aa(sem_color, badge_bg),
                        "{sem_name}/badge({bg_name}) falhou WCAG AA em {preset:?}/{mode:?}"
                    );
                }
            }
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
        assert!(
            svg.contains("stroke-width=\"1.75\""),
            "ícone monoline {name} deve usar traço limpo 1.75px"
        );
        let _cached = icon.cached_handle();
    }
}

#[test]
fn should_provide_17_multicolor_illustrated_icons_with_distinct_fills() {
    assert_eq!(IllustratedIcon::ALL.len(), 17);
    let mut names = HashSet::new();
    for icon in IllustratedIcon::ALL {
        let name = icon.name();
        assert!(!name.is_empty() && name != "stub");
        assert!(
            names.insert(name),
            "nome de ícone ilustrado duplicado: {name}"
        );
        let svg = icon.to_svg_string();
        assert!(
            svg.starts_with("<svg ") && svg.ends_with("</svg>"),
            "SVG ilustrado inválido em {name}"
        );
        assert!(
            svg.matches("fill=\"#").count() >= 2,
            "ícone ilustrado {name} precisa de preenchimentos coloridos reais (encontrado < 2)"
        );
        let _handle = icon.cached_handle();
    }
}

#[test]
fn should_preserve_exact_oliva_organico_core_palette_and_clean_sidebar_metrics() {
    let light = EstudyTheme::resolve(ThemePreset::OlivaOrganico, ThemeMode::Light);
    assert_eq!(light.bg, Color::from_rgb8(0xf6, 0xf3, 0xec));
    assert_eq!(light.panel, Color::from_rgb8(0xe8, 0xe3, 0xd8));
    assert_eq!(light.card, Color::from_rgb8(0xff, 0xff, 0xff));
    assert_eq!(light.accent, Color::from_rgb8(0x37, 0x4c, 0x3b));

    let dark = EstudyTheme::resolve(ThemePreset::OlivaOrganico, ThemeMode::Dark);
    assert_eq!(dark.bg, Color::from_rgb8(0x10, 0x12, 0x0f));
    assert_eq!(dark.accent, Color::from_rgb8(0xb2, 0xc9, 0xa8));

    assert!((SIDEBAR_WIDTH - 216.0).abs() < f32::EPSILON);
    assert!((SIDEBAR_COMPACT_WIDTH - 58.0).abs() < f32::EPSILON);
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

#[test]
fn should_expose_soft_geometry_tonal_surfaces_and_clamped_progress() {
    const {
        assert!(SPACE_1 < SPACE_2 && SPACE_2 < SPACE_3 && SPACE_3 < SPACE_4);
        assert!(SPACE_4 < SPACE_5 && SPACE_5 < SPACE_6 && SPACE_6 < SPACE_8);
        assert!(RADIUS_CARD >= RADIUS_PANEL);
        assert!(RADIUS_PANEL >= RADIUS_CONTROL);
        assert!(RADIUS_CONTROL >= RADIUS_CHIP);
        assert!(RADIUS_PILL >= 900.0);
    }

    assert!((clamp_progress(-0.2) - 0.0).abs() < f32::EPSILON);
    assert!((clamp_progress(0.45) - 0.45).abs() < f32::EPSILON);
    assert!((clamp_progress(1.8) - 1.0).abs() < f32::EPSILON);

    for preset in ThemePreset::ALL {
        for mode in ThemeMode::ALL {
            let theme = EstudyTheme::resolve(preset, mode);
            let tonal = theme.tonal_card_style();
            assert!(tonal.background.is_some());
            assert!(meets_wcag_aa(theme.text_primary, theme.surface_soft));
            assert!(theme.pill_tray_style().background.is_some());
        }
    }
}

#[test]
fn should_embed_nine_static_truetype_fonts_with_true_italic_and_bold() {
    let fonts = embedded_fonts();
    assert_eq!(fonts.len(), 9, "esperadas 9 fontes estáticas OFL embutidas");
    for bytes in &fonts {
        assert!(bytes.len() > 50_000, "fonte embutida muito pequena");
        assert_eq!(&bytes[0..4], &[0x00, 0x01, 0x00, 0x00]);
        let num_tables = u16::from_be_bytes([bytes[4], bytes[5]]) as usize;
        for idx in 0..num_tables {
            let off = 12 + idx * 16;
            let tag = &bytes[off..off + 4];
            assert_ne!(
                tag, b"fvar",
                "proibido usar fonte variável (fvar) para evitar esmagamento de HVAR/Basic"
            );
        }
    }
    assert_ne!(FONT_SANS, FONT_SERIF);
    assert_ne!(FONT_SANS, FONT_MONO);
    assert_ne!(FONT_SERIF, FONT_MONO);
    assert_eq!(FONT_SERIF_ITALIC.style, Style::Italic);
    assert_eq!(FONT_SERIF_MEDIUM.weight, Weight::Medium);
    assert_eq!(FONT_SANS_MEDIUM.weight, Weight::Medium);
    assert_eq!(FONT_SANS_SEMIBOLD.weight, Weight::Semibold);
    assert_eq!(FONT_SANS_BOLD.weight, Weight::Bold);
    assert_eq!(FONT_MONO_MEDIUM.weight, Weight::Medium);
}

#[test]
fn should_expose_readable_typography_scale_with_12px_minimum() {
    const {
        assert!(FONT_SIZE_XS >= 12.0);
        assert!(FONT_SIZE_XS < FONT_SIZE_SM);
        assert!(FONT_SIZE_SM < FONT_SIZE_BASE);
        assert!(FONT_SIZE_BASE >= 14.0);
        assert!(FONT_SIZE_BASE < FONT_SIZE_MD);
        assert!(FONT_SIZE_MD < FONT_SIZE_LG);
        assert!(FONT_SIZE_LG < FONT_SIZE_XL);
        assert!(FONT_SIZE_XL < FONT_SIZE_2XL);
        assert!(FONT_SIZE_2XL < FONT_SIZE_DISPLAY);
    }
}

#[test]
fn should_resolve_seven_category_colors_and_drawer_sidebar_metrics() {
    assert_eq!(CategoryColor::ALL.len(), 7);
    const {
        assert!(RADIUS_DRAWER > RADIUS_CARD);
        assert!(SIDEBAR_WIDTH >= 200.0 && SIDEBAR_WIDTH <= 260.0);
    }

    for preset in ThemePreset::ALL {
        for mode in ThemeMode::ALL {
            let theme = EstudyTheme::resolve(preset, mode);
            assert!(theme.sidebar_style().background.is_some());
            assert!(theme.floating_drawer_style().background.is_some());
            for (idx, cat) in CategoryColor::ALL.iter().enumerate() {
                assert_eq!(CategoryColor::from_index(idx), *cat);
                let solid = cat.solid(theme);
                let tint = cat.tint(theme);
                assert!(solid.a > 0.99);
                assert!(tint.a > 0.05 && tint.a < 0.30);
                for (bg_name, bg) in [
                    ("card", theme.card),
                    ("surface_soft", theme.surface_soft),
                    ("panel", theme.panel),
                    ("card_hover", theme.card_hover),
                ] {
                    assert!(
                        meets_wcag_aa(solid, bg),
                        "categoria {cat:?} falhou WCAG AA sobre {bg_name} em {preset:?}/{mode:?}"
                    );
                }
                let pill_bg = composite_over(solid, theme.surface_soft, 0.13);
                assert!(
                    meets_wcag_aa(solid, pill_bg),
                    "categoria {cat:?} falhou WCAG AA em badge sobre surface_soft em {preset:?}/{mode:?}"
                );
            }
        }
    }
}

#[test]
fn should_normalize_sparkline_series_safely() {
    assert!(SparklineData::new(vec![]).normalized_points().is_empty());

    let single = SparklineData::new(vec![42.0]).normalized_points();
    assert_eq!(single.len(), 2);
    assert!((single[0].1 - 0.5).abs() < f32::EPSILON);

    let flat = SparklineData::new(vec![10.0, 10.0, 10.0]).normalized_points();
    assert_eq!(flat.len(), 3);
    assert!(flat.iter().all(|&(_, y)| (y - 0.5).abs() < f32::EPSILON));

    let ramp = SparklineData::new(vec![0.0, 50.0, 100.0]).normalized_points();
    assert_eq!(ramp.len(), 3);
    assert!((ramp[0].1 - 1.0).abs() < f32::EPSILON);
    assert!((ramp[1].1 - 0.5).abs() < f32::EPSILON);
    assert!((ramp[2].1 - 0.0).abs() < f32::EPSILON);
}
