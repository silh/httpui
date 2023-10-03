use eframe::CreationContext;
use egui::ScrollArea;
use log::error;

fn main() -> eframe::Result<()> {
    env_logger::init();
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
    headers: Vec<(String, String)>,
}

impl HttpUIApp {
    fn new(_: &eframe::CreationContext<'_>) -> Self {
        let (http_ch_tx, http_ch_rx) = std::sync::mpsc::channel();
        let (ui_ch_tx, ui_ch_rx) = std::sync::mpsc::channel();
        let ui_sender_ch = ui_ch_tx.clone();
        std::thread::spawn(move || {
            while let Ok(msg) = http_ch_rx.recv() {
                let txt = reqwest::blocking::get(&msg)
                    .and_then(|response| response.text())
                    .map_or_else(|e| e.to_string(), |r| r);
                if let Err(msg) = ui_sender_ch.send(txt) {
                    error!("Failed to send response result: {}", msg);
                }
            }
        });
        let mut headers = Vec::new();
        headers.push(("Content-Type".to_owned(), "application/json".to_owned()));
        headers.push(("Accept".to_owned(), "application/json".to_owned()));

        return HttpUIApp {
            ui_receiver_ch: ui_ch_rx,
            to_http_sender_ch: http_ch_tx.clone(),
            url: "https://google.com".to_owned(),
            response: String::new(),
            headers,
        };
    }
}

impl eframe::App for HttpUIApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if let Ok(new_response) = self.ui_receiver_ch.try_recv() {
            self.response = new_response;
        }
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered_justified(|ui| {
                ui.horizontal(|ui| {
                    ui.label("URL:");
                    ui.text_edit_singleline(&mut self.url);
                    if ui.button("Submit").clicked() {
                        if let Err(err) = self.to_http_sender_ch.send(self.url.clone()) {
                            error!("Failed to send request: {}", err)
                        }
                    }
                });
                self.headers.iter_mut().for_each(|(ref mut k, ref mut v)| {
                    ui.horizontal(|ui| {
                        ui.text_edit_singleline(k);
                        ui.text_edit_singleline(v);
                    });
                });
                ScrollArea::vertical().show(ui, |ui| {
                    ui.text_edit_multiline(&mut self.response.as_str());
                });
            });
        });
    }
}
