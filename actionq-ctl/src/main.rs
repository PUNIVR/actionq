use std::error::Error;
use crate::{firebase::{DatabaseCtrl, SessionCtrl}};
use clap::{Parser, Subcommand};

use actionq_motion::LuaExercise;

mod firebase;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    /// Google project id
    #[arg(long, default_value = "uvc-unisco")]
    project: String,
    /// In what scope are we working on
    #[command(subcommand)]
    context: ArgsContext,
}

#[derive(Subcommand)]
enum ArgsContext {
    /// Run singe exercise
    Exercise {
        /// Id of the root patient inside the database
        patient: String,
        /// Optional id of the target patient (RSA mode)
        #[arg(long)]
        target_patient: Option<String>,
        /// Id of the exercise inside the database
        exercise: String,
        /// Number of repetitions
        repetitions: u32,
        /// Runtime parameters
        #[arg(short = 'p', value_parser = parse_key_val::<String, f32>)]
        parameters: Vec<(String, f32)>
    },

    Reset {
        /// Id of the patient of the session to reset
        patient: String,
    },

    /// Insert data into the firebase database
    Database {
        /// What operation to do
        #[command(subcommand)]
        command: CmdsDatabase,
    },
}

#[derive(Subcommand)]
enum CmdsDatabase {
    InsertExercise {
        /// Name of the exercise
        name: String,
        /// Description of the exercise
        description: String,
        /// Path to the fsm json file
        fsm: String,
    },
}

/// Parse a single key-value pair
fn parse_key_val<T, U>(s: &str) -> Result<(T, U), Box<dyn Error + Send + Sync + 'static>>
where
    T: std::str::FromStr,
    T::Err: Error + Send + Sync + 'static,
    U: std::str::FromStr,
    U::Err: Error + Send + Sync + 'static,
{
    let pos = s
        .find('=')
        .ok_or_else(|| format!("invalid KEY=value: no `=` found in `{s}`"))?;
    Ok((s[..pos].parse()?, s[pos + 1..].parse()?))
}

fn main() {

    // Create single thread runtime
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_time()
        .enable_io()
        .build()
        .expect("unable to create single-thread tokio runtime");

    let args = Cli::parse();

    match args.context {
        ArgsContext::Exercise {
            patient,
            target_patient,
            exercise,
            repetitions,
            parameters,
        } => {
            let ctrl =
            sync_async!(rt, SessionCtrl::new(&patient, &args.project).await);

            let parameters = if parameters.len() != 0 {
                Some(parameters.into_iter().collect())
            }
            else {
                None
            };

            dbg!(&parameters);
            sync_async!(rt, ctrl.run_exercise(
                exercise, repetitions, target_patient, parameters).await);
        }
        ArgsContext::Reset { patient } => {
            let ctrl =
                sync_async!(rt, SessionCtrl::new(&patient, &args.project).await);
            sync_async!(rt, ctrl.session_stop().await);
        }
        ArgsContext::Database { command } => {
            let ctrl = sync_async!(rt, DatabaseCtrl::new(&args.project).await);
            match command {
                // Load fsm and gif into exercise definition
                CmdsDatabase::InsertExercise {
                    name,
                    description,
                    fsm,
                } => {
                    let fsm_data =
                        std::fs::read_to_string(&fsm).expect("unable to read fsm file");

                    // Create LuaExercise to extract metadata
                    let exercise = LuaExercise::from_string(fsm_data.clone(), "NONE".to_string(), "NONE".to_string(), 0, &[])
                        .expect("Unable to load FSM script");

                    sync_async!(
                        rt,
                        ctrl.add_exercise_definition(&name, &description, &fsm_data, Vec::from_iter(exercise.default_parameters))
                            .await
                    );
                }
            }
        }
    }
}
