//! Regras puras de Pomodoro, Modo Foco e seleção de faixas de áudio DSP.

use chrono::{DateTime, Utc};

use crate::error::DomainError;
use crate::ids::EntityId;

/// Duração mínima real em minutos para que uma sessão de foco gere moedas anti-inflação.
pub const MINIMUM_REWARDABLE_FOCUS_MINUTES: u32 = 15;

/// Presets de ciclo Pomodoro suportados no Modo Foco.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PomodoroPreset {
    /// Ciclo clássico (`25 min` de foco e `5 min` de pausa).
    Classic25_5,
    /// Bloco profundo (`50 min` de foco e `10 min` de pausa).
    Deep50_10,
    /// Sprint curto (`15 min` de foco e `3 min` de pausa).
    Sprint15_3,
    /// Configuração personalizada em minutos.
    Custom {
        /// Minutos de foco (`5..=120`).
        focus_minutes: u32,
        /// Minutos de pausa (`1..=30`).
        break_minutes: u32,
    },
}

impl PomodoroPreset {
    /// Presets predefinidos exibidos na interface.
    pub const STANDARD: [Self; 3] = [Self::Classic25_5, Self::Deep50_10, Self::Sprint15_3];

    /// Retorna a duração de foco em minutos.
    #[must_use]
    pub const fn focus_minutes(self) -> u32 {
        match self {
            Self::Classic25_5 => 25,
            Self::Deep50_10 => 50,
            Self::Sprint15_3 => 15,
            Self::Custom { focus_minutes, .. } => {
                if focus_minutes < 5 {
                    5
                } else if focus_minutes > 120 {
                    120
                } else {
                    focus_minutes
                }
            }
        }
    }

    /// Retorna a duração de pausa curta em minutos.
    #[must_use]
    pub const fn break_minutes(self) -> u32 {
        match self {
            Self::Classic25_5 => 5,
            Self::Deep50_10 => 10,
            Self::Sprint15_3 => 3,
            Self::Custom { break_minutes, .. } => {
                if break_minutes < 1 {
                    1
                } else if break_minutes > 30 {
                    30
                } else {
                    break_minutes
                }
            }
        }
    }

    /// Rótulo curto exibido nos seletores da interface.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Classic25_5 => "25 / 5",
            Self::Deep50_10 => "50 / 10",
            Self::Sprint15_3 => "15 / 3",
            Self::Custom { .. } => "Livre",
        }
    }
}

/// Estado atual da máquina de estados do temporizador Pomodoro.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FocusPhase {
    /// Pronto para iniciar.
    Idle,
    /// Cronômetro de foco em andamento.
    Focusing,
    /// Foco pausado manualmente.
    Paused,
    /// Pausa curta entre blocos.
    ShortBreak,
    /// Bloco concluído aguardando captura pós-sessão.
    Completed,
}

/// Temporizador Pomodoro puro orientado por segundos monotônicos recebidos do shell.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PomodoroTimer {
    /// Preset ativo do ciclo.
    pub preset: PomodoroPreset,
    /// Fase atual do temporizador.
    pub phase: FocusPhase,
    target_seconds: u32,
    elapsed_seconds: u32,
    break_elapsed_seconds: u32,
}

impl PomodoroTimer {
    /// Cria um temporizador pronto no preset informado.
    #[must_use]
    pub const fn new(preset: PomodoroPreset) -> Self {
        Self {
            preset,
            phase: FocusPhase::Idle,
            target_seconds: preset.focus_minutes() * 60,
            elapsed_seconds: 0,
            break_elapsed_seconds: 0,
        }
    }

    /// Altera o preset reiniciando o relógio quando o ciclo está parado.
    pub fn apply_preset(&mut self, preset: PomodoroPreset) {
        self.preset = preset;
        self.phase = FocusPhase::Idle;
        self.target_seconds = preset.focus_minutes() * 60;
        self.elapsed_seconds = 0;
        self.break_elapsed_seconds = 0;
    }

    /// Inicia o bloco de foco a partir do zero.
    pub fn start_focus(&mut self) {
        self.phase = FocusPhase::Focusing;
        self.target_seconds = self.preset.focus_minutes() * 60;
        self.elapsed_seconds = 0;
    }

    /// Pausa um bloco de foco em execução.
    pub fn pause(&mut self) {
        if self.phase == FocusPhase::Focusing {
            self.phase = FocusPhase::Paused;
        }
    }

    /// Retoma um bloco pausado.
    pub fn resume(&mut self) {
        if self.phase == FocusPhase::Paused {
            self.phase = FocusPhase::Focusing;
        }
    }

    /// Inicia a pausa curta prevista pelo preset.
    pub fn start_break(&mut self) {
        self.phase = FocusPhase::ShortBreak;
        self.break_elapsed_seconds = 0;
    }

    /// Reinicia o temporizador mantendo o preset atual.
    pub fn reset(&mut self) {
        self.apply_preset(self.preset);
    }

    /// Conclui antecipadamente a sessão mantendo o tempo realmente focado.
    pub fn finish_early(&mut self) {
        if matches!(self.phase, FocusPhase::Focusing | FocusPhase::Paused) {
            self.phase = FocusPhase::Completed;
        }
    }

    /// Avança o cronômetro em `delta_seconds` medidos pelo relógio monotônico do shell.
    ///
    /// Retorna `true` apenas no exato instante em que a fase atual termina.
    pub fn advance_seconds(&mut self, delta_seconds: u32) -> bool {
        match self.phase {
            FocusPhase::Focusing => {
                self.elapsed_seconds = self
                    .elapsed_seconds
                    .saturating_add(delta_seconds)
                    .min(self.target_seconds);
                if self.elapsed_seconds >= self.target_seconds {
                    self.phase = FocusPhase::Completed;
                    return true;
                }
                false
            }
            FocusPhase::ShortBreak => {
                let break_target = self.preset.break_minutes() * 60;
                self.break_elapsed_seconds = self
                    .break_elapsed_seconds
                    .saturating_add(delta_seconds)
                    .min(break_target);
                if self.break_elapsed_seconds >= break_target {
                    self.phase = FocusPhase::Idle;
                    self.elapsed_seconds = 0;
                    self.break_elapsed_seconds = 0;
                    return true;
                }
                false
            }
            FocusPhase::Idle | FocusPhase::Paused | FocusPhase::Completed => false,
        }
    }

    /// Segundos restantes na fase ativa (`Focusing`/`Paused`/`Idle` ou `ShortBreak`).
    #[must_use]
    pub const fn remaining_seconds(&self) -> u32 {
        match self.phase {
            FocusPhase::ShortBreak => {
                (self.preset.break_minutes() * 60).saturating_sub(self.break_elapsed_seconds)
            }
            FocusPhase::Idle
            | FocusPhase::Focusing
            | FocusPhase::Paused
            | FocusPhase::Completed => self.target_seconds.saturating_sub(self.elapsed_seconds),
        }
    }

    /// Razão normalizada de progresso (`0.0..=1.0`) para desenhar o anel em `Canvas`.
    #[must_use]
    #[allow(clippy::cast_precision_loss)]
    pub fn progress_ratio(&self) -> f32 {
        let (elapsed, total) = match self.phase {
            FocusPhase::ShortBreak => {
                (self.break_elapsed_seconds, self.preset.break_minutes() * 60)
            }
            FocusPhase::Idle
            | FocusPhase::Focusing
            | FocusPhase::Paused
            | FocusPhase::Completed => (self.elapsed_seconds, self.target_seconds),
        };
        if total == 0 {
            0.0
        } else {
            (elapsed as f32 / total as f32).clamp(0.0, 1.0)
        }
    }

    /// Retorna os minutos inteiros efetivamente focados no bloco atual.
    #[must_use]
    pub const fn focused_minutes_rounded(&self) -> u32 {
        self.elapsed_seconds / 60
    }
}

/// Paisagens sonoras sintetizadas pelo motor procedural DSP em `-16 LUFS`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DspTrack {
    /// Motor silenciado.
    Silence,
    /// Chuva suave baseada em ruído marrom filtrado e pingos leves.
    BrownRain,
    /// Brisa contínua baseada em ruído rosa modulado.
    PinkWind,
    /// Acorde harmônico suave em baixa frequência.
    WarmDrone,
    /// Leito contínuo com batimento alfa sutil de 10 Hz entre canais L/R.
    BinauralAlpha,
}

const DSP_TRACK_MAP: &[(DspTrack, &str, &str)] = &[
    (DspTrack::Silence, "silence", "Silêncio"),
    (DspTrack::BrownRain, "brown_rain", "Chuva Marrom"),
    (DspTrack::PinkWind, "pink_wind", "Vento Rosa"),
    (DspTrack::WarmDrone, "warm_drone", "Drone Quente"),
    (DspTrack::BinauralAlpha, "binaural_alpha", "Alfa 10 Hz"),
];

impl DspTrack {
    /// Lista completa de paisagens sonoras disponíveis no motor DSP.
    pub const ALL: [Self; 5] = [
        Self::Silence,
        Self::BrownRain,
        Self::PinkWind,
        Self::WarmDrone,
        Self::BinauralAlpha,
    ];

    /// Chave estável para persistência no `cosmetic_loadout` e preferência.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        DSP_TRACK_MAP
            .iter()
            .find(|(track, _, _)| *track == self)
            .map_or("silence", |(_, key, _)| *key)
    }

    /// Rótulo em português exibido no dock de áudio.
    #[must_use]
    pub fn label(self) -> &'static str {
        DSP_TRACK_MAP
            .iter()
            .find(|(track, _, _)| *track == self)
            .map_or("Silêncio", |(_, _, label)| *label)
    }

    /// Converte uma chave persistida em [`DspTrack`].
    pub fn parse(raw: &str) -> Result<Self, DomainError> {
        let trimmed = raw.trim();
        DSP_TRACK_MAP
            .iter()
            .find(|(_, key, _)| *key == trimmed)
            .map(|(track, _, _)| *track)
            .ok_or_else(|| {
                DomainError::InvalidFocusSession(format!("faixa DSP desconhecida: {trimmed}"))
            })
    }
}

/// Modalidade da sessão gravada em `study_sessions`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FocusModality {
    /// Bloco geral de foco.
    Focus,
    /// Estudo teórico.
    Theory,
    /// Resolução de questões.
    Questions,
    /// Revisão espaçada.
    Review,
    /// Sessão de leitura.
    Reading,
}

impl FocusModality {
    /// Chave aceita pela constraint CHECK da tabela `study_sessions`.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Focus => "focus",
            Self::Theory => "theory",
            Self::Questions => "questions",
            Self::Review => "review",
            Self::Reading => "reading",
        }
    }

    /// Rótulo em português para interface.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Focus => "Foco",
            Self::Theory => "Teoria",
            Self::Questions => "Questões",
            Self::Review => "Revisão",
            Self::Reading => "Leitura",
        }
    }

    /// Todas as modalidades suportadas.
    pub const ALL: [Self; 5] = [
        Self::Focus,
        Self::Theory,
        Self::Questions,
        Self::Review,
        Self::Reading,
    ];
}

/// Calcula as moedas ganhas por um bloco real de foco (`15+` min = `1` moeda a cada `5` min).
#[must_use]
pub const fn calculate_focus_coins(duration_minutes: u32) -> i64 {
    if duration_minutes < MINIMUM_REWARDABLE_FOCUS_MINUTES {
        0
    } else {
        (duration_minutes / 5) as i64
    }
}

/// Rascunho de sessão pós-Pomodoro enviado para validação e gravação.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FocusSessionDraft {
    /// Identificador único (`UUIDv7`).
    pub id: EntityId,
    /// Modalidade da sessão.
    pub modality: FocusModality,
    /// Duração efetivamente focada em minutos (`> 0`).
    pub duration_minutes: u32,
    /// Quantidade total de questões resolvidas (opcional).
    pub questions_total: u32,
    /// Quantidade de acertos (`<= questions_total`).
    pub questions_right: u32,
    /// Nota rápida opcional capturada ao final do bloco.
    pub note_capture: Option<String>,
    /// Tarefa vinculada (quando o foco partiu de uma tarefa/Em foco).
    pub task_id: Option<EntityId>,
    /// Início real do bloco em UTC.
    pub started_at: DateTime<Utc>,
    /// Término real do bloco em UTC.
    pub ended_at: DateTime<Utc>,
}

/// Sessão de foco validada e pronta para persistência transacional.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidatedFocusSession {
    /// Dados originais normalizados.
    pub draft: FocusSessionDraft,
    /// Moedas calculadas (respeitando mínimo de 15 minutos).
    pub awarded_coins: i64,
}

impl FocusSessionDraft {
    /// Valida invariantes da sessão de foco antes da gravação no `SQLite`.
    pub fn validate(mut self) -> Result<ValidatedFocusSession, DomainError> {
        if self.duration_minutes == 0 {
            return Err(DomainError::InvalidFocusSession(
                "duração do bloco deve ser maior que zero".to_string(),
            ));
        }
        if self.questions_right > self.questions_total {
            return Err(DomainError::InvalidFocusSession(
                "acertos não podem superar o total de questões".to_string(),
            ));
        }
        if self.ended_at < self.started_at {
            return Err(DomainError::InvalidFocusSession(
                "término da sessão não pode anteceder o início".to_string(),
            ));
        }

        self.note_capture = self
            .note_capture
            .take()
            .map(|raw| raw.trim().to_string())
            .filter(|clean| !clean.is_empty());

        let awarded_coins = calculate_focus_coins(self.duration_minutes);
        Ok(ValidatedFocusSession {
            draft: self,
            awarded_coins,
        })
    }
}
