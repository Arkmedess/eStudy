//! Estado e transições do Calendário Unificado e Drawer "Hoje".

use chrono::{DateTime, Datelike, Duration, Months, NaiveDate, NaiveTime, Utc};
use estudy_domain::{
    CalendarEvent, CalendarOccurrence, CalendarViewMode, EntityId, Project, RecurrenceRule, Task,
    expand_calendar_items,
};
use std::collections::BTreeSet;

/// Rascunho do formulário rápido "+ Criar evento".
#[derive(Debug, Clone)]
pub struct CreateEventDraft {
    /// Indica se o formulário rápido está aberto.
    pub open: bool,
    /// Título do novo evento.
    pub title: String,
    /// Hora de início (`HH:MM`).
    pub start_hhmm: String,
    /// Hora de fim (`HH:MM`).
    pub end_hhmm: String,
    /// Indica se o novo evento ocupa o dia inteiro.
    pub all_day: bool,
    /// Recorrência do novo evento.
    pub recurrence: Option<RecurrenceRule>,
}

impl Default for CreateEventDraft {
    fn default() -> Self {
        Self {
            open: false,
            title: String::new(),
            start_hhmm: "10:00".to_string(),
            end_hhmm: "11:00".to_string(),
            all_day: false,
            recurrence: None,
        }
    }
}

/// Estado completo da fatia Calendário Unificado + Drawer "Hoje".
#[derive(Debug, Clone)]
pub struct CalendarState {
    /// Eventos salvos.
    pub events: Vec<CalendarEvent>,
    /// Modo de visualização ativo (`Ano / Mês / Semana / Dia`).
    pub view_mode: CalendarViewMode,
    /// Data de referência para navegação no calendário.
    pub cursor_date: NaiveDate,
    /// Visibilidade da barra lateral esquerda contextual (`240px`, `Ctrl+B`).
    pub sidebar_visible: bool,
    /// Projetos marcados nos checkboxes de filtro (vazio = todos visíveis).
    pub selected_project_ids: BTreeSet<EntityId>,
    /// Filtro rápido por tag.
    pub active_tag_filter: Option<String>,
    /// Rascunho do formulário rápido "+ Criar evento".
    pub create_draft: CreateEventDraft,
    /// Controla abertura do Drawer deslizante "Hoje".
    pub today_drawer_open: bool,
    dirty_event_ids: BTreeSet<EntityId>,
}

/// Mensagens da fatia Calendário.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CalendarMessage {
    /// Recolhe ou expande a barra lateral contextual (`Ctrl+B`).
    ToggleSidebar,
    /// Alterna entre `Ano / Mês / Semana / Dia`.
    SetViewMode(CalendarViewMode),
    /// Avança ou retrocede o período atual (`-1` ou `+1`).
    ShiftPeriod(i32),
    /// Seleciona uma data específica (e salta para ela).
    SelectDate(NaiveDate),
    /// Marca ou desmarca checkbox de filtro por projeto.
    ToggleProjectFilter(EntityId),
    /// Filtra por tag.
    ToggleTagFilter(Option<String>),
    /// Abre ou fecha o formulário "+ Criar evento".
    ToggleCreateForm,
    /// Atualiza título do novo evento.
    NewTitleChanged(String),
    /// Atualiza início (`HH:MM`) do novo evento.
    NewStartChanged(String),
    /// Atualiza fim (`HH:MM`) do novo evento.
    NewEndChanged(String),
    /// Alterna dia inteiro no novo evento.
    ToggleNewAllDay,
    /// Define recorrência do novo evento.
    SetNewRecurrence(Option<RecurrenceRule>),
    /// Cria o evento na data selecionada.
    CreateEvent,
    /// Abre o Drawer "Hoje".
    OpenTodayDrawer,
    /// Fecha o Drawer "Hoje".
    CloseTodayDrawer,
}

impl CalendarState {
    /// Cria estado inicial com eventos realistas vinculados à data atual.
    #[must_use]
    pub fn sample(today: NaiveDate, now_utc: DateTime<Utc>, projects: &[Project]) -> Self {
        let first_proj = projects.first().map(|proj| proj.id.clone());
        let second_proj = projects.get(1).map(|proj| proj.id.clone());

        let events = vec![
            CalendarEvent {
                id: EntityId::new_v7(),
                title: "Bloco de revisão diária".to_string(),
                description: "Revisões espaçadas e flashcards".to_string(),
                project_id: second_proj.clone(),
                starts_at: combine_utc(today, 9, 0),
                ends_at: combine_utc(today, 10, 0),
                all_day: false,
                recurrence: Some(RecurrenceRule::Daily),
                color_token: "accent".to_string(),
                tags: vec!["estudos".to_string()],
                created_at: now_utc,
                updated_at: now_utc,
            },
            CalendarEvent {
                id: EntityId::new_v7(),
                title: "Desenvolvimento eStudy".to_string(),
                description: "Sessão focada na entrega atual".to_string(),
                project_id: first_proj,
                starts_at: combine_utc(today, 14, 0),
                ends_at: combine_utc(today, 16, 30),
                all_day: false,
                recurrence: Some(RecurrenceRule::Weekly),
                color_token: "accent".to_string(),
                tags: vec!["rust".to_string()],
                created_at: now_utc,
                updated_at: now_utc,
            },
            CalendarEvent {
                id: EntityId::new_v7(),
                title: "Sprint de Estudos".to_string(),
                description: "Evento multi-dia de preparação".to_string(),
                project_id: second_proj,
                starts_at: combine_utc(today, 0, 0),
                ends_at: combine_utc(today + Duration::days(2), 23, 59),
                all_day: true,
                recurrence: None,
                color_token: "warning".to_string(),
                tags: vec!["foco".to_string()],
                created_at: now_utc,
                updated_at: now_utc,
            },
        ];

        Self {
            events,
            view_mode: CalendarViewMode::Month,
            cursor_date: today,
            sidebar_visible: true,
            selected_project_ids: BTreeSet::new(),
            active_tag_filter: None,
            create_draft: CreateEventDraft::default(),
            today_drawer_open: false,
            dirty_event_ids: BTreeSet::new(),
        }
    }

    /// Extrai e limpa os IDs de eventos modificados para persistência seletiva.
    pub fn take_dirty_events(&mut self) -> Vec<EntityId> {
        std::mem::take(&mut self.dirty_event_ids)
            .into_iter()
            .collect()
    }

    /// Intervalo `(início, fim)` visível no modo de calendário atual.
    #[must_use]
    pub fn visible_range(&self) -> (NaiveDate, NaiveDate) {
        match self.view_mode {
            CalendarViewMode::Year => {
                let start = NaiveDate::from_ymd_opt(self.cursor_date.year(), 1, 1)
                    .unwrap_or(self.cursor_date);
                let end = NaiveDate::from_ymd_opt(self.cursor_date.year(), 12, 31)
                    .unwrap_or(self.cursor_date);
                (start, end)
            }
            CalendarViewMode::Month => {
                let start =
                    NaiveDate::from_ymd_opt(self.cursor_date.year(), self.cursor_date.month(), 1)
                        .unwrap_or(self.cursor_date);
                let end = start
                    .checked_add_months(Months::new(1))
                    .map_or(self.cursor_date, |next| next - Duration::days(1));
                (start, end)
            }
            CalendarViewMode::Week => {
                let offset = i64::from(self.cursor_date.weekday().num_days_from_monday());
                let start = self.cursor_date - Duration::days(offset);
                (start, start + Duration::days(6))
            }
            CalendarViewMode::Day => (self.cursor_date, self.cursor_date),
        }
    }

    /// Expande eventos e tarefas com prazo aplicando os filtros de projeto e tag ativos.
    #[must_use]
    pub fn visible_occurrences(&self, tasks: &[Task]) -> Vec<CalendarOccurrence> {
        let (start, end) = self.visible_range();
        let all = expand_calendar_items(&self.events, tasks, start, end);
        all.into_iter()
            .filter(|occ| self.matches_filters(occ))
            .collect()
    }

    /// Lista todas as tags únicas dos eventos carregados.
    #[must_use]
    pub fn available_tags(&self) -> Vec<String> {
        let mut tags = BTreeSet::new();
        for ev in &self.events {
            for tag in &ev.tags {
                tags.insert(tag.clone());
            }
        }
        tags.into_iter().collect()
    }

    /// Processa mensagens da fatia Calendário.
    pub fn update(&mut self, message: CalendarMessage, now_utc: DateTime<Utc>) {
        match message {
            CalendarMessage::ToggleSidebar => self.sidebar_visible = !self.sidebar_visible,
            CalendarMessage::SetViewMode(mode) => self.view_mode = mode,
            CalendarMessage::ShiftPeriod(delta) => self.shift_period(delta),
            CalendarMessage::SelectDate(date) => self.cursor_date = date,
            CalendarMessage::ToggleProjectFilter(project_id) => {
                if !self.selected_project_ids.remove(&project_id) {
                    self.selected_project_ids.insert(project_id);
                }
            }
            CalendarMessage::ToggleTagFilter(tag) => self.active_tag_filter = tag,
            CalendarMessage::ToggleCreateForm => {
                self.create_draft.open = !self.create_draft.open;
            }
            CalendarMessage::NewTitleChanged(title) => self.create_draft.title = title,
            CalendarMessage::NewStartChanged(val) => self.create_draft.start_hhmm = val,
            CalendarMessage::NewEndChanged(val) => self.create_draft.end_hhmm = val,
            CalendarMessage::ToggleNewAllDay => {
                self.create_draft.all_day = !self.create_draft.all_day;
            }
            CalendarMessage::SetNewRecurrence(rule) => self.create_draft.recurrence = rule,
            CalendarMessage::CreateEvent => self.create_event_from_form(now_utc),
            CalendarMessage::OpenTodayDrawer => self.today_drawer_open = true,
            CalendarMessage::CloseTodayDrawer => self.today_drawer_open = false,
        }
    }

    /// Cria rapidamente um evento no dia selecionado (usado também pelo `Ctrl+K > novo evento`).
    pub fn quick_add_event(&mut self, title: &str, now_utc: DateTime<Utc>) {
        let trimmed = title.trim().to_string();
        if trimmed.is_empty() {
            return;
        }
        let id = EntityId::new_v7();
        self.events.push(CalendarEvent {
            id: id.clone(),
            title: trimmed,
            description: String::new(),
            project_id: self.selected_project_ids.iter().next().cloned(),
            starts_at: combine_utc(self.cursor_date, 10, 0),
            ends_at: combine_utc(self.cursor_date, 11, 0),
            all_day: false,
            recurrence: None,
            color_token: "accent".to_string(),
            tags: Vec::new(),
            created_at: now_utc,
            updated_at: now_utc,
        });
        self.dirty_event_ids.insert(id);
    }

    fn matches_filters(&self, occ: &CalendarOccurrence) -> bool {
        let project_ok = if self.selected_project_ids.is_empty() {
            true
        } else {
            occ.project_id
                .as_ref()
                .is_some_and(|id| self.selected_project_ids.contains(id))
        };

        let tag_ok = match &self.active_tag_filter {
            Some(tag) => occ
                .tags
                .iter()
                .any(|candidate| candidate.eq_ignore_ascii_case(tag)),
            None => true,
        };

        project_ok && tag_ok
    }

    fn shift_period(&mut self, delta: i32) {
        self.cursor_date = match self.view_mode {
            CalendarViewMode::Year => {
                let target_year = self.cursor_date.year() + delta;
                NaiveDate::from_ymd_opt(target_year, self.cursor_date.month(), 1)
                    .unwrap_or(self.cursor_date)
            }
            CalendarViewMode::Month => {
                if delta >= 0 {
                    self.cursor_date
                        .checked_add_months(Months::new(delta.unsigned_abs()))
                        .unwrap_or(self.cursor_date)
                } else {
                    self.cursor_date
                        .checked_sub_months(Months::new(delta.unsigned_abs()))
                        .unwrap_or(self.cursor_date)
                }
            }
            CalendarViewMode::Week => self.cursor_date + Duration::weeks(i64::from(delta)),
            CalendarViewMode::Day => self.cursor_date + Duration::days(i64::from(delta)),
        };
    }

    fn create_event_from_form(&mut self, now_utc: DateTime<Utc>) {
        let title = self.create_draft.title.trim().to_string();
        if title.is_empty() {
            return;
        }
        let (sh, sm) = parse_hhmm(&self.create_draft.start_hhmm).unwrap_or((10, 0));
        let (eh, em) = parse_hhmm(&self.create_draft.end_hhmm).unwrap_or((11, 0));
        let starts_at = combine_utc(self.cursor_date, sh, sm);
        let mut ends_at = combine_utc(self.cursor_date, eh, em);
        if ends_at < starts_at {
            ends_at = starts_at + Duration::hours(1);
        }

        let id = EntityId::new_v7();
        self.events.push(CalendarEvent {
            id: id.clone(),
            title,
            description: String::new(),
            project_id: self.selected_project_ids.iter().next().cloned(),
            starts_at,
            ends_at,
            all_day: self.create_draft.all_day,
            recurrence: self.create_draft.recurrence,
            color_token: "accent".to_string(),
            tags: Vec::new(),
            created_at: now_utc,
            updated_at: now_utc,
        });
        self.dirty_event_ids.insert(id);

        self.create_draft.title.clear();
        self.create_draft.open = false;
    }
}

fn combine_utc(date: NaiveDate, hour: u32, minute: u32) -> DateTime<Utc> {
    let time = NaiveTime::from_hms_opt(hour, minute, 0).unwrap_or(NaiveTime::MIN);
    DateTime::from_naive_utc_and_offset(date.and_time(time), Utc)
}

fn parse_hhmm(raw: &str) -> Option<(u32, u32)> {
    let mut parts = raw.trim().split(':');
    let h: u32 = parts.next()?.parse().ok()?;
    let m: u32 = parts.next()?.parse().ok()?;
    (h < 24 && m < 60).then_some((h, m))
}
