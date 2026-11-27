//! Shell principal do aplicativo **eStudy** (MVU puro, sem Topbar global nem segunda sidebar).

use chrono::Timelike;
use estudy_domain::{CalendarViewMode, ClockPort, EntityId, PomodoroPreset, TaskStatus, TaskTurn};
use estudy_ui_kit::widgets::{
    card, category_pill, color_dot, editorial_page_header, empty_placeholder, hairline_divider,
    icon_svg, illustrated_svg, kbd_hint, modal_surface, section_header, shaped_text, soft_surface,
};
use estudy_ui_kit::{
    CategoryColor, CosmeticLoadout, EstudyTheme, FONT_MONO, FONT_SANS, FONT_SANS_MEDIUM,
    FONT_SANS_SEMIBOLD, FONT_SERIF, FONT_SERIF_ITALIC, FONT_SERIF_MEDIUM, FONT_SIZE_2XL,
    FONT_SIZE_BASE, FONT_SIZE_MD, FONT_SIZE_SM, FONT_SIZE_TIMER, FONT_SIZE_XL, FONT_SIZE_XS, Icon,
    IllustratedIcon, RADIUS_CARD, RADIUS_CONTROL, RADIUS_PILL, SIDEBAR_COMPACT_WIDTH,
    SIDEBAR_WIDTH, SPACE_1, SPACE_2, SPACE_3, SPACE_4, SPACE_5, SPACE_6, ThemeMode, ThemePreset,
};
use iced::keyboard::{self, Key, Modifiers};
use iced::widget::{button, column, container, mouse_area, row, scrollable, stack, text};
use iced::{Alignment, Border, Element, Event, Length, Subscription, mouse};
use std::sync::Arc;

use crate::command_palette::{
    AppRoute, CommandPaletteState, PaletteAction, PaletteCommand, PaletteMessage,
};
use crate::features::calendar::{CalendarMessage, CalendarState, view_calendar, view_today_drawer};
use crate::features::focus::{FocusMessage, FocusState, view_focus_mode};
use crate::features::habits::{HabitsMessage, HabitsState, view_habits};
use crate::features::home::{
    HomeDashboardData, HomeLoadState, HomeMessage, MoodViewMode, view_home,
};
use crate::features::projects::{ProjectsMessage, ProjectsState, view_projects};
use crate::features::second_brain::{
    SecondBrainMessage, SecondBrainState, view_note_editor_modal, view_second_brain,
};

/// Popup rápido contextual sobre a tela ativa.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum QuickPopup {
    /// Leitura e edição rápida multi-linha de uma nota do Segundo Cérebro.
    Note(EntityId),
    /// Checklist rápido de Hábitos de Hoje.
    HabitsQuick,
    /// Controle rápido do Pomodoro sem entrar em tela cheia.
    FocusQuick,
    /// Visão rápida de um Projeto.
    ProjectOverview(EntityId),
    /// Detalhes rápidos de uma tarefa.
    TaskQuick(EntityId),
}

/// Mensagens raiz do shell MVU do eStudy.
#[derive(Debug, Clone)]
pub enum AppMessage {
    /// Navega para uma rota principal.
    Navigate(AppRoute),
    /// Dispara o atalho rápido de Foco/Pomodoro em tela cheia.
    QuickFocusShortcut,
    /// Abre ou fecha o popover de notificações e conquistas.
    ToggleNotifications,
    /// Abre ou fecha o menu compacto de módulos.
    ToggleModulesMenu,
    /// Abre ou fecha o balão contextual do mascote Mochi.
    ToggleMascotBubble,
    /// Fecha menus e overlays abertos uma camada por vez (`Escape`).
    CloseOverlays,
    /// Alterna a barra lateral principal (`216px` <-> `58px`).
    ToggleNavSidebar,
    /// Alterna a barra contextual / principal (`Ctrl+B`).
    ToggleContextSidebar,
    /// Alterna o próximo preset de tema oficial.
    CycleThemePreset,
    /// Seleciona diretamente um preset de tema oficial (tela Ajustes).
    SelectThemePreset(ThemePreset),
    /// Alterna entre modo claro e escuro.
    ToggleThemeMode,
    /// Abre ou fecha a Gaveta "Hoje".
    ToggleTodayDrawer,
    /// Fecha o banner de status transitório.
    DismissBanner,
    /// Abre um popup de visualização rápida sobre a tela atual.
    OpenQuickPopup(QuickPopup),
    /// Fecha o popup de visualização rápida.
    CloseQuickPopup,
    /// Mensagem destinada à Barra de Comando (`Ctrl+K`).
    Palette(PaletteMessage),
    /// Mensagem destinada à fatia Home.
    Home(HomeMessage),
    /// Mensagem destinada à fatia Tarefas / Projetos.
    Projects(ProjectsMessage),
    /// Mensagem destinada à fatia Agenda / Calendário.
    Calendar(CalendarMessage),
    /// Mensagem destinada ao módulo Pomodoro, Modo Foco e Áudio DSP.
    Focus(FocusMessage),
    /// Mensagem destinada ao módulo de Hábitos.
    Habits(HabitsMessage),
    /// Mensagem destinada ao módulo Segundo Cérebro.
    SecondBrain(SecondBrainMessage),
}

/// Estado do balão contextual do gatinho mascote Mochi.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum MascotBubbleState {
    /// Balão recolhido.
    #[default]
    Closed,
    /// Balão visível ao lado da medalha do Mochi.
    Open,
}

impl MascotBubbleState {
    /// Informa se o balão está aberto.
    #[must_use]
    pub const fn is_open(self) -> bool {
        matches!(self, Self::Open)
    }

    const fn toggle(self) -> Self {
        match self {
            Self::Closed => Self::Open,
            Self::Open => Self::Closed,
        }
    }
}

/// Estado completo da interface do eStudy (testável sem janela nem GPU).
pub struct EstudyApp {
    clock: Arc<dyn ClockPort>,
    /// Rota atual exibida no palco principal.
    pub route: AppRoute,
    /// Slots cosméticos e tema ativo.
    pub loadout: CosmeticLoadout,
    /// Estado da Barra de Comando (`Ctrl+K`).
    pub palette: CommandPaletteState,
    /// Estado do popover de notificações/conquistas.
    pub notifications_open: bool,
    /// Estado do menu compacto de módulos.
    pub modules_menu_open: bool,
    /// Estado do balão de fala do mascote Mochi sobre o palco.
    pub mascot_bubble: MascotBubbleState,
    /// Indica se a barra lateral está expandida (`216px`) ou em trilho limpo de ícones (`58px`).
    pub nav_sidebar_open: bool,
    /// Estado da fatia Home.
    pub home_state: HomeLoadState,
    /// Estado da fatia Tarefas & Projetos.
    pub projects_state: ProjectsState,
    /// Estado da fatia Agenda & Calendário.
    pub calendar_state: CalendarState,
    /// Estado da fatia Pomodoro, Modo Foco e Áudio DSP.
    pub focus_state: FocusState,
    /// Estado da fatia Hábitos.
    pub habits_state: HabitsState,
    /// Estado da fatia Segundo Cérebro.
    pub second_brain_state: SecondBrainState,
    /// Popup rápido atualmente aberto sobre a tela, se houver.
    pub active_popup: Option<QuickPopup>,
    /// Mensagem de feedback transitório da ação mais recente.
    pub status_banner: Option<String>,
}

impl EstudyApp {
    /// Inicializa o aplicativo com um relógio injetado e estado inicial da Home.
    #[must_use]
    pub fn new(clock: Arc<dyn ClockPort>, home_state: HomeLoadState) -> Self {
        let now = clock.now_utc();
        let projects_state = ProjectsState::sample(now.date_naive(), now);
        let calendar_state = CalendarState::sample(now.date_naive(), now, &projects_state.projects);
        Self::with_state(clock, home_state, projects_state, calendar_state)
    }

    /// Inicializa o aplicativo com estado de Projetos carregado da persistência.
    #[must_use]
    pub fn with_projects(
        clock: Arc<dyn ClockPort>,
        home_state: HomeLoadState,
        projects_state: ProjectsState,
    ) -> Self {
        let now = clock.now_utc();
        let calendar_state = CalendarState::sample(now.date_naive(), now, &projects_state.projects);
        Self::with_state(clock, home_state, projects_state, calendar_state)
    }

    /// Inicializa o aplicativo com Projetos e Calendário carregados da persistência.
    #[must_use]
    pub fn with_state(
        clock: Arc<dyn ClockPort>,
        home_state: HomeLoadState,
        projects_state: ProjectsState,
        calendar_state: CalendarState,
    ) -> Self {
        let now = clock.now_utc();
        let habits_state = HabitsState::sample(now.date_naive());
        let second_brain_state = SecondBrainState::sample(now);
        Self {
            clock,
            route: AppRoute::Home,
            loadout: CosmeticLoadout::default(),
            palette: CommandPaletteState::new(),
            notifications_open: false,
            modules_menu_open: false,
            mascot_bubble: MascotBubbleState::Closed,
            nav_sidebar_open: false,
            home_state,
            projects_state,
            calendar_state,
            focus_state: FocusState::default(),
            habits_state,
            second_brain_state,
            active_popup: None,
            status_banner: None,
        }
    }

    /// Informa se o temporizador Pomodoro precisa de pulso periódico de 1 segundo.
    #[must_use]
    pub const fn is_focus_ticking(&self) -> bool {
        self.focus_state.needs_ticker()
    }

    /// Informa se há pílula flutuante de foco ou aviso transitório ativo.
    #[must_use]
    pub const fn has_active_status_dock(&self) -> bool {
        self.focus_state.needs_ticker() || self.status_banner.is_some()
    }

    /// Lista canônica de rotas funcionais expostas na navegação primária (zero dead-ends).
    #[must_use]
    pub const fn primary_nav_routes() -> &'static [AppRoute] {
        &[
            AppRoute::Home,
            AppRoute::Projects,
            AppRoute::Habits,
            AppRoute::SecondBrain,
            AppRoute::Calendar,
        ]
    }

    /// Retorna o tema `Iced` resolvido para o loadout atual.
    #[must_use]
    pub const fn active_theme(&self) -> EstudyTheme {
        EstudyTheme::resolve(self.loadout.theme_preset, self.loadout.theme_mode)
    }

    /// Calcula a saudação adequada ao horário fornecido pelo [`ClockPort`].
    #[must_use]
    pub fn greeting(&self) -> &'static str {
        let hour = self.clock.now_utc().hour();
        if hour < 12 {
            "Bom dia"
        } else if hour < 18 {
            "Boa tarde"
        } else {
            "Boa noite"
        }
    }

    /// Quantidade de tarefas ainda em aberto para exibir no badge da barra lateral.
    #[must_use]
    pub fn open_tasks_count(&self) -> usize {
        self.projects_state
            .tasks
            .iter()
            .filter(|task| task.status != TaskStatus::Done)
            .count()
    }

    /// Gera uma fala contextual curta e viva para o gatinho Mochi no rodapé da barra lateral.
    #[must_use]
    pub fn mascot_contextual_message(&self) -> String {
        if self.is_focus_ticking() {
            let mins = self.focus_state.timer.remaining_seconds().div_ceil(60);
            return format!("Silêncio no estúdio — faltam {mins} min neste bloco de foco.");
        }
        let open_tasks = self.open_tasks_count();
        if open_tasks > 0 {
            format!("{open_tasks} tarefas abertas na mesa. Escolha uma leve para começar.")
        } else {
            "Semana em ordem por aqui. Ótimo momento para ler ou anotar ideias.".to_string()
        }
    }

    /// Processa uma mensagem MVU atualizando o estado puro do aplicativo.
    #[allow(clippy::too_many_lines)]
    pub fn update(&mut self, message: AppMessage) {
        match message {
            AppMessage::Navigate(route) => {
                self.projects_state.flush_deferred();
                self.projects_state.context_menu = None;
                self.route = route;
                self.palette.close();
                self.notifications_open = false;
                self.modules_menu_open = false;
                self.mascot_bubble = MascotBubbleState::Closed;
                self.active_popup = None;
            }
            AppMessage::QuickFocusShortcut => {
                self.notifications_open = false;
                self.modules_menu_open = false;
                self.active_popup = None;
                self.focus_state.fullscreen_active = true;
                self.status_banner =
                    Some("Modo Foco aberto (Esc volta mantendo o timer).".to_string());
            }
            AppMessage::ToggleNotifications => {
                self.notifications_open = !self.notifications_open;
                self.modules_menu_open = false;
                self.active_popup = None;
            }
            AppMessage::ToggleModulesMenu => {
                self.modules_menu_open = !self.modules_menu_open;
                self.notifications_open = false;
            }
            AppMessage::ToggleMascotBubble => {
                self.mascot_bubble = self.mascot_bubble.toggle();
            }
            AppMessage::CloseOverlays => self.close_top_overlay(),
            AppMessage::OpenQuickPopup(popup) => {
                self.notifications_open = false;
                self.modules_menu_open = false;
                self.active_popup = Some(popup);
            }
            AppMessage::CloseQuickPopup => {
                self.active_popup = None;
                self.second_brain_state.note_modal_open = false;
            }
            AppMessage::ToggleNavSidebar => {
                self.nav_sidebar_open = !self.nav_sidebar_open;
            }
            AppMessage::ToggleContextSidebar => match self.route {
                AppRoute::Projects => {
                    self.nav_sidebar_open = !self.nav_sidebar_open;
                    self.projects_state
                        .update(ProjectsMessage::ToggleSidebar, self.clock.now_utc());
                }
                AppRoute::Calendar => {
                    self.nav_sidebar_open = !self.nav_sidebar_open;
                    self.calendar_state
                        .update(CalendarMessage::ToggleSidebar, self.clock.now_utc());
                }
                _ => {
                    self.nav_sidebar_open = !self.nav_sidebar_open;
                }
            },
            AppMessage::CycleThemePreset => {
                self.loadout.theme_preset = self.loadout.theme_preset.next();
            }
            AppMessage::SelectThemePreset(preset) => {
                self.loadout.theme_preset = preset;
            }
            AppMessage::ToggleThemeMode => {
                self.loadout.theme_mode = self.loadout.theme_mode.toggle();
            }
            AppMessage::ToggleTodayDrawer => {
                self.calendar_state.today_drawer_open = !self.calendar_state.today_drawer_open;
                self.notifications_open = false;
                self.active_popup = None;
            }
            AppMessage::DismissBanner => {
                self.status_banner = None;
            }
            AppMessage::Palette(msg) => self.handle_palette_message(msg),
            AppMessage::Home(msg) => self.handle_home_message(msg),
            AppMessage::Projects(msg) => self.handle_projects_message(msg),
            AppMessage::Calendar(msg) => {
                let now = self.clock.now_utc();
                self.calendar_state.update(msg, now);
            }
            AppMessage::Focus(msg) => {
                let now = self.clock.now_utc();
                self.focus_state.update(msg, now);
            }
            AppMessage::Habits(msg) => {
                self.habits_state.update(msg);
            }
            AppMessage::SecondBrain(msg) => {
                let now = self.clock.now_utc();
                match &msg {
                    SecondBrainMessage::SelectNote(id) if self.route == AppRoute::Home => {
                        self.active_popup = Some(QuickPopup::Note(id.clone()));
                    }
                    SecondBrainMessage::CloseNoteModal => {
                        if matches!(self.active_popup, Some(QuickPopup::Note(_))) {
                            self.active_popup = None;
                        }
                    }
                    _ => {}
                }
                self.second_brain_state.update(msg, now);
            }
        }
    }

    /// Fecha exatamente a camada superior ativa na pilha previsível de overlays (`Esc`).
    fn close_top_overlay(&mut self) {
        if self.palette.is_open {
            self.palette.close();
        } else if self.projects_state.context_menu.is_some() {
            self.projects_state.context_menu = None;
        } else if self.projects_state.dragging_task_id.is_some() {
            self.projects_state.dragging_task_id = None;
            self.projects_state.hovered_drop_target = None;
        } else if self.projects_state.task_modal.is_some() || self.projects_state.drawer.is_some() {
            self.projects_state.flush_deferred();
            self.projects_state.task_modal = None;
            self.projects_state.drawer = None;
        } else if self.projects_state.day_peek_date.is_some() {
            self.projects_state.day_peek_date = None;
        } else if self.active_popup.is_some() {
            self.active_popup = None;
            self.second_brain_state.note_modal_open = false;
        } else if self.second_brain_state.note_modal_open {
            self.second_brain_state.note_modal_open = false;
        } else if self.calendar_state.create_draft.open {
            self.calendar_state.create_draft.open = false;
        } else if self.notifications_open || self.modules_menu_open || self.mascot_bubble.is_open()
        {
            self.notifications_open = false;
            self.modules_menu_open = false;
            self.mascot_bubble = MascotBubbleState::Closed;
        } else if self.calendar_state.today_drawer_open {
            self.calendar_state.today_drawer_open = false;
        } else if self.focus_state.fullscreen_active {
            self.focus_state.fullscreen_active = false;
        }
    }

    /// Subscrição de atalhos globais (`Ctrl+K`, `Ctrl+B`, `Esc`) e rastreio leve do cursor para menus/Drag & Drop.
    pub fn subscription(&self) -> Subscription<AppMessage> {
        let keys = keyboard::listen().filter_map(|event| match event {
            keyboard::Event::KeyPressed { key, modifiers, .. } => map_shortcut(&key, modifiers),
            _ => None,
        });

        let pointer = iced::event::listen_with(|event, _status, _id| match event {
            Event::Mouse(mouse::Event::CursorMoved { position }) => Some(AppMessage::Projects(
                ProjectsMessage::CursorMoved(position.x, position.y),
            )),
            _ => None,
        });

        Subscription::batch([keys, pointer])
    }

    /// Renderiza o shell limpo (Sidebar esquerda `216px <-> 58px` + Palco amplo + Pílula flutuante de Foco ativo).
    #[must_use]
    pub fn view(&self) -> Element<'_, AppMessage> {
        let theme = self.active_theme();
        if self.focus_state.fullscreen_active {
            let pet_label = match &self.home_state {
                HomeLoadState::Ready(data) => data.pet_badge_label.as_str(),
                _ => "Mochi",
            };
            let focus_screen =
                view_focus_mode(&self.focus_state, pet_label, theme, AppMessage::Focus);
            if self.palette.is_open {
                return stack![focus_screen, self.palette.view(theme, AppMessage::Palette)].into();
            }
            return focus_screen;
        }

        let sidebar = self.view_sidebar_panel(self.nav_sidebar_open, theme);
        let route_stage = self.view_route_body(theme);

        let shell_row: Element<'_, AppMessage> = row![sidebar, route_stage]
            .width(Length::Fill)
            .height(Length::Fill)
            .into();

        let base_shell = container(shell_row)
            .width(Length::Fill)
            .height(Length::Fill)
            .style(move |_| container::Style {
                background: Some(theme.bg.into()),
                text_color: Some(theme.text_primary),
                ..container::Style::default()
            });

        let mut layers: Vec<Element<'_, AppMessage>> = vec![base_shell.into()];

        // Pílula flutuante no canto superior direito apenas quando o cronômetro Pomodoro está rodando.
        if self.is_focus_ticking() {
            layers.push(self.view_floating_focus_pill(theme));
        }

        if self.mascot_bubble.is_open() && !self.nav_sidebar_open {
            layers.push(self.view_floating_mascot_bubble(theme));
        }

        if self.calendar_state.today_drawer_open {
            layers.push(view_today_drawer(
                &self.calendar_state.events,
                &self.projects_state.tasks,
                self.clock.now_utc(),
                theme,
                AppMessage::Calendar,
            ));
        }

        if let Some(popup) = &self.active_popup {
            layers.push(self.view_quick_popup(popup, theme));
        }

        if self.notifications_open {
            layers.push(view_notifications_popover(theme));
        }

        if self.palette.is_open {
            layers.push(self.palette.view(theme, AppMessage::Palette));
        }

        stack(layers)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    fn handle_projects_message(&mut self, msg: ProjectsMessage) {
        if let ProjectsMessage::StartFocusForTask(task_id) = msg {
            let title = self
                .projects_state
                .tasks
                .iter()
                .find(|task| task.id == task_id)
                .map_or_else(|| "Tarefa em foco".to_string(), |task| task.title.clone());
            let now = self.clock.now_utc();
            self.projects_state.context_menu = None;
            self.focus_state.update(
                FocusMessage::OpenWithTarget {
                    title: title.clone(),
                    task_id: Some(task_id),
                    preset: PomodoroPreset::Classic25_5,
                },
                now,
            );
            self.focus_state.update(FocusMessage::Start, now);
            self.status_banner = Some(format!("Pomodoro iniciado para \"{title}\"."));
            return;
        }
        let now = self.clock.now_utc();
        self.projects_state.update(msg, now);
    }

    fn handle_palette_message(&mut self, message: PaletteMessage) {
        match message {
            PaletteMessage::Toggle => {
                if !self.palette.is_open {
                    self.refresh_palette_search_entries();
                }
                self.palette.toggle();
            }
            PaletteMessage::Close => self.palette.close(),
            PaletteMessage::QueryChanged(query) => {
                if self.palette.search_entries.is_empty() {
                    self.refresh_palette_search_entries();
                }
                self.palette.set_query(query);
            }
            PaletteMessage::SelectPrev => self.palette.select_prev(),
            PaletteMessage::SelectNext => self.palette.select_next(),
            PaletteMessage::ExecuteSelected => {
                if let Some(action) = self.palette.resolve_action(None) {
                    self.execute_palette_action(action);
                }
            }
            PaletteMessage::ExecuteIndex(idx) => {
                if let Some(action) = self.palette.resolve_action(Some(idx)) {
                    self.execute_palette_action(action);
                }
            }
        }
    }

    fn refresh_palette_search_entries(&mut self) {
        let mut entries = Vec::new();
        for proj in &self.projects_state.projects {
            entries.push(PaletteCommand {
                id: format!("proj:{}", proj.id),
                label: proj.title.clone(),
                hint: format!("Projeto · {}", proj.status.label()),
                shortcut: None,
                icon: Icon::FolderKanban,
                action: PaletteAction::OpenProject(proj.id.clone()),
            });
        }
        for task in &self.projects_state.tasks {
            entries.push(PaletteCommand {
                id: format!("task:{}", task.id),
                label: task.title.clone(),
                hint: format!("Tarefa · {}", task.status.label()),
                shortcut: None,
                icon: Icon::CheckSquare,
                action: PaletteAction::OpenTask(task.id.clone()),
            });
        }
        for event in &self.calendar_state.events {
            entries.push(PaletteCommand {
                id: format!("event:{}", event.id),
                label: event.title.clone(),
                hint: format!("Compromisso · {}", event.starts_at.format("%d/%m %H:%M")),
                shortcut: None,
                icon: Icon::Calendar,
                action: PaletteAction::Navigate(AppRoute::Calendar),
            });
        }
        self.palette.set_search_entries(entries);
    }

    fn execute_palette_action(&mut self, action: PaletteAction) {
        self.palette.close();
        let now = self.clock.now_utc();
        match action {
            PaletteAction::Navigate(route) => self.route = route,
            PaletteAction::OpenTodayTimeline => {
                self.calendar_state.today_drawer_open = true;
            }
            PaletteAction::OpenProject(project_id) => {
                self.route = AppRoute::Projects;
                self.projects_state
                    .update(ProjectsMessage::SelectProject(Some(project_id)), now);
            }
            PaletteAction::OpenTask(task_id) => {
                self.route = AppRoute::Projects;
                self.projects_state
                    .update(ProjectsMessage::OpenTaskDrawer(task_id), now);
            }
            PaletteAction::CycleThemePreset => {
                self.loadout.theme_preset = self.loadout.theme_preset.next();
            }
            PaletteAction::ToggleThemeMode => {
                self.loadout.theme_mode = self.loadout.theme_mode.toggle();
            }
            PaletteAction::StartFocus(minutes) => {
                let preset = match minutes {
                    50 => PomodoroPreset::Deep50_10,
                    15 => PomodoroPreset::Sprint15_3,
                    25 => PomodoroPreset::Classic25_5,
                    other => PomodoroPreset::Custom {
                        focus_minutes: u32::from(other),
                        break_minutes: u32::from((other / 5).max(1)),
                    },
                };
                self.focus_state.update(FocusMessage::Reset, now);
                self.focus_state.update(
                    FocusMessage::OpenWithTarget {
                        title: format!("Bloco de foco ({minutes} min)"),
                        task_id: None,
                        preset,
                    },
                    now,
                );
                self.status_banner = Some(format!("Bloco de foco de {minutes} min preparado."));
            }
            PaletteAction::QuickCreateTask(Some(title)) => {
                self.route = AppRoute::Projects;
                self.projects_state
                    .update(ProjectsMessage::QuickTaskTitleChanged(title.clone()), now);
                self.projects_state
                    .update(ProjectsMessage::CreateQuickTask, now);
                self.status_banner = Some(format!("Tarefa '{title}' criada."));
            }
            PaletteAction::QuickCreateTask(None) => {
                self.route = AppRoute::Projects;
                self.projects_state.update(
                    ProjectsMessage::OpenCreateTaskModal {
                        date: Some(now.date_naive()),
                        turn: TaskTurn::Anytime,
                    },
                    now,
                );
            }
            PaletteAction::QuickCreateEvent(Some(title)) => {
                self.route = AppRoute::Calendar;
                self.calendar_state.quick_add_event(&title, now);
                self.status_banner = Some(format!("Evento '{title}' criado no calendário."));
            }
            PaletteAction::QuickCreateEvent(None) => {
                self.route = AppRoute::Calendar;
                self.calendar_state.create_draft.open = true;
            }
            PaletteAction::LogMoodQuick => {
                self.route = AppRoute::Home;
            }
            PaletteAction::RandomTbrBook => {
                self.route = AppRoute::Bookshelf;
            }
            PaletteAction::QuickReadingProgress(pages) => {
                self.route = AppRoute::Bookshelf;
                self.status_banner = Some(format!("+{pages} páginas registradas na leitura."));
            }
            PaletteAction::ExportCloudSnapshot => {
                self.route = AppRoute::Settings;
            }
        }
    }

    #[allow(clippy::too_many_lines)]
    fn handle_home_message(&mut self, message: HomeMessage) {
        let now = self.clock.now_utc();
        match message {
            HomeMessage::SelectTodayMood(level) => {
                if let HomeLoadState::Ready(data) = &mut self.home_state {
                    data.set_today_mood(level);
                }
            }
            HomeMessage::ToggleMoodViewMode => {
                if let HomeLoadState::Ready(data) = &mut self.home_state {
                    data.mood_view = match data.mood_view {
                        MoodViewMode::Week => MoodViewMode::Year,
                        MoodViewMode::Year => MoodViewMode::Week,
                    };
                }
            }
            HomeMessage::SelectBoardFrame(frame) => {
                if let HomeLoadState::Ready(data) = &mut self.home_state {
                    data.pinboard.frame = frame;
                }
                self.loadout.board_frame = frame;
            }
            HomeMessage::ToggleTodayItem(idx) => {
                if let HomeLoadState::Ready(data) = &mut self.home_state
                    && let Some(item) = data.today_items.get_mut(idx)
                {
                    item.done = !item.done;
                }
            }
            HomeMessage::OpenTodayTimeline => {
                self.calendar_state.today_drawer_open = true;
            }
            HomeMessage::OpenCalendarWeek => {
                self.route = AppRoute::Calendar;
                self.calendar_state.view_mode = CalendarViewMode::Week;
            }
            HomeMessage::StartFocusForPin(pin_id) => {
                let pin_title = match &self.home_state {
                    HomeLoadState::Ready(data) => data
                        .focus_items
                        .iter()
                        .find(|pin| pin.id == pin_id)
                        .map_or_else(
                            || "Bloco de foco (25 min)".to_string(),
                            |pin| pin.title.clone(),
                        ),
                    _ => "Bloco de foco (25 min)".to_string(),
                };
                self.focus_state.update(
                    FocusMessage::OpenWithTarget {
                        title: pin_title,
                        task_id: None,
                        preset: PomodoroPreset::Classic25_5,
                    },
                    now,
                );
                self.focus_state.update(FocusMessage::Start, now);
                self.focus_state.fullscreen_active = false;
            }
            HomeMessage::PreviewProject(title) => {
                if let Some(proj) = self
                    .projects_state
                    .projects
                    .iter()
                    .find(|p| p.title.eq_ignore_ascii_case(&title))
                    .or_else(|| self.projects_state.projects.first())
                {
                    let target_id = proj.id.clone();
                    self.route = AppRoute::Projects;
                    self.active_popup = None;
                    self.projects_state
                        .update(ProjectsMessage::SelectProject(Some(target_id)), now);
                }
            }
            HomeMessage::PreviewFocusPin(pin_id) => {
                if let Some(task) = self
                    .projects_state
                    .tasks
                    .iter()
                    .find(|t| t.id.to_string() == pin_id)
                    .or_else(|| self.projects_state.tasks.first())
                {
                    let task_id = task.id.clone();
                    self.route = AppRoute::Projects;
                    self.projects_state
                        .update(ProjectsMessage::OpenEditTaskModal(task_id), now);
                }
            }
            HomeMessage::QuickCreateTask => {
                self.projects_state.update(
                    ProjectsMessage::OpenCreateTaskModal {
                        date: Some(now.date_naive()),
                        turn: TaskTurn::Anytime,
                    },
                    now,
                );
            }
            HomeMessage::QuickCreateHabit => {
                self.route = AppRoute::Habits;
            }
            HomeMessage::QuickCreateEvent => {
                self.route = AppRoute::Calendar;
                self.calendar_state.create_draft.open = true;
            }
            HomeMessage::QuickCreateNote => {
                self.route = AppRoute::SecondBrain;
                self.second_brain_state
                    .update(SecondBrainMessage::CreateNote, now);
            }
            HomeMessage::RetryLoad | HomeMessage::SeedStarterWorkspace => {
                self.home_state =
                    HomeLoadState::Ready(Box::new(HomeDashboardData::sample_workspace("Arthur")));
            }
        }
    }

    fn view_sidebar_panel(&self, expanded: bool, theme: EstudyTheme) -> Element<'_, AppMessage> {
        let header = Self::view_sidebar_brand_header(expanded, theme);

        let search_btn: Element<'_, AppMessage> = if expanded {
            button(
                row![
                    icon_svg(Icon::Search, 15.0, theme.text_muted),
                    shaped_text("Buscar...", FONT_SIZE_SM, FONT_SANS, theme.text_muted)
                        .width(Length::Fill),
                    kbd_hint("Ctrl+K", theme),
                ]
                .spacing(SPACE_2)
                .align_y(Alignment::Center),
            )
            .on_press(AppMessage::Palette(PaletteMessage::Toggle))
            .width(Length::Fill)
            .padding([7.0, 10.0])
            .style(move |_, status| theme.ghost_button_style(status))
            .into()
        } else {
            button(
                container(icon_svg(Icon::Search, 18.0, theme.text_muted))
                    .center_x(Length::Fill)
                    .center_y(Length::Fixed(36.0)),
            )
            .on_press(AppMessage::Palette(PaletteMessage::Toggle))
            .width(Length::Fill)
            .height(Length::Fixed(38.0))
            .padding(0.0)
            .style(move |_, status| theme.subtle_button_style(status))
            .into()
        };

        let open_tasks = self.open_tasks_count();
        let (done_habits, total_habits) = self
            .habits_state
            .today_progress(self.clock.now_utc().date_naive());

        let mut nav_col = column![].spacing(4.0);
        if expanded {
            nav_col = nav_col.push(
                container(shaped_text(
                    "SEU ESPAÇO",
                    10.5,
                    FONT_SANS_SEMIBOLD,
                    theme.text_muted,
                ))
                .padding([SPACE_2, 10.0]),
            );
        }

        for &route in Self::primary_nav_routes() {
            let (illus, badge) = match route {
                AppRoute::Home => (IllustratedIcon::Home, None),
                AppRoute::Projects => (
                    IllustratedIcon::Tasks,
                    (open_tasks > 0).then(|| open_tasks.to_string()),
                ),
                AppRoute::Habits => (
                    IllustratedIcon::Habits,
                    (total_habits > 0).then(|| format!("{done_habits}/{total_habits}")),
                ),
                AppRoute::SecondBrain => (IllustratedIcon::SecondBrain, None),
                AppRoute::Calendar => (IllustratedIcon::Calendar, None),
                _ => (IllustratedIcon::Studies, None),
            };
            nav_col = nav_col.push(self.sidebar_nav_item(route, illus, badge, expanded, theme));
        }

        let nav_scroll = scrollable(nav_col.width(Length::Fill)).height(Length::Fill);

        let mascot_footer = self.view_sidebar_mascot_footer(expanded, theme);
        let settings_item = self.sidebar_nav_item(
            AppRoute::Settings,
            IllustratedIcon::Settings,
            None,
            expanded,
            theme,
        );

        let col = column![
            header,
            search_btn,
            hairline_divider(theme),
            nav_scroll,
            mascot_footer,
            settings_item,
        ]
        .spacing(SPACE_2)
        .width(Length::Fill)
        .height(Length::Fill);

        let width = if expanded {
            SIDEBAR_WIDTH
        } else {
            SIDEBAR_COMPACT_WIDTH
        };

        container(col)
            .width(Length::Fixed(width))
            .height(Length::Fill)
            .padding(if expanded {
                [SPACE_4, 12.0]
            } else {
                [SPACE_3, 8.0]
            })
            .style(move |_| theme.sidebar_style())
            .into()
    }

    fn view_sidebar_brand_header<'a>(
        expanded: bool,
        theme: EstudyTheme,
    ) -> Element<'a, AppMessage> {
        let toggle_btn = button(
            container(icon_svg(Icon::PanelLeft, 16.0, theme.text_muted))
                .center_x(Length::Fixed(32.0))
                .center_y(Length::Fixed(32.0)),
        )
        .on_press(AppMessage::ToggleNavSidebar)
        .width(Length::Fixed(34.0))
        .height(Length::Fixed(34.0))
        .padding(0.0)
        .style(move |_, status| theme.subtle_button_style(status));

        if expanded {
            let lockup = button(
                row![
                    text("e")
                        .size(22.0)
                        .font(FONT_SERIF_ITALIC)
                        .color(CategoryColor::Terracotta.solid(theme))
                        .shaping(text::Shaping::Advanced),
                    text("Study")
                        .size(22.0)
                        .font(FONT_SERIF_MEDIUM)
                        .color(theme.text_primary)
                        .shaping(text::Shaping::Advanced),
                ]
                .spacing(1.0)
                .align_y(Alignment::End),
            )
            .on_press(AppMessage::Navigate(AppRoute::Home))
            .padding([2.0, 6.0])
            .style(|_, _| button::Style::default());

            row![
                lockup,
                container(toggle_btn)
                    .align_x(Alignment::End)
                    .width(Length::Fill),
            ]
            .align_y(Alignment::Center)
            .height(Length::Fixed(38.0))
            .into()
        } else {
            container(toggle_btn)
                .center_x(Length::Fill)
                .height(Length::Fixed(38.0))
                .into()
        }
    }

    fn sidebar_nav_item<'a>(
        &self,
        route: AppRoute,
        icon: IllustratedIcon,
        badge: Option<String>,
        expanded: bool,
        theme: EstudyTheme,
    ) -> Element<'a, AppMessage> {
        let is_active = self.route == route;
        let text_color = if is_active {
            theme.text_primary
        } else {
            theme.text_muted
        };
        let font = if is_active {
            FONT_SANS_SEMIBOLD
        } else {
            FONT_SANS_MEDIUM
        };

        let content: Element<'a, AppMessage> = if expanded {
            let mut line = row![
                illustrated_svg(icon, 20.0),
                shaped_text(route.label(), FONT_SIZE_MD, font, text_color),
            ]
            .spacing(10.0)
            .align_y(Alignment::Center)
            .width(Length::Fill);

            if let Some(count_label) = badge {
                line = line.push(sidebar_badge_pill(count_label, is_active, theme));
            }
            line.into()
        } else {
            container(illustrated_svg(icon, 22.0))
                .center_x(Length::Fill)
                .center_y(Length::Fill)
                .width(Length::Fill)
                .height(Length::Fill)
                .into()
        };

        button(content)
            .on_press(AppMessage::Navigate(route))
            .width(Length::Fill)
            .height(Length::Fixed(if expanded { 40.0 } else { 42.0 }))
            .padding(if expanded { [6.0, 10.0] } else { [0.0, 0.0] })
            .style(move |_, status| {
                if is_active {
                    button::Style {
                        background: Some(theme.card.into()),
                        text_color: theme.text_primary,
                        border: Border {
                            color: theme.border_subtle,
                            width: 1.0,
                            radius: RADIUS_CONTROL.into(),
                        },
                        shadow: theme.card_style().shadow,
                        snap: true,
                    }
                } else {
                    theme.subtle_button_style(status)
                }
            })
            .into()
    }

    fn view_sidebar_mascot_footer(
        &self,
        expanded: bool,
        theme: EstudyTheme,
    ) -> Element<'_, AppMessage> {
        if expanded {
            soft_surface(
                row![
                    illustrated_svg(IllustratedIcon::MascotMochi, 36.0),
                    shaped_text(
                        self.mascot_contextual_message(),
                        FONT_SIZE_XS,
                        FONT_SANS,
                        theme.text_muted,
                    )
                    .width(Length::Fill),
                ]
                .spacing(SPACE_2)
                .align_y(Alignment::Center),
                theme,
            )
            .padding(SPACE_2)
            .width(Length::Fill)
            .into()
        } else {
            mouse_area(
                button(
                    container(illustrated_svg(IllustratedIcon::MascotMochi, 34.0))
                        .center_x(Length::Fill)
                        .center_y(Length::Fill),
                )
                .on_press(AppMessage::ToggleMascotBubble)
                .width(Length::Fill)
                .height(Length::Fixed(42.0))
                .padding(0.0)
                .style(move |_, status| theme.subtle_button_style(status)),
            )
            .into()
        }
    }

    fn view_floating_mascot_bubble(&self, theme: EstudyTheme) -> Element<'_, AppMessage> {
        let bubble = container(
            row![
                illustrated_svg(IllustratedIcon::MascotMochi, 28.0),
                shaped_text(
                    self.mascot_contextual_message(),
                    FONT_SIZE_SM,
                    FONT_SANS_MEDIUM,
                    theme.text_primary,
                ),
            ]
            .spacing(SPACE_2)
            .align_y(Alignment::Center),
        )
        .max_width(280.0)
        .padding([SPACE_2, SPACE_3])
        .style(move |_| theme.floating_menu_style());

        mouse_area(
            container(bubble)
                .width(Length::Fill)
                .height(Length::Fill)
                .align_y(Alignment::End)
                .padding(iced::Padding {
                    top: 0.0,
                    right: 0.0,
                    bottom: 58.0,
                    left: SIDEBAR_COMPACT_WIDTH + 10.0,
                }),
        )
        .on_press(AppMessage::ToggleMascotBubble)
        .into()
    }

    fn view_floating_focus_pill(&self, theme: EstudyTheme) -> Element<'_, AppMessage> {
        let rem = self.focus_state.timer.remaining_seconds();
        let title = self
            .focus_state
            .active_target_title
            .as_deref()
            .unwrap_or("Em foco");

        let pill = container(
            row![
                color_dot(CategoryColor::Terracotta.solid(theme)),
                shaped_text(
                    format!("{:02}:{:02}", rem / 60, rem % 60),
                    FONT_SIZE_BASE,
                    FONT_MONO,
                    theme.text_primary,
                ),
                shaped_text(title, FONT_SIZE_SM, FONT_SANS_MEDIUM, theme.text_muted),
                button(icon_svg(Icon::Pause, 14.0, theme.text_primary))
                    .on_press(AppMessage::Focus(FocusMessage::Pause))
                    .padding(4.0)
                    .style(move |_, status| theme.subtle_button_style(status)),
                button(shaped_text(
                    "Expandir",
                    FONT_SIZE_XS,
                    FONT_SANS_SEMIBOLD,
                    theme.on_accent,
                ))
                .on_press(AppMessage::QuickFocusShortcut)
                .padding([4.0, 10.0])
                .style(move |_, status| theme.pill_button_style(true, status)),
            ]
            .spacing(SPACE_2)
            .align_y(Alignment::Center),
        )
        .padding([6.0, 12.0])
        .style(move |_| container::Style {
            background: Some(theme.card.into()),
            border: Border {
                color: theme.border_subtle,
                width: 1.0,
                radius: RADIUS_PILL.into(),
            },
            shadow: theme.floating_menu_style().shadow,
            snap: true,
            ..container::Style::default()
        });

        container(pill)
            .width(Length::Fill)
            .padding([SPACE_3, SPACE_5])
            .align_x(Alignment::End)
            .into()
    }

    fn view_quick_popup(&self, popup: &QuickPopup, theme: EstudyTheme) -> Element<'_, AppMessage> {
        match popup {
            QuickPopup::Note(_) => {
                view_note_editor_modal(&self.second_brain_state, theme, AppMessage::SecondBrain)
            }
            QuickPopup::HabitsQuick => self.view_habits_popup(theme),
            QuickPopup::FocusQuick => self.view_focus_popup(theme),
            QuickPopup::ProjectOverview(project_id) => {
                let title = self
                    .projects_state
                    .projects
                    .iter()
                    .find(|p| &p.id == project_id)
                    .map_or("Projeto", |p| p.title.as_str());
                modal_surface(
                    column![
                        shaped_text(title, FONT_SIZE_2XL, FONT_SERIF_MEDIUM, theme.text_primary),
                        button(shaped_text(
                            "Abrir em Tarefas",
                            FONT_SIZE_BASE,
                            FONT_SANS_SEMIBOLD,
                            theme.on_accent,
                        ))
                        .on_press(AppMessage::Navigate(AppRoute::Projects))
                        .padding([8.0, 16.0])
                        .style(move |_, status| theme.primary_button_style(status)),
                    ]
                    .spacing(SPACE_3),
                    420.0,
                    theme,
                )
            }
            QuickPopup::TaskQuick(task_id) => {
                let title = self
                    .projects_state
                    .tasks
                    .iter()
                    .find(|t| &t.id == task_id)
                    .map_or("Tarefa", |t| t.title.as_str());
                modal_surface(
                    column![
                        category_pill("Tarefa", CategoryColor::Ochre, theme),
                        shaped_text(title, FONT_SIZE_2XL, FONT_SERIF_MEDIUM, theme.text_primary),
                        button(shaped_text(
                            "Fechar",
                            FONT_SIZE_BASE,
                            FONT_SANS_MEDIUM,
                            theme.text_primary,
                        ))
                        .on_press(AppMessage::CloseQuickPopup)
                        .padding([8.0, 14.0])
                        .style(move |_, status| theme.ghost_button_style(status)),
                    ]
                    .spacing(SPACE_3),
                    420.0,
                    theme,
                )
            }
        }
    }

    fn view_habits_popup(&self, theme: EstudyTheme) -> Element<'_, AppMessage> {
        let close_btn = button(icon_svg(Icon::X, 16.0, theme.text_muted))
            .on_press(AppMessage::CloseQuickPopup)
            .padding(SPACE_1)
            .style(move |_, status| theme.subtle_button_style(status));
        let today = self.clock.now_utc().date_naive();
        let (done, total) = self.habits_state.today_progress(today);

        let mut list = column![].spacing(SPACE_2);
        for habit in &self.habits_state.habits {
            let cat = CategoryColor::from_index(habit.color_index);
            let checked = habit.is_done_on(today);
            list = list.push(
                button(
                    row![
                        icon_svg(
                            if checked {
                                Icon::CheckCircle2
                            } else {
                                Icon::CheckSquare
                            },
                            16.0,
                            if checked {
                                cat.solid(theme)
                            } else {
                                theme.text_muted
                            },
                        ),
                        shaped_text(
                            habit.title.as_str(),
                            FONT_SIZE_MD,
                            FONT_SANS_MEDIUM,
                            theme.text_primary,
                        ),
                    ]
                    .spacing(SPACE_2)
                    .align_y(Alignment::Center),
                )
                .on_press(AppMessage::Habits(HabitsMessage::ToggleHabitDate {
                    habit_id: habit.id.clone(),
                    date: today,
                }))
                .width(Length::Fill)
                .padding([8.0, 10.0])
                .style(move |_, status| theme.subtle_button_style(status)),
            );
        }

        modal_surface(
            column![
                row![
                    section_header(
                        Icon::Flame,
                        format!("Hábitos de Hoje ({done}/{total})"),
                        theme,
                    ),
                    container(close_btn)
                        .align_x(Alignment::End)
                        .width(Length::Fill),
                ]
                .align_y(Alignment::Center),
                hairline_divider(theme),
                list,
            ]
            .spacing(SPACE_3),
            440.0,
            theme,
        )
    }

    fn view_focus_popup(&self, theme: EstudyTheme) -> Element<'_, AppMessage> {
        let close_btn = button(icon_svg(Icon::X, 16.0, theme.text_muted))
            .on_press(AppMessage::CloseQuickPopup)
            .padding(SPACE_1)
            .style(move |_, status| theme.subtle_button_style(status));
        let rem = self.focus_state.timer.remaining_seconds();
        let clock_label = format!("{:02}:{:02}", rem / 60, rem % 60);
        let running = self.is_focus_ticking();

        modal_surface(
            column![
                row![
                    section_header(Icon::Timer, "Pomodoro", theme),
                    container(close_btn)
                        .align_x(Alignment::End)
                        .width(Length::Fill),
                ]
                .align_y(Alignment::Center),
                hairline_divider(theme),
                container(shaped_text(
                    clock_label,
                    FONT_SIZE_TIMER,
                    FONT_MONO,
                    theme.accent,
                ))
                .center_x(Length::Fill)
                .width(Length::Fill),
                row![
                    button(shaped_text(
                        if running { "Pausar" } else { "Iniciar" },
                        FONT_SIZE_BASE,
                        FONT_SANS_SEMIBOLD,
                        theme.on_accent,
                    ))
                    .on_press(AppMessage::Focus(if running {
                        FocusMessage::Pause
                    } else {
                        FocusMessage::Start
                    }))
                    .padding([8.0, 18.0])
                    .style(move |_, status| theme.pill_button_style(true, status)),
                    button(shaped_text(
                        "Abrir tela cheia",
                        FONT_SIZE_BASE,
                        FONT_SANS_MEDIUM,
                        theme.text_primary,
                    ))
                    .on_press(AppMessage::QuickFocusShortcut)
                    .padding([8.0, 14.0])
                    .style(move |_, status| theme.ghost_button_style(status)),
                ]
                .spacing(SPACE_2),
            ]
            .spacing(SPACE_3),
            420.0,
            theme,
        )
    }

    fn view_theme_presets_row(&self, theme: EstudyTheme) -> Element<'_, AppMessage> {
        let presets = [
            (ThemePreset::OlivaOrganico, CategoryColor::Sage),
            (ThemePreset::EstudyMenta, CategoryColor::Moss),
            (ThemePreset::CyberMinimalist, CategoryColor::Slate),
            (ThemePreset::SolarAmber, CategoryColor::Ochre),
            (ThemePreset::VelvetSakura, CategoryColor::Coral),
        ];

        let mut preset_row = row![].spacing(SPACE_3).width(Length::Fill);
        for (preset, cat) in presets {
            let active = self.loadout.theme_preset == preset;
            preset_row = preset_row.push(
                button(
                    column![
                        row![
                            color_dot(cat.solid(theme)),
                            shaped_text(
                                preset.label(),
                                FONT_SIZE_MD,
                                FONT_SANS_SEMIBOLD,
                                if active {
                                    theme.on_accent
                                } else {
                                    theme.text_primary
                                },
                            ),
                        ]
                        .spacing(SPACE_2)
                        .align_y(Alignment::Center),
                        shaped_text(
                            if active { "Ativo" } else { "Aplicar" },
                            FONT_SIZE_XS,
                            FONT_SANS,
                            if active {
                                theme.on_accent
                            } else {
                                theme.text_muted
                            },
                        ),
                    ]
                    .spacing(4.0),
                )
                .on_press(AppMessage::SelectThemePreset(preset))
                .width(Length::FillPortion(1))
                .padding([SPACE_3, SPACE_4])
                .style(move |_, status| {
                    if active {
                        theme.primary_button_style(status)
                    } else {
                        theme.ghost_button_style(status)
                    }
                }),
            );
        }
        preset_row.into()
    }

    fn view_settings_screen(&self, theme: EstudyTheme) -> Element<'_, AppMessage> {
        let preset_row = self.view_theme_presets_row(theme);
        let mode_label = match self.loadout.theme_mode {
            ThemeMode::Light => "Modo Claro ativo · alternar para Escuro",
            ThemeMode::Dark => "Modo Escuro ativo · alternar para Claro",
        };
        let mode_icon = match self.loadout.theme_mode {
            ThemeMode::Light => Icon::Moon,
            ThemeMode::Dark => Icon::Sun,
        };

        let header = editorial_page_header(
            "ESTUDY  /  AJUSTES",
            "Ajustes",
            "Aparência, temas e atalhos",
            None,
            theme,
        );

        let content = column![
            header,
            card(
                column![
                    section_header(Icon::Palette, "Temas WCAG AA", theme),
                    preset_row,
                    hairline_divider(theme),
                    button(
                        row![
                            icon_svg(mode_icon, 16.0, theme.text_primary),
                            shaped_text(
                                mode_label,
                                FONT_SIZE_MD,
                                FONT_SANS_MEDIUM,
                                theme.text_primary,
                            ),
                        ]
                        .spacing(SPACE_2)
                        .align_y(Alignment::Center),
                    )
                    .on_press(AppMessage::ToggleThemeMode)
                    .padding([SPACE_2, SPACE_4])
                    .style(move |_, status| theme.ghost_button_style(status)),
                ]
                .spacing(SPACE_4),
                theme,
            ),
            Self::view_settings_shortcuts_card(theme),
        ]
        .spacing(SPACE_4)
        .max_width(980.0)
        .width(Length::Fill);

        scrollable(
            container(content)
                .padding([SPACE_5, SPACE_6])
                .center_x(Length::Fill)
                .width(Length::Fill),
        )
        .height(Length::Fill)
        .into()
    }

    fn view_settings_shortcuts_card<'a>(theme: EstudyTheme) -> Element<'a, AppMessage> {
        card(
            column![
                section_header(Icon::Command, "Atalhos e Fluxo Rápido", theme),
                shortcut_row(
                    "Ctrl+K",
                    "Buscar ou criar rapidamente em qualquer tela",
                    theme
                ),
                shortcut_row("Ctrl+B", "Expandir ou recolher a barra lateral", theme),
                shortcut_row(
                    "Clique direito",
                    "Abrir menu rápido na tarefa (status, prioridade, período, duplicar)",
                    theme
                ),
                shortcut_row(
                    "Arrastar",
                    "Mover tarefas entre Sem dia, dias da semana e períodos",
                    theme
                ),
                shortcut_row("Esc", "Fechar uma camada aberta por vez", theme),
            ]
            .spacing(SPACE_3),
            theme,
        )
        .into()
    }

    fn view_route_body(&self, theme: EstudyTheme) -> Element<'_, AppMessage> {
        let today = self.clock.now_utc().date_naive();
        match self.route {
            AppRoute::Home => {
                let base_home = view_home(
                    &self.home_state,
                    &self.projects_state,
                    &self.habits_state,
                    &self.second_brain_state,
                    self.focus_state.today_focused_minutes,
                    today,
                    self.greeting(),
                    theme,
                    AppMessage::Home,
                    AppMessage::Projects,
                    AppMessage::Habits,
                    AppMessage::SecondBrain,
                    AppMessage::Navigate,
                );
                if let Some(modal) = &self.projects_state.task_modal {
                    stack![
                        base_home,
                        crate::features::projects::modals::view_task_modal(
                            &self.projects_state,
                            modal,
                            today,
                            theme,
                            AppMessage::Projects,
                        )
                    ]
                    .into()
                } else {
                    base_home
                }
            }
            AppRoute::Projects => {
                let today_items = match &self.home_state {
                    HomeLoadState::Ready(data) => data.today_items.as_slice(),
                    _ => &[],
                };
                view_projects(
                    &self.projects_state,
                    today,
                    today_items,
                    theme,
                    AppMessage::Projects,
                )
            }
            AppRoute::Habits => view_habits(&self.habits_state, today, theme, AppMessage::Habits),
            AppRoute::SecondBrain => {
                view_second_brain(&self.second_brain_state, theme, AppMessage::SecondBrain)
            }
            AppRoute::Calendar => view_calendar(
                &self.calendar_state,
                &self.projects_state.projects,
                &self.projects_state.tasks,
                today,
                theme,
                AppMessage::Calendar,
            ),
            AppRoute::Focus => {
                let pet_label = match &self.home_state {
                    HomeLoadState::Ready(data) => data.pet_badge_label.as_str(),
                    _ => "Mochi",
                };
                view_focus_mode(&self.focus_state, pet_label, theme, AppMessage::Focus)
            }
            AppRoute::Settings => self.view_settings_screen(theme),
            other => view_module_placeholder(other, theme),
        }
    }
}

fn shortcut_row<'a>(
    key_label: &'static str,
    desc: &'static str,
    theme: EstudyTheme,
) -> Element<'a, AppMessage> {
    row![
        kbd_hint(key_label, theme),
        shaped_text(desc, FONT_SIZE_MD, FONT_SANS, theme.text_primary),
    ]
    .spacing(SPACE_3)
    .align_y(Alignment::Center)
    .into()
}

fn sidebar_badge_pill<'a>(
    count_label: String,
    is_active: bool,
    theme: EstudyTheme,
) -> Element<'a, AppMessage> {
    container(
        container(shaped_text(
            count_label,
            11.0,
            FONT_MONO,
            if is_active {
                theme.on_accent
            } else {
                theme.text_primary
            },
        ))
        .padding([2.0, 7.0])
        .style(move |_| container::Style {
            background: Some(
                if is_active {
                    CategoryColor::Terracotta.solid(theme)
                } else {
                    theme.surface_soft
                }
                .into(),
            ),
            border: Border {
                radius: RADIUS_PILL.into(),
                ..Border::default()
            },
            ..container::Style::default()
        }),
    )
    .align_x(Alignment::End)
    .width(Length::Fill)
    .into()
}

fn view_notifications_popover<'a>(theme: EstudyTheme) -> Element<'a, AppMessage> {
    let close_btn = button(icon_svg(Icon::X, 14.0, theme.text_muted))
        .on_press(AppMessage::CloseOverlays)
        .padding(4.0)
        .style(move |_, status| theme.subtle_button_style(status));

    let popover = card(
        column![
            row![
                section_header(Icon::Trophy, "Notificações e Conquistas", theme),
                container(close_btn)
                    .align_x(Alignment::End)
                    .width(Length::Fill),
            ]
            .align_y(Alignment::Center),
            hairline_divider(theme),
            empty_placeholder(
                Icon::Trophy,
                "Tudo em dia por aqui",
                "Continue seus blocos de foco para desbloquear novas medalhas.",
                None,
                theme,
            ),
        ]
        .spacing(SPACE_3),
        theme,
    )
    .width(Length::Fixed(340.0));

    container(popover)
        .width(Length::Fill)
        .height(Length::Fill)
        .padding([SPACE_4, SPACE_4])
        .align_x(Alignment::End)
        .into()
}

fn view_module_placeholder<'a>(route: AppRoute, theme: EstudyTheme) -> Element<'a, AppMessage> {
    let back_btn = button(
        row![
            icon_svg(Icon::ChevronLeft, 16.0, theme.on_accent),
            shaped_text(
                "Voltar para Início",
                FONT_SIZE_BASE,
                FONT_SANS_MEDIUM,
                theme.on_accent
            ),
        ]
        .spacing(SPACE_2)
        .align_y(Alignment::Center),
    )
    .on_press(AppMessage::Navigate(AppRoute::Home))
    .padding([SPACE_2, SPACE_4])
    .style(move |_, status| theme.primary_button_style(status));

    container(
        card(
            column![
                text(route.label())
                    .size(FONT_SIZE_2XL)
                    .font(FONT_SERIF)
                    .color(theme.text_primary)
                    .shaping(text::Shaping::Advanced),
                shaped_text(
                    "Este módulo entra nos próximos marcos do Roadmap usando o mesmo sistema tipográfico e de ícones ilustrados.",
                    FONT_SIZE_MD,
                    FONT_SANS,
                    theme.text_muted,
                ),
                back_btn,
            ]
            .spacing(SPACE_3),
            theme,
        )
        .width(Length::Fixed(500.0)),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .center_x(Length::Fill)
    .center_y(Length::Fill)
    .into()
}

fn map_shortcut(key: &Key, modifiers: Modifiers) -> Option<AppMessage> {
    if modifiers.command()
        && let Key::Character(ch) = key
    {
        if ch.eq_ignore_ascii_case("k") {
            return Some(AppMessage::Palette(PaletteMessage::Toggle));
        }
        if ch.eq_ignore_ascii_case("b") {
            return Some(AppMessage::ToggleContextSidebar);
        }
    }
    if matches!(key, Key::Named(keyboard::key::Named::Escape)) {
        return Some(AppMessage::CloseOverlays);
    }
    None
}

/// Helper para alternar entre presets de forma explícita na inicialização.
#[must_use]
pub const fn default_preset() -> ThemePreset {
    ThemePreset::OlivaOrganico
}

#[allow(dead_code)]
const _: (f32, f32) = (FONT_SIZE_XL, RADIUS_CARD);
