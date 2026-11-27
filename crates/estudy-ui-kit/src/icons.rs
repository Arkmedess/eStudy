//! Catálogo único de 87 ícones vetoriais SVG (estilo Lucide, zero emojis).

use iced::Color;
use iced::widget::svg;
use std::sync::LazyLock;

static CACHED_SVG_HANDLES: LazyLock<Vec<svg::Handle>> = LazyLock::new(|| {
    Icon::ALL
        .iter()
        .map(|icon| svg::Handle::from_memory(icon.to_svg_string(Color::WHITE).into_bytes()))
        .collect()
});

/// Catálogo completo dos 87 ícones usados no eStudy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Icon {
    /// Home.
    Home,
    /// Busca.
    Search,
    /// Comando (`Ctrl+K`).
    Command,
    /// Projetos.
    FolderKanban,
    /// Tarefas / Check.
    CheckSquare,
    /// Calendário.
    Calendar,
    /// Relógio / Hoje.
    Clock,
    /// Estudos / Chapéu acadêmico.
    GraduationCap,
    /// Estante / Livros.
    BookOpen,
    /// Jardim / Broto.
    Sprout,
    /// Pet / Pata.
    PawPrint,
    /// Loja / Sacola.
    ShoppingBag,
    /// Configurações.
    Settings,
    /// Foco / Alvo.
    Target,
    /// Timer / Ampulheta.
    Timer,
    /// Áudio / Volume.
    Volume2,
    /// Áudio mudo.
    VolumeX,
    /// Play.
    Play,
    /// Pause.
    Pause,
    /// Reiniciar.
    RotateCcw,
    /// Troféu / Conquistas.
    Trophy,
    /// Medalha.
    Award,
    /// Chama / Streak.
    Flame,
    /// Floco / Streak Freeze.
    Snowflake,
    /// Sol / Light Mode.
    Sun,
    /// Lua / Dark Mode.
    Moon,
    /// Paleta de temas.
    Palette,
    /// Adicionar.
    Plus,
    /// Fechar.
    X,
    /// Editar.
    Pencil,
    /// Lixeira.
    Trash2,
    /// Arquivar.
    Archive,
    /// Etiqueta / Tag.
    Tag,
    /// Bandeira / Prazo.
    Flag,
    /// Pin / Em foco.
    Pin,
    /// Estrela.
    Star,
    /// Coração / Humor ótimo.
    Heart,
    /// Sorriso / Humor bom.
    Smile,
    /// Neutro / Humor estável.
    Meh,
    /// Nuvem / Humor cansado.
    CloudRain,
    /// Raio / Energia.
    Zap,
    /// Gráfico de barras.
    BarChart3,
    /// Gráfico pizza / Donut.
    PieChart,
    /// Tendência crescente.
    TrendingUp,
    /// Camadas / Marcos.
    Layers,
    /// Grade 2x2 / Eisenhower.
    LayoutGrid,
    /// Lista.
    List,
    /// Colunas Kanban.
    Columns3,
    /// Painel lateral.
    PanelLeft,
    /// Seta direita.
    ChevronRight,
    /// Seta esquerda.
    ChevronLeft,
    /// Seta baixo.
    ChevronDown,
    /// Seta cima.
    ChevronUp,
    /// Seta externa.
    ArrowUpRight,
    /// Alerta.
    AlertTriangle,
    /// Informação.
    Info,
    /// Sucesso circular.
    CheckCircle2,
    /// Erro circular.
    XCircle,
    /// Flashcards / Cartas.
    GalleryVerticalEnd,
    /// Cérebro / Active Recall.
    Brain,
    /// Caderno de erros.
    BookX,
    /// Citação.
    Quote,
    /// Marcador de página.
    Bookmark,
    /// Dado / Sorteio TBR.
    Dices,
    /// Filtro.
    Filter,
    /// Ordenação.
    ArrowUpDown,
    /// Upload / Importar.
    Upload,
    /// Download / Exportar.
    Download,
    /// Backup / Banco.
    Database,
    /// Nuvem / Snapshot.
    Cloud,
    /// Cadeado / HMAC.
    Lock,
    /// Escudo / Auditoria.
    ShieldCheck,
    /// Terminal / MCP.
    Terminal,
    /// Usuário / Perfil.
    User,
    /// Sino / Notificações.
    Bell,
    /// Imagem / Quadro.
    Image,
    /// Moldura.
    Frame,
    /// Moeda.
    Coins,
    /// Árvore.
    TreePine,
    /// Flor.
    Flower2,
    /// Gota / Regar.
    Droplets,
    /// Fone / Áudio foco.
    Headphones,
    /// Café / Pausa.
    Coffee,
    /// Olho / Visualizar.
    Eye,
    /// Código / Markdown.
    Code2,
    /// Repetir / Recorrência.
    Repeat,
    /// Ajuda.
    HelpCircle,
}

struct IconSpec {
    icon: Icon,
    name: &'static str,
    geometry: &'static str,
}

const ICON_SPECS: &[IconSpec] = &[
    IconSpec {
        icon: Icon::Home,
        name: "home",
        geometry: r#"<path d="M3 10.5 12 3l9 7.5"/><path d="M5 9.5V20h14V9.5"/><path d="M9 20v-6h6v6"/>"#,
    },
    IconSpec {
        icon: Icon::Search,
        name: "search",
        geometry: r#"<circle cx="11" cy="11" r="7"/><path d="m21 21-4.3-4.3"/>"#,
    },
    IconSpec {
        icon: Icon::Command,
        name: "command",
        geometry: r#"<path d="M18 3a3 3 0 0 0-3 3v12a3 3 0 0 0 3 3 3 3 0 0 0 3-3 3 3 0 0 0-3-3H6a3 3 0 0 0-3 3 3 3 0 0 0 3 3 3 3 0 0 0 3-3V6a3 3 0 0 0-3-3 3 3 0 0 0-3 3 3 3 0 0 0 3 3h12a3 3 0 0 0 3-3 3 3 0 0 0-3-3z"/>"#,
    },
    IconSpec {
        icon: Icon::FolderKanban,
        name: "folder-kanban",
        geometry: r#"<path d="M4 20h16a2 2 0 0 0 2-2V8a2 2 0 0 0-2-2h-7.9l-2-3H4a2 2 0 0 0-2 2v13a2 2 0 0 0 2 2z"/><path d="M8 10v4"/><path d="M12 10v2"/><path d="M16 10v6"/>"#,
    },
    IconSpec {
        icon: Icon::CheckSquare,
        name: "check-square",
        geometry: r#"<rect width="18" height="18" x="3" y="3" rx="2"/><path d="m9 12 2 2 4-4"/>"#,
    },
    IconSpec {
        icon: Icon::Calendar,
        name: "calendar",
        geometry: r#"<rect width="18" height="18" x="3" y="4" rx="2"/><path d="M16 2v4"/><path d="M8 2v4"/><path d="M3 10h18"/>"#,
    },
    IconSpec {
        icon: Icon::Clock,
        name: "clock",
        geometry: r#"<circle cx="12" cy="12" r="9"/><path d="M12 7v5l3 2"/>"#,
    },
    IconSpec {
        icon: Icon::GraduationCap,
        name: "graduation-cap",
        geometry: r#"<path d="M22 10 12 5 2 10l10 5 10-5Z"/><path d="M6 12v5c3 3 9 3 12 0v-5"/>"#,
    },
    IconSpec {
        icon: Icon::BookOpen,
        name: "book-open",
        geometry: r#"<path d="M2 4h7a3 3 0 0 1 3 3v13a2 2 0 0 0-2-2H2z"/><path d="M22 4h-7a3 3 0 0 0-3 3v13a2 2 0 0 1 2-2h8z"/>"#,
    },
    IconSpec {
        icon: Icon::Sprout,
        name: "sprout",
        geometry: r#"<path d="M7 20h10"/><path d="M12 20v-9"/><path d="M12 11c0-4 3-7 7-7 0 4-3 7-7 7Z"/><path d="M12 13c0-4-3-6-7-6 0 4 3 6 7 6Z"/>"#,
    },
    IconSpec {
        icon: Icon::PawPrint,
        name: "paw-print",
        geometry: r#"<circle cx="11" cy="5" r="2"/><circle cx="18" cy="8" r="2"/><circle cx="20" cy="15" r="2"/><circle cx="5" cy="9" r="2"/><path d="M9 14c1.5-2 4.5-2 6 0 1.5 2 2 5-1 5H10c-3 0-2.5-3-1-5Z"/>"#,
    },
    IconSpec {
        icon: Icon::ShoppingBag,
        name: "shopping-bag",
        geometry: r#"<path d="M6 2 3 6v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2V6l-3-4Z"/><path d="M3 6h18"/><path d="M16 10a4 4 0 0 1-8 0"/>"#,
    },
    IconSpec {
        icon: Icon::Settings,
        name: "settings",
        geometry: r#"<circle cx="12" cy="12" r="3"/><path d="M12 2v3"/><path d="M12 19v3"/><path d="M4.9 4.9 7 7"/><path d="m17 17 2.1 2.1"/><path d="M2 12h3"/><path d="M19 12h3"/><path d="m4.9 19.1 2.1-2.1"/><path d="m17 7 2.1-2.1"/>"#,
    },
    IconSpec {
        icon: Icon::Target,
        name: "target",
        geometry: r#"<circle cx="12" cy="12" r="9"/><circle cx="12" cy="12" r="5"/><circle cx="12" cy="12" r="1.5"/>"#,
    },
    IconSpec {
        icon: Icon::Timer,
        name: "timer",
        geometry: r#"<path d="M10 2h4"/><path d="M12 14v-4"/><circle cx="12" cy="14" r="8"/>"#,
    },
    IconSpec {
        icon: Icon::Volume2,
        name: "volume-2",
        geometry: r#"<path d="M11 5 6 9H2v6h4l5 4V5z"/><path d="M15.5 8.5a5 5 0 0 1 0 7"/><path d="M19 5a10 10 0 0 1 0 14"/>"#,
    },
    IconSpec {
        icon: Icon::VolumeX,
        name: "volume-x",
        geometry: r#"<path d="M11 5 6 9H2v6h4l5 4V5z"/><path d="m22 9-6 6"/><path d="m16 9 6 6"/>"#,
    },
    IconSpec {
        icon: Icon::Play,
        name: "play",
        geometry: r#"<polygon points="6 3 20 12 6 21 6 3"/>"#,
    },
    IconSpec {
        icon: Icon::Pause,
        name: "pause",
        geometry: r#"<rect x="6" y="4" width="4" height="16" rx="1"/><rect x="14" y="4" width="4" height="16" rx="1"/>"#,
    },
    IconSpec {
        icon: Icon::RotateCcw,
        name: "rotate-ccw",
        geometry: r#"<path d="M3 12a9 9 0 1 0 3-6.7L3 8"/><path d="M3 3v5h5"/>"#,
    },
    IconSpec {
        icon: Icon::Trophy,
        name: "trophy",
        geometry: r#"<path d="M6 9H4a2 2 0 0 1-2-2V5h4"/><path d="M18 9h2a2 2 0 0 0 2-2V5h-4"/><path d="M8 21h8"/><path d="M12 17v4"/><path d="M7 4h10v6a5 5 0 0 1-10 0V4Z"/>"#,
    },
    IconSpec {
        icon: Icon::Award,
        name: "award",
        geometry: r#"<circle cx="12" cy="8" r="6"/><path d="M15.5 13.5 17 22l-5-3-5 3 1.5-8.5"/>"#,
    },
    IconSpec {
        icon: Icon::Flame,
        name: "flame",
        geometry: r#"<path d="M12 2c2 3 6 6 6 11a6 6 0 1 1-12 0c0-3 2-6 4-8 0 2 1 3 2 3 0-2 0-4 0-6Z"/>"#,
    },
    IconSpec {
        icon: Icon::Snowflake,
        name: "snowflake",
        geometry: r#"<path d="M12 2v20"/><path d="m4.9 6.5 14.2 11"/><path d="m19.1 6.5-14.2 11"/>"#,
    },
    IconSpec {
        icon: Icon::Sun,
        name: "sun",
        geometry: r#"<circle cx="12" cy="12" r="4"/><path d="M12 2v2"/><path d="M12 20v2"/><path d="m4.9 4.9 1.4 1.4"/><path d="m17.7 17.7 1.4 1.4"/><path d="M2 12h2"/><path d="M20 12h2"/><path d="m4.9 19.1 1.4-1.4"/><path d="m17.7 6.3 1.4-1.4"/>"#,
    },
    IconSpec {
        icon: Icon::Moon,
        name: "moon",
        geometry: r#"<path d="M21 12.8A9 9 0 1 1 11.2 3a7 7 0 0 0 9.8 9.8Z"/>"#,
    },
    IconSpec {
        icon: Icon::Palette,
        name: "palette",
        geometry: r#"<path d="M12 2a10 10 0 1 0 0 20 2 2 0 0 0 2-2c0-.5-.2-1-.5-1.3a2 2 0 0 1 1.5-3.3H17a5 5 0 0 0 5-5A8 8 0 0 0 12 2Z"/><circle cx="7.5" cy="10.5" r="1"/><circle cx="12" cy="7.5" r="1"/><circle cx="16.5" cy="10.5" r="1"/>"#,
    },
    IconSpec {
        icon: Icon::Plus,
        name: "plus",
        geometry: r#"<path d="M12 5v14"/><path d="M5 12h14"/>"#,
    },
    IconSpec {
        icon: Icon::X,
        name: "x",
        geometry: r#"<path d="M18 6 6 18"/><path d="m6 6 12 12"/>"#,
    },
    IconSpec {
        icon: Icon::Pencil,
        name: "pencil",
        geometry: r#"<path d="M17 3a2.8 2.8 0 1 1 4 4L7.5 20.5 2 22l1.5-5.5Z"/>"#,
    },
    IconSpec {
        icon: Icon::Trash2,
        name: "trash-2",
        geometry: r#"<path d="M3 6h18"/><path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6"/><path d="M8 6V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"/>"#,
    },
    IconSpec {
        icon: Icon::Archive,
        name: "archive",
        geometry: r#"<rect x="2" y="4" width="20" height="5" rx="1"/><path d="M4 9v9a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V9"/><path d="M10 13h4"/>"#,
    },
    IconSpec {
        icon: Icon::Tag,
        name: "tag",
        geometry: r#"<path d="M12 2H4a2 2 0 0 0-2 2v8l9.5 9.5a2 2 0 0 0 2.8 0l6.2-6.2a2 2 0 0 0 0-2.8L12 2Z"/><circle cx="7.5" cy="7.5" r="1"/>"#,
    },
    IconSpec {
        icon: Icon::Flag,
        name: "flag",
        geometry: r#"<path d="M4 21V4h12l-2 4 2 4H4"/>"#,
    },
    IconSpec {
        icon: Icon::Pin,
        name: "pin",
        geometry: r#"<path d="M12 17v5"/><path d="M9 4h6l-1 6 3 3v2H7v-2l3-3-1-6Z"/>"#,
    },
    IconSpec {
        icon: Icon::Star,
        name: "star",
        geometry: r#"<polygon points="12 2 15.1 8.3 22 9.3 17 14.1 18.2 21 12 17.8 5.8 21 7 14.1 2 9.3 8.9 8.3 12 2"/>"#,
    },
    IconSpec {
        icon: Icon::Heart,
        name: "heart",
        geometry: r#"<path d="M19 14c1.5-1.5 3-3.2 3-5.5A5.5 5.5 0 0 0 16.5 3c-1.8 0-3 .7-4.5 2-1.5-1.3-2.7-2-4.5-2A5.5 5.5 0 0 0 2 8.5c0 2.3 1.5 4 3 5.5l7 7Z"/>"#,
    },
    IconSpec {
        icon: Icon::Smile,
        name: "smile",
        geometry: r#"<circle cx="12" cy="12" r="9"/><path d="M8 14s1.5 2 4 2 4-2 4-2"/><path d="M9 9h.01"/><path d="M15 9h.01"/>"#,
    },
    IconSpec {
        icon: Icon::Meh,
        name: "meh",
        geometry: r#"<circle cx="12" cy="12" r="9"/><path d="M8 15h8"/><path d="M9 9h.01"/><path d="M15 9h.01"/>"#,
    },
    IconSpec {
        icon: Icon::CloudRain,
        name: "cloud-rain",
        geometry: r#"<path d="M4 14.9A7 7 0 1 1 15.7 8h1.8a4.5 4.5 0 0 1 2.5 8.2"/><path d="M8 19v2"/><path d="M12 17v4"/><path d="M16 19v2"/>"#,
    },
    IconSpec {
        icon: Icon::Zap,
        name: "zap",
        geometry: r#"<polygon points="13 2 3 14 12 14 11 22 21 10 12 10 13 2"/>"#,
    },
    IconSpec {
        icon: Icon::BarChart3,
        name: "bar-chart-3",
        geometry: r#"<path d="M3 3v18h18"/><path d="M18 17V9"/><path d="M13 17V5"/><path d="M8 17v-3"/>"#,
    },
    IconSpec {
        icon: Icon::PieChart,
        name: "pie-chart",
        geometry: r#"<path d="M21 12A9 9 0 1 1 12 3v9Z"/><path d="M21.2 10A9 9 0 0 0 14 2.8V10Z"/>"#,
    },
    IconSpec {
        icon: Icon::TrendingUp,
        name: "trending-up",
        geometry: r#"<polyline points="22 7 13.5 15.5 8.5 10.5 2 17"/><polyline points="16 7 22 7 22 13"/>"#,
    },
    IconSpec {
        icon: Icon::Layers,
        name: "layers",
        geometry: r#"<polygon points="12 2 2 7 12 12 22 7 12 2"/><polyline points="2 12 12 17 22 12"/><polyline points="2 17 12 22 22 17"/>"#,
    },
    IconSpec {
        icon: Icon::LayoutGrid,
        name: "layout-grid",
        geometry: r#"<rect width="7" height="7" x="3" y="3" rx="1"/><rect width="7" height="7" x="14" y="3" rx="1"/><rect width="7" height="7" x="14" y="14" rx="1"/><rect width="7" height="7" x="3" y="14" rx="1"/>"#,
    },
    IconSpec {
        icon: Icon::List,
        name: "list",
        geometry: r#"<path d="M8 6h13"/><path d="M8 12h13"/><path d="M8 18h13"/><path d="M3 6h.01"/><path d="M3 12h.01"/><path d="M3 18h.01"/>"#,
    },
    IconSpec {
        icon: Icon::Columns3,
        name: "columns-3",
        geometry: r#"<rect width="18" height="18" x="3" y="3" rx="2"/><path d="M9 3v18"/><path d="M15 3v18"/>"#,
    },
    IconSpec {
        icon: Icon::PanelLeft,
        name: "panel-left",
        geometry: r#"<rect width="18" height="18" x="3" y="3" rx="2"/><path d="M9 3v18"/>"#,
    },
    IconSpec {
        icon: Icon::ChevronRight,
        name: "chevron-right",
        geometry: r#"<path d="m9 18 6-6-6-6"/>"#,
    },
    IconSpec {
        icon: Icon::ChevronLeft,
        name: "chevron-left",
        geometry: r#"<path d="m15 18-6-6 6-6"/>"#,
    },
    IconSpec {
        icon: Icon::ChevronDown,
        name: "chevron-down",
        geometry: r#"<path d="m6 9 6 6 6-6"/>"#,
    },
    IconSpec {
        icon: Icon::ChevronUp,
        name: "chevron-up",
        geometry: r#"<path d="m18 15-6-6-6 6"/>"#,
    },
    IconSpec {
        icon: Icon::ArrowUpRight,
        name: "arrow-up-right",
        geometry: r#"<path d="M7 17 17 7"/><path d="M7 7h10v10"/>"#,
    },
    IconSpec {
        icon: Icon::AlertTriangle,
        name: "alert-triangle",
        geometry: r#"<path d="m12 3 9 16H3L12 3Z"/><path d="M12 9v4"/><path d="M12 17h.01"/>"#,
    },
    IconSpec {
        icon: Icon::Info,
        name: "info",
        geometry: r#"<circle cx="12" cy="12" r="9"/><path d="M12 16v-4"/><path d="M12 8h.01"/>"#,
    },
    IconSpec {
        icon: Icon::CheckCircle2,
        name: "check-circle-2",
        geometry: r#"<circle cx="12" cy="12" r="9"/><path d="m9 12 2 2 4-4"/>"#,
    },
    IconSpec {
        icon: Icon::XCircle,
        name: "x-circle",
        geometry: r#"<circle cx="12" cy="12" r="9"/><path d="m15 9-6 6"/><path d="m9 9 6 6"/>"#,
    },
    IconSpec {
        icon: Icon::GalleryVerticalEnd,
        name: "gallery-vertical-end",
        geometry: r#"<path d="M7 2h10"/><path d="M5 6h14"/><rect width="18" height="12" x="3" y="10" rx="2"/>"#,
    },
    IconSpec {
        icon: Icon::Brain,
        name: "brain",
        geometry: r#"<path d="M12 5a3 3 0 1 0-5.9-1A4 4 0 0 0 4 11a4 4 0 0 0 2 7 3 3 0 0 0 6 0V5Z"/><path d="M12 5a3 3 0 1 1 5.9-1A4 4 0 0 1 20 11a4 4 0 0 1-2 7 3 3 0 0 1-6 0V5Z"/>"#,
    },
    IconSpec {
        icon: Icon::BookX,
        name: "book-x",
        geometry: r#"<path d="M4 19.5v-15A2.5 2.5 0 0 1 6.5 2H20v20H6.5a2.5 2.5 0 0 1 0-5H20"/><path d="m14.5 7-5 5"/><path d="m9.5 7 5 5"/>"#,
    },
    IconSpec {
        icon: Icon::Quote,
        name: "quote",
        geometry: r#"<path d="M10 11H6V7h4v6a3 3 0 0 1-3 3"/><path d="M18 11h-4V7h4v6a3 3 0 0 1-3 3"/>"#,
    },
    IconSpec {
        icon: Icon::Bookmark,
        name: "bookmark",
        geometry: r#"<path d="m19 21-7-4-7 4V5a2 2 0 0 1 2-2h10a2 2 0 0 1 2 2v16Z"/>"#,
    },
    IconSpec {
        icon: Icon::Dices,
        name: "dices",
        geometry: r#"<rect width="12" height="12" x="2" y="10" rx="2"/><path d="m17.9 14 3.5-3.5a2 2 0 0 0 0-2.8l-5.1-5.1a2 2 0 0 0-2.8 0L10 6"/><path d="M6 14h.01"/><path d="M10 18h.01"/>"#,
    },
    IconSpec {
        icon: Icon::Filter,
        name: "filter",
        geometry: r#"<polygon points="22 3 2 3 10 12.5 10 19 14 21 14 12.5 22 3"/>"#,
    },
    IconSpec {
        icon: Icon::ArrowUpDown,
        name: "arrow-up-down",
        geometry: r#"<path d="m21 16-4 4-4-4"/><path d="M17 20V4"/><path d="m3 8 4-4 4 4"/><path d="M7 4v16"/>"#,
    },
    IconSpec {
        icon: Icon::Upload,
        name: "upload",
        geometry: r#"<path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/><polyline points="17 8 12 3 7 8"/><line x1="12" x2="12" y1="3" y2="15"/>"#,
    },
    IconSpec {
        icon: Icon::Download,
        name: "download",
        geometry: r#"<path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/><polyline points="7 10 12 15 17 10"/><line x1="12" x2="12" y1="15" y2="3"/>"#,
    },
    IconSpec {
        icon: Icon::Database,
        name: "database",
        geometry: r#"<ellipse cx="12" cy="5" rx="9" ry="3"/><path d="M3 5v14a9 3 0 0 0 18 0V5"/><path d="M3 12a9 3 0 0 0 18 0"/>"#,
    },
    IconSpec {
        icon: Icon::Cloud,
        name: "cloud",
        geometry: r#"<path d="M17.5 19H9a7 7 0 1 1 6.7-9h1.8a4.5 4.5 0 1 1 0 9Z"/>"#,
    },
    IconSpec {
        icon: Icon::Lock,
        name: "lock",
        geometry: r#"<rect width="18" height="11" x="3" y="11" rx="2"/><path d="M7 11V7a5 5 0 0 1 10 0v4"/>"#,
    },
    IconSpec {
        icon: Icon::ShieldCheck,
        name: "shield-check",
        geometry: r#"<path d="M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10Z"/><path d="m9 12 2 2 4-4"/>"#,
    },
    IconSpec {
        icon: Icon::Terminal,
        name: "terminal",
        geometry: r#"<polyline points="4 17 10 11 4 5"/><line x1="12" x2="20" y1="19" y2="19"/>"#,
    },
    IconSpec {
        icon: Icon::User,
        name: "user",
        geometry: r#"<circle cx="12" cy="8" r="4"/><path d="M4 20a8 8 0 0 1 16 0"/>"#,
    },
    IconSpec {
        icon: Icon::Bell,
        name: "bell",
        geometry: r#"<path d="M6 8a6 6 0 0 1 12 0c0 7 3 9 3 9H3s3-2 3-9"/><path d="M10.3 21a1.9 1.9 0 0 0 3.4 0"/>"#,
    },
    IconSpec {
        icon: Icon::Image,
        name: "image",
        geometry: r#"<rect width="18" height="18" x="3" y="3" rx="2"/><circle cx="9" cy="9" r="2"/><path d="m21 15-3.1-3.1a2 2 0 0 0-2.8 0L6 21"/>"#,
    },
    IconSpec {
        icon: Icon::Frame,
        name: "frame",
        geometry: r#"<line x1="22" x2="2" y1="6" y2="6"/><line x1="22" x2="2" y1="18" y2="18"/><line x1="6" x2="6" y1="2" y2="22"/><line x1="18" x2="18" y1="2" y2="22"/>"#,
    },
    IconSpec {
        icon: Icon::Coins,
        name: "coins",
        geometry: r#"<circle cx="8" cy="8" r="6"/><path d="M18.1 10.4A6 6 0 1 1 10.3 18"/><path d="M7 6h1v4"/>"#,
    },
    IconSpec {
        icon: Icon::TreePine,
        name: "tree-pine",
        geometry: r#"<path d="m12 2-5 6h3l-4 6h4l-4 5h12l-4-5h4l-4-6h3l-5-6Z"/><path d="M12 19v3"/>"#,
    },
    IconSpec {
        icon: Icon::Flower2,
        name: "flower-2",
        geometry: r#"<circle cx="12" cy="9" r="2"/><path d="M12 3a3 3 0 0 1 3 3c0 1.7-3 3-3 3s-3-1.3-3-3a3 3 0 0 1 3-3Z"/><path d="M12 21v-9"/>"#,
    },
    IconSpec {
        icon: Icon::Droplets,
        name: "droplets",
        geometry: r#"<path d="M7 16.3c2.2 0 4-1.8 4-4 0-3.3-4-6.3-4-6.3S3 9 3 12.3c0 2.2 1.8 4 4 4Z"/><path d="M17 20.3c2.2 0 4-1.8 4-4 0-3.3-4-6.3-4-6.3s-4 3-4 6.3c0 2.2 1.8 4 4 4Z"/>"#,
    },
    IconSpec {
        icon: Icon::Headphones,
        name: "headphones",
        geometry: r#"<path d="M3 18v-6a9 9 0 0 1 18 0v6"/><path d="M21 19a2 2 0 0 1-2 2h-1a2 2 0 0 1-2-2v-3a2 2 0 0 1 2-2h3Z"/><path d="M3 19a2 2 0 0 0 2 2h1a2 2 0 0 0 2-2v-3a2 2 0 0 0-2-2H3Z"/>"#,
    },
    IconSpec {
        icon: Icon::Coffee,
        name: "coffee",
        geometry: r#"<path d="M17 8h1a4 4 0 1 1 0 8h-1"/><path d="M3 8h14v9a4 4 0 0 1-4 4H7a4 4 0 0 1-4-4Z"/><line x1="6" x2="6" y1="2" y2="4"/><line x1="10" x2="10" y1="2" y2="4"/><line x1="14" x2="14" y1="2" y2="4"/>"#,
    },
    IconSpec {
        icon: Icon::Eye,
        name: "eye",
        geometry: r#"<path d="M2 12s3.6-7 10-7 10 7 10 7-3.6 7-10 7-10-7-10-7Z"/><circle cx="12" cy="12" r="3"/>"#,
    },
    IconSpec {
        icon: Icon::Code2,
        name: "code-2",
        geometry: r#"<path d="m18 16 4-4-4-4"/><path d="m6 8-4 4 4 4"/><path d="m14.5 4-5 16"/>"#,
    },
    IconSpec {
        icon: Icon::Repeat,
        name: "repeat",
        geometry: r#"<path d="m17 2 4 4-4 4"/><path d="M3 11V9a4 4 0 0 1 4-4h14"/><path d="m7 22-4-4 4-4"/><path d="M21 13v2a4 4 0 0 1-4 4H3"/>"#,
    },
    IconSpec {
        icon: Icon::HelpCircle,
        name: "help-circle",
        geometry: r#"<circle cx="12" cy="12" r="9"/><path d="M9.1 9a3 3 0 0 1 5.8 1c0 2-3 3-3 3"/><path d="M12 17h.01"/>"#,
    },
];

impl Icon {
    /// Lista completa dos 87 ícones suportados pelo eStudy.
    pub const ALL: [Self; 87] = [
        Self::Home,
        Self::Search,
        Self::Command,
        Self::FolderKanban,
        Self::CheckSquare,
        Self::Calendar,
        Self::Clock,
        Self::GraduationCap,
        Self::BookOpen,
        Self::Sprout,
        Self::PawPrint,
        Self::ShoppingBag,
        Self::Settings,
        Self::Target,
        Self::Timer,
        Self::Volume2,
        Self::VolumeX,
        Self::Play,
        Self::Pause,
        Self::RotateCcw,
        Self::Trophy,
        Self::Award,
        Self::Flame,
        Self::Snowflake,
        Self::Sun,
        Self::Moon,
        Self::Palette,
        Self::Plus,
        Self::X,
        Self::Pencil,
        Self::Trash2,
        Self::Archive,
        Self::Tag,
        Self::Flag,
        Self::Pin,
        Self::Star,
        Self::Heart,
        Self::Smile,
        Self::Meh,
        Self::CloudRain,
        Self::Zap,
        Self::BarChart3,
        Self::PieChart,
        Self::TrendingUp,
        Self::Layers,
        Self::LayoutGrid,
        Self::List,
        Self::Columns3,
        Self::PanelLeft,
        Self::ChevronRight,
        Self::ChevronLeft,
        Self::ChevronDown,
        Self::ChevronUp,
        Self::ArrowUpRight,
        Self::AlertTriangle,
        Self::Info,
        Self::CheckCircle2,
        Self::XCircle,
        Self::GalleryVerticalEnd,
        Self::Brain,
        Self::BookX,
        Self::Quote,
        Self::Bookmark,
        Self::Dices,
        Self::Filter,
        Self::ArrowUpDown,
        Self::Upload,
        Self::Download,
        Self::Database,
        Self::Cloud,
        Self::Lock,
        Self::ShieldCheck,
        Self::Terminal,
        Self::User,
        Self::Bell,
        Self::Image,
        Self::Frame,
        Self::Coins,
        Self::TreePine,
        Self::Flower2,
        Self::Droplets,
        Self::Headphones,
        Self::Coffee,
        Self::Eye,
        Self::Code2,
        Self::Repeat,
        Self::HelpCircle,
    ];

    /// Identificador único em kebab-case do ícone.
    #[must_use]
    pub fn name(self) -> &'static str {
        self.spec().name
    }

    /// Gera o documento SVG completo colorido com a cor semântica fornecida.
    #[must_use]
    pub fn to_svg_string(self, color: Color) -> String {
        let hex = color_to_hex(color);
        let geometry = self.spec().geometry;
        format!(
            r#"<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="{hex}" stroke-width="1.75" stroke-linecap="round" stroke-linejoin="round">{geometry}</svg>"#
        )
    }

    /// Retorna o handle SVG pré-carregado em memória estática (zero alocação por frame).
    #[must_use]
    pub fn cached_handle(self) -> svg::Handle {
        let idx = Self::ALL
            .iter()
            .position(|candidate| *candidate == self)
            .unwrap_or(0);
        CACHED_SVG_HANDLES[idx].clone()
    }

    fn spec(self) -> &'static IconSpec {
        ICON_SPECS
            .iter()
            .find(|spec| spec.icon == self)
            .unwrap_or(&ICON_SPECS[0])
    }
}

/// Ícones vetoriais ilustrados multi-cor (sem filtro monocromático) para módulos, turnos, prioridades e mascote.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum IllustratedIcon {
    /// Casinha acolhedora (Início).
    Home,
    /// Prancheta/caderno de tarefas (Tarefas).
    Tasks,
    /// Chama viva orgânica (Hábitos).
    Habits,
    /// Caderno de ideias e lâmpada quente (Segundo Cérebro).
    SecondBrain,
    /// Folha de calendário com topo colorido (Agenda).
    Calendar,
    /// Chapéu/livro de estudos (Estudos).
    Studies,
    /// Pilha de livros artesanal (Estante).
    Bookshelf,
    /// Engrenagem em latão/oliva (Ajustes).
    Settings,
    /// Relógio suave (Período: Qualquer hora).
    TurnAnytime,
    /// Sol dourado matinal (Período: Manhã).
    TurnMorning,
    /// Sol quente vespertino (Período: Tarde).
    TurnAfternoon,
    /// Lua crescente serena (Período: Noite).
    TurnEvening,
    /// Semente/broto discreto (Prioridade Baixa).
    PriorityLow,
    /// Folha equilibrada (Prioridade Média).
    PriorityMedium,
    /// Pinha/chama terracota (Prioridade Alta).
    PriorityHigh,
    /// Alerta coral intenso (Prioridade Urgente).
    PriorityUrgent,
    /// Medalhão ilustrado do gatinho tricolor Mochi.
    MascotMochi,
}

static CACHED_ILLUSTRATED_HANDLES: LazyLock<Vec<svg::Handle>> = LazyLock::new(|| {
    IllustratedIcon::ALL
        .iter()
        .map(|icon| svg::Handle::from_memory(icon.to_svg_string().into_bytes()))
        .collect()
});

struct IllustratedSpec {
    icon: IllustratedIcon,
    name: &'static str,
    view_box: &'static str,
    body: &'static str,
}

const ILLUSTRATED_SPECS: &[IllustratedSpec] = &[
    IllustratedSpec {
        icon: IllustratedIcon::Home,
        name: "illus-home",
        view_box: "0 0 24 24",
        body: r##"<path d="M4.5 10.5L12 4l7.5 6.5V19a1.5 1.5 0 0 1-1.5 1.5H6A1.5 1.5 0 0 1 4.5 19v-8.5Z" fill="#FAF4E8" stroke="#2B241E" stroke-width="1.5" stroke-linejoin="round"/><path d="M2.75 11.25L12 3.25l9.25 8-1.5 1.5L12 6.1 4.25 12.75l-1.5-1.5Z" fill="#C85A36" stroke="#2B241E" stroke-width="1.4" stroke-linejoin="round"/><rect x="9.75" y="13.5" width="4.5" height="7" rx="1" fill="#4E6B52" stroke="#2B241E" stroke-width="1.4"/><circle cx="12" cy="9.75" r="1.5" fill="#F2B84B" stroke="#2B241E" stroke-width="1.2"/>"##,
    },
    IllustratedSpec {
        icon: IllustratedIcon::Tasks,
        name: "illus-tasks",
        view_box: "0 0 24 24",
        body: r##"<rect x="4.5" y="4.5" width="15" height="16.5" rx="2.5" fill="#E3A865" stroke="#2B241E" stroke-width="1.5"/><rect x="6.5" y="6.5" width="11" height="12.5" rx="1.2" fill="#FCF9F2" stroke="#2B241E" stroke-width="1.2"/><rect x="9" y="3" width="6" height="3" rx="1.2" fill="#5D7560" stroke="#2B241E" stroke-width="1.3"/><path d="M8.5 10.5l1.3 1.3 2.4-2.4" fill="none" stroke="#374C3B" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round"/><line x1="13.5" y1="10.5" x2="15.8" y2="10.5" stroke="#8C7E70" stroke-width="1.5" stroke-linecap="round"/><circle cx="9.5" cy="15" r="1.1" fill="#C85A36"/><line x1="12.2" y1="15" x2="15.8" y2="15" stroke="#8C7E70" stroke-width="1.5" stroke-linecap="round"/>"##,
    },
    IllustratedSpec {
        icon: IllustratedIcon::Habits,
        name: "illus-habits",
        view_box: "0 0 24 24",
        body: r##"<path d="M12.3 2.8c1.9 3 5.7 5.7 5.7 10.6a6 6 0 1 1-12 0c0-3 1.8-5.7 3.8-7.6.1 1.7 1 2.7 2 2.8-.2-1.8-.1-3.9.5-5.8Z" fill="#D9623B" stroke="#2B241E" stroke-width="1.5" stroke-linejoin="round"/><path d="M12.2 10.2c1.2 1.6 2.8 3.1 2.8 5.2a3 3 0 0 1-6 0c0-1.6 1-3 2-4 .1.8.6 1.3 1.1 1.3 0-.8 0-1.7.1-2.5Z" fill="#F6C445" stroke="#2B241E" stroke-width="1.2" stroke-linejoin="round"/>"##,
    },
    IllustratedSpec {
        icon: IllustratedIcon::SecondBrain,
        name: "illus-second-brain",
        view_box: "0 0 24 24",
        body: r##"<path d="M9 17.5h6v2a1.5 1.5 0 0 1-1.5 1.5h-3A1.5 1.5 0 0 1 9 19.5v-2Z" fill="#7D8C80" stroke="#2B241E" stroke-width="1.4"/><path d="M12 3.5a6.2 6.2 0 0 1 4.1 10.8c-.8.7-1.3 1.7-1.4 2.7H9.3c-.1-1-.6-2-1.4-2.7A6.2 6.2 0 0 1 12 3.5Z" fill="#F7CE5B" stroke="#2B241E" stroke-width="1.5" stroke-linejoin="round"/><circle cx="10.2" cy="8.3" r="1.4" fill="#FFF7D6"/><path d="M11 17v-3.5m2 3.5v-3.5" stroke="#2B241E" stroke-width="1.3" stroke-linecap="round"/>"##,
    },
    IllustratedSpec {
        icon: IllustratedIcon::Calendar,
        name: "illus-calendar",
        view_box: "0 0 24 24",
        body: r##"<rect x="3.5" y="4.5" width="17" height="16" rx="2.5" fill="#FCF9F2" stroke="#2B241E" stroke-width="1.5"/><path d="M3.5 7A2.5 2.5 0 0 1 6 4.5h12A2.5 2.5 0 0 1 20.5 7v2.5h-17V7Z" fill="#C85A36" stroke="#2B241E" stroke-width="1.5"/><line x1="8" y1="2.8" x2="8" y2="6" stroke="#2B241E" stroke-width="1.6" stroke-linecap="round"/><line x1="16" y1="2.8" x2="16" y2="6" stroke="#2B241E" stroke-width="1.6" stroke-linecap="round"/><circle cx="8.5" cy="13.5" r="1.3" fill="#4E6B52"/><circle cx="12" cy="13.5" r="1.3" fill="#E0A642"/><circle cx="15.5" cy="16.8" r="1.5" fill="#C85A36"/>"##,
    },
    IllustratedSpec {
        icon: IllustratedIcon::Studies,
        name: "illus-studies",
        view_box: "0 0 24 24",
        body: r##"<path d="M3 6.5A2.5 2.5 0 0 1 5.5 4H11a2 2 0 0 1 1 .3A2 2 0 0 1 13 4h5.5A2.5 2.5 0 0 1 21 6.5v12a1 1 0 0 1-1 1h-6a2 2 0 0 0-2 1 2 2 0 0 0-2-1H4a1 1 0 0 1-1-1v-12Z" fill="#FCF9F2" stroke="#2B241E" stroke-width="1.5" stroke-linejoin="round"/><line x1="12" y1="4.5" x2="12" y2="19.8" stroke="#2B241E" stroke-width="1.4"/><path d="M15.5 4v7.5l1.8-1.3 1.7 1.3V4h-3.5Z" fill="#C85A36" stroke="#2B241E" stroke-width="1.2" stroke-linejoin="round"/><line x1="6" y1="9" x2="9.5" y2="9" stroke="#4E6B52" stroke-width="1.5" stroke-linecap="round"/><line x1="6" y1="12.5" x2="9.5" y2="12.5" stroke="#8C7E70" stroke-width="1.5" stroke-linecap="round"/>"##,
    },
    IllustratedSpec {
        icon: IllustratedIcon::Bookshelf,
        name: "illus-bookshelf",
        view_box: "0 0 24 24",
        body: r##"<rect x="4" y="5" width="4.2" height="14.5" rx="1" fill="#4E6B52" stroke="#2B241E" stroke-width="1.4"/><rect x="8.8" y="6.5" width="4.2" height="13" rx="1" fill="#E0A642" stroke="#2B241E" stroke-width="1.4"/><path d="M14.2 6.2l3.9-1 2.4 13.4-3.9 1-2.4-13.4Z" fill="#C85A36" stroke="#2B241E" stroke-width="1.4" stroke-linejoin="round"/><line x1="2.8" y1="20" x2="21.2" y2="20" stroke="#2B241E" stroke-width="1.6" stroke-linecap="round"/>"##,
    },
    IllustratedSpec {
        icon: IllustratedIcon::Settings,
        name: "illus-settings",
        view_box: "0 0 24 24",
        body: r##"<circle cx="12" cy="12" r="7.5" fill="#E3DAC8" stroke="#2B241E" stroke-width="1.5"/><circle cx="12" cy="12" r="3.2" fill="#4E6B52" stroke="#2B241E" stroke-width="1.4"/><path d="M12 2.5v2.2M12 19.3v2.2M2.5 12h2.2M19.3 12h2.2M5.3 5.3l1.6 1.6M17.1 17.1l1.6 1.6M18.7 5.3l-1.6 1.6M6.9 17.1l-1.6 1.6" stroke="#2B241E" stroke-width="1.7" stroke-linecap="round"/>"##,
    },
    IllustratedSpec {
        icon: IllustratedIcon::TurnAnytime,
        name: "illus-turn-anytime",
        view_box: "0 0 24 24",
        body: r##"<circle cx="12" cy="12.5" r="8" fill="#DCE5DA" stroke="#374C3B" stroke-width="1.6"/><circle cx="12" cy="12.5" r="5.2" fill="#FAF7F0"/><path d="M12 8.8v3.9l2.6 1.6" fill="none" stroke="#374C3B" stroke-width="1.7" stroke-linecap="round" stroke-linejoin="round"/><line x1="9.8" y1="2.8" x2="14.2" y2="2.8" stroke="#C85A36" stroke-width="1.8" stroke-linecap="round"/>"##,
    },
    IllustratedSpec {
        icon: IllustratedIcon::TurnMorning,
        name: "illus-turn-morning",
        view_box: "0 0 24 24",
        body: r##"<circle cx="12" cy="12" r="5" fill="#F6C445" stroke="#8C5811" stroke-width="1.5"/><circle cx="10.8" cy="10.8" r="1.6" fill="#FFF3C4"/><path d="M12 3v2.2M12 18.8V21M3 12h2.2M18.8 12H21M5.6 5.6l1.6 1.6M16.8 16.8l1.6 1.6M18.4 5.6l-1.6 1.6M7.2 16.8l-1.6 1.6" stroke="#D98A2B" stroke-width="1.7" stroke-linecap="round"/>"##,
    },
    IllustratedSpec {
        icon: IllustratedIcon::TurnAfternoon,
        name: "illus-turn-afternoon",
        view_box: "0 0 24 24",
        body: r##"<circle cx="12" cy="12" r="5.2" fill="#DF6D3B" stroke="#7D2D14" stroke-width="1.5"/><circle cx="10.8" cy="10.8" r="1.8" fill="#F9B872"/><path d="M12 2.8v2.2M12 19v2.2M2.8 12H5M19 12h2.2M5.5 5.5l1.6 1.6M16.9 16.9l1.6 1.6M18.5 5.5l-1.6 1.6M7.1 16.9l-1.6 1.6" stroke="#C85A36" stroke-width="1.7" stroke-linecap="round"/>"##,
    },
    IllustratedSpec {
        icon: IllustratedIcon::TurnEvening,
        name: "illus-turn-evening",
        view_box: "0 0 24 24",
        body: r##"<path d="M19.5 14.2A7.8 7.8 0 0 1 9.8 4.5a7.8 7.8 0 1 0 9.7 9.7Z" fill="#7D8CE0" stroke="#323B75" stroke-width="1.5" stroke-linejoin="round"/><circle cx="17.2" cy="6.5" r="1.4" fill="#F5C84C"/><circle cx="14.2" cy="9.5" r="0.9" fill="#E3E7FF"/>"##,
    },
    IllustratedSpec {
        icon: IllustratedIcon::PriorityLow,
        name: "illus-priority-low",
        view_box: "0 0 24 24",
        body: r##"<ellipse cx="12" cy="14" rx="5" ry="6" fill="#9BBAA0" stroke="#2F4734" stroke-width="1.5"/><path d="M8.5 11.5h7M8.5 15h7" stroke="#2F4734" stroke-width="1.2" stroke-linecap="round"/><path d="M12 4.5v3.5" stroke="#2F4734" stroke-width="1.6" stroke-linecap="round"/><circle cx="12" cy="8.5" r="2" fill="#587A5F"/>"##,
    },
    IllustratedSpec {
        icon: IllustratedIcon::PriorityMedium,
        name: "illus-priority-medium",
        view_box: "0 0 24 24",
        body: r##"<ellipse cx="12" cy="14" rx="5.3" ry="6.2" fill="#E8B349" stroke="#6E470B" stroke-width="1.5"/><path d="M8.2 11.5h7.6M8.2 15h7.6" stroke="#6E470B" stroke-width="1.2" stroke-linecap="round"/><path d="M12 4.2v3.6" stroke="#6E470B" stroke-width="1.6" stroke-linecap="round"/><circle cx="12" cy="8.2" r="2.1" fill="#B87D1E"/>"##,
    },
    IllustratedSpec {
        icon: IllustratedIcon::PriorityHigh,
        name: "illus-priority-high",
        view_box: "0 0 24 24",
        body: r##"<ellipse cx="12" cy="14.2" rx="5.5" ry="6.3" fill="#D8633B" stroke="#66220E" stroke-width="1.5"/><path d="M8 11.5h8M8 15h8M9.5 18.2h5" stroke="#66220E" stroke-width="1.25" stroke-linecap="round"/><path d="M12 3.8v3.8" stroke="#66220E" stroke-width="1.7" stroke-linecap="round"/><circle cx="12" cy="8" r="2.2" fill="#9E3919"/>"##,
    },
    IllustratedSpec {
        icon: IllustratedIcon::PriorityUrgent,
        name: "illus-priority-urgent",
        view_box: "0 0 24 24",
        body: r##"<path d="M12 3l8.2 15.2a1.4 1.4 0 0 1-1.2 2.1H5a1.4 1.4 0 0 1-1.2-2.1L12 3Z" fill="#E04F43" stroke="#5C1510" stroke-width="1.5" stroke-linejoin="round"/><line x1="12" y1="9" x2="12" y2="14" stroke="#FFF8F5" stroke-width="2" stroke-linecap="round"/><circle cx="12" cy="17.2" r="1.2" fill="#FBE166"/>"##,
    },
    IllustratedSpec {
        icon: IllustratedIcon::MascotMochi,
        name: "illus-mascot-mochi",
        view_box: "0 0 36 36",
        body: r##"<circle cx="18" cy="18" r="16.5" fill="#EFE8DA" stroke="#2B241E" stroke-width="1.4"/><path d="M9.5 16L8 8.2l6.5 3.6c2.2-.6 4.8-.6 7 0L28 8.2 26.5 16c1.5 2.2 2 5 1.2 7.5-1.2 3.5-5.1 5.5-9.7 5.5s-8.5-2-9.7-5.5c-.8-2.5-.3-5.3 1.2-7.5Z" fill="#FCF9F3" stroke="#2B241E" stroke-width="1.4" stroke-linejoin="round"/><path d="M8 8.2l6.2 3.5c-1.8 1.2-3.4 3-4.2 5.2L8 8.2Z" fill="#D9723E"/><path d="M28 8.2l-6.2 3.5c1.9 1.1 3.3 2.8 4.1 5L28 8.2Z" fill="#3B332C"/><circle cx="14.2" cy="18.8" r="1.35" fill="#241E1A"/><circle cx="21.8" cy="18.8" r="1.35" fill="#241E1A"/><ellipse cx="11.6" cy="21" rx="1.6" ry="0.95" fill="#F2A7A0"/><ellipse cx="24.4" cy="21" rx="1.6" ry="0.95" fill="#F2A7A0"/><path d="M16.5 21.3c.8.9 2.2.9 3 0" fill="none" stroke="#241E1A" stroke-width="1.3" stroke-linecap="round"/><path d="M11 26.8c2.2 1.5 11.8 1.5 14 0l-1.2 3.2c-3.4 1.2-8.2 1.2-11.6 0L11 26.8Z" fill="#4E6B52" stroke="#2B241E" stroke-width="1.2" stroke-linejoin="round"/>"##,
    },
];

impl IllustratedIcon {
    /// Lista completa dos 17 ícones ilustrados multi-cor.
    pub const ALL: [Self; 17] = [
        Self::Home,
        Self::Tasks,
        Self::Habits,
        Self::SecondBrain,
        Self::Calendar,
        Self::Studies,
        Self::Bookshelf,
        Self::Settings,
        Self::TurnAnytime,
        Self::TurnMorning,
        Self::TurnAfternoon,
        Self::TurnEvening,
        Self::PriorityLow,
        Self::PriorityMedium,
        Self::PriorityHigh,
        Self::PriorityUrgent,
        Self::MascotMochi,
    ];

    /// Identificador único em kebab-case.
    #[must_use]
    pub fn name(self) -> &'static str {
        self.spec().name
    }

    /// Documento SVG colorido completo preservando preenchimentos e traços próprios.
    #[must_use]
    pub fn to_svg_string(self) -> String {
        let spec = self.spec();
        format!(
            r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="{vb}">{body}</svg>"#,
            vb = spec.view_box,
            body = spec.body
        )
    }

    /// Handle SVG pré-carregado em cache estático (zero alocação por frame).
    #[must_use]
    pub fn cached_handle(self) -> svg::Handle {
        let idx = Self::ALL
            .iter()
            .position(|candidate| *candidate == self)
            .unwrap_or(0);
        CACHED_ILLUSTRATED_HANDLES[idx].clone()
    }

    fn spec(self) -> &'static IllustratedSpec {
        ILLUSTRATED_SPECS
            .iter()
            .find(|spec| spec.icon == self)
            .unwrap_or(&ILLUSTRATED_SPECS[0])
    }
}

fn color_to_hex(color: Color) -> String {
    let to_u8 = |channel: f32| -> u8 {
        let clamped = channel.clamp(0.0, 1.0) * 255.0;
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        {
            clamped.round() as u8
        }
    };
    format!(
        "#{:02x}{:02x}{:02x}",
        to_u8(color.r),
        to_u8(color.g),
        to_u8(color.b)
    )
}
