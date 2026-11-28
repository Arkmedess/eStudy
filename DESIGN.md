# Design System — eStudy

Documento oficial de interface, tokens visuais, tipografia estática, iconografia ilustrada e acessibilidade do **eStudy** (`Iced 0.14` com motor gráfico `wgpu` sRGB por padrão).

---

## 1. Arquitetura Visual, Janela Limpa e Responsividade

O eStudy segue uma estética editorial acolhedora e tátil: zero barras horizontais no topo, zero segunda coluna fixa roubando espaço de leitura e interação direta nos cartões antes de abrir qualquer janela modal. Todas as telas adaptam colunas e barras de filtro para monitores amplos (`1360×840`+) e janelas lado a lado (`950×960`).

- **Sidebar esquerda única (`216px` expandida $\leftrightarrow$ `58px` em trilho de ícones, `Ctrl+B`):**
  - *Modo expandido (`216px`):* marca nominal **`e`***`Study`*, busca global (`Buscar... Ctrl+K`), seção `SEU ESPAÇO` com as 5 rotas ativas (*Início, Tarefas, Hábitos, Segundo cérebro, Agenda*) marcadas por filete lateral quando ativas, medalhão do gatinho **Mochi** com dicas contextuais, atalho `Timeline Hoje`, alternador claro/escuro e `Ajustes`.
  - *Modo recolhido (`58px`):* apenas os ícones ilustrados coloridos (`22px`, sem textos espremidos) e medalhão do **Mochi** clicável com balão flutuante ao lado.
- **Pílula flutuante de Pomodoro e toast `Desfazer`:**
  - A pílula de foco (`● MM:SS · Em foco [Pausar] [Expandir]`) aparece no **canto inferior direito** somente quando o cronômetro está rodando fora da tela de Foco, sem encobrir cabeçalhos nem botões `+ Nova tarefa`.
  - Exclusões de tarefas, hábitos, notas e compromissos exibem um aviso discreto no **canto inferior esquerdo** com botão **`Desfazer`** para restaurar na hora.
- **Busca global (`Ctrl+K`):** além de ações rápidas, indexa projetos, tarefas, notas do Segundo Cérebro e hábitos ativos.
- **Cabeçalho editorial unificado (`editorial_page_header`):** micro-breadcrumb acima da linha principal — título em `Newsreader` (`34px`) acompanhado do subtítulo em `Newsreader Italic` na mesma linha de base — com quebra automática de controles quando a janela fica estreita.
- **Home (`Início`) em 3 faixas:**
  1. *Hero orgânico:* saudação personalizada em `Newsreader` (`36px`), resumo do dia, paisagem vetorial sem caixa dura em Modo Claro e Escuro, e pílulas de criação rápida (`+ Tarefa`, `+ Hábito`, `+ Compromisso`, `+ Nota`, `Timeline Hoje`).
  2. *Faixa de contraste tonal (`4fr / 5fr / 3fr`, reorganizada em `950×960`):* cartão Terracota (*Foco & Ritmo* com disparo rápido), cartão Musgo (*Constância · 14 dias* com curva Bézier suave, preenchimento em gradiente e métricas em `JetBrains Mono`) e *Mini-Calendário* interativo com marcadores de dias ocupados e navegação direta para o dia.
  3. *Faixa acionável alinhada pela base em 3 colunas:* *Tarefas* (check circular direto e abertura em 1 clique), *Hábitos* (check colorido direto) e *Segundo cérebro*.
- **Tarefas (`[ Semana | Calendário | Kanban ]`):**
  - Abre na visão **`Semana`**, com bandeja horizontal **`SEM DIA`** no topo e grade rolável para os **7 dias completos (`248px` por coluna)** divididos nos 4 períodos (*Qualquer hora, Manhã, Tarde, Noite*), zonas tracejadas nos espaços vazios e selo `ONTEM` / `ATRASADA` em atrasos pendentes.
  - **Manipulação direta e gestos sem conflito:** check circular `○ -> ✓` em 1 clique, pílula de status clicável, criação de projetos (`+ Projeto`), atalho `Esta semana` e arraste fluido que só ativa depois de `6px` de movimento para nunca atrapalhar o clique simples.
  - **Menu de contexto flutuante (clique direito):** posicionado dentro dos limites da janela e com submenus que invertem para a esquerda perto da borda direita.
  - **Kanban e Modais:** rolagem horizontal automática em telas estreitas no Kanban, modal `Nova / Editar tarefa` com editor multi-linha real (`Enter` quebra linha), setas rápidas `< >` na data, e modal `Card do Dia` ao clicar numa data no Calendário.
- **Hábitos:** círculos limpos (`30px`, sem anel duplo), exclusão direta por linha com opção de desfazer, gatilho tracejado `+ Novo hábito` que expande formulário inline com 7 cores, barra de progresso mensal e calendário com cabeçalho `DOM..SÁB`.
- **Segundo Cérebro:** grade adaptativa (4, 3 ou 2 colunas conforme a largura), primeiro cartão tracejado `+ Nova nota`, abas de tipo com contagem, filtros por `#tag` em pílulas pastel, data (`dd/mm`) em cada cartão e modal multi-linha com tags removíveis, campo de URL em links e preview formatado de títulos e listas.
- **Agenda (`[ Calendário | Lista ]`):** cliques nos dias abrem o *Card do Dia* para consultar, editar ou excluir compromissos; aba **`Lista`** exibe toda a agenda do mês em sequência rolável com categoria e ações; legenda inferior filtra categorias por clique e o formulário inclui chave `Dia todo`.
- **Modo Foco e Ajustes:** anel de progresso em `Canvas` com marcações horárias e brilho suave, cronômetro em `JetBrains Mono`, trilhas sonoras integradas com barra de volume estilizada, além de **Ajustes** com amostras visuais nos 5 temas e cartão **`Perfil & Ritmo de Foco`** para personalizar seu nome e o bloco Pomodoro padrão.

---

## 2. Tipografia Estática, Iconografia em 2 Camadas e Geometria

- **Motor gráfico padrão (`wgpu`):** `estudy-desktop` e `estudy-preview` usam renderização acelerada por `wgpu` para suavidade nas fontes, gradientes e animações a 60–144 FPS, mantendo o consumo privado de memória (`RssAnon`, memória alocada só do processo) dentro do limite de `45 MiB` (também há modo por CPU com `--no-default-features --features cpu`).
- **9 fontes TrueType estáticas embutidas (`crates/estudy-ui-kit/assets/fonts/`):**
  - `Newsreader 16pt` (`Regular`, `Medium`, `Italic` real) — títulos editoriais, subtítulos em itálico na mesma linha de base e números grandes.
  - `DM Sans` (`Regular`, `Medium`, `SemiBold`, `Bold`) — textos da interface, cartões, botões e formulários.
  - `JetBrains Mono` (`Regular`, `Medium`) — relógios, datas e contadores tabulares alinhados.
  - Todos os helpers de texto aplicam `Shaping::Advanced` (motor de modelagem tipográfica HarfRust) para kerning correto também em strings ASCII.
- **Iconografia em duas camadas (`crates/estudy-ui-kit/src/icons.rs`):**
  - **`IllustratedIcon` (17 SVGs multi-cor em cache):** ícones com preenchimento próprio para módulos da sidebar, períodos do dia, prioridades e medalhão do **Mochi**. O helper `illustrated_svg_themed` adapta os traços escuros (`#2B241E`) para creme suave automaticamente no Modo Escuro.
  - **`Icon` (89 SVGs monoline `1.75px`):** traços limpos em `14px / 16px / 20px / 24px`, incluindo `Check` e `Circle`.
- **Primitivos visuais (`estudy-ui-kit`):** `radio_check_circle`, `dashed_frame`, `strikethrough_text`, `interactive_card_style`, `gradient_card_style`, `slider_style`, `subtle_scrollbar_style`, bordas suaves em `8%–16%` de opacidade e `SparklineCanvas` com curvas Bézier suaves e gradiente vertical.

---

## 3. Os 5 Presets de Tema (10 Variações Verificadas em WCAG 2.2 AA)

As cores centrais do **Oliva Orgânico** e dos demais 4 presets preservam contraste mínimo `4.5:1` validado por testes automáticos antes de cada entrega.

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
