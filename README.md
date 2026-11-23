# eStudy

Aplicativo desktop local-first de produtividade pessoal em **Rust**, reunindo **Projetos, Calendário/Hoje, Estudos, Estante de Leituras, Foco/Áudio e Gamificação leve** em um único binário nativo com consumo alvo de **20–45 MB de RAM**.

> **Status atual:** `v0.0.1` — Fundação do workspace Rust, núcleo de domínio com cadeia HMAC na carteira, camada de persistência SQLite em modo WAL e governança técnica.

---

## Como começar (Desenvolvimento)

### Pré-requisitos
- **Rust** `1.89+` (Edition 2024) com `rustfmt` e `clippy`

### Comandos principais

```bash
# Verificar formatação
cargo fmt --all -- --check

# Análise estática sem warnings
cargo clippy --workspace --all-targets -- -D warnings

# Executar todos os testes unitários e de integração
cargo test --workspace
```

---

## Organização do Workspace

- `crates/estudy-domain`: regras puras de negócio, identificadores cronológicos (`UUIDv7`), relógio virtual (`ClockPort`), eventos pós-commit e cadeia HMAC-SHA256 do razão da carteira.
- `crates/estudy-storage`: diretórios padrão do sistema (`XDG`/`directories` com suporte a `--data-dir` e `ESTUDY_DATA_DIR`), conexão SQLite em modo WAL, schema relacional v1 com `FTS5`, backup automático antes de migrações e snapshot limpo para pastas de nuvem.

Os próximos ciclos adicionam o Design System e Home (`v0.1.0`), Projetos (`v0.2.0`), Calendário e `Ctrl+K` (`v0.3.0`), Pomodoro e áudio procedural (`v0.4.0`), Estudos (`v0.5.0`), Estante (`v0.6.0`), servidor MCP (`v0.7.0`) e Jardim/Pets/Loja (`v0.8.0`). Veja o plano completo em [ROADMAP.md](./ROADMAP.md).

---

## Documentação Adicional

- [DESIGN.md](./DESIGN.md) — Sistema visual, tipografia, layout e os 5 presets de tema WCAG 2.2 AA.
- [ROADMAP.md](./ROADMAP.md) — Entregas planejadas da `v0.0.1` à `v1.0.0`.
- [CHANGELOG.md](./CHANGELOG.md) — Histórico de alterações por versão.
- [docs/architecture/c4-overview.md](./docs/architecture/c4-overview.md) — Visão de contexto, contêineres e fronteiras de dependência.
- [docs/adr/](./docs/adr/) — Registros de decisões arquiteturais (MADR).
