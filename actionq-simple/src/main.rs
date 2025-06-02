use actionq_motion::{LuaExercise, Widget};
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

#[macroquad::main(configuration())]
async fn main() {

    ctrlc::set_handler(move || {
        actionq_zed::finish();
        std::process::exit(0);
    }).expect("unable to set CTRL-C handler");

    let mut exercise = LuaExercise::from_file(
        std::path::Path::new("exercises/demo/alzata_3D.lua"), 
        "curl".to_string(), 
        "".to_string(), 
        2,
        // Custom parameters
        &[
            ("dist_target".to_string(), 25.0)
        ]
    ).expect("unable to load exercise");
    dbg!(&exercise);

    actionq_zed::initialize();

    let mut running = true;
    while running {
        clear_background(BLACK);

        let capture = actionq_zed::extract();
        let result = exercise.process(&capture);
        if result.is_err() {
            actionq_zed::finish();
        }

        let (finish, state) = result.unwrap();
        if let Some(state) = state {
            for widget in &state.metadata.widgets {
                match widget {
                    Widget::Circle { position, .. } => {
                        draw_circle(position.x, position.y, 10.0, WHITE);
                    },
                    Widget::Segment { from, to } => {
                        draw_line(from.x, from.y, to.x, to.y, 10.0, WHITE);
                    },
                    Widget::Arc { center, radius, angle, delta } => {
                        draw_arc(center.x, center.y, 30, *radius, *angle, 5.0, *delta, RED);
                    },
                    Widget::VLine { x } => {
                        draw_line(*x, 0.0, *x, screen_height(), 10.0, WHITE);
                    },
                    Widget::HLine { y } => {
                        draw_line(0.0, *y, screen_width(), *y, 10.0, WHITE);
                    }
                    _ => {}
                }
            }
        }

        if finish { break; }
        next_frame().await
    }
    actionq_zed::finish();
}
