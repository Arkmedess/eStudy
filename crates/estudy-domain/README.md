# `estudy-domain`

## Propósito
Concentra as regras puras de negócio do **eStudy**, totalmente isoladas de banco de dados, sistema de arquivos, rede ou interface gráfica.

## Contrato Público
- `EntityId`: identificador cronologicamente ordenável baseado em `UUIDv7` (`new_v7`, `parse`).
- `ClockPort` (`SystemClock`, `FixedClock`): abstração de relógio UTC para manter regras de negócio e testes independentes do relógio do sistema.
- `EventBusPort` (`InMemoryEventBus`, `DomainEvent`): publicação desacoplada de eventos após a confirmação da transação no banco.
- `WalletChain`, `LedgerEntryType`, `WalletEntryDraft`, `SignedWalletEntry`: derivação de chave por perfil, teto diário anti-inflação (`DAILY_COIN_CAP = 150`), assinatura encadeada e verificação da cadeia HMAC-SHA256.

## Regras e Limites
- `unsafe_code = "forbid"` em todo o crate.
- Fontes recorrentes (`DailyCheckin`, `PomodoroFocus`, `RoutineGoal`) respeitam o teto de `150` moedas por dia; conquistas (`AchievementReward`) e ajustes administrativos (`AdminGrant`) são isentos do teto.
- A verificação da cadeia rejeita lacunas de sequência, divergência de `prev_hash`, assinatura inválida e saldo acumulado negativo.

## Trade-offs Descartados
- **Não** usar `Utc::now()` diretamente dentro das regras de assinatura: o instante sempre chega pelo `WalletEntryDraft` ou `ClockPort` para manter testes reproduzíveis e viabilizar o relógio virtual nos modos `--admin` e `--e2e`.

## Testes
```bash
cargo test -p estudy-domain
```
