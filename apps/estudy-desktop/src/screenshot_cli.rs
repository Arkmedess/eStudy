//! Suporte a captura de tela via CLI (`--screenshot`) para inspeção visual e MCP.

use estudy_ui::AppRoute;
use estudy_ui_kit::{ThemeMode, ThemePreset};
use iced::window::Screenshot;
use std::ffi::OsString;
use std::fs::File;
use std::io::BufWriter;
use std::path::{Path, PathBuf};

/// Opções de linha de comando para captura automatizada de tela.
#[derive(Debug, Clone, PartialEq)]
pub struct ScreenshotOptions {
    /// Caminho do arquivo PNG de saída.
    pub output_path: PathBuf,
    /// Rota inicial exibida antes da captura.
    pub route: AppRoute,
    /// Preset de tema aplicado.
    pub preset: ThemePreset,
    /// Modo de polaridade (`Light` ou `Dark`).
    pub mode: ThemeMode,
    /// Largura lógica da janela.
    pub width: f32,
    /// Altura lógica da janela.
    pub height: f32,
    /// Define se a barra lateral inicia expandida.
    pub sidebar_open: bool,
    /// Subvista, modal ou estado interno opcional para captura (`--subview`).
    pub subview: Option<String>,
}

impl Default for ScreenshotOptions {
    fn default() -> Self {
        Self {
            output_path: PathBuf::from("screenshot.png"),
            route: AppRoute::Home,
            preset: ThemePreset::OlivaOrganico,
            mode: ThemeMode::Light,
            width: 1360.0,
            height: 840.0,
            sidebar_open: false,
            subview: None,
        }
    }
}

/// Faz o parsing de argumentos CLI procurando `--screenshot` e flags relacionadas.
#[must_use]
pub fn parse_screenshot_args(
    args: impl IntoIterator<Item = OsString>,
) -> Option<ScreenshotOptions> {
    let items: Vec<String> = args
        .into_iter()
        .map(|s| s.to_string_lossy().into_owned())
        .collect();

    let mut opts = ScreenshotOptions::default();
    let mut has_screenshot = false;
    let mut idx = 0;

    while idx < items.len() {
        match items[idx].as_str() {
            "--screenshot" => {
                if let Some(val) = items.get(idx + 1) {
                    opts.output_path = PathBuf::from(val);
                    has_screenshot = true;
                    idx += 2;
                    continue;
                }
            }
            "--route" => {
                if let Some(val) = items.get(idx + 1)
                    && let Some(route) = parse_route(val)
                {
                    opts.route = route;
                    idx += 2;
                    continue;
                }
            }
            "--theme" => {
                if let Some(val) = items.get(idx + 1)
                    && let Some(preset) = parse_theme(val)
                {
                    opts.preset = preset;
                    idx += 2;
                    continue;
                }
            }
            "--mode" => {
                if let Some(val) = items.get(idx + 1)
                    && let Some(mode) = parse_mode(val)
                {
                    opts.mode = mode;
                    idx += 2;
                    continue;
                }
            }
            "--size" => {
                if let Some(val) = items.get(idx + 1)
                    && let Some((w, h)) = parse_size(val)
                {
                    opts.width = w;
                    opts.height = h;
                    idx += 2;
                    continue;
                }
            }
            "--sidebar" => {
                if let Some(val) = items.get(idx + 1) {
                    opts.sidebar_open =
                        matches!(val.to_ascii_lowercase().as_str(), "open" | "true" | "1");
                    idx += 2;
                    continue;
                }
            }
            "--subview" => {
                if let Some(val) = items.get(idx + 1) {
                    let trimmed = val.trim();
                    if !trimmed.is_empty() {
                        opts.subview = Some(trimmed.to_ascii_lowercase());
                    }
                    idx += 2;
                    continue;
                }
            }
            _ => {}
        }
        idx += 1;
    }

    has_screenshot.then_some(opts)
}

fn parse_route(raw: &str) -> Option<AppRoute> {
    match raw.to_ascii_lowercase().as_str() {
        "home" | "inicio" => Some(AppRoute::Home),
        "projects" | "tarefas" | "tasks" => Some(AppRoute::Projects),
        "habits" | "habitos" => Some(AppRoute::Habits),
        "second-brain" | "second_brain" | "brain" | "notas" => Some(AppRoute::SecondBrain),
        "calendar" | "agenda" => Some(AppRoute::Calendar),
        "studies" | "estudos" => Some(AppRoute::Studies),
        "bookshelf" | "livros" => Some(AppRoute::Bookshelf),
        "finance" | "financas" => Some(AppRoute::Finance),
        "focus" | "foco" => Some(AppRoute::Focus),
        "garden" | "jardim" => Some(AppRoute::GardenAndPets),
        "shop" | "loja" => Some(AppRoute::Shop),
        "settings" | "ajustes" => Some(AppRoute::Settings),
        _ => None,
    }
}

fn parse_theme(raw: &str) -> Option<ThemePreset> {
    match raw.to_ascii_lowercase().as_str() {
        "oliva" | "oliva-organico" | "oliva_organico" => Some(ThemePreset::OlivaOrganico),
        "menta" | "estudy-menta" | "estudy_menta" => Some(ThemePreset::EstudyMenta),
        "cyber" | "cyber-minimalist" | "cyber_minimalist" => Some(ThemePreset::CyberMinimalist),
        "amber" | "solar-amber" | "solar_amber" => Some(ThemePreset::SolarAmber),
        "sakura" | "velvet-sakura" | "velvet_sakura" => Some(ThemePreset::VelvetSakura),
        _ => None,
    }
}

fn parse_mode(raw: &str) -> Option<ThemeMode> {
    match raw.to_ascii_lowercase().as_str() {
        "light" | "claro" => Some(ThemeMode::Light),
        "dark" | "escuro" => Some(ThemeMode::Dark),
        _ => None,
    }
}

fn parse_size(raw: &str) -> Option<(f32, f32)> {
    let (w_str, h_str) = raw.split_once('x').or_else(|| raw.split_once('X'))?;
    let w: f32 = w_str.trim().parse().ok()?;
    let h: f32 = h_str.trim().parse().ok()?;
    (w >= 320.0 && h >= 240.0).then_some((w, h))
}

/// Codifica e grava um [`Screenshot`] do `iced` em formato PNG RGBA.
pub fn save_screenshot_png(shot: &Screenshot, path: &Path) -> Result<(), String> {
    if let Some(parent) = path.parent()
        && !parent.as_os_str().is_empty()
    {
        std::fs::create_dir_all(parent).map_err(|err| err.to_string())?;
    }

    let file = File::create(path).map_err(|err| err.to_string())?;
    let writer = BufWriter::new(file);
    let mut encoder = png::Encoder::new(writer, shot.size.width, shot.size.height);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    let mut png_writer = encoder.write_header().map_err(|err| err.to_string())?;
    png_writer
        .write_image_data(&shot.rgba)
        .map_err(|err| err.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use iced::Size;

    #[test]
    fn returns_none_without_screenshot_flag() {
        let args = vec![OsString::from("--data-dir"), OsString::from("/tmp/db")];
        assert_eq!(parse_screenshot_args(args), None);
    }

    #[test]
    fn parses_all_screenshot_flags() {
        let args = vec![
            "--screenshot",
            "/tmp/home.png",
            "--route",
            "habits",
            "--theme",
            "cyber",
            "--mode",
            "dark",
            "--size",
            "1600x900",
            "--sidebar",
            "open",
        ]
        .into_iter()
        .map(OsString::from);

        let expected = ScreenshotOptions {
            output_path: PathBuf::from("/tmp/home.png"),
            route: AppRoute::Habits,
            preset: ThemePreset::CyberMinimalist,
            mode: ThemeMode::Dark,
            width: 1600.0,
            height: 900.0,
            sidebar_open: true,
            subview: None,
        };
        assert_eq!(parse_screenshot_args(args), Some(expected));
    }

    #[test]
    fn parses_optional_subview_flag() {
        let args = [
            "estudy-desktop",
            "--screenshot",
            "/tmp/subview.png",
            "--route",
            "projects",
            "--subview",
            "Kanban",
        ]
        .into_iter()
        .map(OsString::from);

        let parsed = parse_screenshot_args(args);
        assert!(parsed.is_some());
        if let Some(opts) = parsed {
            assert_eq!(opts.route, AppRoute::Projects);
            assert_eq!(opts.subview.as_deref(), Some("kanban"));
        }
    }

    #[test]
    fn encodes_valid_png_file() -> Result<(), String> {
        let out = std::env::temp_dir().join(format!("estudy_png_test_{}.png", std::process::id()));
        let shot = Screenshot::new(vec![255, 0, 0, 255, 0, 255, 0, 255], Size::new(2, 1), 1.0);
        save_screenshot_png(&shot, &out)?;
        let bytes = std::fs::read(&out).map_err(|err| err.to_string())?;
        let _ = std::fs::remove_file(&out);
        assert_eq!(&bytes[..8], b"\x89PNG\r\n\x1a\n");
        Ok(())
    }
}
