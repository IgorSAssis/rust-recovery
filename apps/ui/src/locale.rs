/// Supported locales for the UI.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Locale {
    PtBr,
    En,
}

impl Locale {
    pub const ALL: &'static [Self] = &[Self::PtBr, Self::En];

    /// Detects the system locale from environment variables and maps it to a
    /// supported [`Locale`]. Falls back to [`Locale::En`] when the system locale
    /// cannot be determined or is not supported.
    ///
    /// Reads `LANG`, `LC_ALL`, and `LANGUAGE` in order of precedence, covering
    /// Linux, macOS, and Windows terminal environments.
    pub fn detect() -> Self {
        let lang = std::env::var("LANG")
            .or_else(|_| std::env::var("LC_ALL"))
            .or_else(|_| std::env::var("LANGUAGE"))
            .unwrap_or_default()
            .to_lowercase();

        if lang.starts_with("pt") {
            Locale::PtBr
        } else {
            Locale::En
        }
    }
}

impl std::fmt::Display for Locale {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Locale::PtBr => write!(f, "Português (Brasil)"),
            Locale::En => write!(f, "Inglês"),
        }
    }
}

/// All user-facing strings for a single locale.
pub struct Strings {
    locale: Locale,

    // header
    pub app_subtitle: &'static str,

    // navigation
    pub nav_devices: &'static str,
    pub nav_scan: &'static str,
    pub nav_recover: &'static str,

    // devices
    pub detect_btn: &'static str,
    pub detecting: &'static str,
    pub devices_hint: &'static str,

    // scan
    pub source_label: &'static str,
    pub source_placeholder: &'static str,
    pub strategy_label: &'static str,
    pub strategy_carver: &'static str,
    pub strategy_fat32: &'static str,
    pub scan_btn: &'static str,
    pub scanning_btn: &'static str,
    pub scanning_status: &'static str,
    pub scan_error_no_source: &'static str,

    // results
    pub deselect_all: &'static str,
    pub select_all: &'static str,
    pub export_picking: &'static str,
    pub exporting: &'static str,
    pub no_files_found: &'static str,
    pub select_to_preview: &'static str,

    // worker
    pub export_folder_title: &'static str,
}

impl Strings {
    pub fn files_found(&self, count: usize, strategy: &str) -> String {
        match self.locale {
            Locale::PtBr => format!("Foram encontrados {} arquivo(s) via {}.", count, strategy),
            Locale::En => format!("Found {} file(s) via {}.", count, strategy),
        }
    }

    pub fn export_selected(&self, count: usize) -> String {
        match self.locale {
            Locale::PtBr => format!("Exportar Selecionados ({})", count),
            Locale::En => format!("Export Selected ({})", count),
        }
    }

    pub fn export_success(&self, count: usize) -> String {
        match self.locale {
            Locale::PtBr => format!("✓ {} arquivo(s) exportado(s) com sucesso.", count),
            Locale::En => format!("✓ {} file(s) exported successfully.", count),
        }
    }

    pub fn export_error(&self, err: &str) -> String {
        format!("✗ {}", err)
    }
}

pub const PT_BR: Strings = Strings {
    locale: Locale::PtBr,

    app_subtitle: "Ferramenta de Recuperação de Arquivos",

    nav_devices: "Dispositivos",
    nav_scan: "Escanear",
    nav_recover: "Recuperar",

    detect_btn: "Detectar Dispositivos",
    detecting: "Detectando dispositivos…",
    devices_hint: "Clique em 'Detectar Dispositivos' para listar os dispositivos disponíveis.",

    source_label: "Caminho da fonte:",
    source_placeholder: "/dev/sdb1 ou /caminho/para/imagem.img",
    strategy_label: "Estratégia de recuperação:",
    strategy_carver: "Carver — baseado em assinatura (qualquer dispositivo)",
    strategy_fat32: "FAT32 — reconhece o sistema de arquivos (preserva nomes de arquivos)",
    scan_btn: "Escanear",
    scanning_btn: "Escaneando…",
    scanning_status: "Escaneando… isso pode levar um tempo.",
    scan_error_no_source: "Informe o caminho da fonte antes de escanear.",

    deselect_all: "Desmarcar Todos",
    select_all: "Selecionar Todos",
    export_picking: "Abrindo seletor de pasta…",
    exporting: "Exportando…",
    no_files_found: "Nenhum arquivo recuperável foi encontrado.",
    select_to_preview: "Selecione um arquivo para visualizar",

    export_folder_title: "Selecionar pasta de saída para os arquivos recuperados",
};

pub const EN: Strings = Strings {
    locale: Locale::En,

    app_subtitle: "File Recovery Tool",

    nav_devices: "Devices",
    nav_scan: "Scan",
    nav_recover: "Recover",

    detect_btn: "Detect Devices",
    detecting: "Detecting devices…",
    devices_hint: "Click 'Detect Devices' to list available devices.",

    source_label: "Source path:",
    source_placeholder: "/dev/sdb1  or  /path/to/image.img",
    strategy_label: "Recovery strategy:",
    strategy_carver: "Carver — signature-based (any device)",
    strategy_fat32: "FAT32 — filesystem-aware (preserves filenames)",
    scan_btn: "Scan",
    scanning_btn: "Scanning…",
    scanning_status: "Scanning… this may take a while.",
    scan_error_no_source: "Enter a source path before scanning.",

    deselect_all: "Deselect All",
    select_all: "Select All",
    export_picking: "Opening folder picker…",
    exporting: "Exporting…",
    no_files_found: "No recoverable files were found.",
    select_to_preview: "Select a file to preview",

    export_folder_title: "Select output folder for recovered files",
};
