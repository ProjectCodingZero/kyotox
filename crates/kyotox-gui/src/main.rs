use eframe::egui;
use portable_pty::{CommandBuilder, NativePtySystem, PtySize, PtySystem};
use std::io::{Read, Write};
use std::sync::{Arc, Mutex};
use std::thread;

struct TerminalApp {
    // Canal para enviar comandos al PTY
    pty_writer: Box<dyn Write + Send>,
    // Buffer compartido que almacena lo que la terminal responde
    output_buffer: Arc<Mutex<String>>,
}

impl TerminalApp {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        // 1. Inicializar el sistema PTY nativo
        let pty_system = NativePtySystem::default();
        let pty_pair = pty_system
            .openpty(PtySize {
                rows: 24,
                cols: 80,
                pixel_width: 0,
                pixel_height: 0,
            })
            .unwrap();

        // 2. Lanzar la Shell por defecto (ej. bash, zsh o cmd)
        #[cfg(target_os = "windows")]
        let cmd = CommandBuilder::new("cmd.exe");
        #[cfg(not(target_os = "windows"))]
        let cmd = CommandBuilder::new("bash");

        pty_pair.slave.spawn_command(cmd).unwrap();

        let pty_writer = pty_pair.master.take_writer().unwrap();
        let mut pty_reader = pty_pair.master.try_clone_reader().unwrap();

        let output_buffer = Arc::new(Mutex::new(String::new()));
        let output_buffer_clone = Arc::clone(&output_buffer);

        // 3. Hilo secundario para escuchar constantemente la salida del PTY
        thread::spawn(move || {
            let mut buf = [0u8; 1024];
            while let Ok(n) = pty_reader.read(&mut buf) {
                if n == 0 { break; }
                if let Ok(text) = std::str::from_utf8(&buf[..n]) {
                    let mut buffer = output_buffer_clone.lock().unwrap();
                    buffer.push_str(text);
                }
            }
        });

        Self {
            pty_writer,
            output_buffer,
        }
    }
}

impl eframe::App for TerminalApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Mi Emulador de Terminal en Rust");

            // Área de texto con scroll para mostrar la terminal
            egui::ScrollArea::vertical().show(ui, |ui| {
                let buffer = self.output_buffer.lock().unwrap();
                // Usamos un tipo de letra monoespaciada para la consola
                ui.add(
                    egui::TextEdit::multiline(&mut buffer.as_str())
                        .font(egui::TextStyle::Monospace)
                        .interactive(false) // Solo lectura visual directa por ahora
                );
            });

            // Capturar la entrada del teclado físico
            ctx.input(|i| {
                for event in &i.events {
                    if let egui::Event::Text(text) = event {
                        // Enviamos el texto que escribe el usuario directo al proceso del PTY
                        let _ = self.pty_writer.write_all(text.as_bytes());
                        let _ = self.pty_writer.flush();
                    }
                    if let egui::Event::Key { key: egui::Key::Enter, pressed: true, .. } = event {
                        // Manejo manual del salto de línea si es necesario
                        let _ = self.pty_writer.write_all(b"\n");
                        let _ = self.pty_writer.flush();
                    }
                }
            });
        });

        // Forzar a egui a repintar continuamente para ver la salida en tiempo real
        ctx.request_repaint();
    }
}

fn main() -> eframe::Result<()> {
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "Rust Terminal GUI",
        native_options,
        Box::new(|cc| Ok(Box::new(TerminalApp::new(cc)))),
    )
}
