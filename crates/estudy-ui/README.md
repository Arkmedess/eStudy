# `estudy-ui`

## Propósito
Implementa a interface MVU do **eStudy** desacoplada de janela (`EstudyApp`), com moldura limpa sem Topbar global nem segunda sidebar, Sidebar esquerda (`216px` expandida $\leftrightarrow$ `58px` trilho de ícones ilustrados), Barra de Comando (`Ctrl+K`) e módulos fluidos com manipulação direta.

## Contrato Público
- `EstudyApp`, `AppMessage`, `AppRoute`: shell raiz testável sem abrir janela gráfica, com `ClockPort` injetado, rastreio de cursor para menus contextuais / Drag & Drop e pilha previsível da tecla `Esc`.
- `features::home`: Hero acolhedor com pílulas de criação rápida (`+ Tarefa`, `+ Hábito`, `+ Compromisso`, `+ Nota`), faixa central de 3 cartões contrastantes (Terracota, Musgo Escuro com `SparklineCanvas` e Mini-Calendário) e 3 colunas vivas (*Tarefas, Hábitos, Segundo cérebro*).
- `features::projects` (`Tarefas`): modos `[ Semana | Calendário | Kanban ]`, bandeja `SEM DIA`, 4 períodos (`Qualquer hora`, `Manhã`, `Tarde`, `Noite`), check e status em 1 clique no cartão, Drag & Drop entre dias/períodos, Menu de Contexto no cursor (botão direito), Modal `Nova/Editar tarefa` multi-linha real (`text_editor`) e Card do Dia.
- `features::habits`, `features::second_brain` (grade 4 colunas com cartão `+ Nova nota` e modal `text_editor` multi-linha), `features::calendar` (`[ Calendário | Lista ]`) e `features::focus`.

## Testes
```bash
cargo test -p estudy-ui
```
