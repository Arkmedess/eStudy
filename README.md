# eStudy

Aplicativo desktop local-first de produtividade pessoal em **Rust**, reunindo **Home Pessoal, Projetos, Hábitos, Segundo Cérebro, Calendário Unificado/Hoje, Estudos, Estante de Leituras, Foco/Áudio e Gamificação leve** em um único binário nativo com consumo alvo de **20–45 MB de RAM** (atualmente ~35 MiB em repouso no backend padrão `tiny-skia`).

> **Status atual:** `v0.6.0` — Tipografia embutida (`Plus Jakarta Sans`, `Lora`, `JetBrains Mono`), barra lateral recolhível estável (`220px` <-> `64px`) com mascote contextual, Home em Painel Bento com *Hábitos de Hoje* e *Segundo Cérebro* inline, Popups rápidos sem trocar de tela, Projetos com visão semanal por turnos (`Manhã/Tarde/Noite`) e disparo direto para o Pomodoro, módulos completos de **Hábitos** e **Segundo Cérebro**, Calendário Unificado, Gaveta Flutuante "Hoje", `Ctrl+K` com `FTS5` e Modo Foco com áudio procedural DSP (`-16 LUFS`).

---

## Como começar (Desenvolvimento)

### Pré-requisitos
- **Rust** `1.89+` (Edition 2024) com `rustfmt` e `clippy`

### Comandos principais

```bash
# Executar o aplicativo desktop (backend padrão em CPU, ~33 MiB RAM)
cargo run -p estudy-desktop

# Executar com aceleração GPU (opcional via feature wgpu)
cargo run -p estudy-desktop --features gpu

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
- `crates/estudy-ui-kit`: Design System em `Iced` (`0.14.0`) com 5 presets em modo Claro/Escuro validados em contraste WCAG 2.2 AA, 87 ícones SVG com cache estático, 7 molduras, `HeatmapGrid` anual e primitivos suaves.
- `crates/estudy-ui`: interface MVU com barra superior, `Ctrl+K`, tela inicial (Home), módulo de Projetos, Calendário Unificado, Drawer "Hoje" e Modo Foco em tela cheia.
- `apps/estudy-desktop`: binário principal com persistência SQLite, escrita seletiva (*dirty tracking*) e logs diários em JSON.
- `apps/estudy-preview`: sandbox visual com dados de exemplo para testar layouts e temas isoladamente.

Os próximos ciclos entregam polimento de usabilidade/UX (`v0.6.1`), Estudos e Estante (`v0.7.0`), servidor MCP nativo e Jardim/Pets/Loja (`v0.8.0`). Veja o plano completo em [ROADMAP.md](./ROADMAP.md) e o estudo de UX em [docs/ux-architecture-0.6.1.md](./docs/ux-architecture-0.6.1.md).

---

## Documentação Adicional

- [DESIGN.md](./DESIGN.md) — Sistema visual, tipografia, layout e os 5 presets de tema WCAG 2.2 AA.
- [ROADMAP.md](./ROADMAP.md) — Entregas planejadas da `v0.0.1` à `v1.0.0`.
- [CHANGELOG.md](./CHANGELOG.md) — Histórico de alterações e comparações por versão.
- [docs/architecture/c4-overview.md](./docs/architecture/c4-overview.md) — Visão de contexto, contêineres e fronteiras de dependência.
- [docs/adr/](./docs/adr/) — Registros de decisões arquiteturais (MADR).
