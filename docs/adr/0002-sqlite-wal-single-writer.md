# ADR-0002: Persistência Local-First em SQLite WAL (`rusqlite`) com Backup e Snapshot

* **Status:** aceito
* **Data:** 2026-11-22
* **Decisores:** Equipe eStudy
* **Módulos Afetados:** `estudy-storage`

## 1. Contexto e Declaração do Problema
O eStudy armazena dados críticos do usuário (projetos, sessões de estudo, revisões espaçadas, leituras e razão da carteira) localmente e precisa permitir leituras simultâneas entre o aplicativo desktop e o servidor MCP (`estudy-mcp`) sem risco de erro `SQLITE_BUSY` ou corrupção ao sincronizar com serviços de nuvem.

## 2. Direcionadores de Decisão (Drivers)
* Proteção absoluta dos dados do usuário durante atualizações de schema.
* Leituras concorrentes rápidas com teto rígido de cache em RAM (4 MB).
* Exportação segura para pastas de nuvem (Drive, OneDrive, Syncthing, Git) sem arquivos `-wal`/`-shm` dependentes.

## 3. Opções Consideradas
* **Opção 1:** `rusqlite` (SQLite embarcado) em modo WAL + Online Backup API
* **Opção 2:** `sqlx` (SQLite assíncrono em pool)
* **Opção 3:** Arquivos Markdown/JSON soltos em disco

## 4. Decisão Escolhida
Escolhida a **Opção 1 (`rusqlite` em modo WAL)**, porque dá acesso direto à **Online Backup API** nativa do SQLite para backups automáticos pré-migração e exportação de snapshots limpos em arquivo único, além de suportar busca `FTS5` e controle fino de pragmas.

## 5. Prós e Contras das Opções

### `rusqlite` em WAL + Online Backup API
* Boa, porque suporta backups atômicos a quente, `PRAGMA cache_size = -4000`, `FTS5` e `PRAGMA data_version` para reatividade entre processos.
* Ruim, porque exige disciplina explícita nas migrações SQL em vez de geração automática por ORM.

### `sqlx`
* Boa, porque possui verificação de queries em compilação.
* Ruim, porque não expõe ergonomicamente a Online Backup API e adiciona complexidade de pool assíncrono para um banco local embarcado.

### Arquivos Markdown/JSON soltos
* Boa, porque são legíveis diretamente no editor de texto.
* Ruim, porque dificultam transações atômicas, integridade referencial e consultas analíticas rápidas.

## 6. Consequências e Trade-offs
* **Impacto Positivo:** Zero dependência de servidor externo, integridade referencial garantida por chaves estrangeiras e proteção pré-migração com rotação de 5 backups.
* **Impacto Negativo / Débito Aceito:** Mudanças de schema futuras devem seguir política aditiva (*Expand-Only*) com `extra_json` e `deleted_at`.
* **Ações Mitigatórias:** Todo script SQL de migração roda em transação única seguido de `PRAGMA quick_check` e `PRAGMA foreign_key_check`.
