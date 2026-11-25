//! Tipografia oficial embutida do **eStudy** (`Plus Jakarta Sans`, `Lora` e `JetBrains Mono`).

use iced::Font;
use iced::font::{Family, Weight};
use std::borrow::Cow;

/// Bytes TrueType da família `Plus Jakarta Sans` (variável, OFL).
pub const PLUS_JAKARTA_SANS_BYTES: &[u8] = include_bytes!("../assets/fonts/PlusJakartaSans.ttf");

/// Bytes TrueType da família editorial `Lora` (variável, OFL).
pub const LORA_BYTES: &[u8] = include_bytes!("../assets/fonts/Lora.ttf");

/// Bytes TrueType da família monoespaçada `JetBrains Mono` (variável, OFL).
pub const JETBRAINS_MONO_BYTES: &[u8] = include_bytes!("../assets/fonts/JetBrainsMono.ttf");

/// Família padrão de interface (`Plus Jakarta Sans`, Regular).
pub const FONT_SANS: Font = Font {
    family: Family::Name("Plus Jakarta Sans"),
    weight: Weight::Normal,
    ..Font::DEFAULT
};

/// Família de interface em peso médio (`Plus Jakarta Sans`, Medium).
pub const FONT_SANS_MEDIUM: Font = Font {
    family: Family::Name("Plus Jakarta Sans"),
    weight: Weight::Medium,
    ..Font::DEFAULT
};

/// Família de interface em destaque (`Plus Jakarta Sans`, `SemiBold`).
pub const FONT_SANS_SEMIBOLD: Font = Font {
    family: Family::Name("Plus Jakarta Sans"),
    weight: Weight::Semibold,
    ..Font::DEFAULT
};

/// Família editorial serifada (`Lora`, Regular) para saudações, títulos de página e citações.
pub const FONT_SERIF: Font = Font {
    family: Family::Name("Lora"),
    weight: Weight::Normal,
    ..Font::DEFAULT
};

/// Família editorial serifada em peso médio (`Lora`, Medium).
pub const FONT_SERIF_MEDIUM: Font = Font {
    family: Family::Name("Lora"),
    weight: Weight::Medium,
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

/// Retorna todas as fontes embutidas prontas para registro na inicialização do `Iced`.
#[must_use]
pub fn embedded_fonts() -> Vec<Cow<'static, [u8]>> {
    vec![
        Cow::Borrowed(PLUS_JAKARTA_SANS_BYTES),
        Cow::Borrowed(LORA_BYTES),
        Cow::Borrowed(JETBRAINS_MONO_BYTES),
    ]
}
