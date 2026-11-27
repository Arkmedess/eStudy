# Design System — eStudy

Documento oficial de interface, tokens visuais, tipografia estática, iconografia ilustrada e acessibilidade do **eStudy** (`Iced 0.14` + `wgpu` sRGB padrão).

---

## 1. Arquitetura Visual, Janela Limpa e Interatividade Direta

O eStudy segue uma estética editorial acolhedora e tátil: zero barras globais poluídas, zero segundas sidebars roubando largura útil e manipulação direta nos cartões antes de abrir qualquer janela modal.

- **Sem Topbar global e sem 2ª sidebar:** o palco principal ocupa 100% da altura da janela com respiro amplo.
- **Sidebar esquerda limpa (`216px` expandida $\leftrightarrow$ `58px` em trilho de ícones, `Ctrl+B`):**
  - *Modo expandido (`216px`):* marca nominal **`e`***`Study`* (`Newsreader Italic` + `Newsreader Medium`), gatilho discreto de busca (`Buscar... Ctrl+K`), seção `SEU ESPAÇO` com as 5 rotas ativas (*Início, Tarefas, Hábitos, Segundo cérebro, Agenda*), contadores discretos, medalhão ilustrado do gatinho **Mochi** com fala contextual e `Ajustes`.
  - *Modo recolhido (`58px`):* apenas os ícones ilustrados coloridos (`22px`, sem micro-textos espremidos), medalhão do **Mochi** (clique abre balão flutuante ao lado) e `Ajustes`.
- **Pílula flutuante de Pomodoro (sob demanda):** aparece no canto superior direito somente quando o cronômetro de foco está ativo (`● MM:SS · Tarefa [Pausar] [Expandir]`).
- **Cabeçalho editorial unificado (`editorial_page_header`):** micro-breadcrumb (`ESTUDY / TAREFAS`) acima da linha dupla na mesma base — título em `Newsreader` (`34px`) ao lado do subtítulo em **`Newsreader Italic` real** (`18px`) — e no máximo **1 botão calmo de ação** alinhado à direita (`+ Nova tarefa`, `+ Novo compromisso`).
- **Home (`Início`) em 3 faixas:**
  1. *Hero acolhedor:* saudação em `Newsreader` (`38px`), resumo vivo do dia, paisagem vetorial sutil à direita e 4 pílulas rápidas (`+ Tarefa`, `+ Hábito`, `+ Compromisso`, `+ Nota`).
  2. *Faixa de contraste tonal (`4fr / 5fr / 3fr`):* Card Terracota Quente (*Foco & Ritmo* com disparo rápido de 25m), Card Musgo Escuro (*Constância · 14 dias* com `SparklineCanvas` e métricas) e *Mini-Calendário* interativo do mês.
  3. *Faixa acionável em 3 colunas:* *Tarefas* (check circular direto na Home), *Hábitos* (check colorido direto) e *Segundo cérebro*.
- **Tarefas (`[ Semana | Calendário | Kanban ]`):**
  - Abre direto na visão **`Semana`**, com bandeja horizontal **`SEM DIA`** no topo e colunas diárias divididas nos 4 períodos (`Qualquer hora`, `Manhã`, `Tarde`, `Noite`).
  - **Manipulação direta no cartão:** rádio circular `○ -> ✓` em 1 clique, pílula de status clicável (`Falta fazer -> Fazendo -> Concluído`), **Drag & Drop fluido** entre `SEM DIA`, dias e períodos, e **Menu de Contexto flutuante no cursor (clique direito)** (*Editar, Duplicar, Concluir, Iniciar foco, Status ▸, Prioridade ▸, Período ▸, Dia ▸, Excluir*).
  - **Apenas 2 modais previsíveis:** Modal central **`Nova / Editar tarefa`** (com `text_editor` multi-linha real onde `Enter` pula linha, checklist interativa, status, prioridade, toggle *Tem dia*, atalhos *Hoje / Amanhã / +1 semana*, período e projeto) e **Card do Dia** ao clicar numa data do modo Calendário.
- **Segundo Cérebro & Hábitos:** grade ampla de 4 colunas no Segundo Cérebro tendo o **1º cartão `+ Nova nota`** e leitura/edição multi-linha real (`text_editor`) em modal amplo; matriz semanal em Hábitos com `+ Novo hábito` integrado no rodapé do cartão.

---

## 2. Tipografia Estática (`Shaping::Advanced`), Iconografia em 2 Camadas e Geometria

- **Motor gráfico padrão (`wgpu`):** `estudy-desktop` e `estudy-preview` ativam `wgpu` por padrão para blending linear `sRGB` nítido nas fontes, sombras suaves e 60–144 FPS no Drag & Drop, mantendo a memória privada do processo (`RssAnon`) abaixo de `45 MiB` (e modo CPU opcional via `--no-default-features --features cpu`).
- **9 fontes TrueType estáticas OFL embutidas (`crates/estudy-ui-kit/assets/fonts/`, zero eixo variável `fvar`):**
  - `Newsreader 16pt` (`Regular`, `Medium`, **`Italic` real**) — títulos editoriais, subtítulos em itálico na mesma base e métricas principais.
  - `DM Sans` (`Regular`, `Medium`, `SemiBold`, `Bold`) — interface limpa, cartões, filtros e textos de apoio.
  - `JetBrains Mono` (`Regular`, `Medium`) — cronômetros, datas e contadores tabulares.
  - Todos os helpers (`shaped_text`, `editorial_page_header`, `editorial_heading`) forçam `text::Shaping::Advanced` (HarfRust) para garantir kerning `GPOS` completo também em strings ASCII.
- **Iconografia em duas camadas (`crates/estudy-ui-kit/src/icons.rs`):**
  - **`IllustratedIcon` (17 SVGs multi-cor com cache estático):** ícones ilustrados com preenchimento e luz próprios (sem o filtro monocromático do `svg::Style.color`) para módulos da sidebar, períodos do dia (*Qualquer hora, Manhã, Tarde, Noite*), prioridades e medalhão do gatinho **Mochi**.
  - **`Icon` (87 SVGs monoline `1.75px`):** traços limpos renderizados em tamanhos inteiros (`16px / 20px / 24px`) para ações internas de botões.
- **Bordas hairline delicadas e sombras suaves:** `border_subtle` calibrada em `10%–11%` de opacidade e elevação orgânica suave nos cartões, modais e menus flutuantes.

---

## 3. Os 5 Presets de Tema (10 Variações Verificadas em WCAG 2.2 AA)

As cores centrais do **Oliva Orgânico** (e dos demais 4 presets) são preservadas integralmente e validadas por testes automáticos de contraste mínimo `4.5:1` antes de cada release.

| Preset | Modo | `accent` | `on_accent` | `bg` | `panel` | `card` | Contraste `accent/panel` |
| :--- | :---: | :--- | :--- | :--- | :--- | :--- | :---: |
| **1. Oliva Orgânico** *(Padrão)* | Light | `#374c3b` | `#ffffff` | `#f6f3ec` | `#e8e3d8` | `#ffffff` | `7.26:1` |
| **1. Oliva Orgânico** | Dark | `#b2c9a8` | `#10120f` | `#10120f` | `#161915` | `#222720` | `9.98:1` |
| **2. eStudy Menta** | Dark | `#86efac` | `#07080a` | `#07080a` | `#0e1016` | `#181b24` | `13.54:1` |
| **2. eStudy Menta** | Light | `#0f612e` | `#ffffff` | `#f4f7fa` | `#e8eff6` | `#ffffff` | `6.54:1` |
| **3. Cyber Minimalist** | Dark | `#4cc9ff` | `#040711` | `#050814` | `#0b1224` | `#141e36` | `9.86:1` |
| **3. Cyber Minimalist** | Light | `#025e91` | `#ffffff` | `#eff6ff` | `#dfedfc` | `#ffffff` | `5.86:1` |
| **4. Solar Amber** | Dark | `#fcc738` | `#0e0b08` | `#0e0b08` | `#17120d` | `#241d15` | `11.84:1` |
| **4. Solar Amber** | Light | `#84380a` | `#ffffff` | `#fef8ec` | `#f9edd1` | `#ffffff` | `7.08:1` |
| **5. Velvet Sakura** | Dark | `#f786c0` | `#0d080b` | `#0d080b` | `#160e13` | `#23171e` | `8.25:1` |
| **5. Velvet Sakura** | Light | `#aa1452` | `#ffffff` | `#faf5f7` | `#f7e2ee` | `#ffffff` | `5.82:1` |
