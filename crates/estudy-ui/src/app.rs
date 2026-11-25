//! Shell principal do aplicativo **eStudy** (MVU puro desacoplado de janela).

use chrono::Timelike;
use estudy_domain::{CalendarViewMode, ClockPort, EntityId, PomodoroPreset, TaskStatus};
use estudy_ui_kit::widgets::{
    banner_toast, card, category_pill, category_tile, color_dot, empty_placeholder,
    hairline_divider, icon_svg, kbd_hint, modal_surface, section_header, soft_surface,
};
use estudy_ui_kit::{
    CategoryColor, CosmeticLoadout, EstudyTheme, FONT_MONO, FONT_SANS_MEDIUM, FONT_SANS_SEMIBOLD,
    FONT_SERIF_MEDIUM, Icon, RADIUS_PILL, SIDEBAR_COMPACT_WIDTH, SIDEBAR_WIDTH, SPACE_1, SPACE_2,
    SPACE_3, SPACE_4, SPACE_6, TOPBAR_HEIGHT, ThemeMode, ThemePreset,
};
use iced::keyboard::{self, Key, Modifiers};
use iced::widget::{button, column, container, row, scrollable, stack, text};
use iced::{Alignment, Border, Element, Length, Subscription};
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
use crate::features::second_brain::{SecondBrainMessage, SecondBrainState, view_second_brain};

/// Popup rápido de visualização sem sair da tela ativa.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum QuickPopup {
    /// Leitura rápida de uma nota do Segundo Cérebro.
    Note(EntityId),
    /// Checklist rápido de Hábitos de Hoje.
    HabitsQuick,
    /// Controle rápido do Pomodoro sem entrar em tela cheia.
    FocusQuick,
}

/// Mensagens raiz do shell MVU do eStudy.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AppMessage {
    /// Navega para uma rota principal.
    Navigate(AppRoute),
    /// Dispara o atalho rápido de Foco/Pomodoro na topbar.
    QuickFocusShortcut,
    /// Abre ou fecha o popover de notificações e conquistas.
    ToggleNotifications,
    /// Abre ou fecha o menu compacto de módulos.
    ToggleModulesMenu,
    /// Fecha menus e overlays abertos (`Escape`).
    CloseOverlays,
    /// Alterna a barra lateral principal de navegação (`220px`).
    ToggleNavSidebar,
    /// Alterna a barra contextual esquerda na tela ativa (`Ctrl+B`).
    ToggleContextSidebar,
    /// Alterna o próximo preset de tema oficial.
    CycleThemePreset,
    /// Alterna entre modo claro e escuro.
    ToggleThemeMode,
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
    /// Mensagem destinada à fatia Projetos.
    Projects(ProjectsMessage),
    /// Mensagem destinada à fatia Calendário e Drawer "Hoje".
    Calendar(CalendarMessage),
    /// Mensagem destinada ao módulo Pomodoro, Modo Foco e Áudio DSP.
    Focus(FocusMessage),
    /// Mensagem destinada ao módulo de Hábitos.
    Habits(HabitsMessage),
    /// Mensagem destinada ao módulo Segundo Cérebro.
    SecondBrain(SecondBrainMessage),
}

/// Estado completo da interface do eStudy (testável sem janela nem GPU).
pub struct EstudyApp {
    clock: Arc<dyn ClockPort>,
    /// Rota atual exibida no conteúdo principal.
    pub route: AppRoute,
    /// Slots cosméticos e tema ativo.
    pub loadout: CosmeticLoadout,
    /// Estado da Barra de Comando (`Ctrl+K`).
    pub palette: CommandPaletteState,
    /// Estado do popover de notificações/conquistas.
    pub notifications_open: bool,
    /// Estado do menu compacto de módulos.
    pub modules_menu_open: bool,
    /// Indica se a barra lateral principal (`220px`) está expandida (`false` = trilho compacto `64px`).
    pub nav_sidebar_open: bool,
    /// Estado da fatia Home.
    pub home_state: HomeLoadState,
    /// Estado da fatia Projetos (Kanban, Semana por Turnos, frameworks e Drawer da tarefa).
    pub projects_state: ProjectsState,
    /// Estado da fatia Calendário Unificado e Drawer "Hoje".
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
            nav_sidebar_open: true,
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

    /// Gera uma fala contextual curta para o mascote no rodapé da barra lateral.
    #[must_use]
    pub fn mascot_contextual_message(&self) -> String {
        if self.is_focus_ticking() {
            let mins = self.focus_state.timer.remaining_seconds().div_ceil(60);
            return format!("Foco firme! Faltam {mins} min neste bloco.");
        }
        let open_tasks = self.open_tasks_count();
        if open_tasks > 0 {
            format!("{open_tasks} tarefas abertas. Que tal 25 min em uma delas?")
        } else {
            "Tudo limpo por aqui. Ótimo momento para uma leitura calma.".to_string()
        }
    }

    /// Processa uma mensagem MVU atualizando o estado puro do aplicativo.
    pub fn update(&mut self, message: AppMessage) {
        match message {
            AppMessage::Navigate(route) => {
                self.projects_state.flush_deferred();
                self.route = route;
                self.palette.close();
                self.notifications_open = false;
                self.modules_menu_open = false;
                self.active_popup = None;
            }
            AppMessage::QuickFocusShortcut => {
                self.notifications_open = false;
                self.modules_menu_open = false;
                self.focus_state.fullscreen_active = true;
                self.status_banner =
                    Some("Modo Foco aberto (Esc para sair da tela cheia).".to_string());
            }
            AppMessage::ToggleNotifications => {
                self.notifications_open = !self.notifications_open;
                self.modules_menu_open = false;
            }
            AppMessage::ToggleModulesMenu => {
                self.modules_menu_open = !self.modules_menu_open;
                self.notifications_open = false;
            }
            AppMessage::CloseOverlays => {
                self.palette.close();
                self.notifications_open = false;
                self.modules_menu_open = false;
                self.active_popup = None;
                self.projects_state.flush_deferred();
                self.projects_state.drawer = None;
                self.calendar_state.today_drawer_open = false;
                self.focus_state.fullscreen_active = false;
            }
            AppMessage::OpenQuickPopup(popup) => {
                self.notifications_open = false;
                self.modules_menu_open = false;
                self.active_popup = Some(popup);
            }
            AppMessage::CloseQuickPopup => {
                self.active_popup = None;
            }
            AppMessage::ToggleNavSidebar => {
                self.nav_sidebar_open = !self.nav_sidebar_open;
            }
            AppMessage::ToggleContextSidebar => match self.route {
                AppRoute::Projects => self
                    .projects_state
                    .update(ProjectsMessage::ToggleSidebar, self.clock.now_utc()),
                AppRoute::Calendar => self
                    .calendar_state
                    .update(CalendarMessage::ToggleSidebar, self.clock.now_utc()),
                _ => {
                    self.nav_sidebar_open = !self.nav_sidebar_open;
                }
            },
            AppMessage::CycleThemePreset => {
                self.loadout.theme_preset = self.loadout.theme_preset.next();
            }
            AppMessage::ToggleThemeMode => {
                self.loadout.theme_mode = self.loadout.theme_mode.toggle();
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
                if let SecondBrainMessage::SelectNote(ref id) = msg
                    && self.route == AppRoute::Home
                {
                    self.active_popup = Some(QuickPopup::Note(id.clone()));
                }
                self.second_brain_state.update(msg, now);
            }
        }
    }

    /// Subscrição de atalhos globais de teclado (`Ctrl+K` e `Escape`).
    pub fn subscription(&self) -> Subscription<AppMessage> {
        keyboard::listen().filter_map(|event| match event {
            keyboard::Event::KeyPressed { key, modifiers, .. } => map_shortcut(&key, modifiers),
            _ => None,
        })
    }

    /// Renderiza a barra superior (`48px`), o conteúdo da rota atual e o overlay `Ctrl+K`.
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

        let topbar = self.view_topbar(theme);
        let body = self.view_route_body(theme);
        let main_column = column![topbar, body]
            .width(Length::Fill)
            .height(Length::Fill);

        let sidebar = self.view_sidebar(theme);
        let shell_layout: Element<'_, AppMessage> = row![sidebar, main_column]
            .width(Length::Fill)
            .height(Length::Fill)
            .into();

        let shell = container(shell_layout)
            .width(Length::Fill)
            .height(Length::Fill)
            .style(move |_| container::Style {
                background: Some(theme.bg.into()),
                text_color: Some(theme.text_primary),
                ..container::Style::default()
            });

        if self.palette.is_open {
            stack![shell, self.palette.view(theme, AppMessage::Palette)].into()
        } else if let Some(popup) = &self.active_popup {
            stack![shell, self.view_quick_popup(popup, theme)].into()
        } else if self.calendar_state.today_drawer_open {
            stack![
                shell,
                view_today_drawer(
                    &self.calendar_state.events,
                    &self.projects_state.tasks,
                    self.clock.now_utc(),
                    theme,
                    AppMessage::Calendar,
                )
            ]
            .into()
        } else if self.notifications_open {
            stack![shell, view_notifications_popover(theme)].into()
        } else if self.modules_menu_open {
            stack![shell, view_modules_popover(self.route, theme)].into()
        } else {
            shell.into()
        }
    }

    fn handle_projects_message(&mut self, msg: ProjectsMessage) {
        if let ProjectsMessage::StartFocusForTask(task_id) = msg {
            let title = self
                .projects_state
                .tasks
                .iter()
                .find(|task| task.id == task_id)
                .map_or_else(|| "Tarefa em foco".to_string(), |task| task.title.clone());
            self.focus_state.update(
                FocusMessage::OpenWithTarget {
                    title: title.clone(),
                    task_id: Some(task_id),
                    preset: PomodoroPreset::Classic25_5,
                },
                self.clock.now_utc(),
            );
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
                hint: format!("Evento · {}", event.starts_at.format("%d/%m %H:%M")),
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
                self.status_banner = Some("Criação rápida de tarefa pronta.".to_string());
            }
            PaletteAction::QuickCreateEvent(Some(title)) => {
                self.route = AppRoute::Calendar;
                self.calendar_state.quick_add_event(&title, now);
                self.status_banner = Some(format!("Evento '{title}' criado no calendário."));
            }
            PaletteAction::QuickCreateEvent(None) => {
                self.route = AppRoute::Calendar;
                self.calendar_state.create_draft.open = true;
                self.status_banner = Some("Formulário de novo evento aberto.".to_string());
            }
            PaletteAction::LogMoodQuick => {
                self.route = AppRoute::Home;
                self.status_banner = Some("Selecione seu humor na coluna direita.".to_string());
            }
            PaletteAction::RandomTbrBook => {
                self.route = AppRoute::Bookshelf;
                self.status_banner = Some("Estante pronta para sortear leitura.".to_string());
            }
            PaletteAction::QuickReadingProgress(pages) => {
                self.route = AppRoute::Bookshelf;
                self.status_banner = Some(format!("+{pages} páginas registradas na leitura."));
            }
            PaletteAction::ExportCloudSnapshot => {
                self.route = AppRoute::Settings;
                self.status_banner = Some("Snapshot limpo pronto para exportação.".to_string());
            }
        }
    }

    fn handle_home_message(&mut self, message: HomeMessage) {
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
                        .map_or_else(|| "Item em foco".to_string(), |pin| pin.title.clone()),
                    _ => "Item em foco".to_string(),
                };
                self.focus_state.update(
                    FocusMessage::OpenWithTarget {
                        title: pin_title,
                        task_id: None,
                        preset: PomodoroPreset::Classic25_5,
                    },
                    self.clock.now_utc(),
                );
                self.status_banner = Some(format!("Foco vinculado ao item {pin_id}."));
            }
            HomeMessage::RetryLoad | HomeMessage::SeedStarterWorkspace => {
                self.home_state =
                    HomeLoadState::Ready(Box::new(HomeDashboardData::sample_workspace("Arthur")));
            }
        }
    }

    fn view_sidebar(&self, theme: EstudyTheme) -> Element<'_, AppMessage> {
        let expanded = self.nav_sidebar_open;
        let header = Self::view_sidebar_toggle_header(expanded, theme);

        let open_tasks = self.open_tasks_count();
        let (done_habits, total_habits) = self
            .habits_state
            .today_progress(self.clock.now_utc().date_naive());
        let notes_count = self.second_brain_state.notes.len();

        let space_section = column![
            sidebar_group_header("SEU ESPAÇO", expanded, theme),
            self.sidebar_nav_item(AppRoute::Home, Icon::Home, None, expanded, theme),
            self.sidebar_nav_item(
                AppRoute::Projects,
                Icon::FolderKanban,
                (open_tasks > 0).then(|| open_tasks.to_string()),
                expanded,
                theme,
            ),
            self.sidebar_nav_item(
                AppRoute::Habits,
                Icon::Flame,
                (total_habits > 0).then(|| format!("{done_habits}/{total_habits}")),
                expanded,
                theme,
            ),
            self.sidebar_nav_item(
                AppRoute::SecondBrain,
                Icon::Brain,
                (notes_count > 0).then(|| notes_count.to_string()),
                expanded,
                theme,
            ),
            self.sidebar_nav_item(AppRoute::Calendar, Icon::Calendar, None, expanded, theme),
        ]
        .spacing(4.0);

        let depth_section = column![
            sidebar_group_header("APROFUNDAMENTO", expanded, theme),
            self.sidebar_nav_item(
                AppRoute::Studies,
                Icon::GraduationCap,
                None,
                expanded,
                theme,
            ),
            self.sidebar_nav_item(AppRoute::Bookshelf, Icon::BookOpen, None, expanded, theme),
            self.sidebar_nav_item(AppRoute::Finance, Icon::Coins, None, expanded, theme),
            self.sidebar_nav_item(
                AppRoute::Focus,
                Icon::Timer,
                self.is_focus_ticking().then(|| {
                    let rem = self.focus_state.timer.remaining_seconds();
                    format!("{:02}:{:02}", rem / 60, rem % 60)
                }),
                expanded,
                theme,
            ),
        ]
        .spacing(4.0);

        let companion_section = column![
            sidebar_group_header("COMPANHIA", expanded, theme),
            self.sidebar_nav_item(AppRoute::GardenAndPets, Icon::Sprout, None, expanded, theme),
            self.sidebar_nav_item(AppRoute::Shop, Icon::ShoppingBag, None, expanded, theme),
        ]
        .spacing(4.0);

        let nav_scroll = scrollable(
            column![space_section, depth_section, companion_section]
                .spacing(SPACE_3)
                .width(Length::Fill),
        )
        .height(Length::Fill);

        let settings_btn =
            self.sidebar_nav_item(AppRoute::Settings, Icon::Settings, None, expanded, theme);

        let mut col = column![header, hairline_divider(theme), nav_scroll]
            .spacing(SPACE_3)
            .width(Length::Fill)
            .height(Length::Fill);
        if expanded {
            col = col.push(self.view_sidebar_mascot_card(theme));
        }
        col = col.push(settings_btn);

        let width = if expanded {
            SIDEBAR_WIDTH
        } else {
            SIDEBAR_COMPACT_WIDTH
        };
        container(col)
            .width(Length::Fixed(width))
            .height(Length::Fill)
            .padding([SPACE_3, 13.0])
            .style(move |_| theme.sidebar_style())
            .into()
    }

    fn view_sidebar_toggle_header<'a>(
        expanded: bool,
        theme: EstudyTheme,
    ) -> Element<'a, AppMessage> {
        let toggle_icon = if expanded {
            Icon::ChevronLeft
        } else {
            Icon::ChevronRight
        };
        let toggle_btn = button(
            container(icon_svg(toggle_icon, 16.0, theme.text_primary))
                .width(Length::Fixed(38.0))
                .height(Length::Fixed(36.0))
                .center_x(Length::Fixed(38.0))
                .center_y(Length::Fixed(36.0)),
        )
        .on_press(AppMessage::ToggleNavSidebar)
        .width(Length::Fixed(38.0))
        .height(Length::Fixed(36.0))
        .padding(0.0)
        .style(move |_, status| theme.subtle_button_style(status));

        if expanded {
            row![
                toggle_btn,
                button(
                    column![
                        text("eStudy")
                            .size(16)
                            .font(FONT_SERIF_MEDIUM)
                            .color(theme.text_primary),
                        text("estação pessoal").size(10).color(theme.text_muted),
                    ]
                    .spacing(0.0),
                )
                .on_press(AppMessage::Navigate(AppRoute::Home))
                .padding([2.0, SPACE_1])
                .style(move |_, status| theme.ghost_button_style(status)),
            ]
            .spacing(SPACE_2)
            .align_y(Alignment::Center)
            .height(Length::Fixed(36.0))
            .into()
        } else {
            row![toggle_btn]
                .align_y(Alignment::Center)
                .height(Length::Fixed(36.0))
                .into()
        }
    }

    fn view_sidebar_mascot_card(&self, theme: EstudyTheme) -> Element<'_, AppMessage> {
        let pet_name = match &self.home_state {
            HomeLoadState::Ready(data) => data.pet_badge_label.as_str(),
            _ => "Mochi",
        };
        soft_surface(
            column![
                row![
                    category_tile(Icon::PawPrint, CategoryColor::Ochre, 26.0, theme),
                    text(pet_name)
                        .size(12)
                        .font(FONT_SANS_SEMIBOLD)
                        .color(theme.text_primary),
                ]
                .spacing(SPACE_2)
                .align_y(Alignment::Center),
                text(self.mascot_contextual_message())
                    .size(11)
                    .color(theme.text_muted),
            ]
            .spacing(SPACE_2),
            theme,
        )
        .padding(SPACE_3)
        .width(Length::Fill)
        .into()
    }

    fn sidebar_nav_item<'a>(
        &self,
        route: AppRoute,
        icon: Icon,
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
        let icon_color = if is_active {
            theme.accent
        } else {
            theme.text_muted
        };
        let font = if is_active {
            FONT_SANS_SEMIBOLD
        } else {
            FONT_SANS_MEDIUM
        };

        // Slot fixo de 38×36px preserva exatamente a coordenada (x, y) do ícone ao comprimir/expandir.
        let icon_slot = container(icon_svg(icon, 16.0, icon_color))
            .width(Length::Fixed(38.0))
            .height(Length::Fixed(36.0))
            .center_x(Length::Fixed(38.0))
            .center_y(Length::Fixed(36.0));

        let content: Element<'a, AppMessage> = if expanded {
            let mut line = row![
                icon_slot,
                text(route.label()).size(13).font(font).color(text_color),
            ]
            .spacing(SPACE_1)
            .align_y(Alignment::Center)
            .width(Length::Fill);

            if let Some(count_label) = badge {
                line =
                    line.push(
                        container(
                            container(text(count_label).size(10).font(FONT_MONO).color(
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
                                        theme.accent
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
                        .padding([0.0, 8.0])
                        .align_x(Alignment::End)
                        .width(Length::Fill),
                    );
            }
            line.into()
        } else {
            icon_slot.into()
        };

        button(content)
            .on_press(AppMessage::Navigate(route))
            .width(Length::Fill)
            .height(Length::Fixed(36.0))
            .padding(0.0)
            .style(move |_, status| {
                if is_active {
                    button::Style {
                        background: Some(theme.accent_subtle.into()),
                        text_color: theme.text_primary,
                        border: Border {
                            color: theme.border_subtle,
                            width: 1.0,
                            radius: 10.0.into(),
                        },
                        ..button::Style::default()
                    }
                } else {
                    theme.ghost_button_style(status)
                }
            })
            .into()
    }

    fn view_quick_popup(&self, popup: &QuickPopup, theme: EstudyTheme) -> Element<'_, AppMessage> {
        let body = match popup {
            QuickPopup::Note(note_id) => self.view_note_popup(note_id, theme),
            QuickPopup::HabitsQuick => self.view_habits_popup(theme),
            QuickPopup::FocusQuick => self.view_focus_popup(theme),
        };
        modal_surface(body, 480.0, theme)
    }

    fn view_note_popup<'a>(
        &'a self,
        note_id: &EntityId,
        theme: EstudyTheme,
    ) -> Element<'a, AppMessage> {
        let close_btn = button(icon_svg(Icon::X, 15.0, theme.text_muted))
            .on_press(AppMessage::CloseQuickPopup)
            .padding(SPACE_1)
            .style(move |_, status| theme.ghost_button_style(status));

        let Some(note) = self
            .second_brain_state
            .notes
            .iter()
            .find(|n| &n.id == note_id)
        else {
            return empty_placeholder(
                Icon::Brain,
                "Nota não encontrada",
                "Esta nota pode ter sido removida.",
                None,
                theme,
            );
        };

        let mut tags_row = row![].spacing(6.0);
        for tag in &note.tags {
            tags_row = tags_row.push(
                text(format!("#{tag}"))
                    .size(11)
                    .font(FONT_MONO)
                    .color(theme.text_muted),
            );
        }

        column![
            row![
                category_pill(note.kind.label(), CategoryColor::Lavender, theme),
                container(close_btn)
                    .align_x(Alignment::End)
                    .width(Length::Fill),
            ]
            .align_y(Alignment::Center),
            text(&note.title)
                .size(22)
                .font(FONT_SERIF_MEDIUM)
                .color(theme.text_primary),
            hairline_divider(theme),
            soft_surface(
                text(if note.body_md.trim().is_empty() {
                    "Nota em branco."
                } else {
                    note.body_md.as_str()
                })
                .size(13)
                .color(theme.text_primary),
                theme,
            )
            .padding(SPACE_4)
            .width(Length::Fill),
            tags_row,
            button(
                row![
                    icon_svg(Icon::ArrowUpRight, 13.0, theme.on_accent),
                    text("Abrir no Segundo Cérebro")
                        .size(12)
                        .font(FONT_SANS_MEDIUM)
                        .color(theme.on_accent),
                ]
                .spacing(6.0)
                .align_y(Alignment::Center),
            )
            .on_press(AppMessage::Navigate(AppRoute::SecondBrain))
            .padding([6.0, 14.0])
            .style(move |_, status| theme.pill_button_style(true, status)),
        ]
        .spacing(SPACE_3)
        .into()
    }

    fn view_habits_popup(&self, theme: EstudyTheme) -> Element<'_, AppMessage> {
        let close_btn = button(icon_svg(Icon::X, 15.0, theme.text_muted))
            .on_press(AppMessage::CloseQuickPopup)
            .padding(SPACE_1)
            .style(move |_, status| theme.ghost_button_style(status));
        let today = self.clock.now_utc().date_naive();
        let (done, total) = self.habits_state.today_progress(today);

        let mut list = column![].spacing(SPACE_2);
        for habit in &self.habits_state.habits {
            let cat = CategoryColor::from_index(habit.color_index);
            let checked = habit.is_done_on(today);
            list = list.push(
                button(
                    row![
                        color_dot(cat.solid(theme)),
                        icon_svg(
                            if checked {
                                Icon::CheckCircle2
                            } else {
                                Icon::Flame
                            },
                            15.0,
                            if checked {
                                cat.solid(theme)
                            } else {
                                theme.text_muted
                            },
                        ),
                        text(&habit.title)
                            .size(13)
                            .font(FONT_SANS_MEDIUM)
                            .color(theme.text_primary),
                        container(
                            text(format!("{}d", habit.current_streak(today)))
                                .size(11)
                                .font(FONT_MONO)
                                .color(cat.solid(theme)),
                        )
                        .align_x(Alignment::End)
                        .width(Length::Fill),
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
            button(text("Abrir módulo de Hábitos").size(12).color(theme.accent))
                .on_press(AppMessage::Navigate(AppRoute::Habits))
                .padding([4.0, 8.0])
                .style(move |_, status| theme.ghost_button_style(status)),
        ]
        .spacing(SPACE_3)
        .into()
    }

    fn view_focus_popup(&self, theme: EstudyTheme) -> Element<'_, AppMessage> {
        let close_btn = button(icon_svg(Icon::X, 15.0, theme.text_muted))
            .on_press(AppMessage::CloseQuickPopup)
            .padding(SPACE_1)
            .style(move |_, status| theme.ghost_button_style(status));
        let rem = self.focus_state.timer.remaining_seconds();
        let clock_label = format!("{:02}:{:02}", rem / 60, rem % 60);
        let running = self.is_focus_ticking();

        column![
            row![
                section_header(Icon::Timer, "Espiar Pomodoro", theme),
                container(close_btn)
                    .align_x(Alignment::End)
                    .width(Length::Fill),
            ]
            .align_y(Alignment::Center),
            hairline_divider(theme),
            container(
                text(clock_label)
                    .size(38)
                    .font(FONT_MONO)
                    .color(theme.accent),
            )
            .center_x(Length::Fill)
            .width(Length::Fill),
            row![
                button(
                    row![
                        icon_svg(
                            if running { Icon::Pause } else { Icon::Play },
                            14.0,
                            theme.on_accent,
                        ),
                        text(if running { "Pausar" } else { "Iniciar" })
                            .size(12)
                            .font(FONT_SANS_MEDIUM)
                            .color(theme.on_accent),
                    ]
                    .spacing(6.0)
                    .align_y(Alignment::Center),
                )
                .on_press(AppMessage::Focus(if running {
                    FocusMessage::Pause
                } else {
                    FocusMessage::Start
                }))
                .padding([7.0, 16.0])
                .style(move |_, status| theme.pill_button_style(true, status)),
                button(text("Abrir tela cheia").size(12).color(theme.text_primary))
                    .on_press(AppMessage::QuickFocusShortcut)
                    .padding([7.0, 14.0])
                    .style(move |_, status| theme.subtle_button_style(status)),
            ]
            .spacing(SPACE_2)
            .align_y(Alignment::Center),
        ]
        .spacing(SPACE_3)
        .into()
    }

    fn view_topbar(&self, theme: EstudyTheme) -> Element<'_, AppMessage> {
        let breadcrumb = row![
            text("eStudy").size(12).color(theme.text_muted),
            text("/").size(12).color(theme.text_muted),
            text(self.route.label())
                .size(13)
                .font(FONT_SANS_SEMIBOLD)
                .color(theme.text_primary),
        ]
        .spacing(SPACE_2)
        .align_y(Alignment::Center);

        let cmd_btn = button(
            row![
                icon_svg(Icon::Search, 14.0, theme.text_muted),
                text("Buscar").size(12).color(theme.text_muted),
                kbd_hint("Ctrl+K", theme),
            ]
            .spacing(SPACE_2)
            .align_y(Alignment::Center),
        )
        .on_press(AppMessage::Palette(PaletteMessage::Toggle))
        .padding([4.0, SPACE_3])
        .style(move |_, status| theme.pill_button_style(false, status));

        let mode_icon = match self.loadout.theme_mode {
            ThemeMode::Light => Icon::Moon,
            ThemeMode::Dark => Icon::Sun,
        };

        let focus_label = if self.is_focus_ticking() {
            let rem = self.focus_state.timer.remaining_seconds();
            format!("Focar · {:02}:{:02}", rem / 60, rem % 60)
        } else {
            "Focar".to_string()
        };

        let controls = row![
            cmd_btn,
            button(icon_svg(Icon::Eye, 15.0, theme.text_primary))
                .on_press(AppMessage::OpenQuickPopup(QuickPopup::HabitsQuick))
                .padding(SPACE_2)
                .style(move |_, status| theme.ghost_button_style(status)),
            button(
                row![
                    icon_svg(Icon::Timer, 14.0, theme.on_accent),
                    text(focus_label)
                        .size(12)
                        .font(FONT_SANS_MEDIUM)
                        .color(theme.on_accent),
                ]
                .spacing(SPACE_2)
                .align_y(Alignment::Center),
            )
            .on_press(AppMessage::OpenQuickPopup(QuickPopup::FocusQuick))
            .padding([5.0, SPACE_3])
            .style(move |_, status| theme.primary_button_style(status)),
            button(icon_svg(Icon::Bell, 15.0, theme.text_primary))
                .on_press(AppMessage::ToggleNotifications)
                .padding(SPACE_2)
                .style(move |_, status| theme.ghost_button_style(status)),
            button(
                row![
                    icon_svg(Icon::Palette, 14.0, theme.accent),
                    text(self.loadout.theme_preset.label())
                        .size(12)
                        .color(theme.text_primary),
                ]
                .spacing(SPACE_1)
                .align_y(Alignment::Center),
            )
            .on_press(AppMessage::CycleThemePreset)
            .padding([SPACE_1, SPACE_2])
            .style(move |_, status| theme.ghost_button_style(status)),
            button(icon_svg(mode_icon, 15.0, theme.text_primary))
                .on_press(AppMessage::ToggleThemeMode)
                .padding(SPACE_2)
                .style(move |_, status| theme.ghost_button_style(status)),
        ]
        .spacing(SPACE_2)
        .align_y(Alignment::Center);

        container(
            row![
                breadcrumb,
                container(controls)
                    .align_x(Alignment::End)
                    .width(Length::Fill),
            ]
            .align_y(Alignment::Center)
            .width(Length::Fill),
        )
        .height(Length::Fixed(TOPBAR_HEIGHT))
        .padding([0.0, SPACE_4])
        .center_y(Length::Fixed(TOPBAR_HEIGHT))
        .style(move |_| theme.topbar_style())
        .into()
    }

    fn view_route_body(&self, theme: EstudyTheme) -> Element<'_, AppMessage> {
        let banner: Option<Element<'_, AppMessage>> = self.status_banner.as_ref().map(|msg| {
            container(banner_toast(
                Icon::Info,
                msg,
                Some(AppMessage::DismissBanner),
                theme,
            ))
            .padding([SPACE_2, SPACE_6])
            .width(Length::Fill)
            .into()
        });

        let today = self.clock.now_utc().date_naive();
        let main_view = match self.route {
            AppRoute::Home => view_home(
                &self.home_state,
                &self.habits_state,
                &self.second_brain_state,
                today,
                self.greeting(),
                theme,
                AppMessage::Home,
                AppMessage::Habits,
                AppMessage::SecondBrain,
                AppMessage::Navigate,
            ),
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
            other => view_module_placeholder(other, theme),
        };

        if let Some(banner_el) = banner {
            column![banner_el, main_view].height(Length::Fill).into()
        } else {
            main_view
        }
    }
}

fn sidebar_group_header<'a>(
    label: &'static str,
    expanded: bool,
    theme: EstudyTheme,
) -> Element<'a, AppMessage> {
    if expanded {
        container(
            text(label)
                .size(10)
                .font(FONT_SANS_SEMIBOLD)
                .color(theme.text_muted),
        )
        .height(Length::Fixed(22.0))
        .center_y(Length::Fixed(22.0))
        .padding([0.0, 10.0])
        .into()
    } else {
        container(hairline_divider(theme))
            .height(Length::Fixed(22.0))
            .center_y(Length::Fixed(22.0))
            .padding([0.0, 4.0])
            .into()
    }
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
        .padding([TOPBAR_HEIGHT + SPACE_2, SPACE_4])
        .align_x(Alignment::End)
        .into()
}

fn view_modules_popover<'a>(active: AppRoute, theme: EstudyTheme) -> Element<'a, AppMessage> {
    let routes = [
        (AppRoute::Home, Icon::Home),
        (AppRoute::Projects, Icon::FolderKanban),
        (AppRoute::Calendar, Icon::Calendar),
        (AppRoute::Studies, Icon::GraduationCap),
        (AppRoute::Bookshelf, Icon::BookOpen),
        (AppRoute::GardenAndPets, Icon::Sprout),
        (AppRoute::Shop, Icon::ShoppingBag),
        (AppRoute::Settings, Icon::Settings),
    ];

    let mut items = column![].spacing(4.0);
    for (route, icon) in routes {
        let is_active = route == active;
        let text_color = if is_active {
            theme.on_accent
        } else {
            theme.text_primary
        };
        let icon_color = if is_active {
            theme.on_accent
        } else {
            theme.accent
        };

        items = items.push(
            button(
                row![
                    icon_svg(icon, 14.0, icon_color),
                    text(route.label()).size(13).color(text_color),
                ]
                .spacing(SPACE_2)
                .align_y(Alignment::Center),
            )
            .on_press(AppMessage::Navigate(route))
            .width(Length::Fill)
            .padding([SPACE_2, SPACE_3])
            .style(move |_, status| {
                if is_active {
                    theme.primary_button_style(status)
                } else {
                    theme.subtle_button_style(status)
                }
            }),
        );
    }

    container(card(items, theme).width(Length::Fixed(220.0)))
        .width(Length::Fill)
        .height(Length::Fill)
        .padding([TOPBAR_HEIGHT + SPACE_2, SPACE_4])
        .align_x(Alignment::End)
        .into()
}

fn view_module_placeholder<'a>(route: AppRoute, theme: EstudyTheme) -> Element<'a, AppMessage> {
    let back_btn = button(
        row![
            icon_svg(Icon::ChevronLeft, 14.0, theme.on_accent),
            text("Voltar para Home").size(13).color(theme.on_accent),
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
                section_header(Icon::Layers, route.label(), theme),
                text("Este módulo entra nos próximos marcos do Roadmap mantendo a navegação e os presets ativos.")
                    .size(13)
                    .color(theme.text_muted),
                back_btn,
            ]
            .spacing(SPACE_3),
            theme,
        )
        .width(Length::Fixed(480.0)),
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
