use serde::{ser::SerializeMap, Deserialize, Deserializer, Serialize, Serializer};

use crate::{config::term_config, buffer::Mode, gutter::{GutterComponents, LineNumberMode}, graphics::CursorKind};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case", default, deny_unknown_fields)]
pub struct ViewConfig {
    pub mouse: MouseConfig,
    pub document: DocumentConfig,
    pub lsp: LspConfig,
    pub bufferline: BufferLineConfig,
    pub gutter: GutterConfig,
    pub statusline: StatusLineConfig,
    pub search: SearchConfig,
    pub keymap_suggestions: bool,
    #[serde(default)]
    pub shell: Vec<String>,
    // TODO: should not be in helix-view, in helix-term if anything
    pub true_color: bool,
    pub terminal: Option<term_config::TerminalConfig>,
}

impl Default for ViewConfig {
    fn default() -> Self {
        Self {
            keymap_suggestions: true,
            mouse: MouseConfig {
                active: true,
                middle_click_paste: true
            },
            document: DocumentConfig {
                scroll_config: ScrollConfig {
                    scroll_cursor_to_edge_padding: 5,
                    lines_per_scroll: 3,
                },
                save: SaveConfig {
                    auto_format_on_save: true,
                    auto_save_on_focus_lost: false,
                },
                insert_mode_config: InsertModeConfig {
                    auto_pair: helix_core::syntax::AutoPairConfig::default(),
                    auto_complete: AutoCompleteConfig {
                        auto_completion_suggestion: true,
                        min_len_before_auto_completion_suggestion: 2,
                        min_idle_ms_before_auto_completion_suggestion: 400,
                    }
                },
                whitespace: WhitespaceConfig {
                    render: WhitespaceRender::Basic(WhitespaceRenderValue::None),
                    characters: WhitespaceCharacters {
                        space: '·',    // U+00B7
                        nbsp: '⍽',    // U+237D
                        tab: '→',     // U+2192
                        newline: '⏎', // U+23CE
                        tabpad: ' ',
                    },
                },
                indent_guides: IndentGuidesConfig {
                    skip_levels: 0,
                    render: false,
                    character: '│',
                },
                cursor_config: CursorConfig {
                    cursor_shape: [CursorKind::Block; 3],
                    cursorline: false,
                    cursorcolumn: false,
                },
                column_rulers: Vec::new()
            },
            bufferline: BufferLineConfig::Never,
            gutter: GutterConfig {
                gutter_components: vec![
                    GutterComponents::DiagnosticsAndBreakpoints,
                    GutterComponents::Spacer,
                    GutterComponents::LineNumbers,
                    GutterComponents::Spacer,
                    GutterComponents::VcsDiff,
                ],
                line_number_mode: LineNumberMode::Absolute,
            },
            search: SearchConfig {
                smart_case: true,
                wrap_around: true,
                global: SearchPickerConfig {
                    ignore_hidden: true,
                    follow_symlinks: true,
                    ingore_file_from_ancestor: true,
                    use_dot_ignore: true,
                    use_git_ignore: true,
                    use_git_global: true,
                    use_git_exclude: true,
                    max_depth: None,
                },
                file: SearchPickerConfig {
                    ignore_hidden: true,
                    follow_symlinks: true,
                    ingore_file_from_ancestor: true,
                    use_dot_ignore: true,
                    use_git_ignore: true,
                    use_git_global: true,
                    use_git_exclude: true,
                    max_depth: None,
                }
            },
            statusline: StatusLineConfig {
                left: vec![
                    StatusLineElement::Mode,
                    StatusLineElement::LspSpinner,
                    StatusLineElement::FileName
                ],
                center: vec![],
                right: vec![
                    StatusLineElement::Diagnostics,
                    StatusLineElement::SelectionsCount,
                    StatusLineElement::CursorPosition,
                    StatusLineElement::FileEncoding
                ],
                separator: String::from("│"),
                mode_string: ModeString {
                    normal: String::from("NOR"),
                    insert: String::from("INS"),
                    select: String::from("SEL"),
                },
                color_modes: false
            },
            lsp: LspConfig {
                cmd_row_messages: false,
                signature_help_auto_popup: true,
                display_docs_in_signature_help: true,
            },
            shell: if cfg!(windows) { vec!["cmd".to_owned(), "/C".to_owned()] } 
                   else { vec!["sh".to_owned(), "-c".to_owned()] },
            terminal: term_config::get_terminal_provider(),
            true_color: false,
        }
    }
}

pub struct MouseConfig {
    pub active: bool,
    // NOTE: does currently not being ovveridden by mouse = false
    pub middle_click_paste: bool
}

pub struct DocumentConfig {
    pub scroll_config: ScrollConfig,
    pub save: SaveConfig,
    pub insert_mode_config: InsertModeConfig,
    #[serde(default)]
    pub whitespace: WhitespaceConfig,
    pub indent_guides: IndentGuidesConfig,
    pub cursor_config: CursorConfig,
    pub column_rulers: Vec<u16>,
}

pub struct ScrollConfig {
    pub scroll_cursor_to_edge_padding: usize,
    pub lines_per_scroll: isize,
}

pub struct SaveConfig {
    pub auto_format_on_save: bool,
    pub auto_save_on_focus_lost: bool,
}

pub struct InsertModeConfig {
    pub auto_pair: helix_core::syntax::AutoPairConfig,
    pub auto_complete: AutoCompleteConfig
}

pub struct AutoCompleteConfig {
    pub auto_completion_suggestion: bool,
    pub min_len_before_auto_completion_suggestion: u8,
    pub min_idle_ms_before_auto_completion_suggestion: u64,
}

pub struct CursorConfig {
    pub cursor_shape: CursorShapeConfig,
    pub cursorline: bool,
    pub cursorcolumn: bool,
}

pub struct GutterConfig {
        pub gutter_components: Vec<GutterComponents>,
        pub line_number_mode: LineNumberMode,
}

pub struct SearchPickerConfig {
    pub ignore_hidden: bool,
    pub follow_symlinks: bool,
    pub ingore_file_from_ancestor: bool,
    pub use_dot_ignore: bool,
    pub use_git_ignore: bool,
    pub use_git_global: bool,
    pub use_git_exclude: bool,
    pub max_depth: Option<usize>,
}

pub struct LspConfig {
    pub cmd_row_messages: bool,
    pub signature_help_auto_popup: bool,
    pub display_docs_in_signature_help: bool,
}

pub struct SearchConfig {
    pub smart_case: bool,
    pub wrap_around: bool,
    pub global: SearchPickerConfig,
    pub file: SearchPickerConfig
}

// TODO: color_remove now that themes can be customized with ease,
pub struct StatusLineConfig {
    pub left: Vec<StatusLineElement>,
    pub center: Vec<StatusLineElement>,
    pub right: Vec<StatusLineElement>,
    pub separator: String,
    pub mode_string: ModeString,
    pub color_modes: bool,
}

pub struct ModeString {
    pub normal: String,
    pub insert: String,
    pub select: String,
}

pub enum StatusLineElement {
    Mode,
    LspSpinner,
    // + dirty flag
    FileName,
    FileEncoding,
    // CRLF | LF
    FileLineEnding,
    // language ID | "text"
    FileType,
    DiagnosticsCount,
    WorkspaceDiagnosticsCount,
    SelectionsCount,
    PrimarySelectionLength,
    CursorPosition,
    CursorPositionPercentage,
    SeparatorString,
    TotalLineNumbers,
    Spacer,
}


pub enum BufferLineConfig {
    Never,
    Always,
    IfMultiple,
}

pub struct WhitespaceConfig {
    pub render: WhitespaceRender,
    pub characters: WhitespaceCharacters,
}

pub enum WhitespaceRender {
    Basic(WhitespaceRenderValue),
    Specific {
        default: Option<WhitespaceRenderValue>,
        space: Option<WhitespaceRenderValue>,
        nbsp: Option<WhitespaceRenderValue>,
        tab: Option<WhitespaceRenderValue>,
        newline: Option<WhitespaceRenderValue>,
    },
}

pub enum WhitespaceRenderValue {
    None,
    All,
    // TODO: Selection,
}

impl WhitespaceRender {
    pub fn space(&self) -> WhitespaceRenderValue {
        match *self {
            Self::Basic(val) => val,
            Self::Specific { default, space, .. } => {
                space.or(default).unwrap_or(WhitespaceRenderValue::None)
            }
        }
    }
    pub fn nbsp(&self) -> WhitespaceRenderValue {
        match *self {
            Self::Basic(val) => val,
            Self::Specific { default, nbsp, .. } => {
                nbsp.or(default).unwrap_or(WhitespaceRenderValue::None)
            }
        }
    }
    pub fn tab(&self) -> WhitespaceRenderValue {
        match *self {
            Self::Basic(val) => val,
            Self::Specific { default, tab, .. } => {
                tab.or(default).unwrap_or(WhitespaceRenderValue::None)
            }
        }
    }
    pub fn newline(&self) -> WhitespaceRenderValue {
        match *self {
            Self::Basic(val) => val,
            Self::Specific {
                default, newline, ..
            } => newline.or(default).unwrap_or(WhitespaceRenderValue::None),
        }
    }
}

pub struct WhitespaceCharacters {
    pub space: char,
    pub nbsp: char,
    pub tab: char,
    pub tabpad: char,
    pub newline: char,
}

pub struct IndentGuidesConfig {
    pub render: bool,
    pub character: char,
    pub skip_levels: u8,
}

// Cursor shape is read and used on every rendered frame and so needs
// to be fast. Therefore we avoid a hashmap and use an enum indexed array.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CursorShapeConfig([CursorKind; 3]);

impl CursorShapeConfig {
    pub fn from_mode(&self, mode: Mode) -> CursorKind {
        self.get(mode as usize).copied().unwrap_or_default()
    }
}

impl<'de> Deserialize<'de> for CursorShapeConfig {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let m = std::collections::HashMap::<Mode, CursorKind>::deserialize(deserializer)?;
        let into_cursor = |mode: Mode| m.get(&mode).copied().unwrap_or_default();
        Ok(CursorShapeConfig([
            into_cursor(Mode::Normal),
            into_cursor(Mode::Select),
            into_cursor(Mode::Insert),
        ]))
    }
}

impl Serialize for CursorShapeConfig {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut map = serializer.serialize_map(Some(self.len()))?;
        let modes = [Mode::Normal, Mode::Select, Mode::Insert];
        for mode in modes {
            map.serialize_entry(&mode, &self.from_mode(mode))?;
        }
        map.end()
    }
}

impl std::ops::Deref for CursorShapeConfig {
    type Target = [CursorKind; 3];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
