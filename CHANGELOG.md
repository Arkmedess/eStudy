# Changelog

Todas as alterações relevantes do **eStudy** são documentadas neste arquivo seguindo o padrão [Keep a Changelog](https://keepachangelog.com/pt-BR/1.1.0/) e [SemVer 2.0.0](https://semver.org/lang/pt-BR/).

---

## [Unreleased]

---

## [0.2.0] - 2026-11-22

### Adicionado
- Regras de Projetos, Tarefas multi-projeto, Checklists (`0/20`), Marcos e Matriz de Eisenhower em `crates/estudy-domain`.
- Repositório transacional `SqliteProjectRepository` em `crates/estudy-storage` com carregamento em lote sem consultas N+1, tags, checklists e soft delete.
- Visão de Projetos em `crates/estudy-ui` com barra lateral esquerda contextual recolhível (`Ctrl+B` com mini-calendário mensal e resumo "Hoje"), faixa de próximos eventos, filtros por tag, Kanban 4 colunas, frameworks opcionais (*Marcos & Entregas*, *Matriz Eisenhower*, *Foco Semanal*) e Drawer lateral da tarefa (pin *Em foco* limitado a 5, múltiplos projetos, tags, datas/recorrência, checklist `0/20` e descrição Markdown `Escrever / Visualizar`).

---

## [0.1.1] - 2026-11-22

### Corrigido e Aprimorado
- Barra superior ajustada com atalho de foco, notificações/conquistas e menu compacto de módulos (*Home, Projetos, Calendário, Estudos, Estante, Jardim & Pets, Loja, Configurações*).
- Diário de humor com alternância entre os últimos 7 dias e visão anual, e exibição de todas as 7 molduras no quadro pessoal.
- Barra de comandos (`Ctrl+K`) ampliada com `> +10p <livro>`, Loja e Configurações.
- Leitura e criação do perfil inicial diretamente no banco SQLite local ao abrir o aplicativo.
- Perfil de compilação `dev` ajustado com `debug = "line-tables-only"` para acelerar montagens locais preservando o cache.

---

## [0.1.0] - 2026-11-22

### Adicionado
- Design System `crates/estudy-ui-kit` em `Iced` (`0.14.0`) com os 5 presets oficiais (*Oliva Orgânico, eStudy Menta, Cyber Minimalist, Solar Amber, Velvet Sakura*) em Light/Dark validados em contraste WCAG 2.2 AA (`>= 4.5:1`), catálogo de 87 ícones SVG, 7 molduras de quadro e `HeatmapGrid` anual (365 dias) em `Canvas`.
- Shell MVU `crates/estudy-ui` com Barra Superior (`48px`), Barra de Comando (`Ctrl+K`) e fatia Home em 3 colunas (Perfil + "Hoje", Heatmap 365d + Grade de Projetos em 2 colunas, "Em foco" `X/5` + Diário de Humor + Quadro Personalizável) cobrindo estados Loading, Empty, Error e Ready.
- Binário sandbox `apps/estudy-preview` e ponto de entrada desktop `apps/estudy-desktop` conectando `estudy-storage` + `estudy-domain` + `estudy-ui`.

---

## [0.0.1] - 2026-11-22

### Adicionado
- Estrutura inicial do Cargo Workspace (Edition 2024) com `crates/estudy-domain` e `crates/estudy-storage` e lints estritos (`unsafe_code = "forbid"`, `clippy::pedantic`).
- Núcleo de domínio em `estudy-domain` com identificadores cronológicos `EntityId` (`UUIDv7`), `ClockPort` (`SystemClock` / `FixedClock`), barramento de eventos pós-commit (`EventBusPort`) e cadeia criptográfica HMAC-SHA256 para o razão `wallet_ledger` com teto diário anti-inflação (`150` moedas/dia).
- Camada de persistência `estudy-storage` com resolução XDG (`--data-dir` / `ESTUDY_DATA_DIR`), abertura de conexão SQLite em modo WAL (`cache_size = -4000`), executor transacional de migrações com `PRAGMA quick_check` e `PRAGMA foreign_key_check`, backup automático pré-migração (rotação de 5 arquivos) e exportação de snapshot limpo para nuvem.
- Schema SQLite baseline v1 (`0001_initial_v1.sql`) com as 3 camadas de dados (usuário, economia/gamificação/auditoria e sistema/cache) e busca `notes_fts` (`FTS5`) sincronizada via triggers.
- Documentação fundacional: `README.md`, `DESIGN.md`, `ROADMAP.md`, diagramas C4 e ADRs `0001` a `0003`.
