# Design System — eStudy

Documento oficial de interface, tokens visuais, tipografia, geometria e acessibilidade do **eStudy** (`Iced` + `wgpu`).

---

## 1. Direção Visual e Navegação

O eStudy combina a calma tátil de uma estação pessoal de estudos com hierarquia editorial clara, acesso rápido por teclado (`Ctrl+K`) e organização por módulos sem poluição visual.

- **Barra lateral fixa fina (`220px`, recolhível com `Ctrl+B`):** lockup editorial `eStudy` no topo, grupos de navegação claros (*Seu Espaço*: Início, Projetos & Tarefas com badge de abertas, Hábitos, Segundo Cérebro, Agenda; *Aprofundamento*: Estudos, Estante, Financeiro, Pomodoro; *Companhia*: Jardim & Pets, Loja) e **cartão do Mascote** no rodapé com fala contextual + atalho para Ajustes.
- **Barra superior limpa (`44px`):** botão de recolher sidebar, breadcrumb da rota ativa (`eStudy / Início`), busca rápida (`Ctrl+K`), botão primário **Focar** (com timer ao vivo), notificações, seletor rápido de preset e alternador Claro/Escuro.
- **Home em Painel Bento Editorial:**
  - *Cabeçalho editorial:* saudação em serifa (`Lora`), resumo vivo do dia e pílulas de salto rápido (*Timeline Hoje, Hábitos, Segundo Cérebro, Semana*).
  - *Faixa Bento Hero assimétrica (`5fr / 7fr`):* cartão tonal de **Pulso de Foco (14 dias)** com gráfico de linha/área `SparklineCanvas` e cartão de **Ritmo anual (365 dias)** com `HeatmapGrid` e legenda `Menos ■■■■■ Mais`.
  - *Grade principal (`8fr / 4fr`):* à esquerda, grade de **Projetos** com `category_tile` colorido por categoria e contadores com bolinhas por status + cartão **Hoje**; à direita, **Perfil + medalhas**, **Em foco (`X/5`)** com disparo direto de Pomodoro, **Diário de Humor** (7 dias / ano) e **Quadro Personalizável** com 7 molduras (*Polaroid, Photocard, Clean, Moderna, Japandi, Museu, Nenhum*).
- **Gavetas flutuantes à direita (`20px` de raio, margem externa `16px`):** abrir uma tarefa ou clicar em "Timeline Hoje" exibe um painel flutuante arredondado (incluindo faixa semanal, blocos sem horário tintados, timeline vertical em cápsulas proporcionais, janelas livres e régua vermelha `Agora · HH:MM`).
- **Modo Foco em tela cheia (`Esc` sai):** oculta menus e colunas, deixando apenas o anel do Pomodoro, o Pet estudando junto e os controles mínimos de áudio.

---

## 2. Tipografia, Geometria, Cores de Categoria e Mídia Leve

- **Fontes embutidas no binário (`crates/estudy-ui-kit/assets/fonts/`):**
  - `Plus Jakarta Sans` (variável OFL) — família padrão de interface, controles e listas.
  - `Lora` (variável OFL) — família editorial serifada para saudações, títulos de página e citações.
  - `JetBrains Mono` (variável OFL) — temporizadores, horários, contadores e badges numéricos sem tremulação.
- **Paleta semântica de 7 categorias (`CategoryColor`):** `Sage`, `Terracotta`, `Slate`, `Ochre`, `Lavender`, `Moss` e `Coral` — cada uma calibrada para modo Claro e Escuro com variantes `.solid(theme)`, `.tint(theme)` e `.border(theme)` para projetos, tags, hábitos, notas e agenda.
- **Espaçamento e raios:** grade base de `4px` (`SPACE_1` a `SPACE_8`) e raios concêntricos suaves (`20px` para gavetas flutuantes, `16px` para cartões principais e modais, `12px` para painéis internos, `10px` para controles e entradas de texto, `6px` para chips de atalho e `999px` para pílulas/filtros).
- **Primitivos do UI Kit (`crates/estudy-ui-kit`):** `editorial_heading`, `category_tile`, `category_pill`, `floating_drawer`, `SparklineCanvas`, `HeatmapGrid`, `card`, `tonal_card`, `soft_surface`, `hairline_divider`, `segmented_switch`, `pill_chip`, `progress_track`, `metric_stat`, `empty_placeholder`, `banner_toast`, `kbd_hint`, `drawer_backdrop` e `modal_surface`.
- **Iconografia:** catálogo único com 87 ícones Lucide em SVG vetorial (zero emojis na interface).
- **Sprites 2D do Jardim e Pets:** arquivos WebP artesanais (`128×128px`, 2 a 4 frames leves por estado), carregados sob demanda apenas para o que está visível na tela (teto de 1,5 MB de RAM) e animados a 4–6 FPS somente quando a janela está ativa.
- **Ícones de Conquistas:** arquivos WebP `64×64px` em 4 raridades (*Bronze, Prata, Ouro, Platina*).

---

## 3. Os 5 Presets de Tema (10 Variações Verificadas em WCAG 2.2 AA)

Todos os presets passam por verificação automática de contraste mínimo `4.5:1` entre `accent` e `panel`/`card` antes de cada release.

| Preset | Modo | `accent` | `on_accent` | `bg` | `panel` | `card` | Contraste `accent/panel` |
| :--- | :---: | :--- | :--- | :--- | :--- | :--- | :---: |
| **1. Oliva Orgânico** *(Padrão)* | Light | `#4a5d4e` | `#ffffff` | `#f7f5f0` | `#efece4` | `#ffffff` | `6.00:1` |
| **1. Oliva Orgânico** | Dark | `#a3b899` | `#121411` | `#121411` | `#1a1d18` | `#21251f` | `8.01:1` |
| **2. eStudy Menta** | Dark | `#86efac` | `#070709` | `#070709` | `#0f1015` | `#13141a` | `13.53:1` |
| **2. eStudy Menta** | Light | `#15803d` | `#ffffff` | `#f8fafc` | `#f1f5f9` | `#ffffff` | `4.58:1` |
| **3. Cyber Minimalist** | Dark | `#38bdf8` | `#040711` | `#040711` | `#0a1020` | `#0f172a` | `8.85:1` |
| **3. Cyber Minimalist** | Light | `#0369a1` | `#ffffff` | `#f0f7ff` | `#e0f0fe` | `#ffffff` | `5.10:1` |
| **4. Solar Amber** | Dark | `#fbbf24` | `#0e0b08` | `#0e0b08` | `#17130e` | `#1f1a13` | `11.07:1` |
| **4. Solar Amber** | Light | `#92400e` | `#ffffff` | `#fef9ee` | `#fef3c7` | `#ffffff` | `6.37:1` |
| **5. Velvet Sakura** | Dark | `#f472b6` | `#0d080b` | `#0d080b` | `#160e13` | `#1d1319` | `7.17:1` |
| **5. Velvet Sakura** | Light | `#be185d` | `#ffffff` | `#faf7f5` | `#fce7f3` | `#ffffff` | `5.14:1` |
