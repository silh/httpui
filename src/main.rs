use eframe::CreationContext;
use egui::Button;

fn main() -> eframe::Result<()> {
    let native_options = eframe::NativeOptions::default();
    return eframe::run_native(
        "Httpui",
        native_options,
        Box::new(|cc: &CreationContext| Box::new(HttpUIApp::new(cc))));
}

struct HttpUIApp {
    url: String,
    response: String,
    to_http_sender_ch: std::sync::mpsc::Sender<String>,
    ui_receiver_ch: std::sync::mpsc::Receiver<String>,
}

impl HttpUIApp {
    fn new(_: &eframe::CreationContext<'_>) -> Self {
        let (http_ch_tx, http_ch_rx) = std::sync::mpsc::channel();
        let (ui_ch_tx, ui_ch_rx) = std::sync::mpsc::channel();
        let ui_sender_ch = ui_ch_tx.clone();
        std::thread::spawn(move || {
            while let Ok(msg) = http_ch_rx.recv() {
                // TODO handle failed send
                let _ = match reqwest::blocking::get(&msg).and_then(|response| response.text()) {
                    Ok(response) => ui_sender_ch.send(response),
                    Err(err) => ui_sender_ch.send(err.to_string()),
                };
            }
        });
        return HttpUIApp {
            ui_receiver_ch: ui_ch_rx,
            to_http_sender_ch: http_ch_tx.clone(),
            url: String::new(),
            response: String::new(),
        };
    }
}

impl eframe::App for HttpUIApp {
    fn update(&mut self, ctx: &eframe::egui::Context, _frame: &mut eframe::Frame) {
        if let Ok(new_response) = self.ui_receiver_ch.try_recv() {
            self.response = new_response;
        }
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.horizontal_centered(|ui| {
                    ui.label("URL:");
                    ui.text_edit_singleline(&mut self.url);
                    if ui.add(Button::new("Submit")).clicked() {
                        // TODO handle failed send
                        let _ = self.to_http_sender_ch.send(self.url.clone());
                    }
                    ui.text_edit_multiline(&mut self.response);
                });
            })
        });
    }
}
