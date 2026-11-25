//! Regras puras do módulo de Hábitos (constância semanal, sequência e aderência mensal).

use chrono::{Datelike, Duration, NaiveDate};
use std::collections::BTreeSet;

use crate::ids::EntityId;

/// Hábito recorrente acompanhado diariamente pelo usuário.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Habit {
    /// Identificador cronológico (`UUIDv7`).
    pub id: EntityId,
    /// Nome curto do hábito (ex.: `Leitura`, `Academia`, `Estudar Inglês`).
    pub title: String,
    /// Descrição ou meta diária (ex.: `20 minutos por dia`).
    pub subtitle: String,
    /// Índice estável da cor de categoria (`0..7`).
    pub color_index: usize,
    /// Meta de dias ativos por semana (`1..=7`).
    pub weekly_goal: u8,
    /// Datas concluídas (`YYYY-MM-DD`).
    pub completed_dates: BTreeSet<NaiveDate>,
}

impl Habit {
    /// Verifica se o hábito foi cumprido na data informada.
    #[must_use]
    pub fn is_done_on(&self, date: NaiveDate) -> bool {
        self.completed_dates.contains(&date)
    }

    /// Alterna a conclusão do hábito na data informada e retorna o novo estado.
    pub fn toggle_on(&mut self, date: NaiveDate) -> bool {
        if self.completed_dates.remove(&date) {
            false
        } else {
            self.completed_dates.insert(date);
            true
        }
    }

    /// Calcula a sequência atual de dias consecutivos até `today` (ou ontem se hoje ainda não foi marcado).
    #[must_use]
    pub fn current_streak(&self, today: NaiveDate) -> u32 {
        let mut cursor = if self.is_done_on(today) {
            today
        } else {
            today - Duration::days(1)
        };
        let mut streak = 0_u32;
        while self.is_done_on(cursor) {
            streak += 1;
            cursor -= Duration::days(1);
        }
        streak
    }

    /// Quantidade de dias concluídos no mês/ano informado.
    #[must_use]
    pub fn completions_in_month(&self, year: i32, month: u32) -> u32 {
        let count = self
            .completed_dates
            .iter()
            .filter(|d| d.year() == year && d.month() == month)
            .count();
        u32::try_from(count).unwrap_or(u32::MAX)
    }

    /// Porcentagem de aderência (`0..=100`) considerando os dias decorridos no mês até `today`.
    #[must_use]
    pub fn monthly_adherence_pct(&self, year: i32, month: u32, today: NaiveDate) -> u8 {
        let elapsed_days = elapsed_days_in_month(year, month, today).max(1);
        let done = self.completions_in_month(year, month).min(elapsed_days);
        let pct = (done * 100) / elapsed_days;
        u8::try_from(pct.min(100)).unwrap_or(100)
    }
}

/// Calcula o percentual médio mensal (`0..=100`) de um conjunto de hábitos.
#[must_use]
pub fn overall_monthly_adherence_pct(
    habits: &[Habit],
    year: i32,
    month: u32,
    today: NaiveDate,
) -> u8 {
    if habits.is_empty() {
        return 0;
    }
    let total: u32 = habits
        .iter()
        .map(|h| u32::from(h.monthly_adherence_pct(year, month, today)))
        .sum();
    let count = u32::try_from(habits.len()).unwrap_or(1);
    u8::try_from((total / count).min(100)).unwrap_or(100)
}

/// Retorna o total de dias no mês (28..=31).
#[must_use]
pub fn days_in_month(year: i32, month: u32) -> u32 {
    let (next_year, next_month) = if month == 12 {
        (year + 1, 1)
    } else {
        (year, month + 1)
    };
    let Some(first_next) = NaiveDate::from_ymd_opt(next_year, next_month, 1) else {
        return 30;
    };
    (first_next - Duration::days(1)).day()
}

fn elapsed_days_in_month(year: i32, month: u32, today: NaiveDate) -> u32 {
    if year == today.year() && month == today.month() {
        today.day()
    } else {
        days_in_month(year, month)
    }
}
