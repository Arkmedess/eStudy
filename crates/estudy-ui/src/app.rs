//! Shell principal do aplicativo **eStudy** (MVU puro desacoplado de janela).

use chrono::Timelike;
use estudy_domain::{CalendarViewMode, ClockPort, EntityId, PomodoroPreset, TaskStatus};
use estudy_ui_kit::widgets::{
    card, category_pill, category_tile, color_dot, empty_placeholder, hairline_divider, icon_svg,
    kbd_hint, modal_surface, section_header, soft_surface,
};
use estudy_ui_kit::{
    CategoryColor, CosmeticLoadout, EstudyTheme, FONT_MONO, FONT_SANS_MEDIUM, FONT_SANS_SEMIBOLD,
    FONT_SERIF_MEDIUM, Icon, RADIUS_PILL, SIDEBAR_COMPACT_WIDTH, SIDEBAR_WIDTH, SPACE_1, SPACE_2,
    SPACE_3, SPACE_4, SPACE_6, TOPBAR_HEIGHT, ThemeMode, ThemePreset,
};
use iced::keyboard::{self, Key, Modifiers};
use iced::widget::{button, column, container, row, scrollable, stack, text};
use iced::{Alignment, Border, Color, Element, Length, Subscription};
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
    /// Visão rápida de um Projeto e suas tarefas abertas sem sair da tela.
    ProjectOverview(EntityId),
    /// Detalhes rápidos de uma tarefa com disparo direto para o Pomodoro.
    TaskQuick(EntityId),
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
    /// Seleciona diretamente um preset de tema oficial (tela Ajustes).
    SelectThemePreset(ThemePreset),
    /// Alterna entre modo claro e escuro.
    ToggleThemeMode,
    /// Abre ou fecha a Gaveta Flutuante "Hoje" a partir do header.
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

    /// Informa se a barra de status inferior deve aparecer sob demanda.
    #[must_use]
    pub const fn has_active_status_dock(&self) -> bool {
        self.focus_state.needs_ticker() || self.status_banner.is_some()
    }

    /// Lista canônica de rotas funcionais expostas na navegação primária (sem dead-ends).
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
                self.nav_sidebar_open = false;
            }
            AppMessage::QuickFocusShortcut => {
                self.notifications_open = false;
                self.modules_menu_open = false;
                self.active_popup = None;
                self.nav_sidebar_open = false;
                self.focus_state.fullscreen_active = true;
                self.status_banner =
                    Some("Modo Foco aberto (Esc para sair da tela cheia).".to_string());
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
            AppMessage::CloseOverlays => self.close_top_overlay(),
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
                AppRoute::Projects => {
                    self.nav_sidebar_open = false;
                    self.projects_state
                        .update(ProjectsMessage::ToggleSidebar, self.clock.now_utc());
                }
                AppRoute::Calendar => {
                    self.nav_sidebar_open = false;
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
                if let SecondBrainMessage::SelectNote(ref id) = msg
                    && self.route == AppRoute::Home
                {
                    self.active_popup = Some(QuickPopup::Note(id.clone()));
                }
                self.second_brain_state.update(msg, now);
            }
        }
    }

    /// Fecha exatamente a camada superior ativa na pilha de overlays (`Esc`).
    fn close_top_overlay(&mut self) {
        if self.palette.is_open {
            self.palette.close();
        } else if self.active_popup.is_some() {
            self.active_popup = None;
        } else if self.notifications_open || self.modules_menu_open {
            self.notifications_open = false;
            self.modules_menu_open = false;
        } else if self.nav_sidebar_open {
            self.nav_sidebar_open = false;
        } else if self.projects_state.drawer.is_some() {
            self.projects_state.flush_deferred();
            self.projects_state.drawer = None;
        } else if self.calendar_state.today_drawer_open {
            self.calendar_state.today_drawer_open = false;
        } else if self.focus_state.fullscreen_active {
            self.focus_state.fullscreen_active = false;
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
        let mut main_col = column![topbar, body]
            .width(Length::Fill)
            .height(Length::Fill);
        if self.has_active_status_dock() {
            main_col = main_col.push(self.view_status_dock(theme));
        }

        // Trilho compacto fixo (64px) garante largura constante do palco principal (zero reflow).
        let compact_rail = self.view_sidebar_panel(false, theme);
        let shell_layout: Element<'_, AppMessage> = row![compact_rail, main_col]
            .width(Length::Fill)
            .height(Length::Fill)
            .into();

        let base_shell = container(shell_layout)
            .width(Length::Fill)
            .height(Length::Fill)
            .style(move |_| container::Style {
                background: Some(theme.bg.into()),
                text_color: Some(theme.text_primary),
                ..container::Style::default()
            });

        // Quando expandida, a barra lateral sobrepõe o conteúdo com scrim leve clicável.
        let shell: Element<'_, AppMessage> = if self.nav_sidebar_open {
            stack![base_shell, self.view_sidebar_overlay(theme)].into()
        } else {
            base_shell.into()
        };

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
            shell
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
            let now = self.clock.now_utc();
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
            HomeMessage::PreviewProject(title) => {
                if let Some(proj) = self
                    .projects_state
                    .projects
                    .iter()
                    .find(|p| p.title.eq_ignore_ascii_case(&title))
                    .or_else(|| self.projects_state.projects.first())
                {
                    self.active_popup = Some(QuickPopup::ProjectOverview(proj.id.clone()));
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
                    self.active_popup = Some(QuickPopup::TaskQuick(task.id.clone()));
                }
            }
            HomeMessage::RetryLoad | HomeMessage::SeedStarterWorkspace => {
                self.home_state =
                    HomeLoadState::Ready(Box::new(HomeDashboardData::sample_workspace("Arthur")));
            }
        }
    }

    fn view_sidebar_overlay(&self, theme: EstudyTheme) -> Element<'_, AppMessage> {
        let panel = self.view_sidebar_panel(true, theme);
        let scrim = button(container(text("")).width(Length::Fill).height(Length::Fill))
            .on_press(AppMessage::ToggleNavSidebar)
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(0.0)
            .style(move |_, _| button::Style {
                background: Some(Color::from_rgba(0.0, 0.0, 0.0, 0.18).into()),
                ..button::Style::default()
            });

        row![panel, scrim]
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    fn view_sidebar_panel(&self, expanded: bool, theme: EstudyTheme) -> Element<'_, AppMessage> {
        let header = Self::view_sidebar_toggle_header(expanded, theme);

        let open_tasks = self.open_tasks_count();
        let (done_habits, total_habits) = self
            .habits_state
            .today_progress(self.clock.now_utc().date_naive());
        let notes_count = self.second_brain_state.notes.len();

        let mut space_section =
            column![sidebar_group_header("SEU ESPAÇO", expanded, theme)].spacing(4.0);
        for &route in Self::primary_nav_routes() {
            let (icon, badge) = match route {
                AppRoute::Home => (Icon::Home, None),
                AppRoute::Projects => (
                    Icon::CheckSquare,
                    (open_tasks > 0).then(|| open_tasks.to_string()),
                ),
                AppRoute::Habits => (
                    Icon::Flame,
                    (total_habits > 0).then(|| format!("{done_habits}/{total_habits}")),
                ),
                AppRoute::SecondBrain => (
                    Icon::BookOpen,
                    (notes_count > 0).then(|| notes_count.to_string()),
                ),
                AppRoute::Calendar => (Icon::Calendar, None),
                _ => (Icon::Layers, None),
            };
            space_section =
                space_section.push(self.sidebar_nav_item(route, icon, badge, expanded, theme));
        }

        let quick_actions_section = column![
            sidebar_group_header("AÇÕES RÁPIDAS", expanded, theme),
            Self::sidebar_action_item(
                Icon::Clock,
                "Timeline Hoje",
                AppMessage::ToggleTodayDrawer,
                self.calendar_state.today_drawer_open,
                expanded,
                theme,
            ),
            Self::sidebar_action_item(
                Icon::Timer,
                if self.is_focus_ticking() {
                    "Foco ativo"
                } else {
                    "Iniciar foco"
                },
                AppMessage::OpenQuickPopup(QuickPopup::FocusQuick),
                self.is_focus_ticking(),
                expanded,
                theme,
            ),
        ]
        .spacing(4.0);

        let nav_scroll = scrollable(
            column![space_section, quick_actions_section]
                .spacing(SPACE_4)
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
            SIDEBAR_WIDTH + 28.0
        } else {
            SIDEBAR_COMPACT_WIDTH
        };
        container(col)
            .width(Length::Fixed(width))
            .height(Length::Fill)
            .padding([SPACE_3, 13.0])
            .style(move |_| {
                if expanded {
                    container::Style {
                        background: Some(theme.panel.into()),
                        text_color: Some(theme.text_primary),
                        border: Border {
                            color: theme.border_subtle,
                            width: 1.0,
                            radius: 0.0.into(),
                        },
                        ..container::Style::default()
                    }
                } else {
                    theme.sidebar_style()
                }
            })
            .into()
    }

    fn sidebar_action_item<'a>(
        icon: Icon,
        label: &'static str,
        message: AppMessage,
        highlighted: bool,
        expanded: bool,
        theme: EstudyTheme,
    ) -> Element<'a, AppMessage> {
        let icon_color = if highlighted {
            theme.accent
        } else {
            theme.text_muted
        };
        let icon_slot = container(icon_svg(icon, 16.0, icon_color))
            .width(Length::Fixed(46.0))
            .height(Length::Fixed(24.0))
            .center_x(Length::Fixed(46.0))
            .center_y(Length::Fixed(24.0));

        let short_label = if label.contains("Hoje") {
            "Hoje"
        } else {
            "Foco"
        };

        let content: Element<'a, AppMessage> = if expanded {
            row![
                icon_slot,
                text(label)
                    .size(13)
                    .font(FONT_SANS_MEDIUM)
                    .color(if highlighted {
                        theme.accent
                    } else {
                        theme.text_primary
                    }),
            ]
            .spacing(SPACE_1)
            .align_y(Alignment::Center)
            .width(Length::Fill)
            .into()
        } else {
            column![
                icon_slot,
                text(short_label)
                    .size(10)
                    .font(FONT_SANS_MEDIUM)
                    .color(if highlighted {
                        theme.accent
                    } else {
                        theme.text_muted
                    }),
            ]
            .spacing(2.0)
            .align_x(Alignment::Center)
            .width(Length::Fill)
            .into()
        };

        button(content)
            .on_press(message)
            .width(Length::Fill)
            .height(Length::Fixed(46.0))
            .padding([3.0, 0.0])
            .style(move |_, status| theme.ghost_button_style(status))
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

        let icon_slot = container(icon_svg(icon, 16.0, icon_color))
            .width(Length::Fixed(46.0))
            .height(Length::Fixed(24.0))
            .center_x(Length::Fixed(46.0))
            .center_y(Length::Fixed(24.0));

        let short_label = sidebar_short_label(route);

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
            column![
                icon_slot,
                text(short_label).size(10).font(font).color(text_color),
            ]
            .spacing(2.0)
            .align_x(Alignment::Center)
            .width(Length::Fill)
            .into()
        };

        button(content)
            .on_press(AppMessage::Navigate(route))
            .width(Length::Fill)
            .height(Length::Fixed(46.0))
            .padding([3.0, 0.0])
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
            QuickPopup::ProjectOverview(project_id) => self.view_project_popup(project_id, theme),
            QuickPopup::TaskQuick(task_id) => self.view_task_popup(task_id, theme),
        };
        modal_surface(body, 500.0, theme)
    }

    fn view_project_popup<'a>(
        &'a self,
        project_id: &EntityId,
        theme: EstudyTheme,
    ) -> Element<'a, AppMessage> {
        let close_btn = button(icon_svg(Icon::X, 15.0, theme.text_muted))
            .on_press(AppMessage::CloseQuickPopup)
            .padding(SPACE_1)
            .style(move |_, status| theme.ghost_button_style(status));

        let Some(project) = self
            .projects_state
            .projects
            .iter()
            .find(|p| &p.id == project_id)
        else {
            return empty_placeholder(
                Icon::CheckSquare,
                "Projeto não encontrado",
                "Este projeto pode ter sido removido.",
                None,
                theme,
            );
        };

        let mut tasks_list = column![].spacing(SPACE_2);
        let project_tasks: Vec<_> = self
            .projects_state
            .tasks
            .iter()
            .filter(|t| {
                t.project_links
                    .iter()
                    .any(|link| &link.project_id == project_id)
            })
            .take(5)
            .collect();

        if project_tasks.is_empty() {
            tasks_list = tasks_list.push(
                text("Nenhuma tarefa cadastrada ainda neste projeto.")
                    .size(12)
                    .color(theme.text_muted),
            );
        } else {
            for task in project_tasks {
                tasks_list = tasks_list.push(view_project_popup_task_row(task, theme));
            }
        }

        column![
            row![
                category_pill(project.status.label(), CategoryColor::Sage, theme),
                container(close_btn)
                    .align_x(Alignment::End)
                    .width(Length::Fill),
            ]
            .align_y(Alignment::Center),
            text(&project.title)
                .size(22)
                .font(FONT_SERIF_MEDIUM)
                .color(theme.text_primary),
            text(&project.description).size(13).color(theme.text_muted),
            hairline_divider(theme),
            tasks_list,
            row![
                button(
                    text("Abrir projeto completo")
                        .size(12)
                        .font(FONT_SANS_MEDIUM)
                        .color(theme.on_accent),
                )
                .on_press(AppMessage::Navigate(AppRoute::Projects))
                .padding([7.0, 14.0])
                .style(move |_, status| theme.primary_button_style(status)),
            ],
        ]
        .spacing(SPACE_3)
        .into()
    }

    fn view_task_popup<'a>(
        &'a self,
        task_id: &EntityId,
        theme: EstudyTheme,
    ) -> Element<'a, AppMessage> {
        let close_btn = button(icon_svg(Icon::X, 15.0, theme.text_muted))
            .on_press(AppMessage::CloseQuickPopup)
            .padding(SPACE_1)
            .style(move |_, status| theme.ghost_button_style(status));

        let Some(task) = self.projects_state.tasks.iter().find(|t| &t.id == task_id) else {
            return empty_placeholder(
                Icon::CheckSquare,
                "Tarefa não encontrada",
                "Esta tarefa pode ter sido removida.",
                None,
                theme,
            );
        };

        column![
            row![
                category_pill(task.status.label(), CategoryColor::Ochre, theme),
                container(close_btn)
                    .align_x(Alignment::End)
                    .width(Length::Fill),
            ]
            .align_y(Alignment::Center),
            text(&task.title)
                .size(20)
                .font(FONT_SERIF_MEDIUM)
                .color(theme.text_primary),
            soft_surface(
                text(if task.description_md.trim().is_empty() {
                    "Sem descrição detalhada."
                } else {
                    task.description_md.as_str()
                })
                .size(13)
                .color(theme.text_primary),
                theme,
            )
            .padding(SPACE_3)
            .width(Length::Fill),
            row![
                button(
                    row![
                        icon_svg(Icon::Play, 13.0, theme.on_accent),
                        text("Iniciar foco")
                            .size(12)
                            .font(FONT_SANS_MEDIUM)
                            .color(theme.on_accent),
                    ]
                    .spacing(6.0)
                    .align_y(Alignment::Center),
                )
                .on_press(AppMessage::Projects(ProjectsMessage::StartFocusForTask(
                    task.id.clone(),
                )))
                .padding([7.0, 14.0])
                .style(move |_, status| theme.primary_button_style(status)),
                button(text("Abrir em Projetos").size(12).color(theme.text_primary))
                    .on_press(AppMessage::Navigate(AppRoute::Projects))
                    .padding([7.0, 14.0])
                    .style(move |_, status| theme.subtle_button_style(status)),
            ]
            .spacing(SPACE_2),
        ]
        .spacing(SPACE_3)
        .into()
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

    fn view_topbar_breadcrumb(&self, theme: EstudyTheme) -> Element<'_, AppMessage> {
        let context_subtitle = match self.route {
            AppRoute::Home => format!("{}", self.clock.now_utc().format("%d/%m/%Y")),
            AppRoute::Projects => self
                .projects_state
                .active_project()
                .map_or_else(|| "Todos os projetos".to_string(), |p| p.title.clone()),
            AppRoute::Habits => {
                let (done, total) = self
                    .habits_state
                    .today_progress(self.clock.now_utc().date_naive());
                format!("{done}/{total} hoje")
            }
            AppRoute::SecondBrain => format!("{} notas", self.second_brain_state.notes.len()),
            AppRoute::Calendar => self.calendar_state.view_mode.label().to_string(),
            AppRoute::Settings => self.loadout.theme_preset.label().to_string(),
            _ => String::new(),
        };

        let mut breadcrumb = row![
            color_dot(theme.accent),
            text(self.route.label())
                .size(14)
                .font(FONT_SANS_SEMIBOLD)
                .color(theme.text_primary),
        ]
        .spacing(SPACE_2)
        .align_y(Alignment::Center);

        if !context_subtitle.is_empty() {
            breadcrumb = breadcrumb
                .push(text("·").size(12).color(theme.text_muted))
                .push(text(context_subtitle).size(12).color(theme.text_muted));
        }
        breadcrumb.into()
    }

    fn view_topbar(&self, theme: EstudyTheme) -> Element<'_, AppMessage> {
        let breadcrumb = self.view_topbar_breadcrumb(theme);

        let cmd_btn = button(
            row![
                icon_svg(Icon::Search, 14.0, theme.text_muted),
                text("Buscar ou criar...").size(12).color(theme.text_muted),
                container(kbd_hint("Ctrl+K", theme))
                    .align_x(Alignment::End)
                    .width(Length::Fill),
            ]
            .spacing(SPACE_2)
            .align_y(Alignment::Center)
            .width(Length::Fill),
        )
        .on_press(AppMessage::Palette(PaletteMessage::Toggle))
        .width(Length::Fixed(260.0))
        .padding([5.0, SPACE_3])
        .style(move |_, status| theme.pill_button_style(false, status));

        let (done_habits, total_habits) = self
            .habits_state
            .today_progress(self.clock.now_utc().date_naive());
        let habits_chip = button(
            row![
                icon_svg(Icon::Flame, 14.0, theme.accent),
                text(format!("{done_habits}/{total_habits}"))
                    .size(12)
                    .font(FONT_MONO)
                    .color(theme.text_primary),
            ]
            .spacing(5.0)
            .align_y(Alignment::Center),
        )
        .on_press(AppMessage::OpenQuickPopup(QuickPopup::HabitsQuick))
        .padding([5.0, 10.0])
        .style(move |_, status| theme.subtle_button_style(status));

        let today_btn = button(
            row![
                icon_svg(Icon::Clock, 14.0, theme.text_primary),
                text("Hoje")
                    .size(12)
                    .font(FONT_SANS_MEDIUM)
                    .color(theme.text_primary),
            ]
            .spacing(5.0)
            .align_y(Alignment::Center),
        )
        .on_press(AppMessage::ToggleTodayDrawer)
        .padding([5.0, 10.0])
        .style(move |_, status| {
            theme.pill_button_style(self.calendar_state.today_drawer_open, status)
        });

        let focus_label = if self.is_focus_ticking() {
            let rem = self.focus_state.timer.remaining_seconds();
            format!("Focar · {:02}:{:02}", rem / 60, rem % 60)
        } else {
            "Focar".to_string()
        };

        let controls = row![
            today_btn,
            habits_chip,
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
        ]
        .spacing(SPACE_2)
        .align_y(Alignment::Center);

        container(
            row![
                container(breadcrumb).width(Length::FillPortion(1)),
                container(cmd_btn)
                    .center_x(Length::FillPortion(1))
                    .width(Length::FillPortion(1)),
                container(controls)
                    .align_x(Alignment::End)
                    .width(Length::FillPortion(1)),
            ]
            .align_y(Alignment::Center)
            .width(Length::Fill),
        )
        .height(Length::Fixed(TOPBAR_HEIGHT + 4.0))
        .padding([0.0, SPACE_4])
        .center_y(Length::Fixed(TOPBAR_HEIGHT + 4.0))
        .style(move |_| theme.topbar_style())
        .into()
    }

    fn view_status_dock(&self, theme: EstudyTheme) -> Element<'_, AppMessage> {
        let mut left = row![].spacing(SPACE_2).align_y(Alignment::Center);
        if self.is_focus_ticking() {
            let rem = self.focus_state.timer.remaining_seconds();
            let title = self
                .focus_state
                .active_target_title
                .as_deref()
                .unwrap_or("Bloco de foco ativo");
            left = left
                .push(color_dot(theme.accent))
                .push(
                    text(format!("{:02}:{:02}", rem / 60, rem % 60))
                        .size(12)
                        .font(FONT_MONO)
                        .color(theme.accent),
                )
                .push(
                    text(title)
                        .size(12)
                        .font(FONT_SANS_MEDIUM)
                        .color(theme.text_primary),
                );
        } else if let Some(msg) = &self.status_banner {
            left = left
                .push(icon_svg(Icon::Info, 13.0, theme.accent))
                .push(text(msg).size(12).color(theme.text_primary));
        }

        let mut right = row![].spacing(SPACE_2).align_y(Alignment::Center);
        if self.is_focus_ticking() {
            right = right
                .push(
                    button(text("Pausar").size(11).color(theme.text_primary))
                        .on_press(AppMessage::Focus(FocusMessage::Pause))
                        .padding([3.0, 10.0])
                        .style(move |_, status| theme.subtle_button_style(status)),
                )
                .push(
                    button(text("Tela cheia").size(11).color(theme.accent))
                        .on_press(AppMessage::QuickFocusShortcut)
                        .padding([3.0, 10.0])
                        .style(move |_, status| theme.ghost_button_style(status)),
                );
        }
        if self.status_banner.is_some() {
            right = right.push(
                button(icon_svg(Icon::X, 13.0, theme.text_muted))
                    .on_press(AppMessage::DismissBanner)
                    .padding(3.0)
                    .style(move |_, status| theme.ghost_button_style(status)),
            );
        }

        container(
            row![
                left,
                container(right).align_x(Alignment::End).width(Length::Fill),
            ]
            .align_y(Alignment::Center)
            .width(Length::Fill),
        )
        .height(Length::Fixed(34.0))
        .padding([0.0, SPACE_4])
        .center_y(Length::Fixed(34.0))
        .style(move |_| container::Style {
            background: Some(theme.surface_soft.into()),
            border: Border {
                color: theme.border_subtle,
                width: 1.0,
                radius: 0.0.into(),
            },
            ..container::Style::default()
        })
        .into()
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
                            text(preset.label())
                                .size(13)
                                .font(FONT_SANS_SEMIBOLD)
                                .color(if active {
                                    theme.on_accent
                                } else {
                                    theme.text_primary
                                }),
                        ]
                        .spacing(SPACE_2)
                        .align_y(Alignment::Center),
                        text(if active { "Ativo" } else { "Aplicar" })
                            .size(11)
                            .color(if active {
                                theme.on_accent
                            } else {
                                theme.text_muted
                            }),
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
                        theme.subtle_button_style(status)
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

        let content = column![
            column![
                text("Ajustes")
                    .size(28)
                    .font(FONT_SERIF_MEDIUM)
                    .color(theme.text_primary),
                text("Aparência, temas validados em WCAG AA e atalhos rápidos do teclado.")
                    .size(13)
                    .color(theme.text_muted),
            ]
            .spacing(4.0),
            card(
                column![
                    section_header(Icon::Palette, "Aparência e Presets", theme),
                    preset_row,
                    hairline_divider(theme),
                    row![
                        button(
                            row![
                                icon_svg(mode_icon, 15.0, theme.text_primary),
                                text(mode_label)
                                    .size(13)
                                    .font(FONT_SANS_MEDIUM)
                                    .color(theme.text_primary),
                            ]
                            .spacing(SPACE_2)
                            .align_y(Alignment::Center),
                        )
                        .on_press(AppMessage::ToggleThemeMode)
                        .padding([SPACE_2, SPACE_4])
                        .style(move |_, status| theme.subtle_button_style(status)),
                    ],
                ]
                .spacing(SPACE_4),
                theme,
            ),
            Self::view_settings_shortcuts_card(theme),
        ]
        .spacing(SPACE_4)
        .max_width(920.0)
        .width(Length::Fill);

        scrollable(
            container(content)
                .padding([SPACE_6, SPACE_6])
                .center_x(Length::Fill)
                .width(Length::Fill),
        )
        .height(Length::Fill)
        .into()
    }

    fn view_settings_shortcuts_card<'a>(theme: EstudyTheme) -> Element<'a, AppMessage> {
        card(
            column![
                section_header(Icon::Command, "Atalhos de Teclado", theme),
                row![
                    kbd_hint("Ctrl+K", theme),
                    text("Abrir barra de comando e busca global")
                        .size(13)
                        .color(theme.text_primary),
                ]
                .spacing(SPACE_3)
                .align_y(Alignment::Center),
                row![
                    kbd_hint("Ctrl+B", theme),
                    text("Alternar barra lateral ou painel contextual")
                        .size(13)
                        .color(theme.text_primary),
                ]
                .spacing(SPACE_3)
                .align_y(Alignment::Center),
                row![
                    kbd_hint("Esc", theme),
                    text("Fechar uma camada por vez (popup, drawer, barra ou foco)")
                        .size(13)
                        .color(theme.text_primary),
                ]
                .spacing(SPACE_3)
                .align_y(Alignment::Center),
            ]
            .spacing(SPACE_3),
            theme,
        )
        .into()
    }

    fn view_route_body(&self, theme: EstudyTheme) -> Element<'_, AppMessage> {
        let today = self.clock.now_utc().date_naive();
        match self.route {
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
            AppRoute::Settings => self.view_settings_screen(theme),
            other => view_module_placeholder(other, theme),
        }
    }
}

const fn sidebar_short_label(route: AppRoute) -> &'static str {
    match route {
        AppRoute::Home => "Início",
        AppRoute::Projects => "Projetos",
        AppRoute::Habits => "Hábitos",
        AppRoute::SecondBrain => "Notas",
        AppRoute::Calendar => "Agenda",
        AppRoute::Settings => "Ajustes",
        _ => route.label(),
    }
}

fn view_project_popup_task_row(
    task: &estudy_domain::Task,
    theme: EstudyTheme,
) -> Element<'_, AppMessage> {
    let done = task.status == TaskStatus::Done;
    soft_surface(
        row![
            icon_svg(
                if done {
                    Icon::CheckCircle2
                } else {
                    Icon::CheckSquare
                },
                15.0,
                if done { theme.success } else { theme.accent },
            ),
            text(&task.title)
                .size(13)
                .font(FONT_SANS_MEDIUM)
                .color(if done {
                    theme.text_muted
                } else {
                    theme.text_primary
                })
                .width(Length::Fill),
            button(icon_svg(Icon::Play, 13.0, theme.on_accent))
                .on_press(AppMessage::Projects(ProjectsMessage::StartFocusForTask(
                    task.id.clone(),
                )))
                .padding([4.0, 8.0])
                .style(move |_, status| theme.primary_button_style(status)),
        ]
        .spacing(SPACE_2)
        .align_y(Alignment::Center),
        theme,
    )
    .padding([SPACE_2, SPACE_3])
    .into()
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
