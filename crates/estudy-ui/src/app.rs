//! Shell principal do aplicativo **eStudy** (MVU puro desacoplado de janela).

use chrono::Timelike;
use estudy_domain::ClockPort;
use estudy_ui_kit::widgets::{card, icon_svg, section_header};
use estudy_ui_kit::{
    CosmeticLoadout, EstudyTheme, Icon, SPACE_2, SPACE_3, SPACE_4, SPACE_6, TOPBAR_HEIGHT,
    ThemeMode, ThemePreset,
};
use iced::keyboard::{self, Key, Modifiers};
use iced::widget::{button, column, container, row, stack, text};
use iced::{Alignment, Element, Length, Subscription};
use std::sync::Arc;

use crate::command_palette::{AppRoute, CommandPaletteState, PaletteAction, PaletteMessage};
use crate::features::home::{
    HomeDashboardData, HomeLoadState, HomeMessage, MoodViewMode, view_home,
};
use crate::features::projects::{ProjectsMessage, ProjectsState, view_projects};

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
    /// Alterna a barra contextual esquerda na tela ativa (`Ctrl+B`).
    ToggleContextSidebar,
    /// Alterna o próximo preset de tema oficial.
    CycleThemePreset,
    /// Alterna entre modo claro e escuro.
    ToggleThemeMode,
    /// Mensagem destinada à Barra de Comando (`Ctrl+K`).
    Palette(PaletteMessage),
    /// Mensagem destinada à fatia Home.
    Home(HomeMessage),
    /// Mensagem destinada à fatia Projetos.
    Projects(ProjectsMessage),
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
    /// Estado da fatia Home.
    pub home_state: HomeLoadState,
    /// Estado da fatia Projetos (Kanban, frameworks e Drawer da tarefa).
    pub projects_state: ProjectsState,
    /// Mensagem de feedback transitório da ação mais recente.
    pub status_banner: Option<String>,
}

impl EstudyApp {
    /// Inicializa o aplicativo com um relógio injetado e estado inicial da Home.
    #[must_use]
    pub fn new(clock: Arc<dyn ClockPort>, home_state: HomeLoadState) -> Self {
        let now = clock.now_utc();
        let projects_state = ProjectsState::sample(now.date_naive(), now);
        Self::with_projects(clock, home_state, projects_state)
    }

    /// Inicializa o aplicativo com estado de Projetos carregado da persistência.
    #[must_use]
    pub fn with_projects(
        clock: Arc<dyn ClockPort>,
        home_state: HomeLoadState,
        projects_state: ProjectsState,
    ) -> Self {
        Self {
            clock,
            route: AppRoute::Home,
            loadout: CosmeticLoadout::default(),
            palette: CommandPaletteState::new(),
            notifications_open: false,
            modules_menu_open: false,
            home_state,
            projects_state,
            status_banner: None,
        }
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

    /// Processa uma mensagem MVU atualizando o estado puro do aplicativo.
    pub fn update(&mut self, message: AppMessage) {
        match message {
            AppMessage::Navigate(route) => {
                self.route = route;
                self.palette.close();
                self.notifications_open = false;
                self.modules_menu_open = false;
            }
            AppMessage::QuickFocusShortcut => {
                self.notifications_open = false;
                self.modules_menu_open = false;
                self.status_banner = Some("Bloco de foco de 25 min preparado.".to_string());
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
                self.projects_state.drawer = None;
            }
            AppMessage::ToggleContextSidebar => {
                if self.route == AppRoute::Projects {
                    self.projects_state
                        .update(ProjectsMessage::ToggleSidebar, self.clock.now_utc());
                }
            }
            AppMessage::CycleThemePreset => {
                self.loadout.theme_preset = self.loadout.theme_preset.next();
            }
            AppMessage::ToggleThemeMode => {
                self.loadout.theme_mode = self.loadout.theme_mode.toggle();
            }
            AppMessage::Palette(msg) => self.handle_palette_message(msg),
            AppMessage::Home(msg) => self.handle_home_message(msg),
            AppMessage::Projects(msg) => {
                let now = self.clock.now_utc();
                self.projects_state.update(msg, now);
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
        let topbar = self.view_topbar(theme);
        let body = self.view_route_body(theme);

        let shell = container(
            column![topbar, body]
                .width(Length::Fill)
                .height(Length::Fill),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .style(move |_| container::Style {
            background: Some(theme.bg.into()),
            text_color: Some(theme.text_primary),
            ..container::Style::default()
        });

        if self.palette.is_open {
            stack![shell, self.palette.view(theme, AppMessage::Palette)].into()
        } else if self.notifications_open {
            stack![shell, view_notifications_popover(theme)].into()
        } else if self.modules_menu_open {
            stack![shell, view_modules_popover(self.route, theme)].into()
        } else {
            shell.into()
        }
    }

    fn handle_palette_message(&mut self, message: PaletteMessage) {
        match message {
            PaletteMessage::Toggle => self.palette.toggle(),
            PaletteMessage::Close => self.palette.close(),
            PaletteMessage::QueryChanged(query) => self.palette.set_query(query),
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

    fn execute_palette_action(&mut self, action: PaletteAction) {
        self.palette.close();
        match action {
            PaletteAction::Navigate(route) => self.route = route,
            PaletteAction::CycleThemePreset => {
                self.loadout.theme_preset = self.loadout.theme_preset.next();
            }
            PaletteAction::ToggleThemeMode => {
                self.loadout.theme_mode = self.loadout.theme_mode.toggle();
            }
            PaletteAction::StartFocus(minutes) => {
                self.status_banner = Some(format!("Bloco de foco de {minutes} min preparado."));
            }
            PaletteAction::QuickCreateTask => {
                self.route = AppRoute::Projects;
                self.status_banner = Some("Criação rápida de tarefa pronta.".to_string());
            }
            PaletteAction::QuickCreateEvent => {
                self.route = AppRoute::Calendar;
                self.status_banner = Some("Calendário aberto para novo evento.".to_string());
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
            HomeMessage::StartFocusForPin(pin_id) => {
                self.status_banner = Some(format!("Foco vinculado ao item {pin_id}."));
            }
            HomeMessage::RetryLoad | HomeMessage::SeedStarterWorkspace => {
                self.home_state =
                    HomeLoadState::Ready(Box::new(HomeDashboardData::sample_workspace("Arthur")));
            }
        }
    }

    fn view_topbar(&self, theme: EstudyTheme) -> Element<'_, AppMessage> {
        let brand_btn = button(
            row![
                icon_svg(Icon::GraduationCap, 18.0, theme.accent),
                text("eStudy").size(15).color(theme.text_primary),
            ]
            .spacing(SPACE_2)
            .align_y(Alignment::Center),
        )
        .on_press(AppMessage::Navigate(AppRoute::Home))
        .padding([SPACE_2, SPACE_3])
        .style(move |_, status| theme.ghost_button_style(status));

        let cmd_btn = button(
            row![
                icon_svg(Icon::Search, 14.0, theme.text_muted),
                text("Ctrl+K").size(12).color(theme.text_muted),
            ]
            .spacing(SPACE_2)
            .align_y(Alignment::Center),
        )
        .on_press(AppMessage::Palette(PaletteMessage::Toggle))
        .padding([SPACE_2, SPACE_3])
        .style(move |_, status| theme.ghost_button_style(status));

        let mode_icon = match self.loadout.theme_mode {
            ThemeMode::Light => Icon::Moon,
            ThemeMode::Dark => Icon::Sun,
        };

        let controls = row![
            cmd_btn,
            button(
                row![
                    icon_svg(Icon::Timer, 14.0, theme.accent),
                    text("Focar").size(12).color(theme.text_primary),
                ]
                .spacing(SPACE_2)
                .align_y(Alignment::Center),
            )
            .on_press(AppMessage::QuickFocusShortcut)
            .padding([SPACE_2, SPACE_3])
            .style(move |_, status| theme.ghost_button_style(status)),
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
                .spacing(SPACE_2)
                .align_y(Alignment::Center),
            )
            .on_press(AppMessage::CycleThemePreset)
            .padding([SPACE_2, SPACE_3])
            .style(move |_, status| theme.ghost_button_style(status)),
            button(icon_svg(mode_icon, 15.0, theme.text_primary))
                .on_press(AppMessage::ToggleThemeMode)
                .padding(SPACE_2)
                .style(move |_, status| theme.ghost_button_style(status)),
            button(
                row![
                    icon_svg(Icon::LayoutGrid, 14.0, theme.accent),
                    text("Módulos").size(12).color(theme.text_primary),
                    icon_svg(Icon::ChevronDown, 13.0, theme.text_muted),
                ]
                .spacing(SPACE_2)
                .align_y(Alignment::Center),
            )
            .on_press(AppMessage::ToggleModulesMenu)
            .padding([SPACE_2, SPACE_3])
            .style(move |_, status| theme.ghost_button_style(status)),
        ]
        .spacing(SPACE_2)
        .align_y(Alignment::Center);

        container(
            row![
                brand_btn,
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
            container(
                row![
                    icon_svg(Icon::Info, 14.0, theme.accent),
                    text(msg).size(12).color(theme.text_primary),
                ]
                .spacing(SPACE_2)
                .align_y(Alignment::Center),
            )
            .padding([SPACE_2, SPACE_6])
            .width(Length::Fill)
            .into()
        });

        let main_view = match self.route {
            AppRoute::Home => view_home(
                &self.home_state,
                self.greeting(),
                theme,
                AppMessage::Home,
                AppMessage::Navigate,
            ),
            AppRoute::Projects => {
                let today_items = match &self.home_state {
                    HomeLoadState::Ready(data) => data.today_items.as_slice(),
                    _ => &[],
                };
                view_projects(
                    &self.projects_state,
                    self.clock.now_utc().date_naive(),
                    today_items,
                    theme,
                    AppMessage::Projects,
                )
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

fn view_notifications_popover<'a>(theme: EstudyTheme) -> Element<'a, AppMessage> {
    let close_btn = button(icon_svg(Icon::X, 14.0, theme.text_muted))
        .on_press(AppMessage::CloseOverlays)
        .padding(4.0)
        .style(move |_, status| theme.ghost_button_style(status));

    let popover = card(
        column![
            row![
                section_header(Icon::Trophy, "Notificações e Conquistas", theme),
                container(close_btn).align_x(Alignment::End).width(Length::Fill),
            ]
            .align_y(Alignment::Center),
            text("Nenhuma conquista pendente. Continue seu bloco de estudos para desbloquear novas medalhas.")
                .size(12)
                .color(theme.text_muted),
        ]
        .spacing(SPACE_3),
        theme,
    )
    .width(Length::Fixed(320.0));

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
                    theme.ghost_button_style(status)
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
