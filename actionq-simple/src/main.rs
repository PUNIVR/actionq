use actionq_motion::LuaExercise;
use actionq_common::*;

use std::collections::HashMap;
use macroquad::prelude::*;
use ctrlc;

fn configuration() -> Conf {
    Conf {
        window_title: "ActionQ".to_owned(),
        window_width: 1280,
        window_height: 720,
        ..Default::default()
    }
}

fn signed_angle(v1: (f32, f32), v2: (f32, f32)) -> f32 {
    let dot = v1.0 * v2.0 + v1.1 * v2.1;
    let det = v1.0 * v2.1 - v1.1 * v2.0;
    return det.atan2(dot);
}

fn draw(capture: &CaptureData) {
    let pose_2d = &capture.pose.kp2d;

    let rs = pose_2d["right_shoulder"];
    draw_circle(rs.x, rs.y, 10.0, RED);
    let rw = pose_2d["right_wrist"];
    draw_circle(rw.x, rw.y, 10.0, RED);
    let re = pose_2d["right_elbow"];
    draw_circle(re.x, re.y, 10.0, RED);
    let ls = pose_2d["left_shoulder"];
    draw_circle(ls.x, ls.y, 10.0, RED);
    let lw = pose_2d["left_wrist"];
    draw_circle(lw.x, lw.y, 10.0, RED);
    let le = pose_2d["left_elbow"];
    draw_circle(le.x, le.y, 10.0, RED);

    draw_line(rs.x, rs.y, re.x, re.y, 10.0, RED);
    draw_line(rw.x, rw.y, re.x, re.y, 10.0, RED);
    draw_line(ls.x, ls.y, le.x, le.y, 10.0, RED);
    draw_line(lw.x, lw.y, le.x, le.y, 10.0, RED);
}

#[macroquad::main(configuration())]
async fn main() {

    ctrlc::set_handler(move || {
        actionq_zed::finish();
        std::process::exit(0);
    }).expect("unable to set CTRL-C handler");

    let mut exercise = LuaExercise::from_file(
        std::path::Path::new("exercises/ginocchia_3D.lua"), 
        "curl".to_string(), 
        "".to_string(), 
        2,
        // Custom parameters
        &[
            ("dist_target".to_string(), 25.0)
        ]
    ).expect("unable to load exercise");
    dbg!(&exercise);
    //return;

    actionq_zed::initialize();

    let mut running = true;
    while running {
        clear_background(BLACK);

        let capture = actionq_zed::extract();
        draw(&capture);

        let result = exercise.process(&capture);
        if result.is_err() {
            actionq_zed::finish();
        }

        let (finish, state) = result.unwrap();

        if finish { break; }
        next_frame().await
    }
    actionq_zed::finish();
}
