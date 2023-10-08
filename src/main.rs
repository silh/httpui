use eframe::CreationContext;
use egui::ScrollArea;
use hyper::{HeaderMap, Method};
use hyper::http::{HeaderName, HeaderValue};
use log::error;

mod format;
mod request;

fn main() -> eframe::Result<()> {
    env_logger::init();
    let native_options = eframe::NativeOptions::default();
    return eframe::run_native(
        "Httpui",
        native_options,
        Box::new(|cc: &CreationContext| Box::new(HttpUIApp::new(cc))));
}

struct HttpUIApp {
    response: String,
    to_http_sender_ch: std::sync::mpsc::Sender<request::Request>,
    ui_receiver_ch: std::sync::mpsc::Receiver<String>,
    request: request::Request,
}

impl HttpUIApp {
    fn new(_: &CreationContext<'_>) -> Self {
        let (http_ch_tx, http_ch_rx) = std::sync::mpsc::channel::<request::Request>();
        let (ui_ch_tx, ui_ch_rx) = std::sync::mpsc::channel();
        let ui_sender_ch = ui_ch_tx.clone();
        let client = reqwest::blocking::Client::new();
        std::thread::spawn(move || {
            while let Ok(req) = http_ch_rx.recv() {
                let mut headers = HeaderMap::new();
                let vec = req.headers;
                vec.iter().for_each(|(k, v)| {
                    let key = HeaderName::from_bytes(k.as_bytes()).unwrap();
                    let value = HeaderValue::from_bytes(v.as_bytes()).unwrap();
                    headers.insert(key, value);
                });

                let body = reqwest::blocking::Body::from(req.body);
                // should be safe to unwrap as we have the same enum
                let method = Method::from_bytes(req.method.to_string().as_bytes()).unwrap();
                let txt = client.request(method, &req.url)
                    .headers(headers)
                    .body(body)
                    .send()
                    .and_then(|response| {
                        if let Some(_) = response.headers().get("Content-Type").filter(|v| *v == "application/json") {
                            return response.text().map(|txt| format::pretty_format_json(&txt));
                        }
                        return response.text();
                    })
                    .map_or_else(|e| e.to_string(), |r| r);
                if let Err(msg) = ui_sender_ch.send(txt) {
                    error!("Failed to send response result: {}", msg);
                }
            }
        });
        let mut headers = Vec::new();
        headers.push(("Content-Type".to_owned(), "application/json".to_owned()));

        return HttpUIApp {
            ui_receiver_ch: ui_ch_rx,
            to_http_sender_ch: http_ch_tx.clone(),
            response: String::new(),
            request: request::Request {
                url: "http://localhost:8080/".to_owned(),
                headers,
                body: String::new(),
                method: request::Method::Get,
            },
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
                    egui::ComboBox::from_label("Method")
                        .selected_text(self.request.method.to_string())
                        .show_ui(ui, |ui| {
                            // TODO add iteration over enum
                            ui.selectable_value(&mut self.request.method, request::Method::Options, "OPTIONS");
                            ui.selectable_value(&mut self.request.method, request::Method::Get, "GET");
                            ui.selectable_value(&mut self.request.method, request::Method::Post, "POST");
                            ui.selectable_value(&mut self.request.method, request::Method::Put, "PUT");
                            ui.selectable_value(&mut self.request.method, request::Method::Delete, "DELETE");
                            ui.selectable_value(&mut self.request.method, request::Method::Head, "HEAD");
                            ui.selectable_value(&mut self.request.method, request::Method::Trace, "TRACE");
                            ui.selectable_value(&mut self.request.method, request::Method::Connect, "CONNECT");
                            ui.selectable_value(&mut self.request.method, request::Method::Patch, "PATCH");
                        });
                    ui.text_edit_singleline(&mut self.request.url);
                    if ui.button("Submit").clicked() {
                        if let Err(err) = self.to_http_sender_ch.send(self.request.clone()) {
                            error!("Failed to send request: {}", err)
                        }
                    }
                    ui.text_edit_multiline(&mut self.request.body);
                });
                ui.label("Headers:");

                // For every header we want to display its name and value, but we also want to
                // add a delete button. We can't iterate and call delete while iterating, so we do
                // retain.
                self.request.headers.retain_mut(|(ref mut k, ref mut v)| {
                    let mut result = true;
                    ui.horizontal(|ui| {
                        ui.text_edit_singleline(k);
                        ui.text_edit_singleline(v);
                        if ui.button("-").clicked() {
                            result = false;
                        }
                    });
                    return result;
                });
                if ui.button("+").clicked() {
                    self.request.headers.push(("".to_owned(), "".to_owned()));
                }
                ScrollArea::vertical().show(ui, |ui| {
                    ui.text_edit_multiline(&mut self.response.as_str());
                });
            });
        });
    }
}
