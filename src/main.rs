mod syntax;
mod md;

use eframe::egui;
use std::path::PathBuf;
use std::collections::HashSet;
use walkdir::WalkDir;
use std::process::Command;
use std::fs;
use std::sync::mpsc::{channel, Receiver, Sender};

fn main() -> eframe::Result<()> {
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "ML Editor 1.0v",
        native_options,
        Box::new(|cc| {
            setup_custom_fonts(&cc.egui_ctx);
            egui_extras::install_image_loaders(&cc.egui_ctx);
            let mut visuals = egui::Visuals::dark();
            visuals.panel_fill = egui::Color32::from_rgba_unmultiplied(12, 12, 18, 170);
            visuals.window_fill = egui::Color32::from_rgba_unmultiplied(20, 20, 25, 210);
            cc.egui_ctx.set_visuals(visuals);

            cc.egui_ctx.style_mut(|style| {
                style.override_text_style = Some(egui::TextStyle::Monospace);

                style.spacing.indent = 18.0; 
                style.spacing.button_padding = egui::vec2(4.0, 2.0);
                style.spacing.item_spacing = egui::vec2(8.0, 4.0);
            });
            
            Ok(Box::new(Data::default()))
        }),
    )
}
fn setup_custom_fonts(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();

    fonts.font_data.insert(
        "noto_emoji".to_owned(),
        egui::FontData::from_static(include_bytes!("NotoEmoji-Regular.ttf")),
    );

    fonts.families.get_mut(&egui::FontFamily::Proportional)
        .unwrap()
        .push("noto_emoji".to_owned());

    fonts.families.get_mut(&egui::FontFamily::Monospace)
        .unwrap()
        .push("noto_emoji".to_owned());

    ctx.set_fonts(fonts);
}

// ── Toast ──────────────────────────────────────────────────────────────────────
struct Toast {
    message: String,
    timer:   f32,
    color:   egui::Color32,
}
impl Toast {
    fn ok(msg: impl Into<String>) -> Self {
        Self { message: msg.into(), timer: 2.5, color: egui::Color32::from_rgb(80, 200, 120) }
    }
    fn err(msg: impl Into<String>) -> Self {
        Self { message: msg.into(), timer: 3.0, color: egui::Color32::from_rgb(255, 90, 90) }
    }
}

// ── Data types ─────────────────────────────────────────────────────────────────
struct ConsoleLine { spans: Vec<(String, egui::Color32)> }

#[derive(PartialEq, Clone)]
enum FileOp { CreateFile, CreateFolder, Rename, Delete }

struct Tab {
    path:           PathBuf,
    cached_content: String,
    is_dirty:       bool,
}

struct Data {
    content:               String,
    explorer_width:        f32,
    target_explorer_width: f32,
    console_height:        f32,
    target_console_height: f32,
    current_path:          Option<PathBuf>,
    expanded_folders:      HashSet<PathBuf>,
    files:                 Vec<PathBuf>,
    console_visible:       bool,
    console_input:         String,
    console_output:        Vec<ConsoleLine>,
    current_file_path:     Option<PathBuf>,
    latent_selection:      String,
    md_viewer:             md::MdViewer,
    is_md:                 bool,
    show_search:           bool,
    show_command_bar:      bool,
    search_query:          String,
    replace_query:         String,
    language: syntax::Language,
    open_tabs:             Vec<Tab>,
    scroll_to_match:       bool,
    is_image:              bool,
    renaming_target:       Option<String>,
    rename_buffer:         String,
    active_file_op:        Option<(FileOp, PathBuf)>,
    file_op_buffer:        String,
    libs:                  HashSet<String>,
    error_lines:           HashSet<usize>,
    tx:                    Sender<String>,
    rx:                    Receiver<String>,
    toasts:                Vec<Toast>,
    duplicate_pending:     bool,
    comment_pending:       bool,
    move_line_up:          bool,
    move_line_down:        bool,
    cursor_pos:            usize,
}

impl Default for Data {
    fn default() -> Self {
        let (tx, rx) = channel();
        Self {
            content: String::new(),
            explorer_width: 260.0, target_explorer_width: 260.0,
            console_height: 0.0,   target_console_height: 0.0,
            current_path: None, expanded_folders: HashSet::new(),
            files: Vec::new(), console_visible: false,
            console_input: String::new(), console_output: Vec::new(),
            current_file_path: None, latent_selection: String::new(),
            md_viewer: md::MdViewer::default(), is_md: false,
            show_command_bar: false, open_tabs: Vec::new(),
            scroll_to_match: false,
            show_search: false, search_query: String::new(), replace_query: String::new(),
            renaming_target: None, rename_buffer: String::new(),
            active_file_op: None, file_op_buffer: String::new(),
            libs: HashSet::new(), error_lines: HashSet::new(),
            tx, rx,
            is_image: false,
            toasts: Vec::new(),
            language: syntax::Language::Cpp,
            duplicate_pending: false, comment_pending: false,
            move_line_up: false, move_line_down: false,
            cursor_pos: 0,
        }
    }
}

impl Data {
    // ── Tab management ─────────────────────────────────────────────────────
    fn flush_active_tab(&mut self) {
        if let Some(ref path) = self.current_file_path.clone() {
            if let Some(tab) = self.open_tabs.iter_mut().find(|t| &t.path == path) {
                if tab.cached_content != self.content {
                    tab.cached_content = self.content.clone();
                    tab.is_dirty = true;
                }
            }
        }
    }
    fn get_icon_image(path: &std::path::Path) -> egui::Image<'static> {
        let ext = path.extension()
            .and_then(|s| s.to_str())
            .unwrap_or_default()
            .to_lowercase();
        
        let bytes: &[u8] = match ext.as_str() {
            "rs"   => include_bytes!("r.png"),
            "cpp"  => include_bytes!("cpp.png"),
            "c"    => include_bytes!("c.png"),
            "h"    => include_bytes!("h.png"),
            "hpp"  => include_bytes!("hpp.png"),
            "cs"   => include_bytes!("cs.png"),
            "json" => include_bytes!("json.png"),
            "toml" => include_bytes!("toml.png"),
            _      => include_bytes!("file.png"),
        };

        let uri = format!("bytes://{}.png", ext);
        egui::Image::from_bytes(uri, bytes.to_vec())
    }
    fn switch_to_tab(&mut self, idx: usize) {
        self.flush_active_tab();
        self.content           = self.open_tabs[idx].cached_content.clone();
        self.current_file_path = Some(self.open_tabs[idx].path.clone());
        self.is_md             = self.open_tabs[idx].path.extension().map_or(false, |e| e == "md");
    }

    fn close_tab(&mut self, idx: usize) {
        let was_active = self.current_file_path.as_ref() == Some(&self.open_tabs[idx].path);
        self.open_tabs.remove(idx);
        if was_active {
            if let Some(last) = self.open_tabs.last() {
                self.content           = last.cached_content.clone();
                self.current_file_path = Some(last.path.clone());
                self.is_md             = last.path.extension().map_or(false, |e| e == "md");
            } else {
                self.current_file_path = None;
                self.content.clear();
            }
        }
    }

    // ── File I/O ───────────────────────────────────────────────────────────
    fn save_current(&mut self) {
        if let Some(path) = self.current_file_path.clone() {
            match fs::write(&path, &self.content) {
                Ok(_) => {
                    if let Some(tab) = self.open_tabs.iter_mut().find(|t| t.path == path) {
                        tab.is_dirty = false;
                        tab.cached_content = self.content.clone();
                    }
                    let name = path.file_name().unwrap_or_default().to_string_lossy().to_string();
                    self.toasts.push(Toast::ok(format!("✔  Saved  {}", name)));
                }
                Err(e) => self.toasts.push(Toast::err(format!("✘  Save failed: {}", e))),
            }
        }
    }

    fn save_all(&mut self) {
        let dirty: Vec<(PathBuf, String)> = self.open_tabs.iter()
            .filter(|t| t.is_dirty)
            .map(|t| (t.path.clone(), t.cached_content.clone()))
            .collect();
        let n = dirty.len();
        for (path, content) in dirty {
            if fs::write(&path, &content).is_ok() {
                if let Some(tab) = self.open_tabs.iter_mut().find(|t| t.path == path) {
                    tab.is_dirty = false;
                }
            }
        }
        if n > 0 { self.toasts.push(Toast::ok(format!("✔  Saved {} file(s)", n))); }
    }

    fn refresh_files(&mut self) {
        if let Some(ref path) = self.current_path {
            self.files = WalkDir::new(path).max_depth(10).sort_by_file_name()
                .into_iter().filter_map(|e| e.ok()).map(|e| e.path().to_path_buf()).collect();
            self.scan_headers();
        }
    }

    fn scan_headers(&mut self) {
        if let Some(root) = &self.current_path {
            let mut found = HashSet::new();
            for entry in WalkDir::new(root).into_iter().filter_map(|e| e.ok()) {
                let path = entry.path();
                if path.extension().map_or(false, |ext| ext == "h" || ext == "hpp") {
                    if let Ok(content) = fs::read_to_string(path) {
                        for line in content.lines() {
                            let t = line.trim();
                            if t.contains('(') && !t.starts_with("//") && !t.starts_with('#') {
                                if let Some(name) = t.split('(').next().map(|s| s.split_whitespace().last().unwrap_or("")) {
                                    if !name.is_empty() { found.insert(name.to_string()); }
                                }
                            }
                            if t.starts_with("#define") {
                                if let Some(name) = t.split_whitespace().nth(1) { found.insert(name.to_string()); }
                            }
                        }
                    }
                }
            }
            self.libs = found;
        }
    }

    fn toggle_folder(&mut self, folder: PathBuf) {
        if self.expanded_folders.contains(&folder) {
            self.expanded_folders.retain(|p| !p.starts_with(&folder));
        } else {
            self.expanded_folders.insert(folder);
        }
    }

    // ── Terminal ───────────────────────────────────────────────────────────
    fn run_command(&mut self, ctx: &egui::Context) {
        if self.console_input.is_empty() { return; }
        let input = self.console_input.trim().to_string();
        self.error_lines.clear();
        self.console_output.push(ConsoleLine {
            spans: vec![(format!("> {}", input), egui::Color32::from_rgb(0, 255, 170))],
        });
        let current_dir = self.current_path.clone();
        let ctx_clone   = ctx.clone();
        let tx          = self.tx.clone();
        std::thread::spawn(move || {
            let mut cmd = if cfg!(target_os = "windows") {
                let mut c = Command::new("cmd"); c.args(["/C", &input]); c
            } else {
                let mut c = Command::new("sh"); c.args(["-c", &input]); c
            };
            if let Some(path) = current_dir { cmd.current_dir(path); }
            if let Ok(out) = cmd.output() {
                let _ = tx.send(String::from_utf8_lossy(&out.stdout).to_string());
                let _ = tx.send(String::from_utf8_lossy(&out.stderr).to_string());
            } else {
                let _ = tx.send("Error: Failed to execute command".to_string());
            }
            ctx_clone.request_repaint();
        });
        self.console_input.clear();
    }

    fn parse_line(&mut self, text: &str) -> ConsoleLine {
        if text.contains("error") {
            if let Some(n) = text.split(':').nth(1).and_then(|s| s.trim().parse::<usize>().ok()) {
                self.error_lines.insert(n);
            }
        }
        let color = if text.to_lowercase().contains("error")      { egui::Color32::from_rgb(255,  90, 90) }
                    else if text.to_lowercase().contains("warning") { egui::Color32::from_rgb(255, 210,  0) }
                    else                                            { egui::Color32::from_rgb(180, 180, 180) };
        ConsoleLine { spans: vec![(text.to_string(), color)] }
    }

    fn line_range(&self, cursor: usize) -> (usize, usize) {
        let start = self.content[..cursor].rfind('\n').map(|p| p + 1).unwrap_or(0);
        let end   = self.content[cursor..].find('\n').map(|p| cursor + p).unwrap_or(self.content.len());
        (start, end)
    }

    fn duplicate_current_line(&mut self) {
        let (start, end) = self.line_range(self.cursor_pos);
        let line = self.content[start..end].to_string();
        self.content.insert_str(end, &format!("\n{}", line));
    }

    fn toggle_line_comment(&mut self) {
        let (start, end) = self.line_range(self.cursor_pos);
        let line     = self.content[start..end].to_string();
        let trimmed  = line.trim_start();
        let indent: String = line.chars().take_while(|c| c.is_whitespace()).collect();
        let new_line = if trimmed.starts_with("// ") {
            format!("{}{}", indent, &trimmed[3..])
        } else if trimmed.starts_with("//") {
            format!("{}{}", indent, &trimmed[2..])
        } else {
            format!("{}// {}", indent, trimmed)
        };
        self.content.replace_range(start..end, &new_line);
    }

	fn handle_file_click(&mut self, file: &PathBuf) {
        let ext = file.extension().and_then(|s| s.to_str()).unwrap_or_default().to_lowercase();
        let is_img = matches!(ext.as_str(), "png" | "jpg" | "jpeg" | "gif" | "bmp");

        if is_img {
            self.is_image = true;
            self.is_md = false;
            self.current_file_path = Some(file.clone());
        } else if let Ok(c) = fs::read_to_string(file) {
            self.is_image = false;
            self.is_md = ext == "md";
            self.flush_active_tab();

            self.language = match ext.as_str() {
                "rs" => syntax::Language::Rust,
                "cs" => syntax::Language::CSharp,
                _ => syntax::Language::Cpp,
            };

            if let Some(idx) = self.open_tabs.iter().position(|t| t.path == *file) {
                self.content = self.open_tabs[idx].cached_content.clone();
            } else {
                self.content = c.clone();
                self.open_tabs.push(Tab { 
                    path: file.clone(), 
                    cached_content: c, 
                    is_dirty: false 
                });
            }
            self.current_file_path = Some(file.clone());
        }
    }

    fn move_line(&mut self, up: bool) {
        let (start, end) = self.line_range(self.cursor_pos);
        let line = self.content[start..end].to_string();
        if up && start > 0 {
            let prev_start = self.content[..start - 1].rfind('\n').map(|p| p + 1).unwrap_or(0);
            let prev_line  = self.content[prev_start..start - 1].to_string();
            self.content = format!("{}{}\n{}{}", &self.content[..prev_start], line, prev_line, &self.content[end..]);
        } else if !up && end < self.content.len() {
            let next_end  = self.content[end + 1..].find('\n').map(|p| end + 1 + p).unwrap_or(self.content.len());
            let next_line = self.content[end + 1..next_end].to_string();
            self.content = format!("{}{}\n{}{}", &self.content[..start], next_line, line, &self.content[next_end..]);
        }
    }
}

// ── App ────────────────────────────────────────────────────────────────────────
impl eframe::App for Data {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.flush_active_tab();

        while let Ok(msg) = self.rx.try_recv() {
            for line in msg.lines() {
                let p = self.parse_line(line);
                self.console_output.push(p);
            }
        }

        let dt = ctx.input(|i| i.stable_dt);
        self.toasts.retain_mut(|t| { t.timer -= dt; t.timer > 0.0 });

        let glass_frame = egui::Frame::none()
            .fill(egui::Color32::from_rgba_unmultiplied(15, 15, 22, 140))
            .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgba_unmultiplied(255, 255, 255, 15)))
            .inner_margin(8.0);

        self.explorer_width = egui::lerp(self.explorer_width..=self.target_explorer_width, dt * 15.0);
        self.console_height = egui::lerp(self.console_height..=self.target_console_height, dt * 15.0);
        if (self.explorer_width - self.target_explorer_width).abs() > 0.1
            || (self.console_height - self.target_console_height).abs() > 0.1
        { ctx.request_repaint(); }

        // ── Hotkeys ─────────────────────────────────────────────────────────
        let mut do_save     = false;
        let mut do_save_all = false;

        ctx.input_mut(|i| {
            if i.key_pressed(egui::Key::Escape)
               || (i.modifiers.command && i.key_pressed(egui::Key::Backtick))
            {
                self.console_visible       = !self.console_visible;
                self.target_console_height = if self.console_visible { 220.0 } else { 0.0 };
            }
            if i.modifiers.command && i.key_pressed(egui::Key::F) { self.show_search = !self.show_search; }
            if i.modifiers.command && i.key_pressed(egui::Key::P) { self.show_command_bar = !self.show_command_bar; }
            if i.modifiers.command && i.key_pressed(egui::Key::B) {
                self.target_explorer_width = if self.target_explorer_width > 10.0 { 0.0 } else { 260.0 };
            }
            if i.modifiers.command && i.key_pressed(egui::Key::S) {
                if i.modifiers.shift { do_save_all = true; } else { do_save = true; }
            }
            if i.modifiers.command && i.key_pressed(egui::Key::W) {
                if let Some(ref path) = self.current_file_path.clone() {
                    if let Some(idx) = self.open_tabs.iter().position(|t| &t.path == path) {
                        self.close_tab(idx);
                    }
                }
            }
            if i.modifiers.command && i.key_pressed(egui::Key::Tab) && !self.open_tabs.is_empty() {
                if let Some(ref path) = self.current_file_path.clone() {
                    let idx  = self.open_tabs.iter().position(|t| &t.path == path).unwrap_or(0);
                    let next = (idx + 1) % self.open_tabs.len();
                    self.flush_active_tab();
                    self.content           = self.open_tabs[next].cached_content.clone();
                    self.current_file_path = Some(self.open_tabs[next].path.clone());
                    self.is_md             = self.open_tabs[next].path.extension().map_or(false, |e| e == "md");
                }
            }
            if i.modifiers.command && i.key_pressed(egui::Key::Slash) { self.comment_pending = true; }
            if i.modifiers.command && i.key_pressed(egui::Key::D) { self.duplicate_pending = true; }
            if i.modifiers.alt && i.key_pressed(egui::Key::ArrowUp)   { self.move_line_up   = true; }
            if i.modifiers.alt && i.key_pressed(egui::Key::ArrowDown)  { self.move_line_down = true; }
        });

        if do_save     { self.save_current(); }
        if do_save_all { self.save_all(); }

        // ── Top Panel ────────────────────────────────────────────────────────
        egui::TopBottomPanel::top("top_bar").frame(glass_frame.clone()).show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label("MLIDE");
                ui.separator();

                let mut switch_to: Option<usize> = None;
                let mut close_idx: Option<usize> = None;

                for i in 0..self.open_tabs.len() {
                    let path      = self.open_tabs[i].path.clone();
                    let is_active = self.current_file_path.as_ref() == Some(&path);
                    let name      = path.file_name().unwrap_or_default().to_string_lossy();
                    let label     = format!("{}{}", if self.open_tabs[i].is_dirty { "● " } else { "" }, name);
                    let resp      = ui.selectable_label(is_active, label);
                    if resp.clicked() && !is_active { switch_to = Some(i); }
                    resp.context_menu(|ui| {
                        if ui.button("❌ Close Tab").clicked() { close_idx = Some(i); ui.close_menu(); }
                    });
                }

                if let Some(idx) = switch_to { self.switch_to_tab(idx); }
                if let Some(idx) = close_idx { self.close_tab(idx); }

                ui.separator();
                if ui.button("📂 Open Folder").clicked() {
                    if let Some(p) = rfd::FileDialog::new().pick_folder() {
                        self.current_path = Some(p);
                        self.refresh_files();
                    }
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(egui::RichText::new(
                        "Ctrl+S · Ctrl+/ · Ctrl+D · Alt+↑↓ · Ctrl+B · Ctrl+W · Ctrl+`"
                    ).small().color(egui::Color32::from_rgb(70, 70, 90)));
                });
            });
        });
        // ── Explorer ─────────────────────────────────────────────────────────
        egui::SidePanel::left("explorer")
    .frame(egui::Frame::none()
        .fill(egui::Color32::from_rgba_unmultiplied(15, 15, 22, 140))
        .inner_margin(10.0))
    .exact_width(self.explorer_width)
    .show_animated(ctx, self.explorer_width > 0.1, |ui| {
        ui.label(egui::RichText::new("WORKSPACE").strong().color(egui::Color32::GRAY));
        ui.separator();
        
        if let Some(root) = self.current_path.clone() {
            egui::ScrollArea::vertical()
                .id_salt("explorer_scroll")
                .show(ui, |ui| {
                    let files = self.files.clone();
                    for file in &files {
                        let parent = file.parent().unwrap_or(std::path::Path::new(""));
                        

                        if parent != root && !self.expanded_folders.contains(parent) { 
                            continue; 
                        }
                        
                        let depth = file.strip_prefix(&root).map(|p| p.components().count()).unwrap_or(0);
                        let is_dir = file.is_dir();
                        let name = file.file_name().unwrap_or_default().to_string_lossy();
                        let sel = self.current_file_path.as_ref() == Some(file);

                        ui.horizontal(|ui| {
                            ui.add_space(depth as f32 * 12.0);
   
                            if is_dir {
                                let symbol = if self.expanded_folders.contains(file) { "🔽 " } else { "▶ " };
                                ui.label(symbol);
                            } else {
                                ui.add(Data::get_icon_image(file));
                            }

                            let resp = ui.selectable_label(sel, &*name);

                            resp.context_menu(|ui| {
                                if is_dir {
                                    if ui.button("➕ New File").clicked()   { self.active_file_op = Some((FileOp::CreateFile, file.clone())); ui.close_menu(); }
                                    if ui.button("📁 New Folder").clicked() { self.active_file_op = Some((FileOp::CreateFolder, file.clone())); ui.close_menu(); }
                                    ui.separator();
                                }
                                if ui.button("📝 Rename").clicked() {
                                    self.active_file_op = Some((FileOp::Rename, file.clone()));
                                    self.file_op_buffer = name.to_string();
                                    ui.close_menu();
                                }
                                if ui.colored_label(egui::Color32::LIGHT_RED, "🗑 Delete").clicked() {
                                    self.active_file_op = Some((FileOp::Delete, file.clone()));
                                    ui.close_menu();
                                }
                            });

                            if resp.clicked() {
                                if is_dir {
                                    self.toggle_folder(file.clone());
                                } else {
                                    self.handle_file_click(file);
                                }
                            }
                        });
                    }
                });


            let rect = ui.available_rect_before_wrap();
            let resp = ui.interact(rect, ui.id().with("root_ctx"), egui::Sense::click());
            resp.context_menu(|ui| {
                if ui.button("➕ New File").clicked()   { self.active_file_op = Some((FileOp::CreateFile, root.clone())); ui.close_menu(); }
                if ui.button("📁 New Folder").clicked() { self.active_file_op = Some((FileOp::CreateFolder, root.clone())); ui.close_menu(); }
                ui.separator();
                if ui.button("🔄 Refresh").clicked()   { self.refresh_files(); ui.close_menu(); }
            });
        }
    });

        // ── Editor ───────────────────────────────────────────────────────────
        egui::CentralPanel::default()
            .frame(egui::Frame::none().fill(egui::Color32::TRANSPARENT))
            .show(ctx, |ui| {
                if self.is_md {
                    self.md_viewer.render(ui, &self.content);
                } else if self.is_image {
                    if let Some(path) = &self.current_file_path {
                        egui::ScrollArea::both().show(ui, |ui| {
                            ui.add(
                                egui::Image::new(format!("file://{}", path.display()))
                                    .rounding(4.0)
                                    .fit_to_exact_size(ui.available_size() * 0.9) 
                            );
                        });
                    }
                } else {
                    ui.horizontal_top(|ui| {
                        egui::ScrollArea::vertical().id_salt("editor_scroll").show(ui, |ui| {
                            let mut layouter = |ui: &egui::Ui, s: &str, w: f32| {
                                syntax::highlight_code(ui, s, self.language, w, &self.libs, &self.error_lines)
                            };
                            let output = egui::TextEdit::multiline(&mut self.content)
                                .code_editor().layouter(&mut layouter).frame(false)
                                .desired_width(f32::INFINITY).show(ui);

                            if let Some(cr) = output.cursor_range {
                                self.cursor_pos = cr.primary.ccursor.index;
                            }

                            if self.duplicate_pending { self.duplicate_current_line(); self.duplicate_pending = false; }
                            if self.comment_pending   { self.toggle_line_comment();    self.comment_pending   = false; }
                            if self.move_line_up      { self.move_line(true);          self.move_line_up      = false; }
                            if self.move_line_down    { self.move_line(false);         self.move_line_down    = false; }

                            if output.response.has_focus() {
                                let cp  = self.cursor_pos;
                                let last = self.content.chars().take(cp).collect::<String>()
                                    .split(|c: char| !c.is_alphanumeric() && c != '_')
                                    .last().unwrap_or("").to_string();
                                if last.len() > 2 {
                                    if let Some(kw) = self.libs.iter().find(|k| k.starts_with(&last) && *k != &last).cloned() {
                                        egui::show_tooltip(ctx, ui.layer_id(), egui::Id::new("hint"), |ui| {
                                            ui.label(format!("Tab → {}", kw));
                                        });
                                        if ui.input(|i| i.key_pressed(egui::Key::Tab)) {
                                            self.content.insert_str(cp, &kw[last.len()..]);
                                        }
                                    }
                                }
                            }

                            output.response.context_menu(|ui| {
                                if ui.button("📋 Copy").clicked() {
                                    ui.ctx().copy_text(self.latent_selection.clone());
                                    ui.close_menu();
                                }
                                ui.separator();
                                if ui.add_enabled(!self.latent_selection.is_empty(),
                                    egui::Button::new("🔂 Change All Occurrences")).clicked()
                                {
                                    self.renaming_target = Some(self.latent_selection.clone());
                                    self.rename_buffer   = self.latent_selection.clone();
                                    ui.close_menu();
                                }
                            });
                        });
                    });
                }
            });

        // ── Terminal ─────────────────────────────────────────────────────────
        if self.console_height > 0.1 {
            egui::TopBottomPanel::bottom("terminal")
                .frame(egui::Frame::none()
                    .fill(egui::Color32::from_rgba_unmultiplied(5, 5, 10, 235))
                    .inner_margin(10.0))
                .height_range(self.console_height..=self.console_height)
                .show(ctx, |ui| {
                    let out_height = self.console_height - 44.0;
                    let scroll = egui::ScrollArea::vertical()
                        .stick_to_bottom(true).max_height(out_height)
                        .show(ui, |ui| {
                            for line in &self.console_output {
                                ui.horizontal(|ui| {
                                    for (text, color) in &line.spans {
                                        ui.label(egui::RichText::new(text).monospace().color(*color));
                                    }
                                });
                            }
                        });

                    let out_rect = scroll.inner_rect;
                    ui.interact(out_rect, ui.id().with("term_out"), egui::Sense::click())
                        .context_menu(|ui| {
                            if ui.button("📋 Copy All Output").clicked() {
                                let all: String = self.console_output.iter()
                                    .flat_map(|l| l.spans.iter().map(|(t, _)| t.as_str()))
                                    .collect::<Vec<_>>().join("\n");
                                ui.ctx().copy_text(all);
                                ui.close_menu();
                            }
                            if ui.button("🗑 Clear Output").clicked() {
                                self.console_output.clear();
                                ui.close_menu();
                            }
                        });

                    ui.separator();
                    ui.horizontal(|ui| {
                        ui.colored_label(egui::Color32::from_rgb(80, 160, 255), "❯");
                        let res = ui.add(
                            egui::TextEdit::singleline(&mut self.console_input)
                                .desired_width(f32::INFINITY).frame(false)
                                .hint_text("Enter command…"),
                        );

                        if res.has_focus() {
                            ui.input_mut(|i| {
                                if i.modifiers.command && i.key_pressed(egui::Key::V) {
                                    #[cfg(feature = "arboard")]
                                    if let Ok(text) = arboard::Clipboard::new().and_then(|mut c| c.get_text()) {
                                        self.console_input.push_str(&text);
                                    }
                                }
                            });
                        }

                        res.context_menu(|ui| {
                            if ui.button("📋 Paste  Ctrl+V").clicked() {
                                #[cfg(feature = "arboard")]
                                if let Ok(text) = arboard::Clipboard::new().and_then(|mut c| c.get_text()) {
                                    self.console_input.push_str(&text);
                                }
                                ui.close_menu();
                            }
                            if ui.button("📋 Copy Input").clicked() {
                                ui.ctx().copy_text(self.console_input.clone());
                                ui.close_menu();
                            }
                            if ui.button("✖ Clear Input").clicked() {
                                self.console_input.clear();
                                ui.close_menu();
                            }
                        });

                        if res.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                            self.run_command(ctx);
                            res.request_focus();
                        }
                    });
                });
        }


        // ── Refactor modal ───────────────────────────────────────────────────
        if let Some(target) = self.renaming_target.clone() {
            egui::Window::new("Refactor — Change All Occurrences")
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0]).collapsible(false)
                .show(ctx, |ui| {
                    ui.label(format!("Replace:  \"{}\"", target));
                    ui.text_edit_singleline(&mut self.rename_buffer);
                    ui.horizontal(|ui| {
                        if ui.button("Apply").clicked() || ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                            let count = self.content.matches(&target).count();
                            self.content = self.content.replace(&target, &self.rename_buffer);
                            self.renaming_target = None;
                            self.toasts.push(Toast::ok(format!("Replaced {} occurrence(s)", count)));
                        }
                        if ui.button("Cancel").clicked() { self.renaming_target = None; }
                    });
                });
        }

        // ── File op modal ────────────────────────────────────────────────────
        if let Some((op, path)) = self.active_file_op.clone() {
            egui::Window::new("File Action")
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0]).collapsible(false)
                .show(ctx, |ui| {
                    if op == FileOp::Delete {
                        ui.label(format!("Delete  \"{}\"?", path.file_name().unwrap().to_string_lossy()));
                        ui.horizontal(|ui| {
                            if ui.colored_label(egui::Color32::LIGHT_RED, "Delete").clicked() {
                                let _ = if path.is_dir() { fs::remove_dir_all(&path) } else { fs::remove_file(&path) };
                                self.active_file_op = None; self.refresh_files();
                            }
                            if ui.button("Cancel").clicked() { self.active_file_op = None; }
                        });
                    } else {
                        let hint = match op { FileOp::CreateFile => "filename.cpp", FileOp::CreateFolder => "folder_name", _ => "new_name" };
                        ui.add(egui::TextEdit::singleline(&mut self.file_op_buffer).hint_text(hint));
                        ui.horizontal(|ui| {
                            if ui.button("Apply").clicked() {
                                let new_path = match op {
                                    FileOp::CreateFile   => if path.is_dir() { path.join(&self.file_op_buffer) } else { path.parent().unwrap().join(&self.file_op_buffer) },
                                    FileOp::CreateFolder => path.join(&self.file_op_buffer),
                                    FileOp::Rename       => path.parent().unwrap().join(&self.file_op_buffer),
                                    _ => unreachable!(),
                                };
                                match op {
                                    FileOp::CreateFile   => { let _ = fs::write(&new_path, ""); }
                                    FileOp::CreateFolder => { let _ = fs::create_dir(&new_path); }
                                    FileOp::Rename       => { let _ = fs::rename(&path, &new_path); }
                                    _ => {}
                                }
                                self.active_file_op = None; self.file_op_buffer.clear(); self.refresh_files();
                            }
                            if ui.button("Cancel").clicked() { self.active_file_op = None; }
                        });
                    }
                });
        }
    }
}