//! Estado e transições do módulo de Hábitos.

use chrono::{Datelike, Duration, NaiveDate};
use estudy_domain::{EntityId, Habit, overall_monthly_adherence_pct};
use std::collections::BTreeSet;

/// Mensagens de interação da fatia Hábitos.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HabitsMessage {
    /// Alterna a conclusão de um hábito em uma data específica.
    ToggleHabitDate {
        /// Identificador do hábito.
        habit_id: EntityId,
        /// Data alvo (`YYYY-MM-DD`).
        date: NaiveDate,
    },
    /// Avança ou recua a semana exibida na grade (`-1` ou `+1`).
    ShiftWeek(i32),
    /// Avança ou recua o mês exibido no calendário de constância (`-1` ou `+1`).
    ShiftMonth(i32),
    /// Atualiza o título digitado para o novo hábito.
    NewTitleChanged(String),
    /// Atualiza a meta/descrição digitada para o novo hábito.
    NewSubtitleChanged(String),
    /// Cria um novo hábito na lista.
    CreateHabit,
    /// Remove um hábito da lista.
    RemoveHabit(EntityId),
}

/// Estado completo da fatia Hábitos.
#[derive(Debug, Clone)]
pub struct HabitsState {
    /// Lista de hábitos acompanhados.
    pub habits: Vec<Habit>,
    /// Segunda-feira da semana atualmente exibida na grade semanal.
    pub week_start: NaiveDate,
    /// Ano exibido no painel mensal.
    pub view_year: i32,
    /// Mês exibido no painel mensal (`1..=12`).
    pub view_month: u32,
    /// Texto temporário para título de novo hábito.
    pub new_title: String,
    /// Texto temporário para descrição/meta de novo hábito.
    pub new_subtitle: String,
}

impl HabitsState {
    /// Inicializa uma amostra acolhedora de hábitos alinhada à data atual.
    #[must_use]
    pub fn sample(today: NaiveDate) -> Self {
        let week_start = monday_of_week(today);
        let mut reading_dates = BTreeSet::new();
        let mut workout_dates = BTreeSet::new();
        let mut english_dates = BTreeSet::new();
        let mut focus_dates = BTreeSet::new();

        for back in 0..14_i64 {
            let d = today - Duration::days(back);
            if back % 2 == 0 || back < 4 {
                reading_dates.insert(d);
            }
            if back % 3 != 1 {
                workout_dates.insert(d);
            }
            if back != 2 && back != 6 {
                english_dates.insert(d);
            }
            if back < 6 || back % 2 == 1 {
                focus_dates.insert(d);
            }
        }

        Self {
            habits: vec![
                Habit {
                    id: EntityId::new_v7(),
                    title: "Leitura".to_string(),
                    subtitle: "20 páginas antes de dormir".to_string(),
                    color_index: 3,
                    weekly_goal: 6,
                    completed_dates: reading_dates,
                },
                Habit {
                    id: EntityId::new_v7(),
                    title: "Academia".to_string(),
                    subtitle: "Treino de força · 50 min".to_string(),
                    color_index: 0,
                    weekly_goal: 5,
                    completed_dates: workout_dates,
                },
                Habit {
                    id: EntityId::new_v7(),
                    title: "Estudar Inglês".to_string(),
                    subtitle: "Active recall e listening · 25 min".to_string(),
                    color_index: 2,
                    weekly_goal: 6,
                    completed_dates: english_dates,
                },
                Habit {
                    id: EntityId::new_v7(),
                    title: "Revisão Espaçada".to_string(),
                    subtitle: "Flashcards do dia".to_string(),
                    color_index: 4,
                    weekly_goal: 7,
                    completed_dates: focus_dates,
                },
            ],
            week_start,
            view_year: today.year(),
            view_month: today.month(),
            new_title: String::new(),
            new_subtitle: String::new(),
        }
    }

    /// Quantos hábitos foram concluídos hoje e o total acompanhado.
    #[must_use]
    pub fn today_progress(&self, today: NaiveDate) -> (usize, usize) {
        let done = self.habits.iter().filter(|h| h.is_done_on(today)).count();
        (done, self.habits.len())
    }

    /// Percentual médio do mês exibido (`0..=100`).
    #[must_use]
    pub fn monthly_overall_pct(&self, today: NaiveDate) -> u8 {
        overall_monthly_adherence_pct(&self.habits, self.view_year, self.view_month, today)
    }

    /// Processa uma mensagem MVU atualizando o estado de hábitos.
    pub fn update(&mut self, message: HabitsMessage) {
        match message {
            HabitsMessage::ToggleHabitDate { habit_id, date } => {
                if let Some(habit) = self.habits.iter_mut().find(|h| h.id == habit_id) {
                    habit.toggle_on(date);
                }
            }
            HabitsMessage::ShiftWeek(delta_weeks) => {
                self.week_start += Duration::days(i64::from(delta_weeks) * 7);
            }
            HabitsMessage::ShiftMonth(delta_months) => {
                let total =
                    self.view_year * 12 + (self.view_month.cast_signed() - 1) + delta_months;
                self.view_year = total.div_euclid(12);
                self.view_month = u32::try_from(total.rem_euclid(12) + 1).unwrap_or(1);
            }
            HabitsMessage::NewTitleChanged(val) => self.new_title = val,
            HabitsMessage::NewSubtitleChanged(val) => self.new_subtitle = val,
            HabitsMessage::CreateHabit => {
                let title = self.new_title.trim().to_string();
                if title.is_empty() {
                    return;
                }
                let subtitle = if self.new_subtitle.trim().is_empty() {
                    "Rotina diária".to_string()
                } else {
                    self.new_subtitle.trim().to_string()
                };
                let color_index = self.habits.len();
                self.habits.push(Habit {
                    id: EntityId::new_v7(),
                    title,
                    subtitle,
                    color_index,
                    weekly_goal: 5,
                    completed_dates: BTreeSet::new(),
                });
                self.new_title.clear();
                self.new_subtitle.clear();
            }
            HabitsMessage::RemoveHabit(id) => {
                self.habits.retain(|h| h.id != id);
            }
        }
    }
}

fn monday_of_week(date: NaiveDate) -> NaiveDate {
    let offset = i64::from(date.weekday().num_days_from_monday());
    date - Duration::days(offset)
}
