//! Renderização do Pomodoro em `Canvas`, Modo Foco em tela cheia, dock DSP e captura rápida.

use std::f32::consts::FRAC_PI_2;

use estudy_domain::{DspTrack, FocusModality, FocusPhase, PomodoroPreset};
use estudy_ui_kit::widgets::{
    banner_toast, card, hairline_divider, icon_svg, kbd_hint, pill_chip, section_header,
    segmented_switch, soft_surface, status_badge,
};
use estudy_ui_kit::{
    EstudyTheme, FONT_SIZE_BASE, FONT_SIZE_HERO_TIMER, FONT_SIZE_MD, Icon, SPACE_1, SPACE_2,
    SPACE_3, SPACE_4, SPACE_6,
};
use iced::widget::canvas::{self, Geometry, Path, Stroke};
use iced::widget::{
    button, canvas as iced_canvas, column, container, row, slider, text, text_input,
};
use iced::{Alignment, Element, Length, Point, Rectangle, Renderer, Theme};

use super::state::{FocusMessage, FocusState};

/// Renderiza a tela de Modo Foco em tela cheia com anel `Canvas`, Pet, dock DSP e captura.
pub fn view_focus_mode<'a, Message: Clone + 'a>(
    state: &'a FocusState,
    pet_label: &'a str,
    theme: EstudyTheme,
    map_msg: impl Fn(FocusMessage) -> Message + Clone + 'a,
) -> Element<'a, Message> {
    let top_strip = view_focus_header(state, pet_label, theme, map_msg.clone());
    let ring = view_timer_ring(state, theme);
    let controls = view_timer_controls(state, theme, &map_msg);
    let dsp_dock = view_dsp_dock(state, theme, map_msg.clone());

    let mut stack_items = column![top_strip, ring, controls]
        .spacing(SPACE_4)
        .align_x(Alignment::Center);
    if let Some(notice) = &state.notice {
        stack_items = stack_items.push(
            container(banner_toast(Icon::CheckCircle2, notice, None, theme)).max_width(520.0),
        );
    }
    if state.capture.open {
        stack_items = stack_items.push(view_post_session_capture(state, theme, map_msg));
    } else {
        stack_items = stack_items.push(dsp_dock);
    }

    container(
        container(stack_items)
            .width(Length::Fill)
            .max_width(760.0)
            .center_x(Length::Fill),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .padding(SPACE_6)
    .center_x(Length::Fill)
    .center_y(Length::Fill)
    .style(move |_| iced::widget::container::Style {
        background: Some(theme.bg.into()),
        text_color: Some(theme.text_primary),
        ..iced::widget::container::Style::default()
    })
    .into()
}

fn view_focus_header<'a, Message: Clone + 'a>(
    state: &'a FocusState,
    pet_label: &'a str,
    theme: EstudyTheme,
    map_msg: impl Fn(FocusMessage) -> Message + Clone + 'a,
) -> Element<'a, Message> {
    const PRESETS: [(PomodoroPreset, &str); 3] = [
        (PomodoroPreset::Classic25_5, "25 / 5"),
        (PomodoroPreset::Deep50_10, "50 / 10"),
        (PomodoroPreset::Sprint15_3, "15 / 3"),
    ];

    let target_title = state
        .active_target_title
        .as_deref()
        .unwrap_or("Bloco de foco livre");

    let exit_msg = map_msg(FocusMessage::ExitFullscreen);
    let preset_switch = segmented_switch(
        &PRESETS,
        state.timer.preset,
        move |preset| map_msg(FocusMessage::SelectPreset(preset)),
        theme,
    );

    let exit_btn = button(
        row![
            icon_svg(Icon::X, 15.0, theme.text_muted),
            text("Sair do foco")
                .size(FONT_SIZE_BASE)
                .color(theme.text_primary),
            kbd_hint("Esc", theme),
        ]
        .spacing(SPACE_2)
        .align_y(Alignment::Center),
    )
    .on_press(exit_msg)
    .padding([SPACE_1, SPACE_3])
    .style(move |_, status| theme.subtle_button_style(status));

    row![
        column![
            section_header(Icon::Timer, target_title, theme),
            row![
                icon_svg(Icon::PawPrint, 14.0, theme.accent),
                text(format!(
                    "{pet_label} · {} min focados hoje",
                    state.today_focused_minutes
                ))
                .size(FONT_SIZE_BASE)
                .color(theme.text_muted),
            ]
            .spacing(SPACE_1)
            .align_y(Alignment::Center),
        ]
        .spacing(SPACE_1)
        .width(Length::Fill),
        preset_switch,
        exit_btn,
    ]
    .spacing(SPACE_3)
    .align_y(Alignment::Center)
    .width(Length::Fill)
    .into()
}

fn view_timer_ring<'a, Message: 'a>(
    state: &'a FocusState,
    theme: EstudyTheme,
) -> Element<'a, Message> {
    let remaining = state.timer.remaining_seconds();
    let minutes = remaining / 60;
    let seconds = remaining % 60;
    let phase_label = match state.timer.phase {
        FocusPhase::Idle => "Pronto para focar",
        FocusPhase::Focusing => "Foco ativo",
        FocusPhase::Paused => "Pausado",
        FocusPhase::ShortBreak => "Pausa curta",
        FocusPhase::Completed => "Bloco concluído",
    };

    let ring_canvas = iced_canvas(FocusRingProgram {
        progress: state.timer.progress_ratio(),
        is_break: state.timer.phase == FocusPhase::ShortBreak,
        theme,
    })
    .width(Length::Fixed(268.0))
    .height(Length::Fixed(268.0));

    let overlay_clock = container(
        column![
            text(format!("{minutes:02}:{seconds:02}"))
                .size(FONT_SIZE_HERO_TIMER)
                .color(theme.text_primary),
            status_badge(
                phase_label,
                if state.timer.phase == FocusPhase::ShortBreak {
                    theme.success
                } else {
                    theme.accent
                },
                theme,
            ),
        ]
        .spacing(SPACE_2)
        .align_x(Alignment::Center),
    )
    .width(Length::Fixed(268.0))
    .height(Length::Fixed(268.0))
    .center_x(Length::Fixed(268.0))
    .center_y(Length::Fixed(268.0));

    iced::widget::stack![ring_canvas, overlay_clock]
        .width(Length::Fixed(268.0))
        .height(Length::Fixed(268.0))
        .into()
}

fn view_timer_controls<'a, Message: Clone + 'a>(
    state: &'a FocusState,
    theme: EstudyTheme,
    map_msg: &impl Fn(FocusMessage) -> Message,
) -> Element<'a, Message> {
    let mut actions = row![].spacing(SPACE_2).align_y(Alignment::Center);
    match state.timer.phase {
        FocusPhase::Idle => {
            actions = actions.push(pill_chip(
                "Iniciar bloco",
                None,
                Some(Icon::Play),
                true,
                map_msg(FocusMessage::Start),
                theme,
            ));
        }
        FocusPhase::Focusing => {
            actions = actions.push(pill_chip(
                "Pausar",
                None,
                Some(Icon::Pause),
                true,
                map_msg(FocusMessage::Pause),
                theme,
            ));
            actions = actions.push(pill_chip(
                "Concluir agora",
                None,
                Some(Icon::CheckCircle2),
                false,
                map_msg(FocusMessage::FinishEarly),
                theme,
            ));
        }
        FocusPhase::Paused => {
            actions = actions.push(pill_chip(
                "Retomar",
                None,
                Some(Icon::Play),
                true,
                map_msg(FocusMessage::Resume),
                theme,
            ));
            actions = actions.push(pill_chip(
                "Concluir",
                None,
                Some(Icon::CheckCircle2),
                false,
                map_msg(FocusMessage::FinishEarly),
                theme,
            ));
            actions = actions.push(pill_chip(
                "Reiniciar",
                None,
                Some(Icon::RotateCcw),
                false,
                map_msg(FocusMessage::Reset),
                theme,
            ));
        }
        FocusPhase::ShortBreak => {
            actions = actions.push(pill_chip(
                "Voltar ao foco",
                None,
                Some(Icon::RotateCcw),
                false,
                map_msg(FocusMessage::Reset),
                theme,
            ));
        }
        FocusPhase::Completed => {
            actions = actions.push(pill_chip(
                "Iniciar pausa",
                None,
                Some(Icon::Coffee),
                false,
                map_msg(FocusMessage::StartBreak),
                theme,
            ));
            actions = actions.push(pill_chip(
                "Novo bloco",
                None,
                Some(Icon::RotateCcw),
                false,
                map_msg(FocusMessage::Reset),
                theme,
            ));
        }
    }
    actions.into()
}

fn view_dsp_dock<'a, Message: Clone + 'a>(
    state: &'a FocusState,
    theme: EstudyTheme,
    map_msg: impl Fn(FocusMessage) -> Message + Clone + 'a,
) -> Element<'a, Message> {
    let mut track_row = row![].spacing(SPACE_1).align_y(Alignment::Center);
    for track in DspTrack::ALL {
        track_row = track_row.push(pill_chip(
            track.label(),
            None,
            None,
            track == state.dsp_track,
            map_msg(FocusMessage::SelectDspTrack(track)),
            theme,
        ));
    }

    let mute_icon = if state.muted || state.dsp_track == DspTrack::Silence {
        Icon::VolumeX
    } else {
        Icon::Volume2
    };
    let mute_msg = map_msg(FocusMessage::ToggleMute);

    let vol_slider = slider(0..=100, state.volume_percent, move |val| {
        map_msg(FocusMessage::SetVolumePercent(val))
    })
    .width(Length::Fixed(160.0));

    let volume_bar = row![
        button(icon_svg(mute_icon, 16.0, theme.accent))
            .on_press(mute_msg)
            .padding(SPACE_1)
            .style(move |_, status| theme.subtle_button_style(status)),
        vol_slider,
        text(format!("{}%", state.volume_percent))
            .size(FONT_SIZE_BASE)
            .color(theme.text_muted),
        container(status_badge("-16 LUFS DSP", theme.accent, theme))
            .align_x(Alignment::End)
            .width(Length::Fill),
    ]
    .spacing(SPACE_2)
    .align_y(Alignment::Center);

    card(
        column![
            section_header(Icon::Headphones, "Áudio Procedural de Foco", theme),
            hairline_divider(theme),
            track_row,
            volume_bar,
        ]
        .spacing(SPACE_3),
        theme,
    )
    .width(Length::Fill)
    .into()
}

fn view_post_session_capture<'a, Message: Clone + 'a>(
    state: &'a FocusState,
    theme: EstudyTheme,
    map_msg: impl Fn(FocusMessage) -> Message + Clone + 'a,
) -> Element<'a, Message> {
    let mut mod_row = row![].spacing(SPACE_1);
    for modality in FocusModality::ALL {
        mod_row = mod_row.push(pill_chip(
            modality.label(),
            None,
            None,
            state.capture.modality == modality,
            map_msg(FocusMessage::SelectModality(modality)),
            theme,
        ));
    }

    let note_input = text_input(
        "Resumo rápido do bloco (opcional, indexado na busca)...",
        &state.capture.note_capture,
    )
    .on_input({
        let map_msg = map_msg.clone();
        move |val| map_msg(FocusMessage::CaptureNoteChanged(val))
    })
    .padding(SPACE_2)
    .size(FONT_SIZE_MD)
    .style(move |_, status| theme.soft_input_style(status));

    let q_total = text_input("Questões", &state.capture.questions_total_input)
        .on_input({
            let map_msg = map_msg.clone();
            move |val| map_msg(FocusMessage::QuestionsTotalChanged(val))
        })
        .width(Length::Fixed(112.0))
        .padding(SPACE_2)
        .size(FONT_SIZE_BASE)
        .style(move |_, status| theme.soft_input_style(status));

    let q_right = text_input("Acertos", &state.capture.questions_right_input)
        .on_input({
            let map_msg = map_msg.clone();
            move |val| map_msg(FocusMessage::QuestionsRightChanged(val))
        })
        .width(Length::Fixed(112.0))
        .padding(SPACE_2)
        .size(FONT_SIZE_BASE)
        .style(move |_, status| theme.soft_input_style(status));

    let mut body = column![
        section_header(Icon::CheckCircle2, "Captura Rápida Pós-Sessão", theme),
        hairline_divider(theme),
        mod_row,
        note_input,
        soft_surface(
            row![
                text("Questões:")
                    .size(FONT_SIZE_BASE)
                    .color(theme.text_muted),
                q_total,
                text("Acertos:")
                    .size(FONT_SIZE_BASE)
                    .color(theme.text_muted),
                q_right,
            ]
            .spacing(SPACE_2)
            .align_y(Alignment::Center),
            theme,
        ),
    ]
    .spacing(SPACE_3);

    if let Some(err) = &state.capture.error {
        body = body.push(status_badge(err, theme.danger, theme));
    }

    body = body.push(
        row![
            pill_chip(
                "Salvar sessão",
                None,
                Some(Icon::CheckCircle2),
                true,
                map_msg(FocusMessage::SavePostSession),
                theme,
            ),
            pill_chip(
                "Descartar",
                None,
                Some(Icon::X),
                false,
                map_msg(FocusMessage::DiscardPostSession),
                theme,
            ),
        ]
        .spacing(SPACE_2),
    );

    card(body, theme).width(Length::Fill).into()
}

struct FocusRingProgram {
    progress: f32,
    is_break: bool,
    theme: EstudyTheme,
}

impl<Message> canvas::Program<Message> for FocusRingProgram {
    type State = ();

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: iced::mouse::Cursor,
    ) -> Vec<Geometry> {
        let mut frame = canvas::Frame::new(renderer, bounds.size());
        let center = Point::new(bounds.width * 0.5, bounds.height * 0.5);
        let radius = (bounds.width.min(bounds.height) * 0.5) - 14.0;

        let track = Path::circle(center, radius);
        frame.stroke(
            &track,
            Stroke::default()
                .with_color(self.theme.surface_soft)
                .with_width(10.0),
        );

        let clamped = self.progress.clamp(0.0, 1.0);
        if clamped > 0.001 {
            let start = iced::Radians(-FRAC_PI_2);
            let end = iced::Radians(-FRAC_PI_2 + clamped * std::f32::consts::TAU);
            let arc = Path::new(|builder| {
                builder.arc(canvas::path::Arc {
                    center,
                    radius,
                    start_angle: start,
                    end_angle: end,
                });
            });
            let active_color = if self.is_break {
                self.theme.success
            } else {
                self.theme.accent
            };
            frame.stroke(
                &arc,
                Stroke::default()
                    .with_color(active_color)
                    .with_width(10.0)
                    .with_line_cap(canvas::LineCap::Round),
            );
        }

        vec![frame.into_geometry()]
    }
}
