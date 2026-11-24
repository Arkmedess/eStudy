# Roadmap — eStudy (`v0.0.1` $\rightarrow$ `v1.0.0`)

Plano incremental de entregas do **eStudy**. Cada marco entrega um conjunto funcional testado sem quebrar dados existentes no SQLite local.

---

- **v0.0.1 — Fundação, Workspace Rust, Schema SQLite v1 e Governança** *(Concluído)*
  - Workspace Cargo (`estudy-domain` e `estudy-storage`) com regras de lint estritas e zero código `unsafe`.
  - Núcleo de domínio com identificadores cronológicos (`UUIDv7`), relógio virtual injetável, barramento de eventos pós-commit e cadeia HMAC-SHA256 da carteira.
  - Resolução XDG de diretórios (`--data-dir` / `ESTUDY_DATA_DIR`), SQLite em modo WAL com teto de cache de 4 MB, backup automático antes de migrações, exportação de snapshot limpo para nuvem e schema v1 completo com `FTS5`.
  - Documentação base (`README.md`, `DESIGN.md`, `ROADMAP.md`, `CHANGELOG.md`, diagramas C4 e ADRs).

- **v0.1.0 — UI Kit (`Iced`), 5 Presets WCAG AA, Catálogo SVG e Home Pessoal** *(Concluído)*
  - `crates/estudy-ui-kit` com os 5 presets (incluindo *Oliva Orgânico*), 87 ícones Lucide SVG e `apps/estudy-preview` para testar componentes isolados.
  - Home em 3 colunas (Perfil + "Hoje", Mapa de calor 365 dias + Projetos, Em Foco + Diário de Humor + Quadro personalizável) e estrutura inicial da Barra de Comando (`Ctrl+K`).

- **v0.2.0 — Projetos (Kanban + Frameworks Leves) e Gaveta da Tarefa** *(Concluído)*
  - Visão de projeto com Kanban de 4 colunas (`Backlog`, `A fazer`, `Em andamento`, `Concluído`).
  - Gaveta lateral da tarefa com checklist `0/20`, descrição Markdown (`Escrever/Visualizar`), pin *Em foco* e visões opcionais (*Marcos*, *Eisenhower*, *Foco Semanal*).

- **v0.3.0 — Timeline "Hoje", Calendário Unificado e `Ctrl+K` Completo** *(Concluído)*
  - Calendário (`Ano / Mês / Semana / Dia`) unificando eventos recorrentes/multi-dia e tarefas com prazo.
  - Gaveta "Hoje" com cálculo automático de janelas livres, linha *Agora* em tempo real e busca global via `FTS5` + ações rápidas no `Ctrl+K`.

- **v0.4.0 — Pomodoro, Modo Foco e Motor de Áudio Procedural DSP** *(Concluído)*
  - Temporizador em `Canvas` com relógio monotônico, Modo Foco em tela cheia, captura pós-sessão e síntese procedural de áudio (`cpal` + fila lock-free) em `-16 LUFS`.

- **v0.5.0 — Módulo de Estudos Completo**
  - Ciclo contínuo de matérias, revisões espaçadas (`1/7/14/30/60d` e FSRS), flashcards *Active Recall*, Caderno de Erros e 4 gráficos analíticos em `Canvas`.

- **v0.6.0 — Estante de Leituras**
  - Estantes customizáveis, ações em lote, sorteio de próxima leitura (*TBR Picker*), múltiplos formatos, *Reader Companion* com emoções/citações/releituras, metas e cache LRU de capas da Open Library.

- **v0.7.0 — Servidor MCP Nativo (`apps/estudy-mcp`), Modo Admin e QA Ponta a Ponta**
  - Servidor `stdio` compartilhando os casos de uso de `estudy-domain`, permissões configuráveis, trilha `mcp_audit_log`, atualização reativa da UI via `PRAGMA data_version`, modo Admin (`--admin`) e ferramentas de sandbox (`--e2e`).

- **v0.8.0 — Economia, Jardim Interativo, Pets, Conquistas e Loja Modular**
  - Check-in diário, sequências, *Streak Freeze*, teto diário anti-inflação, Jardim interativo em sprites WebP (< 1,5 MB RAM), Pets colecionáveis por raridade, conquistas com toast adiado durante o Modo Foco e loja modular via `manifest.toml`.

- **v0.9.0 — Backup Portátil na Nuvem, Migrador e Atualizador SHA-256**
  - Exportação/importação aberta (Markdown + YAML + JSON), verificação final do orçamento de memória (< 45 MB RAM) e empacotamento multiplataforma.

- **v1.0.0 — Lançamento Estável**
  - Estabilização de contratos públicos, tag `v1.0.0`, `SHA256SUMS` e `sbom.json`.
