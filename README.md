# eStudy

Aplicativo desktop local-first de produtividade pessoal em **Rust**, reunindo **Home Pessoal, Projetos, Calendário Unificado/Hoje, Estudos, Estante de Leituras, Foco/Áudio e Gamificação leve** em um único binário nativo com consumo alvo de **20–45 MB de RAM** (atualmente ~33 MiB em repouso no backend padrão `tiny-skia`).

> **Status atual:** `v0.3.0` — Home em 3 colunas com 5 presets WCAG 2.2 AA, Projetos (Kanban + Marcos/Eisenhower/Foco Semanal + Drawer da tarefa), Calendário Unificado (`Ano/Mês/Semana/Dia`), Drawer "Hoje" com janelas livres, Barra de Comando (`Ctrl+K`) com busca `FTS5`, logs estruturados em JSON e validação contínua.

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
- `crates/estudy-ui-kit`: Design System em `Iced` (`0.14.0`) com 5 presets em modo Claro/Escuro validados em contraste WCAG 2.2 AA, 87 ícones SVG com cache estático, 7 molduras e `HeatmapGrid` anual em `Canvas`.
- `crates/estudy-ui`: interface MVU com barra superior, `Ctrl+K`, tela inicial (Home), módulo de Projetos, módulo de Calendário Unificado e Drawer "Hoje".
- `apps/estudy-desktop`: binário principal com persistência SQLite, escrita seletiva (*dirty tracking*) e logs diários em JSON.
- `apps/estudy-preview`: sandbox visual com dados de exemplo para testar layouts e temas isoladamente.

Os próximos ciclos adicionam Pomodoro e áudio procedural (`v0.4.0`), Estudos (`v0.5.0`), Estante (`v0.6.0`), servidor MCP (`v0.7.0`) e Jardim/Pets/Loja (`v0.8.0`). Veja o plano completo em [ROADMAP.md](./ROADMAP.md).

---

## Documentação Adicional

- [DESIGN.md](./DESIGN.md) — Sistema visual, tipografia, layout e os 5 presets de tema WCAG 2.2 AA.
- [ROADMAP.md](./ROADMAP.md) — Entregas planejadas da `v0.0.1` à `v1.0.0`.
- [CHANGELOG.md](./CHANGELOG.md) — Histórico de alterações e comparações por versão.
- [docs/architecture/c4-overview.md](./docs/architecture/c4-overview.md) — Visão de contexto, contêineres e fronteiras de dependência.
- [docs/adr/](./docs/adr/) — Registros de decisões arquiteturais (MADR).
