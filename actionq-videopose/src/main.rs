use eframe::egui;
use egui::{FontId, RichText, Ui};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

struct MyApp {
    
    /// If true then we must display the webcam and video
    is_running: bool
}

impl MyApp {

    fn bottom_panel(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::bottom("bottom_panel")
            .resizable(true)
            .show(ctx, |ui| {
                ui.horizontal_centered(|ui| {
                    ui.columns(3, |cols| {
                        cols[0].heading("TODO: EXERCISE STATS");

                        if self.is_running {
                            cols[1].add_enabled(false, egui::Button::new("START"));
                            if cols[2].button("STOP").clicked() {
                                videopose::inference_stop();
                                self.is_running = false;
                            }
                            
                            // Request new update of ui
                            ctx.request_repaint();
            
                        } else {
                            if cols[1].button("START").clicked() {
                                videopose::inference_start("/dev/video0", "webrtc://@:8554/output").unwrap();
                                self.is_running = true;
                            }
                            cols[2].add_enabled(false, egui::Button::new("STOP"));
                        }
                    }); 
                    ui.separator();
                });
            });
    }

    fn display_videopose_stream(&mut self, ui: &mut Ui) -> Result<()> {
        let frame_data = videopose::inference_step()?;
        if let Some(frame_data) = frame_data {
            let frame_size = [frame_data.framebuffer.size.0 as usize, frame_data.framebuffer.size.1 as usize];
            let img = egui::ColorImage::from_rgb(frame_size, &frame_data.framebuffer.storage);
            let texture = ui.ctx().load_texture("frame", img, Default::default());
            ui.image(&texture);
        }
        Ok(())
    }
}

impl Default for MyApp {
    fn default() -> Self {

        videopose::create_hpe_engine(
            "/home/nvidia/Repositories/actionq-core/network/pose_resnet18_body.onnx",
            "/home/nvidia/Repositories/actionq-core/network/human_pose.json",
            "/home/nvidia/Repositories/actionq-core/network/colors.txt"
        ).unwrap();

        Self {  
            is_running: false
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        self.bottom_panel(ctx);

        // Show streams
        egui::CentralPanel::default().show(ctx, |ui| {
            let total_width = ui.available_width();

            ui.columns(2, |cols| {

                // Stream from human pose estimator
                cols[0].vertical_centered(|ui| {
                    ui.heading("Human Pose Estimator");
                    if self.is_running {
                        self.display_videopose_stream(ui);
                    } else {
                        ui.add(
                            egui::Image::new(egui::include_image!("/home/nvidia/Repositories/actionq-core/actionq-videopose/images/image1.png"))
                                .rounding(5.0)
                        );
                    }
                });

                // TODO: Stream from reference video
                cols[1].vertical_centered(|ui| {
                    ui.heading("Reference Exercise");
                    ui.add(
                        egui::Image::new(egui::include_image!("/home/nvidia/Repositories/actionq-core/actionq-videopose/images/image2.png"))
                            .rounding(5.0)
                    );
                });
            });
        });
    }
}

fn main() -> Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([1280.0, 720.0]),
        ..Default::default()
    };

    eframe::run_native(
        "ActionQ",
        options,
        Box::new(|cc| {
            // This gives us image support:
            egui_extras::install_image_loaders(&cc.egui_ctx);
            Ok(Box::<MyApp>::default())
        }),
    )?;

    Ok(())
}