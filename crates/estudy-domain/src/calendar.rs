//! Entidades e regras puras do Calendário Unificado e Timeline "Hoje".

use chrono::{DateTime, Duration, Months, NaiveDate, NaiveDateTime, Timelike, Utc};

use crate::error::DomainError;
use crate::ids::EntityId;
use crate::projects::{Task, TaskStatus};

const DAY_START_MINUTE: u16 = 7 * 60;
const DAY_END_MINUTE: u16 = 22 * 60;
const MIN_FREE_WINDOW_MINUTES: u16 = 15;

/// Modos de visualização do Calendário Unificado.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CalendarViewMode {
    /// Visão anual (12 meses resumidos).
    Year,
    /// Grade mensal.
    Month,
    /// Semana (7 dias).
    Week,
    /// Dia detalhado com blocos horários.
    Day,
}

impl CalendarViewMode {
    /// Todos os modos em ordem.
    pub const ALL: [Self; 4] = [Self::Year, Self::Month, Self::Week, Self::Day];

    /// Rótulo em português.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Year => "Ano",
            Self::Month => "Mês",
            Self::Week => "Semana",
            Self::Day => "Dia",
        }
    }
}

/// Regra de recorrência para eventos do calendário.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum RecurrenceRule {
    /// Repete todos os dias.
    Daily,
    /// Repete semanalmente no mesmo dia da semana.
    Weekly,
    /// Repete mensalmente no mesmo dia do mês.
    Monthly,
}

impl RecurrenceRule {
    /// Representação textual persistida no banco.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Daily => "daily",
            Self::Weekly => "weekly",
            Self::Monthly => "monthly",
        }
    }

    /// Converte texto para [`RecurrenceRule`].
    pub fn parse(raw: &str) -> Result<Self, DomainError> {
        match raw.trim() {
            "daily" => Ok(Self::Daily),
            "weekly" => Ok(Self::Weekly),
            "monthly" => Ok(Self::Monthly),
            other => Err(DomainError::InvalidProjectValue {
                field: "recorrência",
                value: other.to_string(),
            }),
        }
    }
}

/// Evento de calendário (recorrente ou multi-dia).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CalendarEvent {
    /// Identificador `UUIDv7`.
    pub id: EntityId,
    /// Título do evento.
    pub title: String,
    /// Descrição opcional.
    pub description: String,
    /// Projeto vinculado opcional.
    pub project_id: Option<EntityId>,
    /// Início em UTC.
    pub starts_at: DateTime<Utc>,
    /// Fim em UTC.
    pub ends_at: DateTime<Utc>,
    /// Evento de dia inteiro.
    pub all_day: bool,
    /// Regra opcional de recorrência.
    pub recurrence: Option<RecurrenceRule>,
    /// Token de cor.
    pub color_token: String,
    /// Tags associadas.
    pub tags: Vec<String>,
    /// Criação em UTC.
    pub created_at: DateTime<Utc>,
    /// Atualização em UTC.
    pub updated_at: DateTime<Utc>,
}

impl CalendarEvent {
    /// Valida a coerência temporal entre início e fim do evento.
    pub fn validate(&self) -> Result<(), DomainError> {
        if self.ends_at < self.starts_at {
            return Err(DomainError::InvalidEventTimeRange {
                starts_at: self.starts_at.to_rfc3339(),
                ends_at: self.ends_at.to_rfc3339(),
            });
        }
        Ok(())
    }
}

/// Ocorrência expandida para exibição em um dia específico do calendário.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CalendarOccurrence {
    /// Dia da ocorrência.
    pub date: NaiveDate,
    /// Identificador de origem (evento ou tarefa).
    pub source_id: EntityId,
    /// Título exibido.
    pub title: String,
    /// Indica se veio de uma tarefa com prazo (lembrete tracejado com bandeira).
    pub is_task_deadline: bool,
    /// Indica se a tarefa de origem já foi concluída.
    pub is_completed: bool,
    /// Indica se ocupa o dia inteiro / sem horário específico.
    pub all_day: bool,
    /// Minuto de início no dia (`0..=1439`).
    pub start_minute: u16,
    /// Minuto de fim no dia (`1..=1440`).
    pub end_minute: u16,
    /// Projeto associado, se houver.
    pub project_id: Option<EntityId>,
    /// Tags do item.
    pub tags: Vec<String>,
}

/// Elemento posicionado na Timeline Vertical "Hoje".
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TimelineSlot {
    /// Bloco ocupado por evento ou tarefa agendada.
    Busy {
        /// Título do bloco.
        title: String,
        /// Horário formatado (`HH:MM – HH:MM`).
        time_range: String,
        /// Minuto de início (`0..=1439`).
        start_minute: u16,
        /// Minuto de fim (`1..=1440`).
        end_minute: u16,
        /// Indica se é prazo de tarefa.
        is_task: bool,
        /// Indica se foi concluído.
        is_completed: bool,
    },
    /// Janela livre calculada automaticamente entre blocos ocupados.
    Free {
        /// Minuto de início.
        start_minute: u16,
        /// Minuto de fim.
        end_minute: u16,
        /// Duração em minutos.
        duration_minutes: u16,
        /// Texto formatado (ex.: `Livre · 1h45`).
        label: String,
    },
}

/// Estrutura completa da Timeline Vertical "Hoje".
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TodayTimeline {
    /// Itens sem horário fixo (eventos dia inteiro ou prazos sem hora) exibidos no topo.
    pub unscheduled: Vec<CalendarOccurrence>,
    /// Blocos horários ocupados e janelas livres em ordem cronológica.
    pub slots: Vec<TimelineSlot>,
    /// Minuto atual do dia (`0..=1439`) para posicionar a linha "Agora".
    pub now_minute: u16,
}

/// Expande eventos (incluindo recorrentes e multi-dia) e tarefas com prazo em um intervalo de datas.
#[must_use]
pub fn expand_calendar_items(
    events: &[CalendarEvent],
    tasks: &[Task],
    range_start: NaiveDate,
    range_end: NaiveDate,
) -> Vec<CalendarOccurrence> {
    if range_end < range_start {
        return Vec::new();
    }

    let mut occurrences = Vec::new();
    for event in events {
        append_event_occurrences(event, range_start, range_end, &mut occurrences);
    }
    for task in tasks {
        append_task_deadline(task, range_start, range_end, &mut occurrences);
    }

    occurrences.sort_by_key(|occ| (occ.date, !occ.all_day, occ.start_minute, occ.title.clone()));
    occurrences
}

/// Constrói a Timeline Vertical "Hoje" com itens sem horário, blocos ocupados, janelas livres e linha "Agora".
#[must_use]
pub fn build_today_timeline(
    events: &[CalendarEvent],
    tasks: &[Task],
    now_utc: DateTime<Utc>,
) -> TodayTimeline {
    let today = now_utc.date_naive();
    let now_minute = u16::try_from(now_utc.hour() * 60 + now_utc.minute()).unwrap_or(0);
    let occurrences = expand_calendar_items(events, tasks, today, today);

    let mut unscheduled = Vec::new();
    let mut timed = Vec::new();
    for occ in occurrences {
        if occ.all_day {
            unscheduled.push(occ);
        } else {
            timed.push(occ);
        }
    }

    timed.sort_by_key(|occ| (occ.start_minute, occ.end_minute));
    let slots = assemble_timeline_slots(&timed);

    TodayTimeline {
        unscheduled,
        slots,
        now_minute,
    }
}

/// Formata uma duração em minutos como `Livre · 1h45` ou `Livre · 30m`.
#[must_use]
pub fn format_free_window_label(minutes: u16) -> String {
    let hours = minutes / 60;
    let rem = minutes % 60;
    if hours == 0 {
        format!("Livre · {rem}m")
    } else if rem == 0 {
        format!("Livre · {hours}h")
    } else {
        format!("Livre · {hours}h{rem:02}")
    }
}

fn append_event_occurrences(
    event: &CalendarEvent,
    range_start: NaiveDate,
    range_end: NaiveDate,
    out: &mut Vec<CalendarOccurrence>,
) {
    let base_start = event.starts_at.date_naive();
    let base_end = event.ends_at.date_naive();
    let span_days = (base_end - base_start).num_days().max(0);
    let start_min = minute_of_day(event.starts_at);
    let end_min = minute_of_day(event.ends_at).max(start_min + 1);

    let Some(rule) = event.recurrence else {
        let clamped_start = base_start.max(range_start);
        let clamped_end = base_end.min(range_end);
        let mut cursor = clamped_start;
        while cursor <= clamped_end {
            out.push(build_event_occurrence(
                event,
                cursor,
                start_min,
                end_min,
                event.all_day || base_end > base_start,
            ));
            cursor += Duration::days(1);
        }
        return;
    };

    let mut step = 0_u32;
    while let Some(occ_start) = advance_by_rule(base_start, rule, step) {
        if occ_start > range_end {
            break;
        }
        let occ_end = occ_start + Duration::days(span_days);
        if occ_end >= range_start {
            let mut cursor = occ_start.max(range_start);
            let last = occ_end.min(range_end);
            while cursor <= last {
                out.push(build_event_occurrence(
                    event,
                    cursor,
                    start_min,
                    end_min,
                    event.all_day || span_days > 0,
                ));
                cursor += Duration::days(1);
            }
        }
        step += 1;
    }
}

fn build_event_occurrence(
    event: &CalendarEvent,
    date: NaiveDate,
    start_minute: u16,
    end_minute: u16,
    all_day: bool,
) -> CalendarOccurrence {
    CalendarOccurrence {
        date,
        source_id: event.id.clone(),
        title: event.title.clone(),
        is_task_deadline: false,
        is_completed: false,
        all_day,
        start_minute: if all_day { 0 } else { start_minute },
        end_minute: if all_day { 24 * 60 } else { end_minute },
        project_id: event.project_id.clone(),
        tags: event.tags.clone(),
    }
}

fn append_task_deadline(
    task: &Task,
    range_start: NaiveDate,
    range_end: NaiveDate,
    out: &mut Vec<CalendarOccurrence>,
) {
    let Some(due_raw) = task.due_at.as_deref() else {
        return;
    };
    let Some((date, opt_minute)) = parse_flexible_due(due_raw) else {
        return;
    };
    if date < range_start || date > range_end {
        return;
    }

    let (all_day, start_minute, end_minute) = match opt_minute {
        Some(min) => (false, min, (min + 30).min(24 * 60)),
        None => (true, 0, 24 * 60),
    };

    out.push(CalendarOccurrence {
        date,
        source_id: task.id.clone(),
        title: task.title.clone(),
        is_task_deadline: true,
        is_completed: task.status == TaskStatus::Done,
        all_day,
        start_minute,
        end_minute,
        project_id: task
            .project_links
            .first()
            .map(|link| link.project_id.clone()),
        tags: task.tags.clone(),
    });
}

fn assemble_timeline_slots(timed: &[CalendarOccurrence]) -> Vec<TimelineSlot> {
    let mut slots = Vec::new();
    let mut cursor = DAY_START_MINUTE;

    for item in timed {
        if item.start_minute > cursor {
            let gap = item.start_minute - cursor;
            if gap >= MIN_FREE_WINDOW_MINUTES {
                slots.push(TimelineSlot::Free {
                    start_minute: cursor,
                    end_minute: item.start_minute,
                    duration_minutes: gap,
                    label: format_free_window_label(gap),
                });
            }
        }

        slots.push(TimelineSlot::Busy {
            title: item.title.clone(),
            time_range: format!(
                "{} – {}",
                format_hhmm(item.start_minute),
                format_hhmm(item.end_minute)
            ),
            start_minute: item.start_minute,
            end_minute: item.end_minute,
            is_task: item.is_task_deadline,
            is_completed: item.is_completed,
        });
        cursor = cursor.max(item.end_minute);
    }

    if DAY_END_MINUTE > cursor {
        let trailing = DAY_END_MINUTE - cursor;
        if trailing >= MIN_FREE_WINDOW_MINUTES {
            slots.push(TimelineSlot::Free {
                start_minute: cursor,
                end_minute: DAY_END_MINUTE,
                duration_minutes: trailing,
                label: format_free_window_label(trailing),
            });
        }
    }

    slots
}

fn advance_by_rule(base: NaiveDate, rule: RecurrenceRule, step: u32) -> Option<NaiveDate> {
    match rule {
        RecurrenceRule::Daily => Some(base + Duration::days(i64::from(step))),
        RecurrenceRule::Weekly => Some(base + Duration::weeks(i64::from(step))),
        RecurrenceRule::Monthly => base.checked_add_months(Months::new(step)),
    }
}

fn minute_of_day(dt: DateTime<Utc>) -> u16 {
    u16::try_from(dt.hour() * 60 + dt.minute()).unwrap_or(0)
}

fn format_hhmm(minute: u16) -> String {
    format!("{:02}:{:02}", minute / 60, minute % 60)
}

fn parse_flexible_due(raw: &str) -> Option<(NaiveDate, Option<u16>)> {
    let trimmed = raw.trim();
    if let Ok(dt) = DateTime::parse_from_rfc3339(trimmed) {
        let min = u16::try_from(dt.hour() * 60 + dt.minute()).unwrap_or(0);
        return Some((dt.date_naive(), Some(min)));
    }
    if let Ok(ndt) = NaiveDateTime::parse_from_str(trimmed, "%Y-%m-%d %H:%M") {
        let min = u16::try_from(ndt.hour() * 60 + ndt.minute()).unwrap_or(0);
        return Some((ndt.date(), Some(min)));
    }
    if let Ok(date) = NaiveDate::parse_from_str(trimmed, "%Y-%m-%d") {
        return Some((date, None));
    }
    None
}
