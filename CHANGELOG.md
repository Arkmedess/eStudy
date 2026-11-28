# Changelog

Todas as alterações relevantes do **eStudy** são documentadas neste arquivo seguindo o padrão [Keep a Changelog](https://keepachangelog.com/pt-BR/1.1.0/) e [SemVer 2.0.0](https://semver.org/lang/pt-BR/).

---

## [Unreleased]

---

## [0.6.3] - 2026-11-28

### Adicionado
- **Captura de Telas via CLI e Servidor MCP Visual** (`apps/estudy-desktop/src/screenshot_cli.rs`, `scripts/estudy_ui_mcp.py`): flags `--screenshot` e `--subview` (`week`, `calendar`, `kanban`, `list`, `task-modal`, `day-peek`, `note-modal`, `today-drawer`, `palette`, `focus-running`) com dimensões fixas sob demanda para inspecionar cada tela e modal em `1360×840` e `950×960`.
- **Ação `Desfazer` para Exclusões e Novos Fluxos Diretos** (`crates/estudy-ui/`): botão `Desfazer` no canto inferior esquerdo após excluir tarefa, hábito, nota ou compromisso; criação de projetos direto na interface (`+ Projeto`); edição do nome de exibição e do bloco Pomodoro padrão em **Ajustes** (`Perfil & Ritmo de Foco`); inclusão de notas e hábitos na busca `Ctrl+K`; e exclusão lógica de eventos da Agenda no SQLite (`soft_delete_event`).

### Aprimorado
- **Polimento Visual, Modo Escuro e Responsividade**: contorno dos 17 `IllustratedIcon` adaptado para Modo Escuro via `illustrated_svg_themed`, paisagem do Hero integrada sem caixa de fundo, gráfico de Constância com curva Bézier e preenchimento em gradiente vertical, `89` ícones monoline (`Icon::Check` e `Icon::Circle` adicionados), círculos de check limpos sem anel duplo (`radio_check_circle`), molduras tracejadas (`dashed_frame`), scrollbars sutis de `5px` e os 3 cartões inferiores da Home alinhados pela base em largura padrão.
- **Tarefário, Agenda, Hábitos, Segundo Cérebro e Foco Sem Cortes**: visão `Semana` com 7 colunas completas roláveis horizontalmente (`248px`), limiar de `6px` antes de iniciar arraste para preservar cliques, pílula flutuante do timer movida para o canto inferior direito (oculta na tela Foco), Card do Dia clicável com edição/exclusão e `Lista` mensal na Agenda, cartões do Segundo Cérebro adaptativos em 2/3/4 colunas com tags removíveis e URL em `Link`, e validação em `usability_flows.rs` ampliada para 12 jornadas (`UF-01..UF-12`).

---

## [0.6.2] - 2026-11-26

### Adicionado
- **Fontes Estáticas OFL + `Shaping::Advanced`** (`crates/estudy-ui-kit/src/fonts.rs`): embutimento de 9 arquivos TrueType estáticos (`Newsreader 16pt` Regular/Medium/Italic real, `DM Sans` Regular/Medium/SemiBold/Bold e `JetBrains Mono` Regular/Medium) com modelagem HarfRust ativa em todos os componentes para eliminar colisão de letras e serrilhado.
- **Iconografia Ilustrada Colorida (`IllustratedIcon`) + Monoline `1.75px`** (`crates/estudy-ui-kit/src/icons.rs`): 17 ícones vetoriais multi-cor sem filtro monocromático para módulos da barra lateral, períodos do dia, prioridades e medalhão do gatinho **Mochi**, acompanhados de 87 ícones de linha limpa (`1.75px`) em tamanhos inteiros.
- **Manipulação Direta e Modais Previsíveis em `Tarefas`** (`crates/estudy-ui/src/features/projects/`): abertura direta em `[ Semana | Calendário | Kanban ]`, bandeja `SEM DIA`, período `Qualquer hora`, conclusão `○ -> ✓` e giro de status em 1 clique no cartão, Drag & Drop entre dias e períodos, menu de contexto no cursor (botão direito), modal de tarefa multi-linha real (`text_editor`), Card do Dia no calendário e exclusão lógica persistida no SQLite (`soft_delete_task`).

### Alterado
- **Motor Gráfico GPU (`wgpu`) por Padrão**: ativado em `estudy-desktop` e `estudy-preview` com correção gama linear nas fontes, sombras suaves em cartões/modais e `RssAnon` medido em `~30.1 MiB` (mantendo a feature `cpu` opcional).
- **Moldura Limpa e Composição Editorial das Rotas** (`crates/estudy-ui/`): remoção da Topbar global e da segunda barra lateral contextual; nova barra lateral única (`216px` expandida ou `58px` apenas com ícones), pílula flutuante de Pomodoro somente quando o timer está rodando, `Home` reestruturada (Hero com `+ações`, 3 cartões contrastantes com Mini-Calendário e 3 colunas acionáveis), `Hábitos` com `+ Novo hábito` integrado, `Segundo Cérebro` em grade ampla com modal multi-linha e `Agenda` enxuta em `[ Calendário | Lista ]` — mantendo intactas todas as cores do tema `Oliva Orgânico`.

---

## [0.6.1] - 2026-11-25

### Adicionado
- **Persistência SQLite v2 (`0002_habits_and_weekly_turns.sql`)**: suporte a `weekly_turns` no `CHECK` de `projects`, novas tabelas `habits` e `habit_logs`, repositórios `SqliteHabitsRepository` e `SqliteSecondBrainRepository` (integrado a `notes` + `FTS5`) e sincronização seletiva (*dirty tracking*) em `apps/estudy-desktop`.
- **Popups de Projeto e Tarefa (`QuickPopup::ProjectOverview`, `QuickPopup::TaskQuick`)**: clicar em um card de Projeto na Home abre visão rápida com tarefas clicáveis e botão `Play` para iniciar o Pomodoro sem sair da tela.
- **Barra de Status Inferior Sob Demanda (`34px`)**: aparece somente quando há Pomodoro ativo ou aviso transitório, liberando 100% da altura quando inativa.
- **Tela funcional de Ajustes (`AppRoute::Settings`)**: seletor direto dos 5 presets de tema, alternância Claro/Escuro e mapa de atalhos (`Ctrl+K`, `Ctrl+B`, `Esc`).
- **Suíte de Testes de Usabilidade (`crates/estudy-ui/tests/usability_flows.rs`)**: 7 jornadas ponta-a-ponta (`UF-01` a `UF-07`) cobrindo ações rápidas sem sair da tela, foco em 1 clique, pilha do `Esc`, regra anti-dupla-sidebar e zero *dead ends*.

### Aprimorado
- **Barra Lateral Sobreposta (Zero Reflow)**: trilho compacto de `72px` com micro-labels legíveis e ícones literais (`CheckSquare` para Projetos, `BookOpen` para Notas); ao expandir (`248px`), sobrepõe o palco via `stack!` com *scrim* clicável, exibindo apenas as 5 rotas ativas + *Ações Rápidas* + *Ajustes*.
- **Header Opinativo (`48px`) e Home Enxuta**: busca centralizada (`Ctrl+K`), 4 utilitários transversais (`Hoje`, `Hábitos X/Y`, `Focar`, `Notificações`) e remoção do cartão decorativo de perfil e do bloco "Hoje" duplicado na Home.
- **Contraste Tipográfico e Fluidez**: aprofundamento de `text_primary`, `text_muted` e bordas sutis em `OlivaOrganico` e `EstudyMenta` (Light/Dark), correção do disparo imediato do timer no botão `Play` de tarefas e remoção da feature `crisp` no `tiny-skia`.

---

## [0.6.0] - 2026-11-25

### Adicionado
- **Módulo de Hábitos** (`estudy-domain/src/habits.rs` e `estudy-ui/src/features/habits/`): grade semanal (`SEG`–`DOM`), criação inline sem sair da tela, resumo "O mês" com percentual global e barras individuais de adesão, além do calendário mensal de constância.
- **Segundo Cérebro** (`estudy-domain/src/second_brain.rs` e `estudy-ui/src/features/second_brain/`): layout *master-detail* na mesma tela (`7fr` grade de notas + `5fr` editor inline `Visualizar | Escrever`), classificação por tipo (*Nota, Ideia, Referência, Link*), pin, tags e exclusão rápida.
- **Projetos — Semana por Turnos**: visão de 5 dias úteis distribuídos em `Manhã`, `Tarde` e `Noite` com criação rápida por turno, cartões Kanban tintados por prioridade e botão direto de foco (`Play`) abrindo o Pomodoro com a tarefa vinculada.
- **Popups de Consulta Rápida (`QuickPopup`)**: leitura de nota, checklist de hábitos do dia e mini-controle de foco acessíveis diretamente da barra superior ou da Home sem trocar de tela (`Esc` fecha).

### Aprimorado
- **Barra Lateral Estável (`220px` <-> `64px`)**: botão de comprimir/expandir fixo no topo esquerdo (`x = 13px`) e ícones alinhados em slots rígidos (`38x36px`) que não se deslocam ao alternar o estado.
- **Alinhamento da Home e Ações na Mesma Tela**: cartões do Hero equalizados (`198px`) e faixa inline de *Hábitos de Hoje* (marcação direta) + *Segundo Cérebro* (abertura em popup).

### Otimizado
- **Fluidez de Scroll em CPU (`tiny-skia`)**: `HeatmapGrid` agrupado em 5 `Path`s retangulares por nível de intensidade (eliminando 1.460 curvas Bézier por frame) e `SparklineCanvas` com `canvas::Cache` por assinatura de dados.
- `scripts/bench_and_validate.sh` ampliado para medir tempo/percentual de CPU em janela de 2s junto de `VmRSS` (`35,06 MiB`, abaixo do teto de `45 MiB`), `VmHWM` e `Threads`.

---

## [0.5.0] - 2026-11-24

### Adicionado
- Tipografia real embutida no binário (`crates/estudy-ui-kit/assets/fonts/`): `Plus Jakarta Sans` (interface), `Lora` (títulos editoriais serifados) e `JetBrains Mono` (horários, timers e métricas numéricas).
- Paleta semântica de 7 categorias (`CategoryColor`: `Sage`, `Terracotta`, `Slate`, `Ochre`, `Lavender`, `Moss`, `Coral`) calibrada para Light e Dark em `estudy-ui-kit`, além do gráfico `SparklineCanvas` e dos primitivos `editorial_heading`, `category_tile`, `category_pill` e `floating_drawer`.
- Nova barra lateral fixa fina (`220px`, recolhível com `Ctrl+B`) com grupos de navegação (*Seu Espaço*, *Aprofundamento*, *Companhia*), badge de tarefas abertas, **cartão do Mascote** com fala contextual e novas rotas no `AppRoute` e `Ctrl+K` (*Hábitos*, *Segundo Cérebro*, *Financeiro*, *Pomodoro*).

### Aprimorado
- Home redesenhada em Painel Bento editorial: saudação serifada, faixa hero assimétrica (`5fr / 7fr`) com cartão tonal **Pulso de Foco 14 dias (`SparklineCanvas`)** e **Ritmo anual 365 dias**, e grade de Projetos com `category_tile` colorido e bolinhas por status.
- Gaveta "Hoje" reconstruída como painel flutuante arredondado (`20px`) com faixa semanal de 7 dias, bloco *Sem horário* tintado, cápsulas proporcionais por duração, divisores `Livre · XhYm` e régua vermelha contínua `Agora · HH:MM`.
- Perfil `dev` do `Cargo.toml` atualizado com `[profile.dev.package."*"] opt-level = 2` para manter a renderização de `tiny-skia` e `cosmic-text` fluida mesmo durante desenvolvimento local.

---

## [0.4.1] - 2026-11-23

### Corrigido e Aprimorado
- Removidas sombras desfocadas (`Shadow`) das superfícies em `estudy-ui-kit` que geravam artefatos visuais e lentidão de redesenho no backend padrão `tiny-skia`, substituindo por hierarquia tonal limpa e borda de `1px`.
- Centralizado e dimensionado dinamicamente o `HeatmapGrid` (365 dias com células arredondadas), respeitando redimensionamento da janela.
- Melhorada a apresentação da Home: removida a saudação duplicada no cartão de perfil, eliminados botões repetidos no cartão "Hoje", colunas equilibradas em largura máxima (`1360px`) e contador regressivo ativo na barra superior (`Focar · MM:SS`).
- Ajustado `apps/estudy-preview` para preencher a largura com equilíbrio e exibir os 87 ícones SVG em grade de 9 colunas sem recortes laterais.

---

## [0.4.0] - 2026-11-23

### Aprimorado
- Polimento visual suave e novos primitivos do `estudy-ui-kit` (cartões com cantos de `16px`, superfícies tonais/macias, divisores finos, seletores segmentados, chips em pílula, barra de progresso, métricas, atalhos `kbd_hint`, `banner_toast` e envelopes padrão de drawer/modal), mantendo contraste WCAG 2.2 AA em todos os 10 temas.

### Adicionado
- Regras de Pomodoro e Modo Foco (`PomodoroTimer`, presets `25/5`, `50/10` e `15/3`, faixas de áudio `DspTrack` e regra de recompensa mínima de `15` minutos) em `crates/estudy-domain`.
- Novo crate `crates/estudy-audio` com motor procedural DSP lock-free (`Silêncio`, `Chuva Marrom`, `Vento Rosa`, `Drone Quente`, `Alfa 10 Hz`) calibrado em `-16 LUFS`, saída via `cpal` e fallback silencioso quando headless.
- Repositório transacional `SqliteFocusRepository` em `crates/estudy-storage` gravando sessão em `study_sessions`, captura rápida em `notes` (`FTS5`) e crédito HMAC-SHA256 respeitando o teto diário de moedas.
- Modo Foco em tela cheia (`Esc` sai mantendo o cronômetro), anel vetorial em `Canvas` (`FocusRingProgram`), dock de áudio DSP, captura pós-sessão e ticker monotônico condicional de 1s em `crates/estudy-ui` e `apps/estudy-desktop`.

---

## [0.3.0] - 2026-11-23

### Adicionado
- Modelo e regras de Calendário Unificado (`CalendarEvent`, recorrência diária/semanal/mensal, eventos multi-dia, prazos de tarefas e cálculo de janelas livres da Timeline "Hoje") em `crates/estudy-domain`.
- Repositórios `SqliteCalendarRepository` e `SqliteSearchRepository` (busca global unificada combinando `notes_fts` via `FTS5` e entidades ativas) em `crates/estudy-storage`.
- Tela de Calendário Unificado (`Ano / Mês / Semana / Dia`) com barra contextual recolhível (`Ctrl+B`), criação rápida de eventos e filtros por projeto/tag, além do Drawer deslizante "Hoje" (seção *Sem horário*, blocos proporcionais, janelas livres `Livre · XhYY` e linha *Agora*).
- Barra de Comando (`Ctrl+K`) completa com busca dinâmica em projetos, tarefas, eventos e notas `FTS5`, além de criação direta via `> nova tarefa <título>`, `> novo evento <título>` e `> focar <N>m`.

### Otimizado e Adicionado (Qualidade, Performance & DevOps)
- Backend `tiny-skia` adotado como padrão no `Iced` (reduzindo o RSS em repouso para ~31 MiB; aceleração GPU `wgpu` disponível via `--features gpu` em `estudy-desktop`), além de `[profile.release]` com `lto = "thin"` e `strip = "symbols"`.
- Cache estático dos 87 handles SVG em `estudy-ui-kit` (coloração via `svg::Style`, zero alocações por frame) e `canvas::Cache` no `HeatmapGrid`.
- Rastreamento seletivo de alterações (*dirty tracking*) em `ProjectsState` e `CalendarState` (zero escritas SQLite durante digitação), cache de entradas ao abrir o `Ctrl+K` e consulta `FTS5` apenas para termos com `>= 2` caracteres.
- Logging estruturado em JSON via `tracing` + `tracing-subscriber` + `tracing-appender` com rotação diária em `cache_dir/logs/` e sem dados sensíveis.
- Script padrão de validação e benchmark [scripts/bench_and_validate.sh](file:///home/arthur/Development/eStudy/scripts/bench_and_validate.sh) e pipeline CI em `.github/workflows/ci.yml`.

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

---

[Unreleased]: https://github.com/Arkmedess/eStudy/compare/v0.6.3...HEAD
[0.6.3]: https://github.com/Arkmedess/eStudy/compare/v0.6.2...v0.6.3
[0.6.2]: https://github.com/Arkmedess/eStudy/compare/v0.6.1...v0.6.2
[0.6.1]: https://github.com/Arkmedess/eStudy/compare/v0.6.0...v0.6.1
[0.6.0]: https://github.com/Arkmedess/eStudy/compare/v0.5.0...v0.6.0
[0.5.0]: https://github.com/Arkmedess/eStudy/compare/v0.4.1...v0.5.0
[0.4.1]: https://github.com/Arkmedess/eStudy/compare/v0.4.0...v0.4.1
[0.4.0]: https://github.com/Arkmedess/eStudy/compare/v0.3.0...v0.4.0
[0.3.0]: https://github.com/Arkmedess/eStudy/compare/v0.2.0...v0.3.0
[0.2.0]: https://github.com/Arkmedess/eStudy/compare/v0.1.1...v0.2.0
[0.1.1]: https://github.com/Arkmedess/eStudy/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/Arkmedess/eStudy/compare/v0.0.1...v0.1.0
[0.0.1]: https://github.com/Arkmedess/eStudy/releases/tag/v0.0.1
