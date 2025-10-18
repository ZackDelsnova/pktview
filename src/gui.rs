use eframe::{egui, App};
use std::sync::{Arc, Mutex, mpsc};
use std::thread;
use pnet::datalink;
use crate::capture::start_capture; // PacketInfo can be ignored if unused

pub struct PktViewApp {
    interfaces: Vec<String>,
    selected_index: Option<usize>,
    logs: Arc<Mutex<Vec<String>>>,
}

impl Default for PktViewApp {
    fn default() -> Self {
        let interfaces = datalink::interfaces()
            .iter()
            .map(|iface| {
                if !iface.description.is_empty() {
                    iface.description.clone()
                } else {
                    iface.name.clone()
                }
            })
            .collect();

        Self {
            interfaces,
            selected_index: None,
            logs: Arc::new(Mutex::new(Vec::new())),
        }
    }
}

impl App for PktViewApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Select Interface:");

            egui::ComboBox::from_label("Interface")
                .selected_text(
                    self.selected_index
                        .map(|i| &self.interfaces[i])
                        .unwrap_or(&"Select...".to_string())
                )
                .show_ui(ui, |cb| {
                    for (i, iface) in self.interfaces.iter().enumerate() {
                        cb.selectable_value(&mut self.selected_index, Some(i), iface);
                    }
                });

            if ui.button("Start Capture").clicked() {
                if let Some(index) = self.selected_index {
                    let iface_name = self.interfaces[index].clone();
                    let logs = self.logs.clone();
                    let (tx, rx) = mpsc::channel();

                    // spawn capture thread
                    thread::spawn(move || {
                        start_capture(tx, &iface_name);
                    });

                    // spawn GUI log updater
                    thread::spawn(move || {
                        for pkt in rx {
                            logs.lock().unwrap().push(format!("{:?}", pkt));
                        }
                    });
                }
            }

            ui.separator();
            ui.heading("Packet Log:");
            egui::ScrollArea::vertical().show(ui, |ui| {
                for log in self.logs.lock().unwrap().iter() {
                    ui.label(log);
                }
            });
        });
    }
}
