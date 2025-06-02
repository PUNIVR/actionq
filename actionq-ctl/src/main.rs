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
        #[arg(default_value_t = String::from("VsBYJPdvik35pFYthJgx"))]
        patient: String,
        /// Optional id of the target patient (RSA mode)
        #[arg(long)]
        target_patient: Option<String>,
        /// Id of the exercise inside the database, the number of repetitions and optional
        /// parameters
        #[arg(value_parser = parse_exercise)]
        exercises: Vec<(String, u32, Vec<(String, f32)>)>,
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
        .ok_or_else(|| format!("invalid KEY:value: no `=` found in `{s}`"))?;
    Ok((s[..pos].parse()?, s[pos + 1..].parse()?))
}

fn parse_exercise(s: &str) -> Result<(String, u32, Vec<(String, f32)>), Box<dyn Error + Send + Sync + 'static>> {
    if let Some(mid) = s.find('@') {
        let exercise = &s[..mid];
        let params = &s[mid+1..];

        let mid = exercise.find(':').ok_or_else(|| format!("invalid exercise:reps, o ':' found in `{exercise}`"))?;
        let reps = exercise[mid + 1..].parse()?;
        let exid = exercise[..mid].parse()?;
       
        let mut p: Vec<(String, f32)> = Vec::new();
        for element in params.split(',') {
            let mid = element.find(':').ok_or_else(|| format!("invalid param:value, o ':' found in `{element}`"))?;
            p.push((
                element[..mid].parse()?,
                element[mid+1..].parse()?
            ));
        }

        Ok((exid, reps, p))

    } else {
        let mid = s.find(':').ok_or_else(|| format!("invalid exercise:reps, o ':' found in `{s}`"))?;
        Ok((s[..mid].parse()?, s[mid + 1..].parse()?, vec![]))
    }
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
            exercises,
        } => {
            dbg!(&exercises);

            let ctrl =
            sync_async!(rt, SessionCtrl::new(&patient, &args.project).await);
            sync_async!(rt, ctrl.run_multiple_exercises(target_patient, exercises).await);
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
