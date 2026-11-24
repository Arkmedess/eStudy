//! Barra de Comando (`Ctrl+K`) com busca global e ações diretas do eStudy.

use estudy_domain::EntityId;
use estudy_ui_kit::widgets::{
    empty_placeholder, hairline_divider, icon_svg, kbd_hint, modal_surface,
};
use estudy_ui_kit::{EstudyTheme, Icon, SPACE_2, SPACE_3};
use iced::widget::{button, column, row, scrollable, text, text_input};
use iced::{Alignment, Element, Length};

/// Rotas principais navegáveis pela Barra de Comando e Topbar.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AppRoute {
    /// Home pessoal em 3 colunas.
    Home,
    /// Projetos (Kanban + frameworks leves).
    Projects,
    /// Calendário unificado e Timeline Hoje.
    Calendar,
    /// Módulo de Estudos.
    Studies,
    /// Estante de Leituras.
    Bookshelf,
    /// Jardim interativo e Pet companheiro.
    GardenAndPets,
    /// Loja modular de cosméticos.
    Shop,
    /// Configurações e backups.
    Settings,
}

impl AppRoute {
    /// Nome legível da rota para menus e busca.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Home => "Home",
            Self::Projects => "Projetos",
            Self::Calendar => "Calendário",
            Self::Studies => "Estudos",
            Self::Bookshelf => "Estante",
            Self::GardenAndPets => "Jardim & Pets",
            Self::Shop => "Loja",
            Self::Settings => "Configurações",
        }
    }
}

/// Ação executável a partir de um item da Barra de Comando.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PaletteAction {
    /// Navega para uma tela do aplicativo.
    Navigate(AppRoute),
    /// Abre o Drawer da Timeline "Hoje".
    OpenTodayTimeline,
    /// Abre um projeto específico em Projetos.
    OpenProject(EntityId),
    /// Abre o Drawer de uma tarefa específica.
    OpenTask(EntityId),
    /// Inicia um bloco de foco Pomodoro com duração em minutos.
    StartFocus(u16),
    /// Cria uma tarefa diretamente (ou abre Projetos se título vazio).
    QuickCreateTask(Option<String>),
    /// Cria um evento diretamente (ou abre formulário no Calendário).
    QuickCreateEvent(Option<String>),
    /// Registra humor do dia.
    LogMoodQuick,
    /// Sorteia a próxima leitura da estante TBR.
    RandomTbrBook,
    /// Registra avanço rápido de leitura em páginas.
    QuickReadingProgress(u16),
    /// Dispara exportação de snapshot limpo para a nuvem.
    ExportCloudSnapshot,
    /// Alterna entre os 5 presets de tema.
    CycleThemePreset,
    /// Alterna entre modo claro e escuro.
    ToggleThemeMode,
}

/// Comando ou resultado de busca exibido na Barra de Comando (`Ctrl+K`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PaletteCommand {
    /// Identificador único do item.
    pub id: String,
    /// Texto principal exibido ao usuário.
    pub label: String,
    /// Descrição curta da ação ou contexto do item encontrado.
    pub hint: String,
    /// Atalho de teclado associado, se houver.
    pub shortcut: Option<&'static str>,
    /// Ícone vetorial SVG do catálogo.
    pub icon: Icon,
    /// Ação executada ao confirmar.
    pub action: PaletteAction,
}

/// Mensagens da Barra de Comando.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PaletteMessage {
    /// Alterna abertura/fechamento via `Ctrl+K` ou botão.
    Toggle,
    /// Fecha explicitamente a barra (`Esc` ou clique fora).
    Close,
    /// Atualiza o termo de busca digitado.
    QueryChanged(String),
    /// Move seleção para cima.
    SelectPrev,
    /// Move seleção para baixo.
    SelectNext,
    /// Executa o comando selecionado atualmente.
    ExecuteSelected,
    /// Executa diretamente o comando no índice filtrado informado.
    ExecuteIndex(usize),
}

/// Estado puro da Barra de Comando (`Ctrl+K`).
#[derive(Debug, Clone)]
pub struct CommandPaletteState {
    /// Indica se a paleta está visível.
    pub is_open: bool,
    /// Texto digitado no campo de busca/comando.
    pub query: String,
    /// Índice selecionado dentro da lista filtrada atual.
    pub selected_index: usize,
    /// Catálogo fixo de comandos disponíveis.
    pub commands: Vec<PaletteCommand>,
    /// Itens dinâmicos vindos da busca global (projetos, tarefas, eventos, notas `FTS5`).
    pub search_entries: Vec<PaletteCommand>,
}

impl Default for CommandPaletteState {
    fn default() -> Self {
        Self::new()
    }
}

impl CommandPaletteState {
    /// Inicializa a Barra de Comando com os atalhos e ações do eStudy.
    #[must_use]
    pub fn new() -> Self {
        Self {
            is_open: false,
            query: String::new(),
            selected_index: 0,
            commands: default_commands(),
            search_entries: Vec::new(),
        }
    }

    /// Abre a paleta limpando a busca anterior.
    pub fn open(&mut self) {
        self.is_open = true;
        self.query.clear();
        self.selected_index = 0;
    }

    /// Fecha a paleta e limpa o filtro.
    pub fn close(&mut self) {
        self.is_open = false;
        self.query.clear();
        self.selected_index = 0;
    }

    /// Alterna entre aberta e fechada.
    pub fn toggle(&mut self) {
        if self.is_open {
            self.close();
        } else {
            self.open();
        }
    }

    /// Atualiza a consulta de busca reposicionando o cursor de seleção no início.
    pub fn set_query(&mut self, query: String) {
        self.query = query;
        self.selected_index = 0;
    }

    /// Atualiza a lista de itens pesquisáveis (projetos, tarefas, eventos e hits `FTS5`).
    pub fn set_search_entries(&mut self, entries: Vec<PaletteCommand>) {
        self.search_entries = entries;
    }

    /// Retorna comandos e resultados de busca que casam com a consulta atual.
    #[must_use]
    pub fn filtered_commands(&self) -> Vec<PaletteCommand> {
        let trimmed = self.query.trim();
        if let Some(inline_cmd) = parse_inline_action_command(trimmed) {
            return vec![inline_cmd];
        }

        let needle = trimmed.to_lowercase();
        if needle.is_empty() {
            return self.commands.clone();
        }

        let mut results: Vec<PaletteCommand> = self
            .commands
            .iter()
            .filter(|cmd| matches_needle(cmd, &needle))
            .cloned()
            .collect();

        if !trimmed.starts_with('>') {
            results.extend(
                self.search_entries
                    .iter()
                    .filter(|entry| matches_needle(entry, &needle))
                    .cloned(),
            );
        }

        results
    }

    /// Move a seleção para o item anterior de forma circular.
    pub fn select_prev(&mut self) {
        let count = self.filtered_commands().len();
        if count == 0 {
            self.selected_index = 0;
            return;
        }
        self.selected_index = if self.selected_index == 0 {
            count - 1
        } else {
            self.selected_index - 1
        };
    }

    /// Move a seleção para o próximo item de forma circular.
    pub fn select_next(&mut self) {
        let count = self.filtered_commands().len();
        if count == 0 {
            self.selected_index = 0;
            return;
        }
        self.selected_index = (self.selected_index + 1) % count;
    }

    /// Retorna a ação do item selecionado (ou do índice filtrado informado).
    #[must_use]
    pub fn resolve_action(&self, index: Option<usize>) -> Option<PaletteAction> {
        let filtered = self.filtered_commands();
        let target_idx = index.unwrap_or(self.selected_index);
        filtered.get(target_idx).map(|cmd| cmd.action.clone())
    }

    /// Renderiza o overlay modal da Barra de Comando.
    pub fn view<'a, Message: Clone + 'a>(
        &'a self,
        theme: EstudyTheme,
        map_msg: impl Fn(PaletteMessage) -> Message + Clone + 'a,
    ) -> Element<'a, Message> {
        let filtered = self.filtered_commands();

        let query_input = text_input(
            "Buscar notas, tarefas, projetos ou digitar > comando...",
            &self.query,
        )
        .on_input({
            let map_msg = map_msg.clone();
            move |value| map_msg(PaletteMessage::QueryChanged(value))
        })
        .on_submit(map_msg(PaletteMessage::ExecuteSelected))
        .padding(SPACE_3)
        .size(15)
        .style(move |_, status| theme.soft_input_style(status));

        let header = row![
            icon_svg(Icon::Command, 18.0, theme.accent),
            query_input,
            button(icon_svg(Icon::X, 16.0, theme.text_muted))
                .on_press(map_msg(PaletteMessage::Close))
                .padding(SPACE_2)
                .style(move |_, status| theme.subtle_button_style(status)),
        ]
        .spacing(SPACE_2)
        .align_y(Alignment::Center);

        let body: Element<'a, Message> = if filtered.is_empty() {
            empty_placeholder(
                Icon::Search,
                "Nenhum resultado encontrado",
                "Tente outro termo ou digite > nova tarefa / > novo evento.",
                None,
                theme,
            )
        } else {
            let mut list = column![].spacing(4.0);
            for (idx, cmd) in filtered.iter().enumerate() {
                let is_selected = idx == self.selected_index;
                let mut row_content = row![
                    icon_svg(cmd.icon, 16.0, theme.accent),
                    column![
                        text(cmd.label.clone()).size(14).color(theme.text_primary),
                        text(cmd.hint.clone()).size(12).color(theme.text_muted),
                    ]
                    .spacing(2.0)
                    .width(Length::Fill),
                ]
                .spacing(SPACE_3)
                .align_y(Alignment::Center);

                if let Some(shortcut) = cmd.shortcut.filter(|s| !s.is_empty()) {
                    row_content = row_content.push(kbd_hint(shortcut, theme));
                }

                let item_btn = button(row_content)
                    .on_press(map_msg(PaletteMessage::ExecuteIndex(idx)))
                    .width(Length::Fill)
                    .padding([SPACE_2, SPACE_3])
                    .style(move |_, status| {
                        let mut style = theme.subtle_button_style(status);
                        if is_selected {
                            style.background = Some(theme.accent_subtle.into());
                            style.border.color = theme.accent;
                            style.border.width = 1.0;
                        }
                        style
                    });
                list = list.push(item_btn);
            }
            scrollable(list).height(Length::Fixed(320.0)).into()
        };

        let footer = row![
            kbd_hint("Enter", theme),
            text("executar").size(11).color(theme.text_muted),
            kbd_hint("Esc", theme),
            text("fechar").size(11).color(theme.text_muted),
            kbd_hint("Ctrl+K", theme),
            text("alternar").size(11).color(theme.text_muted),
        ]
        .spacing(SPACE_2)
        .align_y(Alignment::Center);

        modal_surface(
            column![
                header,
                hairline_divider(theme),
                body,
                hairline_divider(theme),
                footer
            ]
            .spacing(SPACE_3),
            620.0,
            theme,
        )
    }
}

fn matches_needle(cmd: &PaletteCommand, needle: &str) -> bool {
    cmd.label.to_lowercase().contains(needle)
        || cmd.hint.to_lowercase().contains(needle)
        || cmd.id.to_lowercase().contains(needle)
}

fn parse_inline_action_command(trimmed: &str) -> Option<PaletteCommand> {
    if let Some(rest) = trimmed.strip_prefix("> nova tarefa ") {
        let title = rest.trim();
        if !title.is_empty() {
            return Some(PaletteCommand {
                id: "inline-new-task".to_string(),
                label: format!("Criar tarefa: {title}"),
                hint: "Adiciona diretamente no projeto ativo".to_string(),
                shortcut: Some("Enter"),
                icon: Icon::CheckSquare,
                action: PaletteAction::QuickCreateTask(Some(title.to_string())),
            });
        }
    }

    if let Some(rest) = trimmed.strip_prefix("> novo evento ") {
        let title = rest.trim();
        if !title.is_empty() {
            return Some(PaletteCommand {
                id: "inline-new-event".to_string(),
                label: format!("Criar evento hoje: {title}"),
                hint: "Adiciona evento na data selecionada do Calendário".to_string(),
                shortcut: Some("Enter"),
                icon: Icon::Calendar,
                action: PaletteAction::QuickCreateEvent(Some(title.to_string())),
            });
        }
    }

    if let Some(rest) = trimmed.strip_prefix("> focar ") {
        let digits: String = rest
            .trim()
            .trim_end_matches('m')
            .chars()
            .filter(char::is_ascii_digit)
            .collect();
        if let Ok(minutes) = digits.parse::<u16>()
            && minutes > 0
        {
            return Some(PaletteCommand {
                id: "inline-focus".to_string(),
                label: format!("Iniciar foco de {minutes} minutos"),
                hint: "Prepara o bloco de concentração imediato".to_string(),
                shortcut: Some("Enter"),
                icon: Icon::Timer,
                action: PaletteAction::StartFocus(minutes),
            });
        }
    }

    None
}

fn default_commands() -> Vec<PaletteCommand> {
    let mut commands = action_commands();
    commands.extend(navigation_commands());
    commands
}

fn action_commands() -> Vec<PaletteCommand> {
    vec![
        cmd(
            "focus-25",
            "> focar 25m",
            "Iniciar sessão Pomodoro de 25 minutos",
            Some("Alt+F"),
            Icon::Timer,
            PaletteAction::StartFocus(25),
        ),
        cmd(
            "today-timeline",
            "> timeline hoje",
            "Abrir gaveta vertical com blocos e janelas livres de hoje",
            None,
            Icon::Clock,
            PaletteAction::OpenTodayTimeline,
        ),
        cmd(
            "new-task",
            "> nova tarefa",
            "Criar tarefa rápida com prazo e projeto",
            None,
            Icon::CheckSquare,
            PaletteAction::QuickCreateTask(None),
        ),
        cmd(
            "new-event",
            "> novo evento",
            "Adicionar bloco ou compromisso no calendário",
            None,
            Icon::Calendar,
            PaletteAction::QuickCreateEvent(None),
        ),
        cmd(
            "log-mood",
            "> humor",
            "Registrar humor do dia na Home",
            None,
            Icon::Smile,
            PaletteAction::LogMoodQuick,
        ),
        cmd(
            "tbr-pick",
            "> sortear leitura",
            "Escolher próximo livro da fila TBR",
            None,
            Icon::Dices,
            PaletteAction::RandomTbrBook,
        ),
        cmd(
            "reading-10p",
            "> +10p <livro>",
            "Registrar 10 páginas lidas na obra atual",
            None,
            Icon::BookOpen,
            PaletteAction::QuickReadingProgress(10),
        ),
        cmd(
            "cloud-backup",
            "> backup nuvem",
            "Exportar snapshot SQLite limpo para pasta de nuvem",
            None,
            Icon::Cloud,
            PaletteAction::ExportCloudSnapshot,
        ),
        cmd(
            "theme-preset",
            "Alternar preset de tema",
            "Ciclar entre os 5 temas oficiais WCAG AA",
            None,
            Icon::Palette,
            PaletteAction::CycleThemePreset,
        ),
        cmd(
            "theme-mode",
            "Alternar modo claro / escuro",
            "Trocar polaridade Light e Dark preservando o preset",
            None,
            Icon::Sun,
            PaletteAction::ToggleThemeMode,
        ),
    ]
}

fn navigation_commands() -> Vec<PaletteCommand> {
    vec![
        cmd(
            "nav-home",
            "Ir para Home",
            "Visão pessoal em 3 colunas",
            None,
            Icon::Home,
            PaletteAction::Navigate(AppRoute::Home),
        ),
        cmd(
            "nav-projects",
            "Ir para Projetos",
            "Kanban 4 colunas e frameworks leves",
            None,
            Icon::FolderKanban,
            PaletteAction::Navigate(AppRoute::Projects),
        ),
        cmd(
            "nav-calendar",
            "Ir para Calendário",
            "Visão unificada de eventos e prazos",
            None,
            Icon::Calendar,
            PaletteAction::Navigate(AppRoute::Calendar),
        ),
        cmd(
            "nav-studies",
            "Ir para Estudos",
            "Ciclo contínuo, revisões espaçadas e flashcards",
            None,
            Icon::GraduationCap,
            PaletteAction::Navigate(AppRoute::Studies),
        ),
        cmd(
            "nav-bookshelf",
            "Ir para Estante",
            "Leituras, estantes customizáveis e citações",
            None,
            Icon::BookOpen,
            PaletteAction::Navigate(AppRoute::Bookshelf),
        ),
        cmd(
            "nav-garden",
            "Ir para Jardim & Pets",
            "Explorar canteiros e interagir com seu pet",
            None,
            Icon::Sprout,
            PaletteAction::Navigate(AppRoute::GardenAndPets),
        ),
        cmd(
            "nav-shop",
            "Ir para Loja",
            "Trocar moedas por temas, pets, molduras e faixas",
            None,
            Icon::ShoppingBag,
            PaletteAction::Navigate(AppRoute::Shop),
        ),
        cmd(
            "nav-settings",
            "Ir para Configurações",
            "Backup na nuvem, permissões MCP e preferências",
            None,
            Icon::Settings,
            PaletteAction::Navigate(AppRoute::Settings),
        ),
    ]
}

fn cmd(
    id: &str,
    label: &str,
    hint: &str,
    shortcut: Option<&'static str>,
    icon: Icon,
    action: PaletteAction,
) -> PaletteCommand {
    PaletteCommand {
        id: id.to_string(),
        label: label.to_string(),
        hint: hint.to_string(),
        shortcut,
        icon,
        action,
    }
}
