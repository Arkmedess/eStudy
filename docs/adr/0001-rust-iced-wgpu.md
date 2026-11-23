# ADR-0001: Stack Desktop Nativa em Rust (Edition 2024) + Iced (`wgpu`)

* **Status:** aceito
* **Data:** 2026-11-22
* **Decisores:** Equipe eStudy
* **Módulos Afetados:** Workspace raiz, futura camada `estudy-ui` / `estudy-ui-kit`

## 1. Contexto e Declaração do Problema
O aplicativo precisa reunir projetos, calendário, estudos, leitura e gamificação em uma interface fluida e personalizável sem repetir o alto consumo de memória RAM de aplicações baseadas em Electron/Chromium ou dependências externas pesadas de runtime.

## 2. Direcionadores de Decisão (Drivers)
* Orçamento de memória RAM em repouso entre **20 e 45 MB**.
* Uso de CPU próximo de zero quando a janela estiver ociosa.
* Renderização vetorial nativa (SVG e Canvas para gráficos, timer e heatmap) e distribuição em binário único multiplataforma.

## 3. Opções Consideradas
* **Opção 1:** Rust + `Iced` (`wgpu`, arquitetura Elm/MVU)
* **Opção 2:** Tauri (Rust + WebView do sistema)
* **Opção 3:** Electron (Node.js + Chromium)

## 4. Decisão Escolhida
Escolhida a **Opção 1 (Rust + `Iced`)**, porque elimina processos WebView/Chromium do consumo de memória, unifica a base de código em uma única linguagem segura em tempo de compilação e fornece arquitetura MVU testável sem precisar subir GPU nos testes de estado.

## 5. Prós e Contras das Opções

### Rust + Iced
* Boa, porque mantém consumo de RAM baixo (~20–45 MB), compila para binário único e oferece Canvas e SVG nativos.
* Ruim, porque exige construir componentes customizados (como gavetas e gráficos) diretamente em `estudy-ui-kit`.

### Tauri
* Boa, porque aproveita HTML/CSS existentes.
* Ruim, porque depende do WebView instalado no sistema operacional e eleva o consumo base de RAM acima de 100 MB.

### Electron
* Boa, porque possui ecossistema web amplo.
* Ruim, porque viola diretamente o orçamento de memória RAM e distribui um navegador inteiro junto ao aplicativo.

## 6. Consequências e Trade-offs
* **Impacto Positivo:** Baixo consumo de RAM e CPU, zero runtime JavaScript/Node.js e separação clara entre estado testável e renderização.
* **Impacto Negativo / Débito Aceito:** Componentes ricos de interface exigem implementação própria no crate `estudy-ui-kit` com sandbox visual (`estudy-preview`).
* **Ações Mitigatórias:** Construir um UI kit modular com tokens verificados em WCAG 2.2 AA antes de montar as telas finais.
