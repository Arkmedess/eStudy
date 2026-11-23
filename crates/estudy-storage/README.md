# `estudy-storage`

## Propósito
Responsável pela persistência local-first do **eStudy** em SQLite (modo WAL), resolução padrão de diretórios do sistema operacional, migrações seguras, backups pré-migração, snapshots limpos para nuvem e gravação transacional do razão `wallet_ledger`.

## Contrato Público
- `AppPaths::resolve(cli_data_dir)` e `AppPaths::from_sandbox_root(root)`: resolve diretórios `data`, `config` e `cache` seguindo XDG/SO, aceitando override via `--data-dir` ou `ESTUDY_DATA_DIR`.
- `open_connection(db_path)` e `open_in_memory()`: aplica `journal_mode = WAL`, `foreign_keys = ON`, `synchronous = NORMAL`, `busy_timeout = 5000`, `cache_size = -4000` (4 MB) e `temp_store = MEMORY`.
- `apply_migrations(conn, backups_dir, now)`: executa migrações pendentes em transação única, gera backup prévio quando já existe versão anterior e valida o banco com `PRAGMA quick_check` e `PRAGMA foreign_key_check`.
- `create_pre_migration_backup(...)` e `export_clean_snapshot(...)`: usam a Online Backup API do SQLite para gerar arquivos `.db` autônomos e rotacionar até 5 backups prévios.
- `SqliteWalletRepository`: grava entradas idempotentes no `wallet_ledger` dentro de transação única e emite evento no `EventBusPort` somente após o commit.

## Regras e Limites
- Dados do usuário ficam isolados em `data_dir`, separados de capas e logs descartáveis em `cache_dir`.
- O índice de busca `notes_fts` (`FTS5` com `remove_diacritics 2`) é mantido atualizado por triggers em `notes`.

## Trade-offs Descartados
- **Não** copiar manualmente o arquivo `estudy.db` enquanto o banco está em modo WAL: backups e snapshots passam obrigatoriamente pela Online Backup API para evitar cópias inconsistentes sem os dados ainda no `-wal`.

## Dependências
- `estudy-domain` (regras de domínio, `EntityId`, `WalletChain` e `EventBusPort`).

## Testes
```bash
cargo test -p estudy-storage
```
