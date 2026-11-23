//! Tokens visuais, os 5 presets WCAG 2.2 AA e slots cosméticos do eStudy.

use iced::widget::{button, container};
use iced::{Border, Color, Shadow, Vector};

/// Unidade base de espaçamento (`4px`).
pub const SPACE_1: f32 = 4.0;
/// Espaçamento compacto (`8px`).
pub const SPACE_2: f32 = 8.0;
/// Espaçamento padrão entre controles (`12px`).
pub const SPACE_3: f32 = 12.0;
/// Padding interno de painéis/cartões (`16px`).
pub const SPACE_4: f32 = 16.0;
/// Margem entre colunas principais (`24px`).
pub const SPACE_6: f32 = 24.0;

/// Altura fixa da barra superior discreta (`48px`).
pub const TOPBAR_HEIGHT: f32 = 48.0;

/// Raio externo de cartões e painéis (`12px`).
pub const RADIUS_PANEL: f32 = 12.0;
/// Raio de controles internos (`8px`, mantendo concentricidade).
pub const RADIUS_CONTROL: f32 = 8.0;
/// Raio de badges e chips (`6px`).
pub const RADIUS_CHIP: f32 = 6.0;

/// Presets oficiais de tema do eStudy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ThemePreset {
    /// Oliva Orgânico (padrão acolhedor).
    OlivaOrganico,
    /// eStudy Menta.
    EstudyMenta,
    /// Cyber Minimalist.
    CyberMinimalist,
    /// Solar Amber.
    SolarAmber,
    /// Velvet Sakura.
    VelvetSakura,
}

impl ThemePreset {
    /// Todos os 5 presets oficiais.
    pub const ALL: [Self; 5] = [
        Self::OlivaOrganico,
        Self::EstudyMenta,
        Self::CyberMinimalist,
        Self::SolarAmber,
        Self::VelvetSakura,
    ];

    /// Nome legível exibido na interface.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::OlivaOrganico => "Oliva Orgânico",
            Self::EstudyMenta => "eStudy Menta",
            Self::CyberMinimalist => "Cyber Minimalist",
            Self::SolarAmber => "Solar Amber",
            Self::VelvetSakura => "Velvet Sakura",
        }
    }

    /// Avança ciclicamente para o próximo preset oficial.
    #[must_use]
    pub fn next(self) -> Self {
        let idx = Self::ALL
            .iter()
            .position(|preset| *preset == self)
            .unwrap_or(0);
        Self::ALL[(idx + 1) % Self::ALL.len()]
    }
}

/// Modo de polaridade visual do tema.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ThemeMode {
    /// Modo claro.
    Light,
    /// Modo escuro.
    Dark,
}

impl ThemeMode {
    /// Todos os modos suportados.
    pub const ALL: [Self; 2] = [Self::Light, Self::Dark];

    /// Alterna entre claro e escuro.
    #[must_use]
    pub const fn toggle(self) -> Self {
        match self {
            Self::Light => Self::Dark,
            Self::Dark => Self::Light,
        }
    }

    /// Nome curto do modo.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Light => "Claro",
            Self::Dark => "Escuro",
        }
    }
}

/// Estilos de moldura disponíveis para o Quadro Personalizável da Home.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BoardFrameStyle {
    /// Borda inferior larga estilo foto instantânea.
    Polaroid,
    /// Moldura limpa com canto arredondado suave.
    Photocard,
    /// Borda fina uniforme.
    Clean,
    /// Contraste editorial moderno.
    Moderna,
    /// Madeira suave e respiro orgânico.
    Japandi,
    /// Passe-partout clássico de galeria.
    Museu,
    /// Sem moldura visível.
    Nenhum,
}

impl BoardFrameStyle {
    /// Lista completa de molduras suportadas na v0.1.0.
    pub const ALL: [Self; 7] = [
        Self::Polaroid,
        Self::Photocard,
        Self::Clean,
        Self::Moderna,
        Self::Japandi,
        Self::Museu,
        Self::Nenhum,
    ];

    /// Nome legível da moldura.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Polaroid => "Polaroid",
            Self::Photocard => "Photocard",
            Self::Clean => "Clean",
            Self::Moderna => "Moderna",
            Self::Japandi => "Japandi",
            Self::Museu => "Museu",
            Self::Nenhum => "Nenhum",
        }
    }
}

/// Slots cosméticos modulares combináveis independentemente do tema ativo.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CosmeticLoadout {
    /// Preset de tema selecionado.
    pub theme_preset: ThemePreset,
    /// Modo claro ou escuro.
    pub theme_mode: ThemeMode,
    /// Moldura do quadro na coluna direita da Home.
    pub board_frame: BoardFrameStyle,
    /// Chave do pet ativo.
    pub active_pet: String,
    /// Chave do acessório equipado no pet.
    pub pet_accessory: Option<String>,
    /// Chave do solo/estação do jardim.
    pub garden_ground: String,
    /// Faixa de áudio DSP padrão.
    pub dsp_track: String,
}

impl Default for CosmeticLoadout {
    fn default() -> Self {
        Self {
            theme_preset: ThemePreset::OlivaOrganico,
            theme_mode: ThemeMode::Light,
            board_frame: BoardFrameStyle::Japandi,
            active_pet: "calico_cat".to_string(),
            pet_accessory: None,
            garden_ground: "meadow_spring".to_string(),
            dsp_track: "brown_rain".to_string(),
        }
    }
}

/// Paleta semântica resolvida para renderização em `Iced`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EstudyTheme {
    /// Preset de origem.
    pub preset: ThemePreset,
    /// Modo claro ou escuro.
    pub mode: ThemeMode,
    /// Fundo base da janela.
    pub bg: Color,
    /// Superfície de painéis e barra lateral.
    pub panel: Color,
    /// Superfície de cartões elevados.
    pub card: Color,
    /// Superfície de destaque suave em hover.
    pub card_hover: Color,
    /// Borda hairline sutil de 1px.
    pub border_subtle: Color,
    /// Texto principal.
    pub text_primary: Color,
    /// Texto secundário / metadados.
    pub text_muted: Color,
    /// Cor de destaque principal.
    pub accent: Color,
    /// Texto sobre fundo `accent`.
    pub on_accent: Color,
    /// Fundo tonal sutil derivado do `accent`.
    pub accent_subtle: Color,
    /// Cor semântica de sucesso.
    pub success: Color,
    /// Cor semântica de alerta.
    pub warning: Color,
    /// Cor semântica de erro/urgência.
    pub danger: Color,
}

const fn rgb(hex: u32) -> Color {
    #[allow(clippy::cast_possible_truncation)]
    let r = ((hex >> 16) & 0xff) as u8;
    #[allow(clippy::cast_possible_truncation)]
    let g = ((hex >> 8) & 0xff) as u8;
    #[allow(clippy::cast_possible_truncation)]
    let b = (hex & 0xff) as u8;
    Color::from_rgb8(r, g, b)
}

const fn rgba(hex: u32, alpha: f32) -> Color {
    let base = rgb(hex);
    Color { a: alpha, ..base }
}

impl EstudyTheme {
    /// Constrói o tema correspondente ao par `(preset, mode)`.
    #[must_use]
    pub const fn resolve(preset: ThemePreset, mode: ThemeMode) -> Self {
        match preset {
            ThemePreset::OlivaOrganico => Self::oliva_organico(mode),
            ThemePreset::EstudyMenta => Self::estudy_menta(mode),
            ThemePreset::CyberMinimalist => Self::cyber_minimalist(mode),
            ThemePreset::SolarAmber => Self::solar_amber(mode),
            ThemePreset::VelvetSakura => Self::velvet_sakura(mode),
        }
    }

    const fn oliva_organico(mode: ThemeMode) -> Self {
        match mode {
            ThemeMode::Light => Self::make(
                ThemePreset::OlivaOrganico,
                mode,
                rgb(0xf7_f5_f0),
                rgb(0xef_ec_e4),
                rgb(0xff_ff_ff),
                rgb(0xf4_f1_ea),
                rgba(0x1f_26_20, 0.12),
                rgb(0x1c_22_1d),
                rgb(0x4e_58_4f),
                rgb(0x4a_5d_4e),
                rgb(0xff_ff_ff),
                rgba(0x4a_5d_4e, 0.12),
            ),
            ThemeMode::Dark => Self::make(
                ThemePreset::OlivaOrganico,
                mode,
                rgb(0x12_14_11),
                rgb(0x1a_1d_18),
                rgb(0x21_25_1f),
                rgb(0x28_2d_25),
                rgba(0xff_ff_ff, 0.10),
                rgb(0xec_f0_ea),
                rgb(0xab_b5_a8),
                rgb(0xa3_b8_99),
                rgb(0x12_14_11),
                rgba(0xa3_b8_99, 0.16),
            ),
        }
    }

    const fn estudy_menta(mode: ThemeMode) -> Self {
        match mode {
            ThemeMode::Dark => Self::make(
                ThemePreset::EstudyMenta,
                mode,
                rgb(0x07_07_09),
                rgb(0x0f_10_15),
                rgb(0x13_14_1a),
                rgb(0x1a_1c_24),
                rgba(0xff_ff_ff, 0.10),
                rgb(0xf1_f5_f9),
                rgb(0x9c_a8_b8),
                rgb(0x86_ef_ac),
                rgb(0x07_07_09),
                rgba(0x86_ef_ac, 0.15),
            ),
            ThemeMode::Light => Self::make(
                ThemePreset::EstudyMenta,
                mode,
                rgb(0xf8_fa_fc),
                rgb(0xf1_f5_f9),
                rgb(0xff_ff_ff),
                rgb(0xf4_f8_fb),
                rgba(0x0f_17_2a, 0.11),
                rgb(0x0f_17_2a),
                rgb(0x47_55_69),
                rgb(0x15_80_3d),
                rgb(0xff_ff_ff),
                rgba(0x15_80_3d, 0.11),
            ),
        }
    }

    const fn cyber_minimalist(mode: ThemeMode) -> Self {
        match mode {
            ThemeMode::Dark => Self::make(
                ThemePreset::CyberMinimalist,
                mode,
                rgb(0x04_07_11),
                rgb(0x0a_10_20),
                rgb(0x0f_17_2a),
                rgb(0x16_20_38),
                rgba(0xe2_e8_f0, 0.11),
                rgb(0xf1_f5_f9),
                rgb(0x94_a3_b8),
                rgb(0x38_bd_f8),
                rgb(0x04_07_11),
                rgba(0x38_bd_f8, 0.16),
            ),
            ThemeMode::Light => Self::make(
                ThemePreset::CyberMinimalist,
                mode,
                rgb(0xf0_f7_ff),
                rgb(0xe0_f0_fe),
                rgb(0xff_ff_ff),
                rgb(0xf2_f8_ff),
                rgba(0x0c_4a_6e, 0.13),
                rgb(0x0c_1a_2e),
                rgb(0x33_4e_68),
                rgb(0x03_69_a1),
                rgb(0xff_ff_ff),
                rgba(0x03_69_a1, 0.12),
            ),
        }
    }

    const fn solar_amber(mode: ThemeMode) -> Self {
        match mode {
            ThemeMode::Dark => Self::make(
                ThemePreset::SolarAmber,
                mode,
                rgb(0x0e_0b_08),
                rgb(0x17_13_0e),
                rgb(0x1f_1a_13),
                rgb(0x28_21_18),
                rgba(0xfe_f3_c7, 0.11),
                rgb(0xfc_f6_ec),
                rgb(0xc2_b2_9b),
                rgb(0xfb_bf_24),
                rgb(0x0e_0b_08),
                rgba(0xfb_bf_24, 0.16),
            ),
            ThemeMode::Light => Self::make(
                ThemePreset::SolarAmber,
                mode,
                rgb(0xfe_f9_ee),
                rgb(0xfe_f3_c7),
                rgb(0xff_ff_ff),
                rgb(0xfe_f7_e5),
                rgba(0x78_35_0f, 0.14),
                rgb(0x29_1d_0e),
                rgb(0x5c_43_24),
                rgb(0x92_40_0e),
                rgb(0xff_ff_ff),
                rgba(0x92_40_0e, 0.12),
            ),
        }
    }

    const fn velvet_sakura(mode: ThemeMode) -> Self {
        match mode {
            ThemeMode::Dark => Self::make(
                ThemePreset::VelvetSakura,
                mode,
                rgb(0x0d_08_0b),
                rgb(0x16_0e_13),
                rgb(0x1d_13_19),
                rgb(0x26_19_21),
                rgba(0xfc_e7_f3, 0.11),
                rgb(0xfb_f1_f6),
                rgb(0xc4_a1_b4),
                rgb(0xf4_72_b6),
                rgb(0x0d_08_0b),
                rgba(0xf4_72_b6, 0.16),
            ),
            ThemeMode::Light => Self::make(
                ThemePreset::VelvetSakura,
                mode,
                rgb(0xfa_f7_f5),
                rgb(0xfc_e7_f3),
                rgb(0xff_ff_ff),
                rgb(0xfd_f2_f8),
                rgba(0x83_18_43, 0.13),
                rgb(0x28_10_1c),
                rgb(0x63_33_4b),
                rgb(0xbe_18_5d),
                rgb(0xff_ff_ff),
                rgba(0xbe_18_5d, 0.11),
            ),
        }
    }

    #[allow(clippy::too_many_arguments)]
    const fn make(
        preset: ThemePreset,
        mode: ThemeMode,
        bg: Color,
        panel: Color,
        card: Color,
        card_hover: Color,
        border_subtle: Color,
        text_primary: Color,
        text_muted: Color,
        accent: Color,
        on_accent: Color,
        accent_subtle: Color,
    ) -> Self {
        let (success, warning, danger) = match mode {
            ThemeMode::Light => (rgb(0x15_80_3d), rgb(0xb4_53_09), rgb(0xb9_1c_1c)),
            ThemeMode::Dark => (rgb(0x4a_de_80), rgb(0xfb_bf_24), rgb(0xf8_71_71)),
        };

        Self {
            preset,
            mode,
            bg,
            panel,
            card,
            card_hover,
            border_subtle,
            text_primary,
            text_muted,
            accent,
            on_accent,
            accent_subtle,
            success,
            warning,
            danger,
        }
    }

    /// Estilo de painel lateral ou faixa de apoio.
    #[must_use]
    pub fn panel_style(&self) -> container::Style {
        container::Style {
            background: Some(self.panel.into()),
            text_color: Some(self.text_primary),
            border: Border {
                color: self.border_subtle,
                width: 1.0,
                radius: RADIUS_PANEL.into(),
            },
            shadow: Shadow::default(),
            snap: true,
        }
    }

    /// Estilo de cartão elevado principal.
    #[must_use]
    pub fn card_style(&self) -> container::Style {
        container::Style {
            background: Some(self.card.into()),
            text_color: Some(self.text_primary),
            border: Border {
                color: self.border_subtle,
                width: 1.0,
                radius: RADIUS_PANEL.into(),
            },
            shadow: Shadow {
                color: Color {
                    a: match self.mode {
                        ThemeMode::Light => 0.05,
                        ThemeMode::Dark => 0.20,
                    },
                    ..Color::BLACK
                },
                offset: Vector::new(0.0, 2.0),
                blur_radius: 8.0,
            },
            snap: true,
        }
    }

    /// Estilo da barra superior (`48px`).
    #[must_use]
    pub fn topbar_style(&self) -> container::Style {
        container::Style {
            background: Some(self.panel.into()),
            text_color: Some(self.text_primary),
            border: Border {
                color: self.border_subtle,
                width: 1.0,
                radius: 0.0.into(),
            },
            shadow: Shadow::default(),
            snap: true,
        }
    }

    /// Estilo de moldura para o Quadro Personalizável da Home.
    #[must_use]
    pub fn board_frame_style(&self, frame: BoardFrameStyle) -> container::Style {
        let (background, border_color, border_width, radius) = match frame {
            BoardFrameStyle::Polaroid => (rgb(0xff_ff_ff), rgba(0x00_00_00, 0.14), 2.0, 4.0),
            BoardFrameStyle::Photocard => (self.card, self.accent, 2.0, 14.0),
            BoardFrameStyle::Clean => (self.card, self.border_subtle, 1.0, RADIUS_CONTROL),
            BoardFrameStyle::Moderna => (self.bg, self.text_primary, 2.0, 2.0),
            BoardFrameStyle::Japandi => (self.panel, rgb(0xb8_9f_80), 3.0, 10.0),
            BoardFrameStyle::Museu => (rgb(0xf7_f3_eb), rgb(0x8c_6d_3f), 4.0, 4.0),
            BoardFrameStyle::Nenhum => (Color::TRANSPARENT, Color::TRANSPARENT, 0.0, 0.0),
        };

        container::Style {
            background: Some(background.into()),
            text_color: Some(self.text_primary),
            border: Border {
                color: border_color,
                width: border_width,
                radius: radius.into(),
            },
            shadow: Shadow::default(),
            snap: true,
        }
    }

    /// Estilo para botão primário de ação.
    #[must_use]
    pub fn primary_button_style(&self, status: button::Status) -> button::Style {
        let bg = match status {
            button::Status::Hovered | button::Status::Pressed => Color {
                a: 0.90,
                ..self.accent
            },
            _ => self.accent,
        };
        button::Style {
            background: Some(bg.into()),
            text_color: self.on_accent,
            border: Border {
                color: self.accent,
                width: 1.0,
                radius: RADIUS_CONTROL.into(),
            },
            shadow: Shadow::default(),
            snap: true,
        }
    }

    /// Estilo para botão secundário ou sutil.
    #[must_use]
    pub fn ghost_button_style(&self, status: button::Status) -> button::Style {
        let bg = match status {
            button::Status::Hovered => Some(self.card_hover.into()),
            button::Status::Pressed => Some(self.accent_subtle.into()),
            _ => None,
        };
        button::Style {
            background: bg,
            text_color: self.text_primary,
            border: Border {
                color: self.border_subtle,
                width: 1.0,
                radius: RADIUS_CONTROL.into(),
            },
            shadow: Shadow::default(),
            snap: true,
        }
    }
}

/// Calcula a razão de contraste WCAG 2.2 entre duas cores (entre `1.0` e `21.0`).
#[must_use]
pub fn contrast_ratio(foreground: Color, background: Color) -> f32 {
    let l1 = relative_luminance(foreground);
    let l2 = relative_luminance(background);
    let (bright, dark) = if l1 >= l2 { (l1, l2) } else { (l2, l1) };
    (bright + 0.05) / (dark + 0.05)
}

/// Verifica se o contraste atinge o mínimo WCAG 2.2 AA para texto normal (`>= 4.5:1`).
#[must_use]
pub fn meets_wcag_aa(foreground: Color, background: Color) -> bool {
    contrast_ratio(foreground, background) >= 4.5
}

fn relative_luminance(color: Color) -> f32 {
    let r = linearize_channel(color.r);
    let g = linearize_channel(color.g);
    let b = linearize_channel(color.b);
    0.2126 * r + 0.7152 * g + 0.0722 * b
}

fn linearize_channel(channel: f32) -> f32 {
    if channel <= 0.04045 {
        channel / 12.92
    } else {
        ((channel + 0.055) / 1.055).powf(2.4)
    }
}
