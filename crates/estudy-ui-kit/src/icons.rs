//! Catálogo único de 87 ícones vetoriais SVG (estilo Lucide, zero emojis).

use iced::Color;

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
            r#"<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="{hex}" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">{geometry}</svg>"#
        )
    }

    fn spec(self) -> &'static IconSpec {
        ICON_SPECS
            .iter()
            .find(|spec| spec.icon == self)
            .unwrap_or(&ICON_SPECS[0])
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
