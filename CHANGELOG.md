# Changelog

Todas as alterações relevantes do **eStudy** são documentadas neste arquivo seguindo o padrão [Keep a Changelog](https://keepachangelog.com/pt-BR/1.1.0/) e [SemVer 2.0.0](https://semver.org/lang/pt-BR/).

---

## [Unreleased]

---

## [0.0.1] - 2026-11-22

### Adicionado
- Estrutura inicial do Cargo Workspace (Edition 2024) com `crates/estudy-domain` e `crates/estudy-storage` e lints estritos (`unsafe_code = "forbid"`, `clippy::pedantic`).
- Núcleo de domínio em `estudy-domain` com identificadores cronológicos `EntityId` (`UUIDv7`), `ClockPort` (`SystemClock` / `FixedClock`), barramento de eventos pós-commit (`EventBusPort`) e cadeia criptográfica HMAC-SHA256 para o razão `wallet_ledger` com teto diário anti-inflação (`150` moedas/dia).
- Camada de persistência `estudy-storage` com resolução XDG (`--data-dir` / `ESTUDY_DATA_DIR`), abertura de conexão SQLite em modo WAL (`cache_size = -4000`), executor transacional de migrações com `PRAGMA quick_check` e `PRAGMA foreign_key_check`, backup automático pré-migração (rotação de 5 arquivos) e exportação de snapshot limpo para nuvem.
- Schema SQLite baseline v1 (`0001_initial_v1.sql`) com as 3 camadas de dados (usuário, economia/gamificação/auditoria e sistema/cache) e busca `notes_fts` (`FTS5`) sincronizada via triggers.
- Documentação fundacional: `README.md`, `DESIGN.md`, `ROADMAP.md`, diagramas C4 e ADRs `0001` a `0003`.
