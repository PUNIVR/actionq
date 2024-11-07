use std::time::{Duration, Instant};
use webp_animation::prelude::*;
use eframe::{egui, App, NativeOptions};
use egui::{Button, Rect, TextureOptions, Ui};
use tokio::sync::mpsc::{Sender, Receiver};

use videopose::Framebuffer;
use motion::{
    Warning, 
    Event, 
    ProgresState, 
    StateId
};

#[derive(Debug)]
pub enum Command {
    ExerciseStart {
        exercise_id: String,
    },
    Update {
        progress: ProgresState,
        frame: Framebuffer,
    },
    ExerciseEnd
}

#[derive(Debug)]
pub struct UiProxy(pub Sender<Command>);
impl UiProxy {
    // Show exercise on UI
    pub async fn exercise_show(&self, exercise_id: String) {
        self.0.send(Command::ExerciseStart { exercise_id } ).await.unwrap();
    }
    // Display framedata
    pub async fn update(&self, progress: ProgresState, frame: Framebuffer) {
        self.0.send(Command::Update{ progress, frame }).await.unwrap();
    }
    // Stop showing exercise
    pub async fn exercise_stop(&self) {
        self.0.send(Command::ExerciseEnd).await.unwrap();
    }
}

fn non_uniform_columns(ui: &mut Ui) -> Vec<Ui> {

    let width_part = ui.available_width() / 4.0;
    let top_left_a = ui.cursor().min;
    let top_left_b = top_left_a + egui::vec2(width_part, 0.0);
    let rect_a = egui::Rect::from_min_max(top_left_a, egui::pos2(top_left_a.x + width_part * 1.0, ui.max_rect().right_bottom().y));
    let rect_b = egui::Rect::from_min_max(top_left_b, egui::pos2(top_left_a.x + width_part * 3.0, ui.max_rect().right_bottom().y));

    let mut child_a = ui.new_child(egui::UiBuilder::new().max_rect(rect_a).layout(egui::Layout::top_down_justified(egui::Align::LEFT)));
    child_a.set_width(width_part * 1.0);

    let mut child_b = ui.new_child(egui::UiBuilder::new().max_rect(rect_b).layout(egui::Layout::top_down_justified(egui::Align::LEFT)));
    child_b.set_width(width_part * 3.0);

    vec![child_a, child_b]
}

struct ExerciseGif {
    frames: Vec<egui::ColorImage>,
    current_exercise_frame: usize,
    last_time: Instant
}

impl ExerciseGif {
    pub fn update_current_frame(&mut self, delay: Duration) -> egui::ColorImage {
        let elapsed = Instant::now() - self.last_time;
        if elapsed >= delay {
            self.current_exercise_frame += 1;
            self.last_time = Instant::now();

            // Wrap
            if self.current_exercise_frame >= self.frames.len() {
                self.current_exercise_frame = 0;
            }
        }

        self.frames[self.current_exercise_frame].clone()
    }
}

struct MyUi {
    is_running: bool,
    repetition_count: u32,

    exercise_gif: Option<ExerciseGif>,
    current_frame: Option<egui::ColorImage>,

    // Receive all cmd messages
    cmds: Receiver<Command>
}

impl MyUi {
    fn render_top_menu_bar(&mut self, ctx: &egui::Context) {
        let _ = egui::TopBottomPanel::top("header").show(ctx, |ui| {
            ui.horizontal_centered(|ui| {
                ui.heading("ActionQ");
            });
        });
    }
    
    fn render_viewport_stream(&mut self, ui: &mut Ui) {
        ui.vertical_centered(|ui| {

            ui.add_space(25.0);
            ui.heading("Viewport");
            ui.add_space(25.0);

            // Show video stream if available
            if let Some(frame) = &self.current_frame {
                // FIXME: cache this texture, this allocates a new one at each render
                let texture: egui::TextureHandle = ui.ctx().load_texture("stream-tex", frame.clone(), Default::default());
                ui.add(
                    egui::Image::from_texture(&texture)
                        .maintain_aspect_ratio(true)
                        .fit_to_fraction([0.9, 0.9].into())
                        .rounding(10.0)
                );
                ui.add_space(5.0);

                // Label for repetitions
                ui.add_sized([400.0, 100.0], 
                    egui::Label::new(&format!("RIPETIZIONI: {}", self.repetition_count)));
            }
        });
    }
    
    fn render_viewport_exercise(&mut self, ui: &mut Ui) {
        ui.vertical_centered(|ui| {

            ui.add_space(25.0);
            ui.heading("Exercise reference");
            ui.add_space(25.0);

            // Show gif if available
            if let Some(ref mut exercise_gif) = self.exercise_gif {
                let frame = exercise_gif.update_current_frame(Duration::from_millis(50 /* 20 FPS */));
                let texture: egui::TextureHandle = ui.ctx().load_texture("reference-tex", frame, Default::default());
                ui.add(
                    egui::Image::from_texture(&texture)
                        .maintain_aspect_ratio(true)
                        .fit_to_fraction([0.9, 0.9].into())
                        .rounding(10.0)
                );
                ui.add_space(5.0);
            }
        });
    }
    
    fn render_viewports(&mut self, ctx: &egui::Context) {
        let _ = egui::CentralPanel::default().show(ctx, |ui| {
            let mut cols = non_uniform_columns(ui);
            self.render_viewport_exercise(&mut cols[0]);
            self.render_viewport_stream(&mut cols[1]);
        });
    }
}

impl App for MyUi {
    #[tracing::instrument(skip_all, fields(cmd))]
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {

        // Get new messages if available
        if let Ok(cmd) = self.cmds.try_recv() {
            match cmd {
                Command::ExerciseStart { ref exercise_id } => {
                    tracing::trace!("start exercise display");
                    
                    // Load the gif
                    let exercise_data = std::fs::read(&format!("/home/nvidia/Repositories/actionq-core/exercises/{}.webp", exercise_id)).unwrap();
                    let exercise_frames = webp_animation::Decoder::new(&exercise_data).unwrap();
                    let exercise_frames: Vec<egui::ColorImage> = exercise_frames.into_iter()
                        .map(|f| { 
                            let size = [f.dimensions().0 as usize, f.dimensions().1 as usize];
                            egui::ColorImage::from_rgba_unmultiplied(size, f.data())
                        }).collect();

                    self.is_running = true;
                    self.repetition_count = 0;
                    self.exercise_gif = Some(ExerciseGif {
                        frames: exercise_frames,
                        current_exercise_frame: 0,
                        last_time: Instant::now()
                    });
                },
                Command::Update { progress, frame } => {
                    tracing::trace!("display single frame");

                    let frame = egui::ColorImage::from_rgb([frame.size.0 as usize, frame.size.1 as usize], &frame.storage);
                    self.current_frame = Some(frame);

                    // Increase repetition count if necessary
                    for event in &progress.events {
                        match event {
                            Event::RepetitionComplete => self.repetition_count += 1,
                            _ => { }
                        }
                    }
                },
                Command::ExerciseEnd => {
                    tracing::trace!("stop exercise display");

                    self.is_running = false;
                    self.exercise_gif = None;
                    self.current_frame = None;
                    self.repetition_count = 0;
                },
                _ => {}
            }
        }

        self.render_top_menu_bar(ctx);
        if self.is_running {
            self.render_viewports(ctx);
        }

        // Request update of the ui, we always want this
        ctx.request_repaint();
    }
}

fn eframe_options() -> NativeOptions {
    // TODO: add more options
    NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_fullscreen(true),
        ..Default::default()
    }
}

pub fn run_ui_blocking(rx: Receiver<Command>) {
    eframe::run_native(
        "ActionQ", 
        eframe_options(), 
        Box::new(|cc| {
            egui_extras::install_image_loaders(&cc.egui_ctx);
            Ok(Box::new(MyUi {
                is_running: false,
                repetition_count: 0,
                cmds: rx,
                exercise_gif: None,
                current_frame: None,
            }))
        }),
    ).expect("Unable to run eframe");
}