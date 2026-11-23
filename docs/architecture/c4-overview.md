# Arquitetura C4 e Fronteiras do Workspace — eStudy

Este documento descreve como os módulos do **eStudy** se organizam em um grafo acíclico direcionado (DAG) sem ciclos de dependência, seguindo **Arquitetura Hexagonal (Ports & Adapters)** combinada com **Fatias Verticais** na camada de interface.

---

## 1. Nível 1 — Contexto de Sistema

```mermaid
graph TD
    User["Usuário (Estudante / Pesquisador)"]
    AI["Agente de IA Externo (Claude / Cursor / Antigravity)"]
    App["eStudy Desktop (Binário Rust)"]
    MCP["estudy-mcp (Servidor stdio)"]
    DB[("SQLite Local WAL + FTS5")]
    Cloud["Pasta de Nuvem do Usuário (Snapshot .db)"]
    OpenLib["Open Library API (Sob Demanda)"]

    User -->|"Usa interface Iced e Ctrl+K"| App
    AI -->|"Invoca Tools/Resources via stdio"| MCP
    App -->|"Lê e grava via casos de uso"| DB
    MCP -->|"Passa pelas mesmas regras de domínio"| DB
    App -->|"Exporta snapshot limpo e Markdown"| Cloud
    App -->|"Busca capas em cache LRU"| OpenLib
```

---

## 2. Nível 2 — Topologia do Cargo Workspace (v0.0.1 $\rightarrow$ v1.0.0)

Na etapa atual (`v0.0.1`), estão ativos os crates **`estudy-domain`** e **`estudy-storage`**. Os demais crates entram nas etapas seguintes respeitando a mesma direção de dependências:

```mermaid
flowchart TB
    Desktop["apps/estudy-desktop\n(Composition Root + Janela Iced)"]
    McpBin["apps/estudy-mcp\n(Servidor MCP stdio)"]
    Preview["apps/estudy-preview\n(Sandbox de componentes)"]

    UI["crates/estudy-ui\n(Fatias MVU + Ctrl+K)"]
    UIKit["crates/estudy-ui-kit\n(EstudyTheme, 5 Presets, SVGs, Canvas)"]
    McpCore["crates/estudy-mcp-server\n(Tools, Resources, Permissões & Auditoria)"]

    Domain["crates/estudy-domain [ATIVO v0.0.1]\n(Entidades, ClockPort, EventBus, Cadeia HMAC)"]

    Storage["crates/estudy-storage [ATIVO v0.0.1]\n(rusqlite WAL, FTS5, Migrações, Snapshot Nuvem)"]
    Audio["crates/estudy-audio\n(DSP Procedural + Canal Lock-Free)"]
    Integrations["crates/estudy-integrations\n(Open Library + Updater SHA-256)"]

    Desktop --> UI
    Desktop --> Storage
    Desktop --> Audio
    Desktop --> Integrations
    Desktop --> Domain

    McpBin --> McpCore
    McpBin --> Storage
    McpBin --> Domain
    McpCore --> Domain

    Preview --> UIKit
    UI --> UIKit
    UI --> Domain

    Storage --> Domain
    Audio --> Domain
    Integrations --> Domain
```

---

## 3. Regras de Fronteira

1. **`estudy-domain` é puro:** não depende de `rusqlite`, `directories`, sistema de arquivos, rede ou `Iced`. Recebe tempo via `ClockPort` e notifica efeitos colaterais pós-commit via `EventBusPort`.
2. **`estudy-storage` depende apenas de `estudy-domain`:** traduz modelos e regras do domínio para transações SQLite seguras sem expor detalhes SQL para as camadas superiores.
3. **Nenhuma operação de I/O entra no domínio:** futuras integrações (`estudy-audio`, `estudy-integrations`, `estudy-ui`) seguem a mesma inversão de dependência.
