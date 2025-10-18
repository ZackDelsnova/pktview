mod capture;
mod gui;

use eframe::NativeOptions;

fn main() -> eframe::Result<()> {
    let options = NativeOptions::default();
    eframe::run_native(
        "pktview - netwrok packet visualizer", 
        options, 
        Box::new(|_cc| Box::new(gui::PktViewApp::default())),
    )
}