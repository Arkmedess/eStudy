# `estudy-ui`

## Propósito
Implementa o shell MVU da interface do **eStudy** desacoplado da janela nativa (`EstudyApp`), incluindo a barra superior (`48px`), a Barra de Comando (`Ctrl+K`) e a fatia Home em 3 colunas.

## Contrato Público
- `EstudyApp`, `AppMessage`, `AppRoute`: estado raiz testável sem GPU, com `ClockPort` injetado para saudações e atalhos globais (`Ctrl+K`, `Escape`).
- `CommandPaletteState` e `PaletteAction`: busca e filtro instantâneo de ações rápidas (`> focar 25m`, `> nova tarefa`, `> novo evento`, `> humor`, `> sortear leitura`, `> backup nuvem`) e navegação entre módulos.
- `features::home`: `HomeLoadState` (`Loading`, `Error`, `Ready`), `HomeDashboardData`, limite de `5` itens em *Em foco* (`MAX_FOCUS_PINS`), limite de `3` medalhas no perfil (`MAX_PINNED_MEDALS`), Diário de Humor (7 dias com ícones SVG) e Quadro Personalizável.

## Testes
```bash
cargo test -p estudy-ui
```
