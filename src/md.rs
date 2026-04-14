use eframe::egui;
use egui_commonmark::{CommonMarkCache, CommonMarkViewer};

/* Base Markdown Viewer */
pub struct MdViewer {
    cache: CommonMarkCache,
}

impl Default for MdViewer {
    fn default() -> Self {
        Self {
            cache: CommonMarkCache::default(),
        }
    }
}

impl MdViewer {
    pub fn render(&mut self, ui: &mut egui::Ui, content: &str) {
        egui::ScrollArea::vertical()
            .id_salt("md_viewer_scroll")
            .show(ui, |ui| {
                CommonMarkViewer::new()
                    .show(ui, &mut self.cache, content);
            });
    }
}