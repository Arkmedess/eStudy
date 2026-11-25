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

- **v0.5.0 — Redesign Editorial, Fontes Embutidas, Sidebar 220px, Home Bento e Timeline em Cápsulas** *(Concluído)*
  - Fontes variáveis OFL embutidas no binário (`Plus Jakarta Sans`, `Lora` e `JetBrains Mono`), 7 cores de categoria (`CategoryColor`) e gráfico de área `SparklineCanvas`.
  - Navegação principal em sidebar fina (`220px`, recolhível com `Ctrl+B`), badges vivos, cartão do Mascote contextual no rodapé e topbar limpa (`44px`) com breadcrumb.
  - Home redesenhada em Painel Bento editorial (Pulso de Foco 14d + Ritmo Anual 365d + Projetos coloridos por categoria) e Gaveta Flutuante "Hoje" com faixa semanal, cápsulas proporcionais e régua `Agora`.
  - Perfil `dev` otimizando dependências de renderização (`tiny-skia` / `cosmic-text`) com `opt-level = 2` para fluidez local.

- **v0.6.0 — Projetos & Tarefas Turbinados, Hábitos, Segundo Cérebro e Financeiro** *(Em andamento)*
  - Visão semanal de tarefas por turnos (`Manhã`, `Tarde`, `Noite`), Kanban com cartões tintados, gaveta flutuante da tarefa e disparo direto de foco para o Pomodoro.
  - Módulo de Hábitos (grade semanal, constância mensal em barras e calendário), Segundo Cérebro (notas, ideias, referências e links integrados ao `FTS5`) e Financeiro pessoal simples.

- **v0.7.0 — Módulo de Estudos Completo e Estante de Leituras**
  - Ciclo contínuo de matérias com "Matéria da Vez", revisões espaçadas (`1/7/14/30/60d` e FSRS), flashcards *Active Recall*, Caderno de Erros e gráficos analíticos em `Canvas`.
  - Estante de Leituras com prateleiras customizáveis, sorteio *TBR Picker*, *Reader Companion* (`+5`/`+10` páginas, citações, estrelas) e capas da Open Library.

- **v0.8.0 — Servidor MCP Nativo (`apps/estudy-mcp`), Economia, Jardim Interativo, Pets e Loja Modular**
  - Servidor `stdio` compartilhando os casos de uso de `estudy-domain`, trilha `mcp_audit_log`, atualização reativa da UI via `PRAGMA data_version` e modo Admin (`--admin`).
  - Check-in diário, sequências, *Streak Freeze*, Jardim interativo em sprites WebP (< 1,5 MB RAM), Pets colecionáveis, conquistas e loja modular via `manifest.toml`.

- **v0.9.0 — Backup Portátil na Nuvem, Migrador e Atualizador SHA-256**
  - Exportação/importação aberta (Markdown + YAML + JSON), verificação final do orçamento de memória (< 45 MB RAM) e empacotamento multiplataforma.

- **v1.0.0 — Lançamento Estável**
  - Estabilização de contratos públicos, tag `v1.0.0`, `SHA256SUMS` e `sbom.json`.
