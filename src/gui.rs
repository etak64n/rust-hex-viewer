use anyhow::{Result, anyhow};
use eframe::{
    App as EguiApp, Frame, NativeOptions,
    egui::{self, Align, Event, Key, RichText, ScrollArea, SelectableLabel, TopBottomPanel, vec2},
};

use crate::app::{App, RowText};

const GUI_ROW_HEIGHT: f32 = 20.0;

/// Launch the egui-based GUI frontend.
pub fn run(app: App, debug: bool) -> Result<()> {
    let options = NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Rust Hex Viewer")
            .with_inner_size([1000.0, 700.0])
            .with_min_inner_size([600.0, 400.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Rust Hex Viewer",
        options,
        Box::new(move |_cc| Box::new(HexGui::new(app, debug))),
    )
    .map_err(|e| anyhow!("Failed to run eframe: {e}"))
}

struct HexGui {
    app: App,
    selected_offset: Option<usize>,
    debug: bool,
    last_vertical_move: Option<VerticalMove>,
}

#[derive(Clone, Copy, Debug)]
enum VerticalMove {
    Up,
    Down,
}

impl HexGui {
    fn new(app: App, debug: bool) -> Self {
        let initial_selection = if app.bytes_len() > 0 { Some(0) } else { None };

        Self {
            app,
            selected_offset: initial_selection,
            debug,
            last_vertical_move: None,
        }
    }

    fn is_selected(&self, offset: usize) -> bool {
        self.selected_offset == Some(offset)
    }

    fn select(&mut self, offset: usize) {
        self.selected_offset = Some(offset);
    }

    fn log_scroll(&self, direction: &str, row_offset: usize, rect_edge: f32, clip_edge: f32) {
        if self.debug {
            eprintln!(
                "[gui-scroll] dir={direction} row=0x{row_offset:08X} rect={rect_edge:.2} clip={clip_edge:.2}"
            );
        }
    }

    fn handle_keyboard_navigation(&mut self, ctx: &egui::Context) -> bool {
        let stride = self.app.bytes_per_row().max(1) as isize;
        let mut deltas: Vec<isize> = Vec::new();

        ctx.input(|input| {
            for event in &input.events {
                if let Event::Key { key, pressed, .. } = event {
                    if !pressed {
                        continue;
                    }
                    let movement = match key {
                        Key::ArrowRight => Some(("right", 1)),
                        Key::ArrowLeft => Some(("left", -1)),
                        Key::ArrowDown => Some(("down", stride)),
                        Key::ArrowUp => Some(("up", -stride)),
                        _ => None,
                    };
                    if let Some((dir, d)) = movement {
                        self.last_vertical_move = match dir {
                            "down" => Some(VerticalMove::Down),
                            "up" => Some(VerticalMove::Up),
                            _ => None,
                        };
                        if self.debug {
                            eprintln!("[gui-key] key={dir} delta={d}");
                        }
                        deltas.push(d);
                    }
                }
            }
        });

        let moved = !deltas.is_empty();
        for delta in deltas {
            self.move_selection_by(delta);
        }

        moved
    }

    fn move_selection_by(&mut self, delta: isize) {
        let len = self.app.bytes_len();
        if len == 0 {
            self.selected_offset = None;
            return;
        }
        let max_index = len.saturating_sub(1) as isize;
        let current = self.selected_offset.unwrap_or(0) as isize;
        let next = (current + delta).clamp(0, max_index);
        self.selected_offset = Some(next as usize);
    }

    fn row_contains_selected(&self, row: &RowText) -> bool {
        if let Some(sel) = self.selected_offset {
            let start = row.offset;
            let end = row.offset + row.bytes.len();
            sel >= start && sel < end
        } else {
            false
        }
    }
}

impl EguiApp for HexGui {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut Frame) {
        let keyboard_moved = self.handle_keyboard_navigation(ctx);

        let mut selection_update: Option<usize> = None;
        let total_rows = self.app.total_rows();

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.add_space(4.0);
            if total_rows == 0 {
                ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        ui.label("File is empty.");
                    });
                return;
            }

            draw_header(ui, self.app.bytes_per_row());
            ui.add_space(2.0);

            ScrollArea::vertical()
                .auto_shrink([false, false])
                .id_source("hex_scroll_area")
                .show_rows(ui, GUI_ROW_HEIGHT, total_rows, |ui, row_range| {
                    let rows: Vec<RowText> =
                        self.app.lines_for_range(row_range.start, row_range.len());
                    for row in rows {
                        let row_selected = keyboard_moved && self.row_contains_selected(&row);
                        let row_response = ui.horizontal(|ui| {
                            ui.monospace(format!("{:08X}", row.offset));
                            ui.add_space(4.0);
                            ui.separator();
                            ui.add_space(4.0);
                            ui.scope(|ui| {
                                let spacing = &mut ui.style_mut().spacing;
                                spacing.item_spacing.x = 4.0;
                                spacing.button_padding = vec2(2.0, 0.0);

                                for (idx, byte) in row.bytes.iter().enumerate() {
                                    if idx == self.app.bytes_per_row() / 2 {
                                        ui.add_space(4.0);
                                    }
                                    let cell_offset = row.offset + idx;
                                    let text = RichText::new(format!("{:02X}", byte)).monospace();
                                    let response = ui.add(SelectableLabel::new(
                                        self.is_selected(cell_offset),
                                        text,
                                    ));
                                    if response.clicked() {
                                        selection_update = Some(cell_offset);
                                    }
                                }
                            });

                            ui.add_space(6.0);
                            ui.separator();
                            ui.add_space(4.0);
                            ui.scope(|ui| {
                                let spacing = &mut ui.style_mut().spacing;
                                spacing.item_spacing.x = 0.0;
                                spacing.button_padding = vec2(2.0, 0.0);

                                for (idx, byte) in row.bytes.iter().enumerate() {
                                    let cell_offset = row.offset + idx;
                                    let text = RichText::new(printable_ascii(*byte)).monospace();
                                    let response = ui.add(SelectableLabel::new(
                                        self.is_selected(cell_offset),
                                        text,
                                    ));
                                    if response.clicked() {
                                        selection_update = Some(cell_offset);
                                    }
                                }
                            });
                        });

                        if row_selected {
                            let clip = ui.clip_rect();
                            let rect = row_response.response.rect;
                            let needs_scroll =
                                rect.top() < clip.top() || rect.bottom() > clip.bottom();
                            if needs_scroll {
                                match self.last_vertical_move {
                                    Some(VerticalMove::Up) => {
                                        self.log_scroll("up", row.offset, rect.top(), clip.top());
                                        row_response.response.scroll_to_me(Some(Align::Min));
                                    }
                                    Some(VerticalMove::Down) => {
                                        self.log_scroll(
                                            "down",
                                            row.offset,
                                            rect.bottom(),
                                            clip.bottom(),
                                        );
                                        row_response.response.scroll_to_me(Some(Align::Max));
                                    }
                                    None => {
                                        if rect.top() < clip.top() {
                                            self.log_scroll(
                                                "up",
                                                row.offset,
                                                rect.top(),
                                                clip.top(),
                                            );
                                            row_response.response.scroll_to_me(Some(Align::Min));
                                        } else {
                                            self.log_scroll(
                                                "down",
                                                row.offset,
                                                rect.bottom(),
                                                clip.bottom(),
                                            );
                                            row_response.response.scroll_to_me(Some(Align::Max));
                                        }
                                    }
                                }
                            }
                        }
                    }
                });
        });

        if let Some(offset) = selection_update {
            self.select(offset);
            self.last_vertical_move = None;
        }

        TopBottomPanel::bottom("status_panel").show(ctx, |ui| {
            ui.label(self.app.file_name());
            ui.horizontal(|ui| {
                ui.label(format!(
                    "bytes: {} | rows: {} | width: {}",
                    self.app.bytes_len(),
                    self.app.total_rows(),
                    self.app.bytes_per_row()
                ));
                if let Some(offset) = self.selected_offset {
                    ui.label(format!("Selection: 0x{offset:08X}"));
                } else {
                    ui.label("Selection: none");
                }
            });
        });
    }
}

fn printable_ascii(byte: u8) -> String {
    if byte.is_ascii_graphic() || byte == b' ' {
        (byte as char).to_string()
    } else {
        ".".to_string()
    }
}

fn draw_header(ui: &mut egui::Ui, bytes_per_row: usize) {
    ui.horizontal(|ui| {
        ui.monospace(format!("{:<8}", "Offset"));
        ui.add_space(4.0);
        ui.separator();
        ui.add_space(4.0);
        ui.scope(|ui| {
            let spacing = &mut ui.style_mut().spacing;
            spacing.item_spacing.x = 4.0;
            spacing.button_padding = vec2(2.0, 0.0);

            for idx in 0..bytes_per_row {
                if idx == bytes_per_row / 2 {
                    ui.add_space(4.0);
                }
                let text = RichText::new(format!("{idx:02X}")).monospace();
                ui.add(SelectableLabel::new(false, text));
            }
        });
        ui.add_space(6.0);
        ui.separator();
        ui.add_space(4.0);
        ui.scope(|ui| {
            let spacing = &mut ui.style_mut().spacing;
            spacing.item_spacing.x = 0.0;
            spacing.button_padding = vec2(2.0, 0.0);

            for _ in 0..bytes_per_row {
                ui.add(SelectableLabel::new(false, RichText::new(" ").monospace()));
            }
        });
    });
    ui.separator();
}
