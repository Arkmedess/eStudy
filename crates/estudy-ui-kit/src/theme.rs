//! Tokens visuais, os 5 presets WCAG 2.2 AA e slots cosméticos do eStudy.

use iced::gradient::Linear;
use iced::widget::{button, container, scrollable, slider, text_input};
use iced::{Background, Border, Color, Gradient, Radians, Shadow, Vector};

/// Unidade base de espaçamento (`4px`).
pub const SPACE_1: f32 = 4.0;
/// Espaçamento compacto (`8px`).
pub const SPACE_2: f32 = 8.0;
/// Espaçamento padrão entre controles (`12px`).
pub const SPACE_3: f32 = 12.0;
/// Padding interno de painéis/cartões (`16px`).
pub const SPACE_4: f32 = 16.0;
/// Padding generoso para cartões principais (`20px`).
pub const SPACE_5: f32 = 20.0;
/// Margem entre colunas principais (`24px`).
pub const SPACE_6: f32 = 24.0;
/// Espaçamento amplo de seção (`32px`).
pub const SPACE_8: f32 = 32.0;

/// Altura fixa da barra superior discreta legada (`50px`).
pub const TOPBAR_HEIGHT: f32 = 50.0;
/// Largura padrão da barra lateral principal expandida (`216px`).
pub const SIDEBAR_WIDTH: f32 = 216.0;
/// Largura da barra lateral principal recolhida em trilho limpo de ícones (`58px`).
pub const SIDEBAR_COMPACT_WIDTH: f32 = 58.0;

/// Raio amplo e orgânico de gavetas flutuantes (`20px`).
pub const RADIUS_DRAWER: f32 = 20.0;
/// Raio amplo e orgânico de cartões principais (`16px`).
pub const RADIUS_CARD: f32 = 16.0;
/// Raio externo de painéis e gavetas (`12px`).
pub const RADIUS_PANEL: f32 = 12.0;
/// Raio de controles internos (`10px`, mantendo concentricidade).
pub const RADIUS_CONTROL: f32 = 10.0;
/// Raio de badges e chips compactos (`6px`).
pub const RADIUS_CHIP: f32 = 6.0;
/// Raio completo para pílulas, tags e seletores segmentados (`999px`).
pub const RADIUS_PILL: f32 = 999.0;

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
    /// Superfície suave intermediária para bandejas, linhas alternadas e apoios leves.
    pub surface_soft: Color,
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
                rgb(0xf6_f3_ec),
                rgb(0xea_e5_db),
                rgb(0xe8_e3_d8),
                rgb(0xff_ff_ff),
                rgb(0xf0_ec_e3),
                rgba(0x1f_26_20, 0.10),
                rgb(0x11_16_12),
                rgb(0x2f_3a_31),
                rgb(0x37_4c_3b),
                rgb(0xff_ff_ff),
                rgba(0x37_4c_3b, 0.14),
            ),
            ThemeMode::Dark => Self::make(
                ThemePreset::OlivaOrganico,
                mode,
                rgb(0x10_12_0f),
                rgb(0x1b_20_19),
                rgb(0x16_19_15),
                rgb(0x22_27_20),
                rgb(0x2b_31_28),
                rgba(0xff_ff_ff, 0.11),
                rgb(0xf6_f9_f4),
                rgb(0xc9_d3_c6),
                rgb(0xb2_c9_a8),
                rgb(0x10_12_0f),
                rgba(0xb2_c9_a8, 0.20),
            ),
        }
    }

    const fn estudy_menta(mode: ThemeMode) -> Self {
        match mode {
            ThemeMode::Dark => Self::make(
                ThemePreset::EstudyMenta,
                mode,
                rgb(0x07_08_0a),
                rgb(0x13_16_1d),
                rgb(0x000e_1016),
                rgb(0x18_1b_24),
                rgb(0x0022_2632),
                rgba(0xff_ff_ff, 0.11),
                rgb(0xf8_fa_fc),
                rgb(0xc3_cf_de),
                rgb(0x86_ef_ac),
                rgb(0x07_08_0a),
                rgba(0x86_ef_ac, 0.18),
            ),
            ThemeMode::Light => Self::make(
                ThemePreset::EstudyMenta,
                mode,
                rgb(0xf4_f7_fa),
                rgb(0xe6_ed_f4),
                rgb(0xe8_ef_f6),
                rgb(0xff_ff_ff),
                rgb(0xed_f3_f9),
                rgba(0x0f_17_2a, 0.10),
                rgb(0x0a_10_1f),
                rgb(0x2c_3a_4e),
                rgb(0x0f_61_2e),
                rgb(0xff_ff_ff),
                rgba(0x0f_61_2e, 0.14),
            ),
        }
    }

    const fn cyber_minimalist(mode: ThemeMode) -> Self {
        match mode {
            ThemeMode::Dark => Self::make(
                ThemePreset::CyberMinimalist,
                mode,
                rgb(0x05_08_14),
                rgb(0x10_18_2d),
                rgb(0x0b_12_24),
                rgb(0x14_1e_36),
                rgb(0x1c_29_47),
                rgba(0xe2_e8_f0, 0.11),
                rgb(0xf4_f8_fc),
                rgb(0xb2_c1_d6),
                rgb(0x4c_c9_ff),
                rgb(0x04_07_11),
                rgba(0x4c_c9_ff, 0.18),
            ),
            ThemeMode::Light => Self::make(
                ThemePreset::CyberMinimalist,
                mode,
                rgb(0xef_f6_ff),
                rgb(0xdd_eb_fa),
                rgb(0xdf_ed_fc),
                rgb(0xff_ff_ff),
                rgb(0xe8_f2_fe),
                rgba(0x0c_4a_6e, 0.10),
                rgb(0x0b_18_2b),
                rgb(0x27_42_5c),
                rgb(0x02_5e_91),
                rgb(0xff_ff_ff),
                rgba(0x02_5e_91, 0.14),
            ),
        }
    }

    const fn solar_amber(mode: ThemeMode) -> Self {
        match mode {
            ThemeMode::Dark => Self::make(
                ThemePreset::SolarAmber,
                mode,
                rgb(0x0e_0b_08),
                rgb(0x1e_18_12),
                rgb(0x17_12_0d),
                rgb(0x24_1d_15),
                rgb(0x2f_26_1c),
                rgba(0xfe_f3_c7, 0.11),
                rgb(0xfd_f8_ef),
                rgb(0xd1_c1_ac),
                rgb(0xfc_c7_38),
                rgb(0x0e_0b_08),
                rgba(0xfc_c7_38, 0.18),
            ),
            ThemeMode::Light => Self::make(
                ThemePreset::SolarAmber,
                mode,
                rgb(0xfe_f8_ec),
                rgb(0xf7_ea_cc),
                rgb(0xf9_ed_d1),
                rgb(0xff_ff_ff),
                rgb(0xfc_f2_da),
                rgba(0x78_35_0f, 0.10),
                rgb(0x24_18_0a),
                rgb(0x4b_35_19),
                rgb(0x84_38_0a),
                rgb(0xff_ff_ff),
                rgba(0x84_38_0a, 0.14),
            ),
        }
    }

    const fn velvet_sakura(mode: ThemeMode) -> Self {
        match mode {
            ThemeMode::Dark => Self::make(
                ThemePreset::VelvetSakura,
                mode,
                rgb(0x0d_08_0b),
                rgb(0x1c_12_18),
                rgb(0x16_0e_13),
                rgb(0x23_17_1e),
                rgb(0x2e_1f_28),
                rgba(0xfc_e7_f3, 0.11),
                rgb(0xfd_f3_f8),
                rgb(0xd4_b4_c6),
                rgb(0xf7_86_c0),
                rgb(0x0d_08_0b),
                rgba(0xf7_86_c0, 0.18),
            ),
            ThemeMode::Light => Self::make(
                ThemePreset::VelvetSakura,
                mode,
                rgb(0xfa_f5_f7),
                rgb(0xf5_de_ea),
                rgb(0xf7_e2_ee),
                rgb(0xff_ff_ff),
                rgb(0xf9_ea_f2),
                rgba(0x83_18_43, 0.10),
                rgb(0x24_0d_18),
                rgb(0x52_26_3c),
                rgb(0xaa_14_52),
                rgb(0xff_ff_ff),
                rgba(0xaa_14_52, 0.13),
            ),
        }
    }

    #[allow(clippy::too_many_arguments)]
    const fn make(
        preset: ThemePreset,
        mode: ThemeMode,
        bg: Color,
        surface_soft: Color,
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
            ThemeMode::Light => (rgb(0x0f_61_2e), rgb(0x84_37_06), rgb(0xa6_17_17)),
            ThemeMode::Dark => (rgb(0x5b_e4_8f), rgb(0xfc_c7_38), rgb(0xfb_85_85)),
        };

        Self {
            preset,
            mode,
            bg,
            surface_soft,
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

    const fn card_shadow(&self) -> Shadow {
        let alpha = match self.mode {
            ThemeMode::Light => 0.045,
            ThemeMode::Dark => 0.22,
        };
        Shadow {
            color: Color::from_rgba(0.08, 0.06, 0.04, alpha),
            offset: Vector::new(0.0, 2.0),
            blur_radius: 10.0,
        }
    }

    const fn elevated_shadow(&self) -> Shadow {
        let alpha = match self.mode {
            ThemeMode::Light => 0.14,
            ThemeMode::Dark => 0.42,
        };
        Shadow {
            color: Color::from_rgba(0.06, 0.05, 0.04, alpha),
            offset: Vector::new(0.0, 8.0),
            blur_radius: 26.0,
        }
    }

    /// Estilo de cartão elevado principal com cantos suaves, hairline delicada e sombra sutil.
    #[must_use]
    pub fn card_style(&self) -> container::Style {
        container::Style {
            background: Some(self.card.into()),
            text_color: Some(self.text_primary),
            border: Border {
                color: self.border_subtle,
                width: 1.0,
                radius: RADIUS_CARD.into(),
            },
            shadow: self.card_shadow(),
            snap: true,
        }
    }

    /// Estilo de cartão focal tonal usando o acento ativo com contraste AA+.
    #[must_use]
    pub fn tonal_card_style(&self) -> container::Style {
        container::Style {
            background: Some(self.accent.into()),
            text_color: Some(self.on_accent),
            border: Border {
                color: Color::TRANSPARENT,
                width: 0.0,
                radius: RADIUS_CARD.into(),
            },
            shadow: self.card_shadow(),
            snap: true,
        }
    }

    /// Superfície suave sem sombra para blocos internos ou linhas alternadas.
    #[must_use]
    pub fn soft_surface_style(&self) -> container::Style {
        container::Style {
            background: Some(self.surface_soft.into()),
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

    /// Bandeja em cápsula para agrupar seletores segmentados (`Semana | Calendário`).
    #[must_use]
    pub fn pill_tray_style(&self) -> container::Style {
        container::Style {
            background: Some(self.surface_soft.into()),
            text_color: Some(self.text_primary),
            border: Border {
                color: self.border_subtle,
                width: 1.0,
                radius: RADIUS_PILL.into(),
            },
            shadow: Shadow::default(),
            snap: true,
        }
    }

    /// Superfície padrão de gavetas deslizantes laterais.
    #[must_use]
    pub fn drawer_surface_style(&self) -> container::Style {
        container::Style {
            background: Some(self.card.into()),
            text_color: Some(self.text_primary),
            border: Border {
                color: self.border_subtle,
                width: 1.0,
                radius: RADIUS_CARD.into(),
            },
            shadow: Shadow::default(),
            snap: true,
        }
    }

    /// Superfície elevada para modais e Barra de Comando (`Ctrl+K`).
    #[must_use]
    pub fn modal_surface_style(&self) -> container::Style {
        container::Style {
            background: Some(self.card.into()),
            text_color: Some(self.text_primary),
            border: Border {
                color: self.border_subtle,
                width: 1.0,
                radius: RADIUS_CARD.into(),
            },
            shadow: self.elevated_shadow(),
            snap: true,
        }
    }

    /// Superfície flutuante para menus de contexto no cursor e cartões em Drag & Drop.
    #[must_use]
    pub fn floating_menu_style(&self) -> container::Style {
        container::Style {
            background: Some(self.card.into()),
            text_color: Some(self.text_primary),
            border: Border {
                color: self.border_subtle,
                width: 1.0,
                radius: RADIUS_PANEL.into(),
            },
            shadow: self.elevated_shadow(),
            snap: true,
        }
    }

    /// Hairline sutil de 1px para dividir listas e seções internas sem ruído.
    #[must_use]
    pub fn divider_style(&self) -> container::Style {
        container::Style {
            background: Some(self.border_subtle.into()),
            ..container::Style::default()
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
            BoardFrameStyle::Nenhum => (self.surface_soft, self.border_subtle, 1.0, RADIUS_PANEL),
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

    /// Estilo para botão leve sem borda em repouso (ideal para topbar e itens de lista).
    #[must_use]
    pub fn subtle_button_style(&self, status: button::Status) -> button::Style {
        let bg = match status {
            button::Status::Hovered => Some(self.card_hover.into()),
            button::Status::Pressed => Some(self.accent_subtle.into()),
            _ => None,
        };
        button::Style {
            background: bg,
            text_color: self.text_primary,
            border: Border {
                color: Color::TRANSPARENT,
                width: 0.0,
                radius: RADIUS_CONTROL.into(),
            },
            shadow: Shadow::default(),
            snap: true,
        }
    }

    /// Estilo para botão em cápsula (`RADIUS_PILL`) de filtros, tags e seletores segmentados.
    #[must_use]
    pub fn pill_button_style(&self, active: bool, status: button::Status) -> button::Style {
        if active {
            let bg = match status {
                button::Status::Hovered | button::Status::Pressed => Color {
                    a: 0.92,
                    ..self.accent
                },
                _ => self.accent,
            };
            button::Style {
                background: Some(bg.into()),
                text_color: self.on_accent,
                border: Border {
                    color: Color::TRANSPARENT,
                    width: 0.0,
                    radius: RADIUS_PILL.into(),
                },
                shadow: Shadow::default(),
                snap: true,
            }
        } else {
            let bg = match status {
                button::Status::Hovered => Some(self.card_hover.into()),
                button::Status::Pressed => Some(self.accent_subtle.into()),
                _ => Some(self.card.into()),
            };
            button::Style {
                background: bg,
                text_color: self.text_primary,
                border: Border {
                    color: self.border_subtle,
                    width: 1.0,
                    radius: RADIUS_PILL.into(),
                },
                shadow: Shadow::default(),
                snap: true,
            }
        }
    }

    /// Estilo suave para campos de texto alinhados ao tema ativo.
    #[must_use]
    pub fn soft_input_style(&self, status: text_input::Status) -> text_input::Style {
        let (border_color, border_width) = match status {
            text_input::Status::Focused { .. } => (self.accent, 1.5),
            text_input::Status::Hovered => (
                Color {
                    a: 0.65,
                    ..self.text_muted
                },
                1.0,
            ),
            text_input::Status::Active | text_input::Status::Disabled => (
                Color {
                    a: 0.40,
                    ..self.text_muted
                },
                1.0,
            ),
        };
        text_input::Style {
            background: self.surface_soft.into(),
            border: Border {
                color: border_color,
                width: border_width,
                radius: RADIUS_CONTROL.into(),
            },
            icon: self.text_muted,
            placeholder: self.text_muted,
            value: self.text_primary,
            selection: self.accent_subtle,
        }
    }

    /// Estilo da barra lateral fixa esquerda com hairline na borda direita.
    #[must_use]
    pub fn sidebar_style(&self) -> container::Style {
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

    /// Estilo de gaveta flutuante arredondada à direita com margem externa.
    #[must_use]
    pub fn floating_drawer_style(&self) -> container::Style {
        container::Style {
            background: Some(self.card.into()),
            text_color: Some(self.text_primary),
            border: Border {
                color: self.border_subtle,
                width: 1.0,
                radius: RADIUS_DRAWER.into(),
            },
            shadow: Shadow::default(),
            snap: true,
        }
    }

    /// Estilo de cartão clicável com elevação progressiva no hover e feedback tátil ao pressionar.
    #[must_use]
    pub fn interactive_card_style(&self, status: button::Status) -> button::Style {
        let hover_shadow = Shadow {
            offset: Vector::new(0.0, 5.0),
            blur_radius: 16.0,
            color: Color {
                a: match self.mode {
                    ThemeMode::Light => 0.09,
                    ThemeMode::Dark => 0.34,
                },
                ..self.card_shadow().color
            },
        };
        let pressed_shadow = Shadow {
            offset: Vector::new(0.0, 1.0),
            blur_radius: 4.0,
            ..self.card_shadow()
        };

        let (bg, border_color, shadow) = match status {
            button::Status::Hovered => (
                self.card_hover,
                Color {
                    a: 0.28,
                    ..self.accent
                },
                hover_shadow,
            ),
            button::Status::Pressed => (self.accent_subtle, self.accent, pressed_shadow),
            _ => (self.card, self.border_subtle, self.card_shadow()),
        };

        button::Style {
            background: Some(bg.into()),
            text_color: self.text_primary,
            border: Border {
                color: border_color,
                width: 1.0,
                radius: RADIUS_CARD.into(),
            },
            shadow,
            snap: true,
        }
    }

    /// Estilo de cartão com gradiente linear suave entre duas cores e texto de alto contraste.
    #[must_use]
    pub fn gradient_card_style(
        &self,
        start: Color,
        end: Color,
        angle_radians: f32,
        text_color: Color,
    ) -> container::Style {
        let linear = Linear::new(Radians(angle_radians))
            .add_stop(0.0, start)
            .add_stop(1.0, end);
        container::Style {
            background: Some(Background::Gradient(Gradient::Linear(linear))),
            text_color: Some(text_color),
            border: Border {
                color: self.border_subtle,
                width: 1.0,
                radius: RADIUS_CARD.into(),
            },
            shadow: self.card_shadow(),
            snap: true,
        }
    }

    /// Estilo harmonizado ao tema para controles deslizantes (`slider`), substituindo o azul padrão.
    #[must_use]
    pub fn slider_style(&self, status: slider::Status) -> slider::Style {
        let (handle_bg, handle_radius, border_color) = match status {
            slider::Status::Hovered => (self.card_hover, 9.0, self.accent),
            slider::Status::Dragged => (self.accent, 9.5, self.accent),
            slider::Status::Active => (self.card, 8.0, self.accent),
        };

        slider::Style {
            rail: slider::Rail {
                backgrounds: (self.accent.into(), self.surface_soft.into()),
                width: 5.0,
                border: Border {
                    radius: RADIUS_PILL.into(),
                    ..Border::default()
                },
            },
            handle: slider::Handle {
                shape: slider::HandleShape::Circle {
                    radius: handle_radius,
                },
                background: handle_bg.into(),
                border_width: 2.0,
                border_color,
            },
        }
    }

    /// Estilo sutil e discreto para barras de rolagem que combina com os presets do eStudy.
    #[must_use]
    pub fn subtle_scrollbar_style(&self, status: scrollable::Status) -> scrollable::Style {
        let is_active = matches!(
            status,
            scrollable::Status::Hovered { .. } | scrollable::Status::Dragged { .. }
        );
        let scroller_color = Color {
            a: if is_active { 0.42 } else { 0.22 },
            ..self.text_muted
        };
        let rail = scrollable::Rail {
            background: None,
            border: Border {
                radius: RADIUS_PILL.into(),
                ..Border::default()
            },
            scroller: scrollable::Scroller {
                background: scroller_color.into(),
                border: Border {
                    radius: RADIUS_PILL.into(),
                    ..Border::default()
                },
            },
        };
        scrollable::Style {
            container: container::Style::default(),
            vertical_rail: rail,
            horizontal_rail: rail,
            gap: None,
            auto_scroll: scrollable::AutoScroll {
                background: self.card.into(),
                border: Border {
                    color: self.border_subtle,
                    width: 1.0,
                    radius: RADIUS_PILL.into(),
                },
                shadow: self.card_shadow(),
                icon: self.text_primary,
            },
        }
    }
}

/// Paleta semântica de 7 categorias para projetos, tags, hábitos, notas e agenda.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CategoryColor {
    /// Verde sálvia (Saúde, Rotina, Equilíbrio).
    Sage,
    /// Terracota / argila (Urgência, Provas, Prazos).
    Terracotta,
    /// Azul ardósia (Faculdade, Estudos, Foco).
    Slate,
    /// Ocre / dourado (Financeiro, Metas, Energia).
    Ochre,
    /// Lavanda suave (Leituras, Referências, Criatividade).
    Lavender,
    /// Verde musgo profundo (Projetos principais, Conteúdo).
    Moss,
    /// Coral suave (Pessoal, Casa, Eventos sociais).
    Coral,
}

impl CategoryColor {
    /// Lista completa das 7 cores semânticas de categoria.
    pub const ALL: [Self; 7] = [
        Self::Sage,
        Self::Terracotta,
        Self::Slate,
        Self::Ochre,
        Self::Lavender,
        Self::Moss,
        Self::Coral,
    ];

    /// Seleciona de forma estável uma cor de categoria a partir de um índice ou chave.
    #[must_use]
    pub const fn from_index(index: usize) -> Self {
        Self::ALL[index % Self::ALL.len()]
    }

    /// Cor sólida de destaque (ícones, pontos, texto de tag) adaptada ao modo claro/escuro.
    #[must_use]
    pub const fn solid(self, theme: EstudyTheme) -> Color {
        match theme.mode {
            ThemeMode::Light => match self {
                Self::Sage => rgb(0x0034_5B40),
                Self::Terracotta => rgb(0x0091_3A23),
                Self::Slate => rgb(0x0037_5469),
                Self::Ochre => rgb(0x0075_4B0C),
                Self::Lavender => rgb(0x005A_4970),
                Self::Moss => rgb(0x002C_4734),
                Self::Coral => rgb(0x0092_3528),
            },
            ThemeMode::Dark => match self {
                Self::Sage => rgb(0x0096_CFA5),
                Self::Terracotta => rgb(0x00F0_A28C),
                Self::Slate => rgb(0x009B_C2DE),
                Self::Ochre => rgb(0x00F0_C67A),
                Self::Lavender => rgb(0x00C5_B5DF),
                Self::Moss => rgb(0x008C_C79C),
                Self::Coral => rgb(0x00F2_AB9E),
            },
        }
    }

    /// Fundo suave translúcido para cartões tintados, tiles de ícone e pílulas.
    #[must_use]
    pub const fn tint(self, theme: EstudyTheme) -> Color {
        let base = self.solid(theme);
        let alpha = match theme.mode {
            ThemeMode::Light => 0.13,
            ThemeMode::Dark => 0.18,
        };
        Color { a: alpha, ..base }
    }

    /// Borda sutil derivada da categoria para cartões Kanban e eventos tracejados.
    #[must_use]
    pub const fn border(self, theme: EstudyTheme) -> Color {
        let base = self.solid(theme);
        let alpha = match theme.mode {
            ThemeMode::Light => 0.28,
            ThemeMode::Dark => 0.34,
        };
        Color { a: alpha, ..base }
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
