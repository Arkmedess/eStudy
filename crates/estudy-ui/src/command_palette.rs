//! Barra de Comando (`Ctrl+K`) e pesquisa rápida global do eStudy.

use estudy_ui_kit::widgets::{card, icon_svg};
use estudy_ui_kit::{EstudyTheme, Icon, SPACE_2, SPACE_3, SPACE_4};
use iced::widget::{button, column, container, row, scrollable, text, text_input};
use iced::{Alignment, Color, Element, Length};

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
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PaletteAction {
    /// Navega para uma tela do aplicativo.
    Navigate(AppRoute),
    /// Inicia um bloco de foco Pomodoro com duração em minutos.
    StartFocus(u16),
    /// Abre fluxo de criação de tarefa.
    QuickCreateTask,
    /// Abre fluxo de criação de evento no calendário.
    QuickCreateEvent,
    /// Registra humor do dia.
    LogMoodQuick,
    /// Sorteia a próxima leitura da estante TBR.
    RandomTbrBook,
    /// Dispara exportação de snapshot limpo para a nuvem.
    ExportCloudSnapshot,
    /// Alterna entre os 5 presets de tema.
    CycleThemePreset,
    /// Alterna entre modo claro e escuro.
    ToggleThemeMode,
}

/// Comando indexado na Barra de Comando (`Ctrl+K`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PaletteCommand {
    /// Identificador único do comando.
    pub id: &'static str,
    /// Texto exibido ao usuário (ex.: `> focar 25m`).
    pub label: &'static str,
    /// Descrição curta da ação.
    pub hint: &'static str,
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
    /// Catálogo de comandos disponíveis.
    pub commands: Vec<PaletteCommand>,
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

    /// Retorna os comandos que casam com a consulta atual.
    #[must_use]
    pub fn filtered_commands(&self) -> Vec<&PaletteCommand> {
        let needle = self.query.trim().to_lowercase();
        if needle.is_empty() {
            return self.commands.iter().collect();
        }

        self.commands
            .iter()
            .filter(|cmd| {
                cmd.label.to_lowercase().contains(&needle)
                    || cmd.hint.to_lowercase().contains(&needle)
                    || cmd.id.to_lowercase().contains(&needle)
            })
            .collect()
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
        filtered.get(target_idx).map(|cmd| cmd.action)
    }

    /// Renderiza o overlay modal da Barra de Comando.
    pub fn view<'a, Message: Clone + 'a>(
        &'a self,
        theme: EstudyTheme,
        map_msg: impl Fn(PaletteMessage) -> Message + Clone + 'a,
    ) -> Element<'a, Message> {
        let filtered = self.filtered_commands();

        let query_input = text_input(
            "Buscar telas ou digitar comando (ex.: > focar 25m)...",
            &self.query,
        )
        .on_input({
            let map_msg = map_msg.clone();
            move |value| map_msg(PaletteMessage::QueryChanged(value))
        })
        .on_submit(map_msg(PaletteMessage::ExecuteSelected))
        .padding(SPACE_3)
        .size(15);

        let header = row![
            icon_svg(Icon::Command, 18.0, theme.accent),
            query_input,
            button(icon_svg(Icon::X, 16.0, theme.text_muted))
                .on_press(map_msg(PaletteMessage::Close))
                .padding(SPACE_2)
                .style(move |_, status| theme.ghost_button_style(status)),
        ]
        .spacing(SPACE_2)
        .align_y(Alignment::Center);

        let body: Element<'a, Message> = if filtered.is_empty() {
            container(
                column![
                    icon_svg(Icon::Search, 24.0, theme.text_muted),
                    text("Nenhum comando encontrado para esse filtro.")
                        .size(13)
                        .color(theme.text_muted),
                ]
                .spacing(SPACE_2)
                .align_x(Alignment::Center),
            )
            .padding(SPACE_4)
            .width(Length::Fill)
            .into()
        } else {
            let mut list = column![].spacing(SPACE_2);
            for (idx, cmd) in filtered.iter().enumerate() {
                let is_selected = idx == self.selected_index;
                let row_content = row![
                    icon_svg(cmd.icon, 16.0, theme.accent),
                    column![
                        text(cmd.label).size(14).color(theme.text_primary),
                        text(cmd.hint).size(12).color(theme.text_muted),
                    ]
                    .spacing(2.0)
                    .width(Length::Fill),
                    text(cmd.shortcut.unwrap_or(""))
                        .size(11)
                        .color(theme.text_muted),
                ]
                .spacing(SPACE_3)
                .align_y(Alignment::Center);

                let item_btn = button(row_content)
                    .on_press(map_msg(PaletteMessage::ExecuteIndex(idx)))
                    .width(Length::Fill)
                    .padding(SPACE_2)
                    .style(move |_, status| {
                        let mut style = theme.ghost_button_style(status);
                        if is_selected {
                            style.background = Some(theme.accent_subtle.into());
                            style.border.color = theme.accent;
                        }
                        style
                    });
                list = list.push(item_btn);
            }
            scrollable(list).height(Length::Fixed(300.0)).into()
        };

        let modal = card(
            column![
                header,
                body,
                row![
                    text("Enter executa · Esc fecha · Ctrl+K alterna")
                        .size(11)
                        .color(theme.text_muted),
                ]
            ]
            .spacing(SPACE_3),
            theme,
        )
        .width(Length::Fixed(580.0));

        container(modal)
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(48.0)
            .align_x(Alignment::Center)
            .style(|_| container::Style {
                background: Some(Color::from_rgba(0.0, 0.0, 0.0, 0.45).into()),
                ..container::Style::default()
            })
            .into()
    }
}

fn default_commands() -> Vec<PaletteCommand> {
    let mut commands = action_commands();
    commands.extend(navigation_commands());
    commands
}

fn action_commands() -> Vec<PaletteCommand> {
    vec![
        PaletteCommand {
            id: "focus-25",
            label: "> focar 25m",
            hint: "Iniciar sessão Pomodoro de 25 minutos",
            shortcut: Some("Alt+F"),
            icon: Icon::Timer,
            action: PaletteAction::StartFocus(25),
        },
        PaletteCommand {
            id: "new-task",
            label: "> nova tarefa",
            hint: "Criar tarefa rápida com prazo e projeto",
            shortcut: None,
            icon: Icon::CheckSquare,
            action: PaletteAction::QuickCreateTask,
        },
        PaletteCommand {
            id: "new-event",
            label: "> novo evento",
            hint: "Adicionar bloco ou compromisso no calendário",
            shortcut: None,
            icon: Icon::Calendar,
            action: PaletteAction::QuickCreateEvent,
        },
        PaletteCommand {
            id: "log-mood",
            label: "> humor",
            hint: "Registrar humor do dia na Home",
            shortcut: None,
            icon: Icon::Smile,
            action: PaletteAction::LogMoodQuick,
        },
        PaletteCommand {
            id: "tbr-pick",
            label: "> sortear leitura",
            hint: "Escolher próximo livro da fila TBR",
            shortcut: None,
            icon: Icon::Dices,
            action: PaletteAction::RandomTbrBook,
        },
        PaletteCommand {
            id: "cloud-backup",
            label: "> backup nuvem",
            hint: "Exportar snapshot SQLite limpo para pasta de nuvem",
            shortcut: None,
            icon: Icon::Cloud,
            action: PaletteAction::ExportCloudSnapshot,
        },
        PaletteCommand {
            id: "theme-preset",
            label: "Alternar preset de tema",
            hint: "Ciclar entre os 5 temas oficiais WCAG AA",
            shortcut: None,
            icon: Icon::Palette,
            action: PaletteAction::CycleThemePreset,
        },
        PaletteCommand {
            id: "theme-mode",
            label: "Alternar modo claro / escuro",
            hint: "Trocar polaridade Light e Dark preservando o preset",
            shortcut: None,
            icon: Icon::Sun,
            action: PaletteAction::ToggleThemeMode,
        },
    ]
}

fn navigation_commands() -> Vec<PaletteCommand> {
    vec![
        PaletteCommand {
            id: "nav-home",
            label: "Ir para Home",
            hint: "Visão pessoal em 3 colunas",
            shortcut: None,
            icon: Icon::Home,
            action: PaletteAction::Navigate(AppRoute::Home),
        },
        PaletteCommand {
            id: "nav-projects",
            label: "Ir para Projetos",
            hint: "Kanban 4 colunas e frameworks leves",
            shortcut: None,
            icon: Icon::FolderKanban,
            action: PaletteAction::Navigate(AppRoute::Projects),
        },
        PaletteCommand {
            id: "nav-calendar",
            label: "Ir para Calendário",
            hint: "Visão unificada de eventos e prazos",
            shortcut: None,
            icon: Icon::Calendar,
            action: PaletteAction::Navigate(AppRoute::Calendar),
        },
        PaletteCommand {
            id: "nav-studies",
            label: "Ir para Estudos",
            hint: "Ciclo contínuo, revisões espaçadas e flashcards",
            shortcut: None,
            icon: Icon::GraduationCap,
            action: PaletteAction::Navigate(AppRoute::Studies),
        },
        PaletteCommand {
            id: "nav-bookshelf",
            label: "Ir para Estante",
            hint: "Leituras, estantes customizáveis e citações",
            shortcut: None,
            icon: Icon::BookOpen,
            action: PaletteAction::Navigate(AppRoute::Bookshelf),
        },
        PaletteCommand {
            id: "nav-garden",
            label: "Ir para Jardim & Pets",
            hint: "Explorar canteiros e interagir com seu pet",
            shortcut: None,
            icon: Icon::Sprout,
            action: PaletteAction::Navigate(AppRoute::GardenAndPets),
        },
    ]
}
