# eStudy

Aplicativo desktop local-first de produtividade pessoal em **Rust**, reunindo **Início Pessoal, Tarefas (Semana por Períodos, Calendário e Kanban), Hábitos, Segundo Cérebro, Agenda, Estudos, Estante de Leituras, Foco/Áudio e Gamificação leve** em um único binário nativo com memória privada alvo abaixo de **45 MiB (`RssAnon`)** e aceleração **`wgpu` (`sRGB`)** ativa por padrão.

> **Status atual:** `v0.6.2` — Redesign editorial completo com tipografia estática (`Newsreader` Regular/Medium/Italic + `DM Sans` + `JetBrains Mono` via `Shaping::Advanced`), ícones ilustrados coloridos (`IllustratedIcon` + mascote **Mochi**) e traço limpo `1.75px`, janela limpa sem Topbar global nem 2ª sidebar, manipulação direta nos cartões (check `○ -> ✓`, status em 1 clique, **Drag & Drop** entre `SEM DIA`/dias/períodos e **Menu de Contexto no clique direito**), modais multi-linha reais (`text_editor`) e persistência **SQLite v2**.

---

## Como começar (Desenvolvimento)

### Pré-requisitos
- **Rust** `1.89+` (Edition 2024) com `rustfmt` e `clippy`

### Comandos principais

```bash
# Executar o aplicativo desktop (aceleração GPU wgpu sRGB ativa por padrão, 60–144 FPS)
cargo run -p estudy-desktop

# Executar em modo somente CPU opcional (tiny-skia)
cargo run -p estudy-desktop --no-default-features --features cpu

# Executar a sandbox visual para inspecionar temas e componentes
cargo run -p estudy-preview

# Verificar formatação e análise estática sem warnings
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings

# Executar todos os testes unitários e de integração
cargo test --workspace

# Validação completa, build release e benchmark de RAM/logs
bash scripts/bench_and_validate.sh
```

---

## Organização do Workspace

- `crates/estudy-domain`: regras puras de negócio (projetos, tarefas, calendário recorrente, janelas livres), identificadores cronológicos (`UUIDv7`), relógio virtual (`ClockPort`), eventos pós-commit e cadeia HMAC-SHA256 da carteira.
- `crates/estudy-storage`: diretórios padrão do sistema (`XDG`/`directories` com `--data-dir` e `ESTUDY_DATA_DIR`), SQLite em modo WAL, schema relacional v1 com `FTS5`, backup pré-migração, repositórios e snapshot limpo para nuvem.
- `crates/estudy-audio`: motor procedural DSP lock-free (`cpal` com fallback silencioso) sintetizando 5 faixas de foco em `-16 LUFS` sem buffers em RAM.
- `crates/estudy-ui-kit`: Design System em `Iced` (`0.14.0`) com 9 fontes estáticas OFL (`Shaping::Advanced`), 5 presets em modo Claro/Escuro validados em contraste WCAG 2.2 AA, 17 ícones ilustrados multi-cor (`IllustratedIcon`) + 87 ícones de linha `1.75px`, 7 molduras, `HeatmapGrid` anual e primitivos suaves.
- `crates/estudy-ui`: interface MVU com barra lateral única (`216px` / `58px`), `Ctrl+K`, Home editorial, Tarefas (`Semana | Calendário | Kanban` com Drag & Drop, menu no cursor e modal multi-linha), Hábitos, Segundo Cérebro, Agenda e Modo Foco.
- `apps/estudy-desktop`: binário principal com aceleração GPU `wgpu` por padrão (`RssAnon ~30 MiB`), persistência SQLite v2, escrita seletiva (*dirty tracking* + *soft delete*) e logs diários em JSON.
- `apps/estudy-preview`: sandbox visual com dados de exemplo para testar layouts, temas e ícones ilustrados isoladamente.

Os próximos ciclos entregam Estudos e Estante (`v0.7.0`), servidor MCP nativo e Jardim/Pets/Loja (`v0.8.0`). Veja o plano completo em [ROADMAP.md](./ROADMAP.md) e o guia visual em [DESIGN.md](./DESIGN.md).

---

## Documentação Adicional

- [DESIGN.md](./DESIGN.md) — Sistema visual, tipografia, layout e os 5 presets de tema WCAG 2.2 AA.
- [ROADMAP.md](./ROADMAP.md) — Entregas planejadas da `v0.0.1` à `v1.0.0`.
- [CHANGELOG.md](./CHANGELOG.md) — Histórico de alterações e comparações por versão.
- [docs/architecture/c4-overview.md](./docs/architecture/c4-overview.md) — Visão de contexto, contêineres e fronteiras de dependência.
- [docs/adr/](./docs/adr/) — Registros de decisões arquiteturais (MADR).
