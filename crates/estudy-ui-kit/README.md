# `estudy-ui-kit`

## Propósito
Fornece os tokens visuais, os 5 presets de tema (10 variações Light/Dark verificadas em WCAG 2.2 AA), o catálogo com 87 ícones vetoriais SVG, o gráfico anual de constância em `Canvas` e os cartões/badges base em `Iced`.

## Contrato Público
- `EstudyTheme`, `ThemePreset` (*Oliva Orgânico, eStudy Menta, Cyber Minimalist, Solar Amber, Velvet Sakura*), `ThemeMode` (`Light` / `Dark`).
- `contrast_ratio` e `meets_wcag_aa`: cálculo de contraste WCAG 2.2 AA (`>= 4.5:1`).
- `CosmeticLoadout` e `BoardFrameStyle` (7 molduras independentes do tema ativo).
- `Icon` (`Icon::ALL` com 87 variantes e `to_svg_string`).
- `HeatmapData` e `HeatmapGrid` (53 semanas × 7 dias em `Canvas`).
- Primitivos `icon_svg`, `card`, `panel`, `status_badge` e `section_header`.

## Testes e Preview Visual
```bash
cargo test -p estudy-ui-kit
cargo run -p estudy-preview
```
