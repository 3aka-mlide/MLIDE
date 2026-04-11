use eframe::egui;
use egui::text::LayoutJob;
use std::sync::Arc;
use std::collections::HashSet;

// ─────────────────────────────────────────────
//  VSCode Dark+ colour palette (accurate)
// ─────────────────────────────────────────────
mod col {
    use eframe::egui::Color32;

    // Types / storage class  (#4EC9B0 teal / #569CD6 blue)
    pub const TYPE_KEYWORD: Color32    = Color32::from_rgb(86,  156, 214); // blue  — int, char, bool…
    pub const TYPE_USER:    Color32    = Color32::from_rgb(78,  201, 176); // teal  — user structs / classes
    // Control flow (#C586C0 purple)
    pub const CONTROL:      Color32    = Color32::from_rgb(197, 134, 192);
    // Functions / macros (#DCDCAA yellow)
    pub const FUNCTION:     Color32    = Color32::from_rgb(220, 220, 170);
    // Preprocessor (#9B59B6 lavender, VSCode uses #C8A0DC)
    pub const PREPROCESSOR: Color32    = Color32::from_rgb(189, 147, 249);
    // String literals (#CE9178 orange-brown)
    pub const STRING:       Color32    = Color32::from_rgb(206, 145, 120);
    // Numeric literals (#B5CEA8 green-grey)
    pub const NUMBER:       Color32    = Color32::from_rgb(181, 206, 168);
    // Comments (#6A9955 green)
    pub const COMMENT:      Color32    = Color32::from_rgb(106, 153,  85);
    // Operators / punctuation (#D4D4D4)
    pub const OPERATOR:     Color32    = Color32::from_rgb(212, 212, 212);
    // Plain text
    pub const DEFAULT:      Color32    = Color32::from_rgb(212, 212, 212);
    // Error underline
    pub const ERROR:        Color32    = Color32::from_rgb(255,  80,  80);
    // Namespace / special identifiers (#4FC1FF light-blue)
    pub const NAMESPACE:    Color32    = Color32::from_rgb( 79, 193, 255);
    // Constants / enum values (#4FC1FF or teal)
    pub const CONSTANT:     Color32    = Color32::from_rgb(100, 180, 255);
}
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Language {
    Cpp,
    CSharp,
    Rust,
    Json,
    Toml,
    GitIgnore,
    Makefile,
    CMake,
}
// ─────────────────────────────────────────────
//  Keyword sets  (categorised like VSCode Dark+)
// ─────────────────────────────────────────────
const CSHARP_KEYWORDS: &[&str] = &[
    "abstract", "as", "base", "bool", "break", "byte", "case", "catch", "char", "checked",
    "class", "const", "continue", "decimal", "default", "delegate", "do", "double", "else",
    "enum", "event", "explicit", "extern", "false", "finally", "fixed", "float", "for",
    "foreach", "goto", "if", "implicit", "in", "int", "interface", "internal", "is", "lock",
    "long", "namespace", "new", "null", "object", "operator", "out", "override", "params",
    "private", "protected", "public", "readonly", "ref", "return", "sbyte", "sealed",
    "short", "sizeof", "stackalloc", "static", "string", "struct", "switch", "this",
    "throw", "true", "try", "typeof", "uint", "ulong", "unchecked", "unsafe", "ushort",
    "using", "virtual", "void", "volatile", "while", "get", "set", "var", "async", "await",
    "yield", "record", "init",
];

const CMAKE_FUNCTIONS: &[&str] = &[
    "project", "cmake_minimum_required", "add_executable", "add_library", 
    "target_link_libraries", "target_include_directories", "set", "find_package",
    "if", "else", "endif", "foreach", "endforeach", "include", "option", "message"
];

const MAKEFILE_KEYWORDS: &[&str] = &["ifeq", "ifneq", "else", "endif", "include", "define", "endef", "export"];

const JSON_TOML_CONSTANTS: &[&str] = &["true", "false", "null", "inf", "nan"];

const RUST_KEYWORDS: &[&str] = &[
    "as", "async", "await", "break", "const", "continue", "crate", "dyn", "else", "enum",
    "extern", "false", "fn", "for", "if", "impl", "in", "let", "loop", "match", "mod",
    "move", "mut", "pub", "ref", "return", "self", "Self", "static", "struct", "super",
    "trait", "true", "type", "union", "unsafe", "use", "where", "while", "async", "await",
    "abstract", "become", "box", "do", "final", "macro", "override", "priv", "typeof",
    "unsized", "virtual", "yield", "try",
];

const RUST_BUILTINS: &[&str] = &[
    "i8", "i16", "i32", "i64", "i128", "isize",
    "u8", "u16", "u32", "u64", "u128", "usize",
    "f32", "f64", "str", "String", "Option", "Result", "Vec",
];

const CPP_TYPE_KEYWORDS: &[&str] = &[
    "int", "char", "void", "bool", "double", "float", "long",
    "unsigned", "signed", "short", "wchar_t", "auto", "size_t",
    "int8_t", "int16_t", "int32_t", "int64_t",
    "uint8_t", "uint16_t", "uint32_t", "uint64_t",
    "ptrdiff_t", "intptr_t", "uintptr_t",
    "static", "const", "constexpr", "consteval", "constinit",
    "volatile", "mutable", "inline", "extern", "register",
    "thread_local", "explicit", "friend",
    "class", "struct", "enum", "union", "typedef", "typename",
    "template", "virtual", "override", "final", "abstract",
    "public", "private", "protected",
    "namespace", "using",
    "noexcept", "nullptr", "true", "false",
    "operator", "sizeof", "alignof", "decltype", "typeof",
    "static_cast", "dynamic_cast", "reinterpret_cast", "const_cast",
];

const CPP_CONTROL_KEYWORDS: &[&str] = &[
    "if", "else", "while", "for", "do",
    "return", "break", "continue", "goto",
    "switch", "case", "default",
    "try", "catch", "throw",
    "new", "delete", "co_await", "co_return", "co_yield",
    // kernel / C
    "kfree", "kmalloc", "free", "malloc", "calloc", "realloc",
];

const CPP_BUILTIN_FUNCTIONS: &[&str] = &[
    "main", "cout", "cin", "cerr", "clog", "endl", "flush",
    "printf", "fprintf", "sprintf", "snprintf",
    "scanf",  "fscanf",  "sscanf",
    "puts",   "gets",    "fgets", "fputs",
    "strlen", "strcpy",  "strncpy", "strcat", "strncat", "strcmp", "strncmp", "strstr",
    "memcpy", "memmove", "memset", "memcmp",
    "atoi",   "atof",    "atol",
    "abs",    "fabs",    "sqrt",  "pow",   "ceil",  "floor", "round", "fmod",
    "exit",   "abort",   "atexit",
    "assert",
    "std",    "string",  "vector", "map",  "set",   "pair",
    "unique_ptr", "shared_ptr", "weak_ptr", "make_unique", "make_shared",
    "move",   "forward", "swap",  "sort",  "find",  "begin", "end",
    "push_back", "pop_back", "emplace_back",
    "size",   "empty",   "clear", "reserve", "resize",
    "open",   "close",   "read",  "write", "seek",  "tell",
    "this",   "self",
];

const NAMESPACE_NAMES: &[&str] = &[
    "std", "boost", "fmt", "ranges", "views", "chrono", "filesystem",
    "literals", "string_literals", "complex_literals",
];

// ─────────────────────────────────────────────
//  Hover tooltip info
// ─────────────────────────────────────────────
pub struct KeywordInfo {
    pub meaning: &'static str,
    pub fix: &'static str,
}

pub fn get_info(word: &str) -> Option<KeywordInfo> {
    match word {
        "int" => Some(KeywordInfo {
            meaning: "Integer: 32-bit whole number (-2,147,483,648 to 2,147,483,647).",
            fix: "Use 'long long' for larger ranges. Beware signed overflow (UB in C++).",
        }),
        "bool" => Some(KeywordInfo {
            meaning: "Boolean: true / false (1 byte).",
            fix: "Never use '=' inside an if-condition. '==' compares, '=' assigns.",
        }),
        "float" | "double" => Some(KeywordInfo {
            meaning: "Floating Point: decimal numbers (float=32-bit, double=64-bit).",
            fix: "Never compare with '=='. Use: std::abs(a - b) < epsilon.",
        }),
        "void" => Some(KeywordInfo {
            meaning: "Void: absence of type. Functions returning void must not return a value.",
            fix: "If you need to return something, change the return type.",
        }),
        "auto" => Some(KeywordInfo {
            meaning: "Auto: compiler deduces the type automatically (C++11+).",
            fix: "Be careful with proxy types (e.g. vector<bool>) — use explicit types when in doubt.",
        }),
        "nullptr" => Some(KeywordInfo {
            meaning: "Null Pointer: type-safe replacement for NULL / 0.",
            fix: "Always initialise pointers to nullptr. Check before dereferencing.",
        }),
        "new" | "malloc" => Some(KeywordInfo {
            meaning: "Memory Allocation: reserves space on the heap.",
            fix: "Every 'new' must have a matching 'delete'. Prefer smart pointers.",
        }),
        "delete" | "free" | "kfree" => Some(KeywordInfo {
            meaning: "Memory Release: returns heap memory to the system.",
            fix: "Set pointer to nullptr after freeing to prevent use-after-free / double-free.",
        }),
        "struct" | "class" => Some(KeywordInfo {
            meaning: "Data Container: groups variables and methods together.",
            fix: "CRITICAL: closing brace must be followed by a semicolon ';'.",
        }),
        "if" => Some(KeywordInfo {
            meaning: "Decision Gate: executes block when condition is true.",
            fix: "Bug: if(x = 5) assigns, not compares. Use if(x == 5).",
        }),
        "for" | "while" | "do" => Some(KeywordInfo {
            meaning: "Loop: repeats code while condition holds.",
            fix: "Guard against infinite loops. Ensure the condition eventually becomes false.",
        }),
        "switch" => Some(KeywordInfo {
            meaning: "Branch: compares a variable against multiple 'case' values.",
            fix: "Always add 'break;' after each case, or execution falls through.",
        }),
        "template" => Some(KeywordInfo {
            meaning: "Template: generic programming — write code for multiple types.",
            fix: "Errors can be cryptic. Use 'static_assert' to constrain types early.",
        }),
        "constexpr" => Some(KeywordInfo {
            meaning: "Compile-time constant: evaluated at compile time when possible.",
            fix: "constexpr functions must have a single return statement (C++11); relaxed in C++14+.",
        }),
        "noexcept" => Some(KeywordInfo {
            meaning: "No-throw guarantee: hints to the compiler this function won't throw.",
            fix: "Move constructors should be noexcept for container optimisations.",
        }),
        _ => None,
    }
}

// ─────────────────────────────────────────────
//  Tokeniser state machine
// ─────────────────────────────────────────────
#[derive(PartialEq)]
enum State {
    Normal,
    LineComment,
    BlockComment,
    StringLit { quote: char, verbatim: bool },
    Preprocessor,
}

pub fn highlight_code(
    ui: &egui::Ui,
    source: &str,
    lang: Language,
    wrap_width: f32,
    libs: &HashSet<String>,
    errors: &HashSet<usize>,
) -> Arc<egui::Galley> {
    let mut job = LayoutJob::default();
    let font = egui::FontId::monospace(14.0);
    let mut state = State::Normal;
    let mut current_line: usize = 1;

    let chars: Vec<(usize, char)> = source.char_indices().collect();
    let len = chars.len();
    let mut i = 0;

    let push = |job: &mut LayoutJob, text: &str, color: egui::Color32, line: usize, font: &egui::FontId| {
        if text.is_empty() { return; }
        let underline = if errors.contains(&line) { egui::Stroke::new(1.5, col::ERROR) } else { egui::Stroke::NONE };
        job.append(text, 0.0, egui::TextFormat { font_id: font.clone(), color, underline, ..Default::default() });
    };

    while i < len {
        let (byte_pos, ch) = chars[i];
        let peek = chars.get(i + 1).map(|&(_, c)| c);

        match &state {
            State::LineComment => {
                if ch == '\n' {
                    push(&mut job, "\n", col::COMMENT, current_line, &font);
                    current_line += 1;
                    state = State::Normal;
                } else {
                    push(&mut job, &ch.to_string(), col::COMMENT, current_line, &font);
                }
            }

            State::BlockComment => {
                if ch == '*' && peek == Some('/') {
                    push(&mut job, "*/", col::COMMENT, current_line, &font);
                    state = State::Normal;
                    i += 1;
                } else {
                    if ch == '\n' { current_line += 1; }
                    push(&mut job, &ch.to_string(), col::COMMENT, current_line, &font);
                }
            }

            State::StringLit { quote, verbatim } => {
                let q = *quote;
                if !*verbatim && ch == '\\' && peek.is_some() {
                    let escaped = &source[byte_pos..byte_pos + ch.len_utf8() + chars[i+1].1.len_utf8()];
                    push(&mut job, escaped, col::STRING, current_line, &font);
                    i += 1;
                } else if ch == q {
                    push(&mut job, &ch.to_string(), col::STRING, current_line, &font);
                    state = State::Normal;
                } else {
                    if ch == '\n' { current_line += 1; }
                    push(&mut job, &ch.to_string(), col::STRING, current_line, &font);
                }
            }

            State::Preprocessor => {
                if ch == '\n' {
                    push(&mut job, "\n", col::DEFAULT, current_line, &font);
                    current_line += 1;
                    state = State::Normal;
                } else {
                    push(&mut job, &ch.to_string(), col::PREPROCESSOR, current_line, &font);
                }
            }

            State::Normal => {
                if ch == '\n' {
                    push(&mut job, "\n", col::DEFAULT, current_line, &font);
                    current_line += 1;
                } 
                else if (ch == '/' && peek == Some('/')) || 
                        (matches!(lang, Language::CMake | Language::Makefile | Language::Toml | Language::GitIgnore) && ch == '#') {
                    let symbol = if ch == '#' { "#" } else { i += 1; "//" };
                    push(&mut job, symbol, col::COMMENT, current_line, &font);
                    state = State::LineComment;
                } 
                else if ch == '/' && peek == Some('*') {
                    push(&mut job, "/*", col::COMMENT, current_line, &font);
                    state = State::BlockComment;
                    i += 1;
                } 
                else if ch == '#' && lang == Language::Cpp {
                    push(&mut job, "#", col::PREPROCESSOR, current_line, &font);
                    state = State::Preprocessor;
                } 
                else if ch == '"' || ch == '\'' {
                    state = State::StringLit { quote: ch, verbatim: false };
                    push(&mut job, &ch.to_string(), col::STRING, current_line, &font);
                } 
                else if ch == '$' && peek == Some('{') && lang == Language::CMake {
                    let start = byte_pos;
                    let mut j = i;
                    while j < len && chars[j].1 != '}' { j += 1; }
                    if j < len { j += 1; }
                    push(&mut job, &source[start..chars[j-1].0 + chars[j-1].1.len_utf8()], col::NAMESPACE, current_line, &font);
                    i = j - 1;
                }
                else if ch.is_ascii_digit() {
                    let start = byte_pos;
                    let mut j = i;
                    while j < len && (chars[j].1.is_ascii_alphanumeric() || chars[j].1 == '.') { j += 1; }
                    push(&mut job, &source[start..chars[j-1].0 + chars[j-1].1.len_utf8()], col::NUMBER, current_line, &font);
                    i = j - 1;
                } 
                else if ch.is_alphabetic() || ch == '_' {
                    let start = byte_pos;
                    let mut j = i;
                    while j < len && (chars[j].1.is_alphanumeric() || chars[j].1 == '_' || chars[j].1 == '-') { j += 1; }
                    let word = &source[start..chars[j-1].0 + chars[j-1].1.len_utf8()];
                    
                    let color = match lang {
                        Language::Rust => if RUST_KEYWORDS.contains(&word) { col::CONTROL } else { col::DEFAULT },
                        Language::CSharp => if CSHARP_KEYWORDS.contains(&word) { col::TYPE_KEYWORD } else { col::DEFAULT },
                        Language::Cpp => if CPP_TYPE_KEYWORDS.contains(&word) { col::TYPE_KEYWORD } 
                                         else if CPP_CONTROL_KEYWORDS.contains(&word) { col::CONTROL } 
                                         else { col::DEFAULT },
                        Language::CMake => if CMAKE_FUNCTIONS.contains(&word) { col::FUNCTION } else { col::DEFAULT },
                        Language::Makefile => if MAKEFILE_KEYWORDS.contains(&word) { col::CONTROL } else { col::DEFAULT },
                        Language::Json | Language::Toml => if JSON_TOML_CONSTANTS.contains(&word) { col::CONTROL } else { col::DEFAULT },
                        _ => col::DEFAULT,
                    };
                    
                    push(&mut job, word, if libs.contains(word) { col::TYPE_USER } else { color }, current_line, &font);
                    i = j - 1;
                } 
                else {
                    let color = match ch {
                        '{'|'}'|'('|')'|'['|']' => egui::Color32::from_rgb(255, 215, 0),
                        ':' if lang == Language::Makefile => col::FUNCTION,
                        _ => col::DEFAULT,
                    };
                    push(&mut job, &ch.to_string(), color, current_line, &font);
                }
            }
        }
        i += 1;
    }

    job.wrap.max_width = wrap_width;
    ui.fonts(|f| f.layout_job(job))
}