# ADR-0003: Razão Contábil Append-Only com Cadeia HMAC-SHA256 na Carteira

* **Status:** aceito
* **Data:** 2026-11-22
* **Decisores:** Equipe eStudy
* **Módulos Afetados:** `estudy-domain`, `estudy-storage`

## 1. Contexto e Declaração do Problema
Como o banco SQLite fica no disco do próprio usuário, guardar o saldo de moedas em uma coluna simples tornaria trivial alterar o valor manualmente. Por outro lado, implementar criptografia pesada com infraestrutura externa seria desproporcional para um aplicativo local-first.

## 2. Direcionadores de Decisão (Drivers)
* Impedir adulteração casual direta no arquivo `.db` sem penalizar o desempenho de inicialização (< 1 ms).
* Garantir que **nenhum dado de estudo, projeto ou leitura** seja perdido mesmo se a cadeia da carteira for quebrada.
* Evitar inflação econômica por cliques repetidos usando teto diário e chaves de idempotência.

## 3. Opções Consideradas
* **Opção 1:** Razão append-only (`wallet_ledger`) com cadeia encadeada HMAC-SHA256 (`seq`, `prev_hash`, `hmac_sig`)
* **Opção 2:** Coluna numérica simples `coins` no perfil do usuário
* **Opção 3:** Validação de saldo em servidor remoto

## 4. Decisão Escolhida
Escolhida a **Opção 1 (cadeia HMAC-SHA256 encadeada)**, porque mantém o aplicativo 100% local e gratuito enquanto detecta imediatamente alterações, remoções ou inserções manuais no banco.

## 5. Prós e Contras das Opções

### Cadeia HMAC-SHA256 no `wallet_ledger`
* Boa, porque torna o saldo derivado do histórico auditável, verifica milhares de linhas em menos de 1 ms e aceita lançamentos legítimos via `--admin` (`admin_grant`).
* Ruim, porque a chave derivada fica embarcada no binário local (proteção contra edição direta no SQLite, não contra engenharia reversa avançada).

### Coluna simples `coins`
* Boa, porque exige menos código.
* Ruim, porque perde histórico de transações, idempotência e qualquer resistência contra alteração manual.

### Servidor remoto
* Boa, porque oferece autoridade externa.
* Ruim, porque quebra o princípio local-first e gera custo de infraestrutura.

## 6. Consequências e Trade-offs
* **Impacto Positivo:** Rastreabilidade completa de ganhos/gastos, teto diário (`150` moedas/dia em fontes recorrentes) e proteção contra lançamentos duplicados via `idempotency_key`.
* **Impacto Negativo / Débito Aceito:** Em caso de quebra da cadeia, a loja congela novas compras até que o usuário restaure um backup íntegro ou reinicie a carteira.
* **Ações Mitigatórias:** Testes automatizados garantem detecção imediata de adulteração de valores e lacunas de sequência.
