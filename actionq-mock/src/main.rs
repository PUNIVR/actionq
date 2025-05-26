/* Registering a new sample exercise
 * -
*/

use std::{
    collections::HashMap,
    error::Error,
    hash::Hash,
    io::{Cursor, Read, Write},
    net::{TcpListener, TcpStream},
    path::Path,
};

use clap::{Parser, Subcommand};
use glam::Vec2;
use image::codecs::jpeg::JpegEncoder;
use imageproc::image::{self, ExtendedColorType, ImageReader};
use ndarray::Array;
use nokhwa::pixel_format::RgbFormat;
use ort::{inputs, session::SessionOutputs, value::Tensor};
use serde::{Deserialize, Serialize};
use show_image::{ImageInfo, ImageView};

use motion::{LuaExercise, Metadata, Skeleton, StateEvent, Widget};
use tungstenite::{accept, WebSocket};
use yolo::Point2;

mod camera;
mod yolo;

/// Parse a single key-value pair
fn parse_key_val<T, U>(s: &str) -> Result<(T, U), Box<dyn Error + Send + Sync + 'static>>
where
    T: std::str::FromStr,
    T::Err: Error + Send + Sync + 'static,
    U: std::str::FromStr,
    U::Err: Error + Send + Sync + 'static,
{
    let pos = s
        .find(':')
        .ok_or_else(|| format!("invalid ID:REPS, no `:` found in `{s}`"))?;
    Ok((s[..pos].parse()?, s[pos + 1..].parse()?))
}

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Register a new exercise sample
    Register {
        /// Ids and repetitions target of the exercises to be registered
        #[arg(value_parser = parse_key_val::<String, u32>)]
        exercises: Vec<(String, u32)>,
        /// Output filename
        output: String,
    },

    /// Stream a registered exercise
    Stream {
        /// Filepath to the registered exercise
        binary: String,
        /// URL of the target server
        url: String,
    },
}

#[inline(always)]
fn skeleton_id_to_name(keypoints: &Vec<Point2>) -> Skeleton {
    let mut result = Skeleton::new();
    for (i, kp) in keypoints.iter().enumerate() {
        let name = match i {
            0 => "nose",
            1 => "left_eye",
            2 => "right_eye",
            3 => "left_ear",
            4 => "right_ear",
            5 => "left_shoulder",
            6 => "right_shoulder",
            7 => "left_elbow",
            8 => "right_elbow",
            9 => "left_wrist",
            10 => "right_wrist",
            11 => "left_hip",
            12 => "right_hip",
            13 => "left_knee",
            14 => "right_knee",
            15 => "left_ankle",
            16 => "right_ankle",
            _ => unimplemented!(),
        };

        if kp.c > 0.5 {
            result.insert(name.to_owned(), Vec2::new(kp.x, kp.y));
        }
    }
    result
}

fn compress_raw_rgb(raw_data: &[u8], width: u32, height: u32) -> Vec<u8> {
    let mut buffer = Cursor::new(Vec::new());
    let mut encoder = JpegEncoder::new_with_quality(&mut buffer, 80);
    encoder
        .encode(raw_data, width, height, ExtendedColorType::Rgb8)
        .unwrap();
    buffer.into_inner()
}

fn decompress_jpeg_to_rgb(jpeg_bytes: &[u8]) -> (Vec<u8>, u32, u32) {
    // Load the JPEG from memory
    let rgb_image = ImageReader::new(std::io::Cursor::new(jpeg_bytes))
        .with_guessed_format()
        .unwrap()
        .decode()
        .unwrap();

    // Convert to RGB8 (in case it's grayscale or CMYK, etc.)
    let rgb_image = rgb_image.to_rgb8();

    let (width, height) = rgb_image.dimensions();
    let raw_rgb = rgb_image.into_raw(); // this is a Vec<u8> with RGBRGB...

    (raw_rgb, width, height)
}

// What to store on disk
#[derive(Debug, Serialize, Deserialize)]
struct StorageFrame {
    pub frame: Vec<u8>,
    pub skeleton: Skeleton,
    pub metadata: Option<Metadata>,
    pub repetitions: u32,
}

// What to store for each exercise
#[derive(Debug, Serialize, Deserialize)]
struct Exercise {
    pub frames: Vec<StorageFrame>,
    pub repetitions_target: u32,
    pub id: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Storage {
    pub exercises: Vec<Exercise>,
    pub resolution: (u32, u32),
    pub frame_rate: u32,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum GUICommand {
    SessionStart {
        /// Number of exercises
        exercises_count: u32,
        /// Ids of the exercises in the session
        exercise_ids: Vec<String>,
        /// Resolution of the screen
        resolution: (u32, u32),
        /// Framerate of the camera
        frame_rate: u32,
    },

    ExerciseStart {
        /// Id of the current exercise
        exercise_id: String,
        /// Repetitions number to reach
        repetitions_target: u32,
    },

    ExerciseUpdate {
        /// FSM metadata
        metadata: Option<Metadata>,
        /// Current 2D skeleton
        skeleton: Skeleton,
        /// Current number of repetitions
        repetitions: u32,
        /// Current framebuffer
        frame: Vec<u8>,
    },

    ExerciseEnd,
    SessionEnd,
}

fn register(exercises: &[(String, u32)], output: &str) -> ort::Result<()> {
    let window = show_image::create_window("ActionQ", Default::default()).unwrap();

    // Load model, store in cache
    let model = yolo::load_model("models/yolov8m-pose.onnx")?;

    // Attach camera
    let mut camera = camera::initialize().unwrap();

    let (resolution, frame_rate, frame_format) = (
        camera.resolution(),
        camera.frame_rate(),
        camera.frame_format(),
    );

    println!("camera resolution: {}", resolution);
    println!("camera framerate: {}", frame_rate);
    println!("camera format: {}", frame_format);

    camera.open_stream().unwrap();

    let mut storage = Storage {
        exercises: vec![],
        resolution: (resolution.x(), resolution.y()),
        frame_rate,
    };

    for (exersise_id, repetitions_target) in exercises {
        // Current exercise lua script
        let mut exercise = LuaExercise::from_file(
            &Path::new("exercises").join(&format!("{}.lua", exersise_id)),
            "<exercise>".to_owned(),
            "<desc>".to_owned(),
            *repetitions_target,
        )
        .unwrap();

        // Current exercise storage
        let mut exercise_storage = Exercise {
            repetitions_target: *repetitions_target,
            id: exersise_id.to_owned(),
            frames: vec![],
        };

        let mut repetitions = 0;
        loop {
            let frame = camera.frame().unwrap();
            //println!("frame");

            // Send frame to ONNX runtime
            let mut frame = frame.decode_image::<RgbFormat>().unwrap();
            let mut input = Array::zeros((1, 3, 640, 640));
            for pixel in frame.enumerate_pixels() {
                let x = pixel.0 as _;
                let y = pixel.1 as _;
                let [r, g, b] = pixel.2 .0;
                input[[0, 0, y, x]] = (r as f32) / 255.;
                input[[0, 1, y, x]] = (g as f32) / 255.;
                input[[0, 2, y, x]] = (b as f32) / 255.;
            }

            // Compress frame into JPEG for later storage
            let frame_jpeg = frame.clone();
            let frame_jpeg = compress_raw_rgb(&frame_jpeg.into_raw(), 640, 480);

            // Run YOLOv8 inference
            let output: SessionOutputs = model.run(inputs! {
                "images" => Tensor::from_array(input)?
            }?)?;

            let mut output = output["output0"].try_extract_tensor::<f32>()?.into_owned();
            let y = yolo::parse_output(&mut output);
            let keypoints = skeleton_id_to_name(&y[0].1);
            //dbg!(keypoints.get("nose"));

            for (_name, kp) in &keypoints {
                imageproc::drawing::draw_filled_circle_mut(
                    &mut frame,
                    (kp.x as i32, kp.y as i32),
                    4,
                    image::Rgb([255, 255, 255]),
                );
            }

            // Evaluate and store
            let (completed, result) = exercise.process(&keypoints).unwrap();

            let mut metadata = None;
            if let Some(result) = &result {
                metadata = Some(result.metadata.clone());
                if result
                    .metadata
                    .events
                    .iter()
                    .any(|x| *x == StateEvent::Repetition)
                {
                    repetitions += 1;
                }

                for widget in &result.metadata.widgets {
                    if let Widget::Arc {
                        center,
                        radius,
                        from,
                        to,
                    } = widget
                    {
                        let angle_span = (to - from).abs() % 360.0;
                        let angle_step = angle_span / 5.0;

                        let clockwise = to < from;

                        let mut points: Vec<(f32, f32)> = vec![];
                        for i in 0..6 {
                            let angle = from
                                + if clockwise {
                                    -angle_step * i as f32
                                } else {
                                    angle_step * i as f32
                                };
                            let angle = (angle % 360.0).to_radians();

                            let x = center.x + radius * angle.cos();
                            let y = center.y - radius * angle.sin();

                            points.push((x, y));
                        }

                        for i in 0..(points.len() - 1) {
                            imageproc::drawing::draw_line_segment_mut(
                                &mut frame,
                                points[i],
                                points[i + 1],
                                image::Rgb([255, 255, 255]),
                            );
                        }
                    }
                }
            }

            exercise_storage.frames.push(StorageFrame {
                frame: frame_jpeg,
                skeleton: keypoints,
                metadata: metadata,
                repetitions,
            });

            let image = ImageView::new(ImageInfo::rgb8(640, 480), &frame);
            window.set_image("inference", image).unwrap();

            //let memory: usize = storage.iter().map(|e| e.frame.len()).sum();
            //println!("memory: {} MB", memory as f32 / 1e6);

            if completed {
                break;
            }
        }

        // Add exersise storage to final binary
        storage.exercises.push(exercise_storage);
    }
    camera.stop_stream().unwrap();

    // Save exercise to disk
    let output_path = Path::new("samples").join(&format!("{}.bin", output));
    let bytes = bincode::serialize(&storage).unwrap();
    let mut file = std::fs::File::create(&output_path).unwrap();
    file.write_all(&bytes).unwrap();

    Ok(())
}

fn send_command(stream: &mut WebSocket<TcpStream>, cmd: GUICommand) {
    let buffer = serde_json::to_string(&cmd)
        .expect(&format!("unable to serialize command to JSON: {:?}", cmd));

    stream
        .send(tungstenite::Message::Text(buffer))
        .expect("unable to stream initial command");
}

fn stream(binary_filepath: &str, url: &str) {
    // Read file content
    let filepath = Path::new("samples").join(&binary_filepath);
    println!("reading {:?}", filepath);

    let mut file = std::fs::File::open(&filepath).expect("no file found");
    let metadata = std::fs::metadata(&filepath).expect("unable to read metadata");
    let mut bytes = vec![0; metadata.len() as usize];
    file.read(&mut bytes).expect("buffer overflow");

    // Deserialize to Storage object
    let storage: Storage = bincode::deserialize(&bytes).expect("unable to deserialize");

    // Create websocket for data-stream
    let listener = TcpListener::bind(url).unwrap();
    println!("webSocket server listening on ws://{} ...", url);

    // Wait for a connection
    while let Ok((stream, _)) = listener.accept() {
        let mut ws_stream = accept(stream).expect("unable to accept websocket connection");
        println!("connection accepted! streaming data...");

        send_command(
            &mut ws_stream,
            GUICommand::SessionStart {
                exercises_count: storage.exercises.len() as u32,
                exercise_ids: storage.exercises.iter().map(|e| e.id.clone()).collect(),
                resolution: storage.resolution,
                frame_rate: storage.frame_rate,
            },
        );

        // Loop over all exercises
        for exercise in &storage.exercises {
            println!("streaming of exercise {} in progress", exercise.id);

            send_command(
                &mut ws_stream,
                GUICommand::ExerciseStart {
                    exercise_id: exercise.id.clone(),
                    repetitions_target: exercise.repetitions_target,
                },
            );

            // Send all frames
            for frame in &exercise.frames {
                send_command(
                    &mut ws_stream,
                    GUICommand::ExerciseUpdate {
                        metadata: frame.metadata.clone(),
                        skeleton: frame.skeleton.clone(),
                        repetitions: frame.repetitions,
                        frame: frame.frame.clone(),
                    },
                );

                // Simulate framerate
                let delay = 1000.0 / storage.frame_rate as f32;
                std::thread::sleep(std::time::Duration::from_millis(delay as u64));
            }

            send_command(&mut ws_stream, GUICommand::ExerciseEnd);
            println!("streaming of exercise {} complete!", exercise.id);

            // Simulate wait between exercises
            println!("waiting between exercises...");
            std::thread::sleep(std::time::Duration::from_millis(2000));
        }

        send_command(&mut ws_stream, GUICommand::SessionEnd);
        println!("streaming complete!");
    }
}

#[show_image::main]
fn main() {
    let args = Args::parse();
    match &args.command {
        Commands::Register { exercises, output } => register(exercises, output).unwrap(),
        Commands::Stream { binary, url } => stream(binary, url),
    }
}
