//! Tokens visuais, os 5 presets WCAG 2.2 AA e slots cosméticos do eStudy.

use iced::widget::{button, container, text_input};
use iced::{Border, Color, Shadow};

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

/// Altura fixa da barra superior discreta (`44px`).
pub const TOPBAR_HEIGHT: f32 = 44.0;
/// Largura padrão da barra lateral principal expandida (`220px`).
pub const SIDEBAR_WIDTH: f32 = 220.0;
/// Largura da barra lateral principal comprimida em trilho de ícones (`72px`).
pub const SIDEBAR_COMPACT_WIDTH: f32 = 72.0;

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
                rgb(0xf7_f4_ee),
                rgb(0xf2_ee_e6),
                rgb(0xef_ec_e4),
                rgb(0xff_ff_ff),
                rgb(0xf4_f1_ea),
                rgba(0x1f_26_20, 0.12),
                rgb(0x14_19_15),
                rgb(0x3a_44_3b),
                rgb(0x3f_54_43),
                rgb(0xff_ff_ff),
                rgba(0x3f_54_43, 0.12),
            ),
            ThemeMode::Dark => Self::make(
                ThemePreset::OlivaOrganico,
                mode,
                rgb(0x12_14_11),
                rgb(0x16_19_15),
                rgb(0x1a_1d_18),
                rgb(0x21_25_1f),
                rgb(0x28_2d_25),
                rgba(0xff_ff_ff, 0.12),
                rgb(0xf4_f7_f2),
                rgb(0xc2_cb_bf),
                rgb(0xa8_be_9e),
                rgb(0x12_14_11),
                rgba(0xa8_be_9e, 0.18),
            ),
        }
    }

    const fn estudy_menta(mode: ThemeMode) -> Self {
        match mode {
            ThemeMode::Dark => Self::make(
                ThemePreset::EstudyMenta,
                mode,
                rgb(0x07_07_09),
                rgb(0x0c_0d_11),
                rgb(0x0f_10_15),
                rgb(0x13_14_1a),
                rgb(0x1a_1c_24),
                rgba(0xff_ff_ff, 0.12),
                rgb(0xf8_fa_fc),
                rgb(0xb8_c4_d4),
                rgb(0x86_ef_ac),
                rgb(0x07_07_09),
                rgba(0x86_ef_ac, 0.16),
            ),
            ThemeMode::Light => Self::make(
                ThemePreset::EstudyMenta,
                mode,
                rgb(0xf8_fa_fc),
                rgb(0xf4_f7_fb),
                rgb(0xf1_f5_f9),
                rgb(0xff_ff_ff),
                rgb(0xf4_f8_fb),
                rgba(0x0f_17_2a, 0.11),
                rgb(0x0a_10_1f),
                rgb(0x33_41_55),
                rgb(0x15_80_3d),
                rgb(0xff_ff_ff),
                rgba(0x15_80_3d, 0.12),
            ),
        }
    }

    const fn cyber_minimalist(mode: ThemeMode) -> Self {
        match mode {
            ThemeMode::Dark => Self::make(
                ThemePreset::CyberMinimalist,
                mode,
                rgb(0x04_07_11),
                rgb(0x07_0c_18),
                rgb(0x0a_10_20),
                rgb(0x0f_17_2a),
                rgb(0x16_20_38),
                rgba(0xe2_e8_f0, 0.10),
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
                rgb(0xe8_f3_fe),
                rgb(0xe0_f0_fe),
                rgb(0xff_ff_ff),
                rgb(0xf2_f8_ff),
                rgba(0x0c_4a_6e, 0.09),
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
                rgb(0x13_10_0b),
                rgb(0x17_13_0e),
                rgb(0x1f_1a_13),
                rgb(0x28_21_18),
                rgba(0xfe_f3_c7, 0.10),
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
                rgb(0xfe_f6_db),
                rgb(0xfe_f3_c7),
                rgb(0xff_ff_ff),
                rgb(0xfe_f7_e5),
                rgba(0x78_35_0f, 0.10),
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
                rgb(0x12_0b_0f),
                rgb(0x16_0e_13),
                rgb(0x1d_13_19),
                rgb(0x26_19_21),
                rgba(0xfc_e7_f3, 0.10),
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
                rgb(0xfd_f0_f7),
                rgb(0xfc_e7_f3),
                rgb(0xff_ff_ff),
                rgb(0xfd_f2_f8),
                rgba(0x83_18_43, 0.09),
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
            ThemeMode::Light => (rgb(0x15_80_3d), rgb(0xb4_53_09), rgb(0xb9_1c_1c)),
            ThemeMode::Dark => (rgb(0x4a_de_80), rgb(0xfb_bf_24), rgb(0xf8_71_71)),
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

    /// Estilo de cartão elevado principal com cantos suaves e borda hairline.
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
            shadow: Shadow::default(),
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
            shadow: Shadow::default(),
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
            shadow: Shadow::default(),
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
        let border_color = match status {
            text_input::Status::Focused { .. } => self.accent,
            text_input::Status::Hovered => self.text_muted,
            text_input::Status::Active | text_input::Status::Disabled => self.border_subtle,
        };
        text_input::Style {
            background: self.surface_soft.into(),
            border: Border {
                color: border_color,
                width: 1.0,
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
                Self::Sage => rgb(0x004E_7A5B),
                Self::Terracotta => rgb(0x00B8_5D43),
                Self::Slate => rgb(0x004A_6B82),
                Self::Ochre => rgb(0x00B0_7A2A),
                Self::Lavender => rgb(0x0073_628A),
                Self::Moss => rgb(0x003D_5A45),
                Self::Coral => rgb(0x00C2_6D5E),
            },
            ThemeMode::Dark => match self {
                Self::Sage => rgb(0x008C_C49B),
                Self::Terracotta => rgb(0x00E8_947C),
                Self::Slate => rgb(0x008D_B7D4),
                Self::Ochre => rgb(0x00E8_B96B),
                Self::Lavender => rgb(0x00B8_A6D4),
                Self::Moss => rgb(0x007E_B88E),
                Self::Coral => rgb(0x00EC_9E90),
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
