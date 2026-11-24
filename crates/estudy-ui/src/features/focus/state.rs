//! Estado MVU puro para Pomodoro, captura pós-sessão e controle do motor DSP.

use chrono::{DateTime, Duration, Utc};
use estudy_domain::{
    DspTrack, EntityId, FocusModality, FocusPhase, FocusSessionDraft, PomodoroPreset, PomodoroTimer,
};

/// Rascunho do cartão de captura rápida exibido ao concluir um bloco de foco.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PostSessionCaptureDraft {
    /// Controla visibilidade do formulário pós-sessão.
    pub open: bool,
    /// Modalidade de estudo registrada.
    pub modality: FocusModality,
    /// Nota rápida opcional gravada no `SQLite` / `FTS5`.
    pub note_capture: String,
    /// Quantidade total de questões (texto editável).
    pub questions_total_input: String,
    /// Quantidade de acertos (texto editável).
    pub questions_right_input: String,
    /// Erro de validação exibido ao tentar salvar com dados inválidos.
    pub error: Option<String>,
}

impl Default for PostSessionCaptureDraft {
    fn default() -> Self {
        Self {
            open: false,
            modality: FocusModality::Focus,
            note_capture: String::new(),
            questions_total_input: "0".to_string(),
            questions_right_input: "0".to_string(),
            error: None,
        }
    }
}

/// Mensagens da fatia Pomodoro & Modo Foco.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FocusMessage {
    /// Entra no Modo Foco em tela cheia com o estado atual.
    EnterFullscreen,
    /// Sai da tela cheia preservando o temporizador ativo.
    ExitFullscreen,
    /// Entra no Modo Foco vinculado a uma tarefa ou meta específica.
    OpenWithTarget {
        /// Título contextual exibido no foco.
        title: String,
        /// Identificador opcional da tarefa vinculada.
        task_id: Option<EntityId>,
        /// Preset inicial aplicado.
        preset: PomodoroPreset,
    },
    /// Seleciona um preset de duração (`25/5`, `50/10`, `15/3` ou `Livre`).
    SelectPreset(PomodoroPreset),
    /// Inicia o bloco de foco atual.
    Start,
    /// Pausa o foco em andamento.
    Pause,
    /// Retoma o foco pausado.
    Resume,
    /// Conclui o bloco atual antecipadamente abrindo a captura pós-sessão.
    FinishEarly,
    /// Reinicia o ciclo atual para `Idle`.
    Reset,
    /// Inicia o intervalo curto de descanso.
    StartBreak,
    /// Avança o relógio interno em `delta_seconds` monotônicos.
    TickSeconds(u32),
    /// Troca a faixa procedural do motor DSP.
    SelectDspTrack(DspTrack),
    /// Altera o volume (`0..=100`).
    SetVolumePercent(u8),
    /// Alterna mudo rápido no dock de áudio.
    ToggleMute,
    /// Seleciona a modalidade na captura pós-sessão.
    SelectModality(FocusModality),
    /// Atualiza a nota rápida capturada ao final do bloco.
    CaptureNoteChanged(String),
    /// Atualiza o campo numérico de questões feitas.
    QuestionsTotalChanged(String),
    /// Atualiza o campo numérico de acertos.
    QuestionsRightChanged(String),
    /// Valida e enfileira a sessão concluída para persistência transacional.
    SavePostSession,
    /// Descarta a captura atual e volta ao repouso.
    DiscardPostSession,
}

/// Estado completo do módulo Pomodoro, Modo Foco e Áudio DSP.
#[derive(Debug, Clone)]
pub struct FocusState {
    /// Indica se o Modo Foco ocupa toda a janela ocultando menus e barra superior.
    pub fullscreen_active: bool,
    /// Máquina de estados pura do cronômetro Pomodoro.
    pub timer: PomodoroTimer,
    /// Instante UTC de início do bloco corrente.
    pub active_started_at: Option<DateTime<Utc>>,
    /// Título da tarefa ou estudo vinculado ao bloco atual.
    pub active_target_title: Option<String>,
    /// ID da tarefa vinculada (quando houver).
    pub active_task_id: Option<EntityId>,
    /// Paisagem sonora DSP selecionada.
    pub dsp_track: DspTrack,
    /// Volume em porcentagem (`0..=100`).
    pub volume_percent: u8,
    /// Estado de mudo explícito.
    pub muted: bool,
    /// Indica se o áudio DSP deve estar tocando.
    pub audio_playing: bool,
    /// Rascunho de captura rápida pós-sessão.
    pub capture: PostSessionCaptureDraft,
    /// Minutos focados acumulados hoje (carregados do `SQLite` e somados ao salvar).
    pub today_focused_minutes: u32,
    /// Sessões concluídas aguardando gravação transacional pelo shell desktop.
    pub pending_sessions: Vec<FocusSessionDraft>,
    /// Último feedback informativo exibido na tela de foco.
    pub notice: Option<String>,
}

impl Default for FocusState {
    fn default() -> Self {
        Self {
            fullscreen_active: false,
            timer: PomodoroTimer::new(PomodoroPreset::Classic25_5),
            active_started_at: None,
            active_target_title: None,
            active_task_id: None,
            dsp_track: DspTrack::BrownRain,
            volume_percent: 70,
            muted: false,
            audio_playing: false,
            capture: PostSessionCaptureDraft::default(),
            today_focused_minutes: 0,
            pending_sessions: Vec::new(),
            notice: None,
        }
    }
}

impl FocusState {
    /// Atualiza o estado MVU do Pomodoro, áudio DSP e captura pós-sessão.
    pub fn update(&mut self, message: FocusMessage, now_utc: DateTime<Utc>) {
        match message {
            FocusMessage::EnterFullscreen => self.fullscreen_active = true,
            FocusMessage::ExitFullscreen => self.fullscreen_active = false,
            FocusMessage::OpenWithTarget {
                title,
                task_id,
                preset,
            } => self.open_with_target(title, task_id, preset),
            FocusMessage::SelectPreset(preset) => {
                if self.timer.phase == FocusPhase::Idle || self.timer.phase == FocusPhase::Completed
                {
                    self.timer.apply_preset(preset);
                    self.capture.open = false;
                }
            }
            FocusMessage::Start => self.start_timer(now_utc),
            FocusMessage::Pause => {
                self.timer.pause();
                self.audio_playing = false;
            }
            FocusMessage::Resume => {
                self.timer.resume();
                self.audio_playing = self.timer.phase == FocusPhase::Focusing;
            }
            FocusMessage::FinishEarly => {
                self.timer.finish_early();
                self.audio_playing = false;
                if self.timer.phase == FocusPhase::Completed {
                    self.capture.open = true;
                }
            }
            FocusMessage::Reset => {
                self.timer.reset();
                self.active_started_at = None;
                self.audio_playing = false;
                self.capture = PostSessionCaptureDraft::default();
            }
            FocusMessage::StartBreak => {
                self.timer.start_break();
                self.audio_playing = false;
            }
            FocusMessage::TickSeconds(delta) => self.handle_tick(delta),
            FocusMessage::SelectDspTrack(track) => self.dsp_track = track,
            FocusMessage::SetVolumePercent(percent) => {
                self.volume_percent = percent.min(100);
            }
            FocusMessage::ToggleMute => self.muted = !self.muted,
            FocusMessage::SelectModality(modality) => self.capture.modality = modality,
            FocusMessage::CaptureNoteChanged(note) => self.capture.note_capture = note,
            FocusMessage::QuestionsTotalChanged(raw) => {
                self.capture.questions_total_input = raw;
                self.capture.error = None;
            }
            FocusMessage::QuestionsRightChanged(raw) => {
                self.capture.questions_right_input = raw;
                self.capture.error = None;
            }
            FocusMessage::SavePostSession => self.save_post_session(now_utc),
            FocusMessage::DiscardPostSession => {
                self.capture = PostSessionCaptureDraft::default();
                self.timer.reset();
                self.active_started_at = None;
            }
        }
    }

    /// Retira sessões concluídas pendentes de gravação no `SQLite`.
    pub fn take_pending_sessions(&mut self) -> Vec<FocusSessionDraft> {
        std::mem::take(&mut self.pending_sessions)
    }

    /// Informa se o temporizador exige ticks de 1 segundo no loop de eventos.
    #[must_use]
    pub const fn needs_ticker(&self) -> bool {
        matches!(
            self.timer.phase,
            FocusPhase::Focusing | FocusPhase::ShortBreak
        )
    }

    fn open_with_target(
        &mut self,
        title: String,
        task_id: Option<EntityId>,
        preset: PomodoroPreset,
    ) {
        self.fullscreen_active = true;
        self.active_target_title = Some(title);
        self.active_task_id = task_id;
        if self.timer.phase == FocusPhase::Idle {
            self.timer.apply_preset(preset);
        }
    }

    fn start_timer(&mut self, now_utc: DateTime<Utc>) {
        self.capture = PostSessionCaptureDraft::default();
        self.notice = None;
        self.active_started_at = Some(now_utc);
        self.timer.start_focus();
        self.audio_playing = true;
    }

    fn handle_tick(&mut self, delta_seconds: u32) {
        let finished = self.timer.advance_seconds(delta_seconds);
        if finished {
            self.audio_playing = false;
            if self.timer.phase == FocusPhase::Completed {
                self.capture.open = true;
                self.fullscreen_active = true;
            }
        }
    }

    fn save_post_session(&mut self, now_utc: DateTime<Utc>) {
        let Ok(questions_total) = self.capture.questions_total_input.trim().parse::<u32>() else {
            self.capture.error = Some("Informe um número válido em total de questões.".to_string());
            return;
        };
        let Ok(questions_right) = self.capture.questions_right_input.trim().parse::<u32>() else {
            self.capture.error = Some("Informe um número válido em acertos.".to_string());
            return;
        };

        let duration_minutes = self.timer.focused_minutes_rounded().max(1);
        let started_at = self
            .active_started_at
            .unwrap_or_else(|| now_utc - Duration::minutes(i64::from(duration_minutes)));

        let draft = FocusSessionDraft {
            id: EntityId::new_v7(),
            modality: self.capture.modality,
            duration_minutes,
            questions_total,
            questions_right,
            note_capture: Some(self.capture.note_capture.clone()),
            task_id: self.active_task_id.clone(),
            started_at,
            ended_at: now_utc,
        };

        match draft.clone().validate() {
            Ok(validated) => {
                self.today_focused_minutes = self
                    .today_focused_minutes
                    .saturating_add(validated.draft.duration_minutes);
                self.pending_sessions.push(validated.draft);
                self.notice = Some(format!(
                    "Sessão de {} min registrada (+{} moedas).",
                    duration_minutes, validated.awarded_coins
                ));
                self.capture = PostSessionCaptureDraft::default();
                self.timer.reset();
                self.active_started_at = None;
            }
            Err(err) => {
                self.capture.error = Some(err.to_string());
            }
        }
    }
}
