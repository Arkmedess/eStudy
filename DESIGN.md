# Design System — eStudy

Documento oficial de interface, tokens visuais, tipografia, geometria e acessibilidade do **eStudy** (`Iced` + `wgpu`).

---

## 1. Direção Visual e Navegação

O eStudy evita barras laterais pesadas de navegação. A interface prioriza respiro visual, contraste claro e acesso direto pelo teclado (`Ctrl+K`).

- **Barra superior discreta (`48px`):** marca `eStudy` à esquerda (retorna à Home); à direita apenas busca/comandos (`Ctrl+K`), atalho rápido de Pomodoro/Foco, notificações de conquistas e menu compacto de módulos (*Estudos, Estante, Jardim & Pets, Loja, Configurações*).
- **Home em 3 colunas:**
  - *Esquerda:* perfil local, vitrine com até 3 medalhas fixadas e cartão **"Hoje"** (próximos blocos horários, prazos e atalhos para *Semana* e *Calendário*).
  - *Centro:* mapa de calor anual (365 dias) com o Pet no canto e grade de Projetos em 2 colunas com contadores por status.
  - *Direita:* lista **"Em foco"** (até 5 itens pinados), **Diário de Humor** (7 dias + visão anual) e **Quadro Personalizável** com molduras (*Polaroid, Photocard, Clean, Moderna, Japandi, Museu, Nenhum*).
- **Barra esquerda contextual (`240px`, recolhível com `Ctrl+B`):** aparece apenas dentro de telas como Projetos e Calendário, exibindo conteúdo útil daquela visão (mini-calendário mensal, resumo "Hoje" ou filtros por projeto).
- **Gavetas deslizantes à direita:** abrir uma tarefa ou clicar em "Hoje" abre um painel sobreposto à direita sem fazer o usuário perder o contexto da tela principal.
- **Modo Foco em tela cheia (`Esc` sai):** oculta menus e colunas, deixando apenas o anel do Pomodoro, o Pet estudando junto e os controles mínimos de áudio.

---

## 2. Tipografia, Geometria e Mídia Leve

- **Fontes:**
  - `Inter` — textos e controles da interface.
  - `JetBrains Mono` (`tabular-nums`) — temporizadores, contadores e badges numéricos para evitar tremulação durante atualizações.
  - `Source Serif 4` — restrita a citações de livros e destaques editoriais.
- **Espaçamento e raios:** grade base de `4px` (`SPACE_1` a `SPACE_8`) e raios concêntricos suaves (`16px` para cartões principais e modais, `12px` para painéis internos, `10px` para controles e entradas de texto, `6px` para chips de atalho e `999px` para pílulas/filtros).
- **Primitivos do UI Kit (`crates/estudy-ui-kit`):** cartões elevados e tonais (`card`, `tonal_card`), superfícies macias (`soft_surface`), divisores finos (`hairline_divider`), seletores em cápsula (`segmented_switch`, `pill_chip`), barras de progresso (`progress_track`), métricas (`metric_stat`), estados vazios (`empty_placeholder`), banners (`banner_toast`), atalhos (`kbd_hint`) e envelopes padrão de drawer/modal (`drawer_backdrop`, `modal_surface`).
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
