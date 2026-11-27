//! Tipografia oficial estática embutida do **eStudy** (`Newsreader 16pt`, `DM Sans` e `JetBrains Mono`).

use iced::Font;
use iced::font::{Family, Style, Weight};
use std::borrow::Cow;

/// Bytes TrueType estáticos da família editorial `Newsreader 16pt` (Regular, OFL).
pub const NEWSREADER_REGULAR_BYTES: &[u8] =
    include_bytes!("../assets/fonts/Newsreader-Regular.ttf");
/// Bytes TrueType estáticos da família editorial `Newsreader 16pt` (Medium, OFL).
pub const NEWSREADER_MEDIUM_BYTES: &[u8] = include_bytes!("../assets/fonts/Newsreader-Medium.ttf");
/// Bytes TrueType estáticos da família editorial `Newsreader 16pt` (Italic real, OFL).
pub const NEWSREADER_ITALIC_BYTES: &[u8] = include_bytes!("../assets/fonts/Newsreader-Italic.ttf");

/// Bytes TrueType estáticos da família `DM Sans` (Regular, OFL).
pub const DM_SANS_REGULAR_BYTES: &[u8] = include_bytes!("../assets/fonts/DMSans-Regular.ttf");
/// Bytes TrueType estáticos da família `DM Sans` (Medium, OFL).
pub const DM_SANS_MEDIUM_BYTES: &[u8] = include_bytes!("../assets/fonts/DMSans-Medium.ttf");
/// Bytes TrueType estáticos da família `DM Sans` (`SemiBold`, OFL).
pub const DM_SANS_SEMIBOLD_BYTES: &[u8] = include_bytes!("../assets/fonts/DMSans-SemiBold.ttf");
/// Bytes TrueType estáticos da família `DM Sans` (Bold, OFL).
pub const DM_SANS_BOLD_BYTES: &[u8] = include_bytes!("../assets/fonts/DMSans-Bold.ttf");

/// Bytes TrueType estáticos da família monoespaçada `JetBrains Mono` (Regular, OFL).
pub const JETBRAINS_MONO_REGULAR_BYTES: &[u8] =
    include_bytes!("../assets/fonts/JetBrainsMono-Regular.ttf");
/// Bytes TrueType estáticos da família monoespaçada `JetBrains Mono` (Medium, OFL).
pub const JETBRAINS_MONO_MEDIUM_BYTES: &[u8] =
    include_bytes!("../assets/fonts/JetBrainsMono-Medium.ttf");

/// Família padrão de interface (`DM Sans`, Regular).
pub const FONT_SANS: Font = Font {
    family: Family::Name("DM Sans"),
    weight: Weight::Normal,
    ..Font::DEFAULT
};

/// Família de interface em peso médio (`DM Sans`, Medium).
pub const FONT_SANS_MEDIUM: Font = Font {
    family: Family::Name("DM Sans"),
    weight: Weight::Medium,
    ..Font::DEFAULT
};

/// Família de interface em destaque (`DM Sans`, `SemiBold`).
pub const FONT_SANS_SEMIBOLD: Font = Font {
    family: Family::Name("DM Sans"),
    weight: Weight::Semibold,
    ..Font::DEFAULT
};

/// Família de interface em peso forte (`DM Sans`, Bold).
pub const FONT_SANS_BOLD: Font = Font {
    family: Family::Name("DM Sans"),
    weight: Weight::Bold,
    ..Font::DEFAULT
};

/// Família editorial serifada (`Newsreader 16pt`, Regular) para títulos e números de destaque.
pub const FONT_SERIF: Font = Font {
    family: Family::Name("Newsreader 16pt"),
    weight: Weight::Normal,
    ..Font::DEFAULT
};

/// Família editorial serifada em peso médio (`Newsreader 16pt`, Medium).
pub const FONT_SERIF_MEDIUM: Font = Font {
    family: Family::Name("Newsreader 16pt"),
    weight: Weight::Medium,
    ..Font::DEFAULT
};

/// Família editorial serifada em itálico real (`Newsreader 16pt`, Italic) para subtítulos na mesma base.
pub const FONT_SERIF_ITALIC: Font = Font {
    family: Family::Name("Newsreader 16pt"),
    weight: Weight::Normal,
    style: Style::Italic,
    ..Font::DEFAULT
};

/// Família monoespaçada (`JetBrains Mono`, Regular) para horários, timers e métricas.
pub const FONT_MONO: Font = Font {
    family: Family::Name("JetBrains Mono"),
    weight: Weight::Normal,
    ..Font::DEFAULT
};

/// Família monoespaçada em peso médio (`JetBrains Mono`, Medium).
pub const FONT_MONO_MEDIUM: Font = Font {
    family: Family::Name("JetBrains Mono"),
    weight: Weight::Medium,
    ..Font::DEFAULT
};

/// Tamanho mínimo legível para micro-etiquetas, atalhos de teclado e legendas (`12px`).
pub const FONT_SIZE_XS: f32 = 12.0;
/// Tamanho compacto para metadados, contadores secundários, tags e badges (`13px`).
pub const FONT_SIZE_SM: f32 = 13.0;
/// Tamanho base padrão para controles, inputs, chips e descrições (`14px`).
pub const FONT_SIZE_BASE: f32 = 14.0;
/// Tamanho principal de leitura para itens de lista, navegação e tarefas (`15px`).
pub const FONT_SIZE_MD: f32 = 15.0;
/// Tamanho de destaque para cabeçalhos de seção e títulos de cartão (`16px`).
pub const FONT_SIZE_LG: f32 = 16.0;
/// Tamanho de subtítulo editorial em itálico e títulos de entrada (`18px`).
pub const FONT_SIZE_XL: f32 = 18.0;
/// Tamanho de título de modal ou cartão de resumo (`24px`).
pub const FONT_SIZE_2XL: f32 = 24.0;
/// Tamanho editorial de abertura de página e métricas principais (`34px`).
pub const FONT_SIZE_DISPLAY: f32 = 34.0;
/// Tamanho de relógio compacto para popups do Pomodoro (`44px`).
pub const FONT_SIZE_TIMER: f32 = 44.0;
/// Tamanho de relógio principal no Modo Foco em tela cheia (`54px`).
pub const FONT_SIZE_HERO_TIMER: f32 = 54.0;

/// Retorna todas as 9 fontes estáticas OFL prontas para registro na inicialização do `Iced`.
#[must_use]
pub fn embedded_fonts() -> Vec<Cow<'static, [u8]>> {
    vec![
        Cow::Borrowed(NEWSREADER_REGULAR_BYTES),
        Cow::Borrowed(NEWSREADER_MEDIUM_BYTES),
        Cow::Borrowed(NEWSREADER_ITALIC_BYTES),
        Cow::Borrowed(DM_SANS_REGULAR_BYTES),
        Cow::Borrowed(DM_SANS_MEDIUM_BYTES),
        Cow::Borrowed(DM_SANS_SEMIBOLD_BYTES),
        Cow::Borrowed(DM_SANS_BOLD_BYTES),
        Cow::Borrowed(JETBRAINS_MONO_REGULAR_BYTES),
        Cow::Borrowed(JETBRAINS_MONO_MEDIUM_BYTES),
    ]
}
