# `estudy-ui-kit`

## Propósito
Fornece o Design System oficial do **eStudy** em `Iced 0.14`: os 5 presets de tema (10 variações Light/Dark verificadas em WCAG 2.2 AA com bordas hairline suaves e sombras orgânicas), as 9 fontes TrueType estáticas OFL (`Newsreader 16pt` com Itálico real, `DM Sans` e `JetBrains Mono`), o sistema duplo de iconografia (`IllustratedIcon` multi-cor + `Icon` monoline `1.75px`) e os primitivos editoriais reutilizáveis.

## Contrato Público
- `EstudyTheme`, `ThemePreset` (*Oliva Orgânico, eStudy Menta, Cyber Minimalist, Solar Amber, Velvet Sakura*), `ThemeMode` (`Light` / `Dark`), `CategoryColor` (7 categorias semânticas).
- `embedded_fonts()`, `FONT_SERIF`, `FONT_SERIF_MEDIUM`, `FONT_SERIF_ITALIC`, `FONT_SANS`, `FONT_SANS_MEDIUM`, `FONT_SANS_SEMIBOLD`, `FONT_SANS_BOLD`, `FONT_MONO`, `FONT_MONO_MEDIUM`.
- `IllustratedIcon` (`17` SVGs coloridos artesanais para módulos, períodos, prioridades e mascote Mochi) e `Icon` (`87` SVGs monoline `1.75px`).
- Primitivos `shaped_text` (`Shaping::Advanced` obrigatório), `editorial_page_header`, `illustrated_svg`, `icon_svg`, `card`, `tonal_card`, `segmented_switch`, `pill_chip` e `modal_surface`.

## Testes e Preview Visual
```bash
cargo test -p estudy-ui-kit
cargo run -p estudy-preview
```
