//! Repositório transacional `SQLite` para Hábitos e registros diários (`habits` / `habit_logs`).

use chrono::{DateTime, NaiveDate, SecondsFormat, Utc};
use estudy_domain::{EntityId, Habit};
use rusqlite::{Connection, params};
use std::collections::{BTreeSet, HashMap};

use crate::error::StorageError;

/// Operações de persistência de Hábitos em `SQLite` WAL sem consultas N+1.
pub struct SqliteHabitsRepository;

impl SqliteHabitsRepository {
    /// Carrega todos os hábitos ativos e seus dias concluídos em 2 consultas (sem N+1).
    pub fn list_habits(conn: &Connection) -> Result<Vec<Habit>, StorageError> {
        let mut logs_by_habit: HashMap<String, BTreeSet<NaiveDate>> = HashMap::new();
        {
            let mut log_stmt =
                conn.prepare("SELECT habit_id, log_date FROM habit_logs ORDER BY log_date ASC")?;
            let rows = log_stmt.query_map([], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
            })?;
            for row in rows {
                let (habit_id, raw_date) = row?;
                if let Ok(parsed) = NaiveDate::parse_from_str(&raw_date, "%Y-%m-%d") {
                    logs_by_habit.entry(habit_id).or_default().insert(parsed);
                }
            }
        }

        let mut stmt = conn.prepare(
            "SELECT id, title, subtitle, color_index, weekly_goal
             FROM habits
             WHERE deleted_at IS NULL
             ORDER BY sort_order ASC, created_at ASC",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, i64>(3)?,
                row.get::<_, i64>(4)?,
            ))
        })?;

        let mut habits = Vec::new();
        for row in rows {
            let (raw_id, title, subtitle, color_idx, weekly_goal) = row?;
            let id = EntityId::parse(&raw_id)?;
            let completed_dates = logs_by_habit.remove(&raw_id).unwrap_or_default();
            habits.push(Habit {
                id,
                title,
                subtitle,
                color_index: usize::try_from(color_idx).unwrap_or(0),
                weekly_goal: u8::try_from(weekly_goal).unwrap_or(7).clamp(1, 7),
                completed_dates,
            });
        }

        Ok(habits)
    }

    /// Salva (upsert) um hábito e sincroniza seu conjunto de datas concluídas na mesma transação.
    pub fn save_habit(
        conn: &mut Connection,
        habit: &Habit,
        now: DateTime<Utc>,
    ) -> Result<(), StorageError> {
        let now_iso = now.to_rfc3339_opts(SecondsFormat::Secs, true);
        let id_str = habit.id.to_string();
        let color_i64 = i64::try_from(habit.color_index).unwrap_or(0);
        let goal_i64 = i64::from(habit.weekly_goal.clamp(1, 7));

        let tx = conn.transaction()?;
        tx.execute(
            "INSERT INTO habits (id, title, subtitle, color_index, weekly_goal, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?6)
             ON CONFLICT(id) DO UPDATE SET
                title = excluded.title,
                subtitle = excluded.subtitle,
                color_index = excluded.color_index,
                weekly_goal = excluded.weekly_goal,
                updated_at = excluded.updated_at,
                deleted_at = NULL",
            params![id_str, habit.title, habit.subtitle, color_i64, goal_i64, now_iso],
        )?;

        tx.execute(
            "DELETE FROM habit_logs WHERE habit_id = ?1",
            params![id_str],
        )?;
        for date in &habit.completed_dates {
            let date_str = date.format("%Y-%m-%d").to_string();
            tx.execute(
                "INSERT INTO habit_logs (habit_id, log_date, created_at) VALUES (?1, ?2, ?3)",
                params![id_str, date_str, now_iso],
            )?;
        }

        tx.commit()?;
        Ok(())
    }

    /// Marca um hábito como excluído (soft delete).
    pub fn delete_habit(
        conn: &Connection,
        habit_id: &EntityId,
        now: DateTime<Utc>,
    ) -> Result<(), StorageError> {
        let now_iso = now.to_rfc3339_opts(SecondsFormat::Secs, true);
        conn.execute(
            "UPDATE habits SET deleted_at = ?1, updated_at = ?1 WHERE id = ?2",
            params![now_iso, habit_id.to_string()],
        )?;
        Ok(())
    }
}
