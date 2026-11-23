//! Estado e regras de interação da fatia Home Pessoal Viva.

use estudy_ui_kit::{BoardFrameStyle, HeatmapData, Icon};

/// Limite máximo de itens fixados em "Em foco".
pub const MAX_FOCUS_PINS: usize = 5;

/// Limite máximo de medalhas fixadas na vitrine do perfil.
pub const MAX_PINNED_MEDALS: usize = 3;

/// Estado de carregamento da Home cobrindo os 5 estados obrigatórios de UI.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HomeLoadState {
    /// Carregamento inicial em andamento.
    Loading,
    /// Falha ao carregar dados com opção de retentativa.
    Error(String),
    /// Dados carregados (pode estar vazio ou populado).
    Ready(Box<HomeDashboardData>),
}

/// Níveis de humor registrados no Diário de Humor (zero emojis, 100% ícones SVG).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MoodLevel {
    /// Ótimo / inspirado.
    Great,
    /// Bom / disposto.
    Good,
    /// Neutro / estável.
    Steady,
    /// Cansado / sobrecarregado.
    Tired,
}

impl MoodLevel {
    /// Todos os níveis selecionáveis na interface.
    pub const ALL: [Self; 4] = [Self::Great, Self::Good, Self::Steady, Self::Tired];

    /// Rótulo legível em português.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Great => "Ótimo",
            Self::Good => "Bom",
            Self::Steady => "Estável",
            Self::Tired => "Cansado",
        }
    }

    /// Ícone SVG correspondente.
    #[must_use]
    pub const fn icon(self) -> Icon {
        match self {
            Self::Great => Icon::Heart,
            Self::Good => Icon::Smile,
            Self::Steady => Icon::Meh,
            Self::Tired => Icon::CloudRain,
        }
    }
}

/// Registro diário de humor dos últimos 7 dias.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MoodDayEntry {
    /// Abreviação do dia (ex.: `Seg`).
    pub day_label: String,
    /// Humor registrado, se houver.
    pub level: Option<MoodLevel>,
}

/// Medalha exibida na vitrine do Perfil.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MedalBadge {
    /// Título curto da conquista.
    pub title: String,
    /// Rótulo de raridade.
    pub rarity: String,
}

/// Resumo do perfil local na coluna esquerda.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProfileSummary {
    /// Nome exibido do usuário.
    pub display_name: String,
    /// Dias seguidos de constância.
    pub streak_days: u32,
    /// Saldo atual de moedas verificado.
    pub coins: i64,
    /// Até 3 medalhas fixadas.
    pub pinned_medals: Vec<MedalBadge>,
}

/// Bloco de horário ou prazo listado no cartão "Hoje".
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TodayItem {
    /// Horário ou prazo curto (ex.: `14:00` ou `Prazo 18h`).
    pub time_label: String,
    /// Título da atividade ou tarefa.
    pub title: String,
    /// Projeto ou disciplina associada.
    pub context: String,
    /// Indica se já foi concluído.
    pub done: bool,
}

/// Resumo de projeto na grade central (2 colunas).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectCardSummary {
    /// Identificador do projeto.
    pub id: String,
    /// Nome do projeto.
    pub title: String,
    /// Status principal único (máximo 1 badge por cartão).
    pub status_label: String,
    /// Contagem em `A fazer`.
    pub todo_count: u16,
    /// Contagem em `Em andamento`.
    pub in_progress_count: u16,
    /// Contagem em `Concluído`.
    pub done_count: u16,
}

/// Item pinado na lista "Em foco" (`X/5`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FocusPinItem {
    /// Identificador da tarefa ou estudo.
    pub id: String,
    /// Título do item em foco.
    pub title: String,
    /// Origem resumida.
    pub context_label: String,
    /// Pomodoros realizados.
    pub completed_pomos: u16,
    /// Pomodoros estimados.
    pub estimated_pomos: u16,
}

/// Estado do Quadro Personalizável na coluna direita.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PinboardState {
    /// Título ou legenda exibida no quadro.
    pub caption: String,
    /// Moldura ativa escolhida pelo usuário.
    pub frame: BoardFrameStyle,
}

/// Dados consolidados da tela Home em 3 colunas.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HomeDashboardData {
    /// Perfil e vitrine de medalhas.
    pub profile: ProfileSummary,
    /// Blocos e prazos do dia ("Hoje").
    pub today_items: Vec<TodayItem>,
    /// Heatmap anual (365 dias).
    pub heatmap: HeatmapData,
    /// Nome e espécie do Pet exibido no canto do Heatmap.
    pub pet_badge_label: String,
    /// Projetos ativos exibidos na grade central.
    pub projects: Vec<ProjectCardSummary>,
    /// Itens fixados em "Em foco" (limitado a 5).
    pub focus_items: Vec<FocusPinItem>,
    /// Últimos 7 dias do Diário de Humor.
    pub mood_week: Vec<MoodDayEntry>,
    /// Quadro personalizável com moldura.
    pub pinboard: PinboardState,
}

/// Mensagens da fatia Home.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HomeMessage {
    /// Seleciona o humor do dia atual (último item da semana).
    SelectTodayMood(MoodLevel),
    /// Altera a moldura do Quadro Personalizável.
    SelectBoardFrame(BoardFrameStyle),
    /// Alterna conclusão de um item do cartão "Hoje".
    ToggleTodayItem(usize),
    /// Inicia sessão Pomodoro a partir de um item "Em foco".
    StartFocusForPin(String),
    /// Simula/dispara recarregamento após erro.
    RetryLoad,
    /// Cria o primeiro projeto ou tarefa a partir do estado vazio.
    SeedStarterWorkspace,
}

impl HomeDashboardData {
    /// Cria um estado inicial vazio para novos perfis (estado Empty acionável).
    #[must_use]
    pub fn empty(display_name: impl Into<String>) -> Self {
        Self {
            profile: ProfileSummary {
                display_name: display_name.into(),
                streak_days: 0,
                coins: 0,
                pinned_medals: Vec::new(),
            },
            today_items: Vec::new(),
            heatmap: HeatmapData::default(),
            pet_badge_label: "Mochi · Gato Calico".to_string(),
            projects: Vec::new(),
            focus_items: Vec::new(),
            mood_week: default_mood_week(None),
            pinboard: PinboardState {
                caption: "Seu espaço de inspiração e foco".to_string(),
                frame: BoardFrameStyle::Japandi,
            },
        }
    }

    /// Cria um conjunto inicial realista para preview e onboarding local.
    #[must_use]
    pub fn sample_workspace(display_name: impl Into<String>) -> Self {
        Self {
            profile: ProfileSummary {
                display_name: display_name.into(),
                streak_days: 14,
                coins: 340,
                pinned_medals: vec![
                    MedalBadge {
                        title: "Primeiro Bloco".to_string(),
                        rarity: "Bronze".to_string(),
                    },
                    MedalBadge {
                        title: "Constância 14d".to_string(),
                        rarity: "Prata".to_string(),
                    },
                ],
            },
            today_items: sample_today_items(),
            heatmap: sample_heatmap(),
            pet_badge_label: "Mochi · Focado".to_string(),
            projects: sample_projects(),
            focus_items: sample_focus_items(),
            mood_week: default_mood_week(Some(MoodLevel::Good)),
            pinboard: PinboardState {
                caption: "Um bloco de foco por vez.".to_string(),
                frame: BoardFrameStyle::Japandi,
            },
        }
    }

    /// Indica se o usuário ainda não possui projetos, blocos do dia nem itens em foco.
    #[must_use]
    pub fn is_empty_workspace(&self) -> bool {
        self.projects.is_empty() && self.today_items.is_empty() && self.focus_items.is_empty()
    }

    /// Registra o humor do dia atual mantendo os 7 dias da semana.
    pub fn set_today_mood(&mut self, level: MoodLevel) {
        if let Some(today) = self.mood_week.last_mut() {
            today.level = Some(level);
        }
    }

    /// Adiciona um item à lista "Em foco" respeitando o limite máximo de 5 itens.
    pub fn pin_focus_item(&mut self, item: FocusPinItem) -> bool {
        if self.focus_items.len() >= MAX_FOCUS_PINS {
            return false;
        }
        self.focus_items.push(item);
        true
    }
}

fn sample_heatmap() -> HeatmapData {
    let mut heatmap = HeatmapData::default();
    for (idx, cell) in heatmap.days.iter_mut().enumerate() {
        let pattern = (idx * 7 + 3) % 13;
        #[allow(clippy::cast_possible_truncation)]
        {
            *cell = if pattern < 4 { 0 } else { (pattern - 3) as u16 };
        }
    }
    heatmap
}

fn sample_today_items() -> Vec<TodayItem> {
    vec![
        TodayItem {
            time_label: "09:00".to_string(),
            title: "Cálculo II · Integrais impróprias".to_string(),
            context: "Estudos".to_string(),
            done: true,
        },
        TodayItem {
            time_label: "14:30".to_string(),
            title: "Revisar wireframes do módulo Projetos".to_string(),
            context: "eStudy".to_string(),
            done: false,
        },
        TodayItem {
            time_label: "19:00".to_string(),
            title: "Leitura · Cap. 4 Clean Architecture".to_string(),
            context: "Estante".to_string(),
            done: false,
        },
    ]
}

fn sample_projects() -> Vec<ProjectCardSummary> {
    vec![
        ProjectCardSummary {
            id: "proj-1".to_string(),
            title: "eStudy Desktop".to_string(),
            status_label: "Em andamento".to_string(),
            todo_count: 4,
            in_progress_count: 2,
            done_count: 8,
        },
        ProjectCardSummary {
            id: "proj-2".to_string(),
            title: "Semestre Engenharia".to_string(),
            status_label: "Em andamento".to_string(),
            todo_count: 6,
            in_progress_count: 1,
            done_count: 5,
        },
        ProjectCardSummary {
            id: "proj-3".to_string(),
            title: "Clube de Leitura".to_string(),
            status_label: "Planejando".to_string(),
            todo_count: 3,
            in_progress_count: 0,
            done_count: 2,
        },
        ProjectCardSummary {
            id: "proj-4".to_string(),
            title: "Notas de Física".to_string(),
            status_label: "Concluído".to_string(),
            todo_count: 0,
            in_progress_count: 0,
            done_count: 12,
        },
    ]
}

fn sample_focus_items() -> Vec<FocusPinItem> {
    vec![
        FocusPinItem {
            id: "pin-1".to_string(),
            title: "Fechar revisão espaçada de Cálculo".to_string(),
            context_label: "Estudos".to_string(),
            completed_pomos: 1,
            estimated_pomos: 2,
        },
        FocusPinItem {
            id: "pin-2".to_string(),
            title: "Implementar grade de projetos".to_string(),
            context_label: "eStudy".to_string(),
            completed_pomos: 2,
            estimated_pomos: 3,
        },
    ]
}

fn default_mood_week(today_mood: Option<MoodLevel>) -> Vec<MoodDayEntry> {
    vec![
        MoodDayEntry {
            day_label: "Seg".to_string(),
            level: Some(MoodLevel::Good),
        },
        MoodDayEntry {
            day_label: "Ter".to_string(),
            level: Some(MoodLevel::Great),
        },
        MoodDayEntry {
            day_label: "Qua".to_string(),
            level: Some(MoodLevel::Steady),
        },
        MoodDayEntry {
            day_label: "Qui".to_string(),
            level: Some(MoodLevel::Good),
        },
        MoodDayEntry {
            day_label: "Sex".to_string(),
            level: Some(MoodLevel::Tired),
        },
        MoodDayEntry {
            day_label: "Sáb".to_string(),
            level: Some(MoodLevel::Great),
        },
        MoodDayEntry {
            day_label: "Hoje".to_string(),
            level: today_mood,
        },
    ]
}
