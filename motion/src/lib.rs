//! A motion analyzer is a FSM that can emit events
//! regarding the execution of an exercise.
//! It is composed of multiple states that represent
//! the parts that make up a single exercise, each
//! state has a number of conditions that must be
//! kept by the subject in order to consider the repetition
//! valid, and a set of transition conditions to move
//! to a different state.

use serde::{Deserialize, Serialize};
use serde_yaml;
use std::{
    collections::{HashMap, HashSet},
    fs,
    ops::{Range, RangeBounds},
};

pub type StateId = String;
pub type CFId = String;

/// Interesting characteristics of the pose
pub type ControlFactor = f32;
pub type ControlFactorMap = HashMap<CFId, ControlFactor>;

/// All possible conditions
#[derive(Clone, Debug, Serialize, Deserialize)]
enum Condition {
    /// Check if CF is in range
    InRange { range: Range<f32> },
    /// Check if CF is not in range
    NotInRange { range: Range<f32> },
    /// Check if CF is greater than some value
    GreaterThanValue { value: f32 },
    /// Check if CF is less that some value
    LessThanValue { value: f32 },
}

/// Condition associated with a control factor
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MappedCondition {
    control_factor: CFId,
    condition: Condition,
}

impl MappedCondition {
    pub fn is_valid(&self, cfs: &ControlFactorMap) -> bool {
        if let Some(cf) = cfs.get(&self.control_factor) {
            match &self.condition {
                Condition::InRange { range } => range.contains(&cf),
                Condition::NotInRange { range } => !range.contains(&cf),
                Condition::GreaterThanValue { value } => cf >= value,
                Condition::LessThanValue { value } => cf <= value,
            }
        } else {
            false
        }
    }
}

/// Used to inform the patient of non-optimal joint pose
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Warning {
    name: String,
    description: String,
    condition: MappedCondition,
}

impl Warning {
    pub fn is_valid(&self, cfs: &ControlFactorMap) -> bool {
        self.condition.is_valid(cfs)
    }
}

/// Exercise's events, such a single repetition completed
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Event {
    RepetitionComplete,
}

/// TODO
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Transition {
    pub conditions: Vec<MappedCondition>,
    pub to: String,
    pub emit: Vec<Event>,
}

pub struct TransitionState {
    pub transition: Transition,
    pub progress: f32,
}

/// Types that implement this trait are single
/// exercises, they are composed by multiple states
/// and transitions between states.
pub trait Exercise {
    /// Returns the set of all states
    fn states(&self) -> Vec<String>;
    /// Returns the initial state
    fn initial_state(&self) -> String;

    /// Returns the transitions from a state
    fn state_transitions(&self, state: &String) -> Vec<Transition>;
    /// Returns the warning of a state
    fn state_warnings(&self, state: &StateId) -> Vec<Warning>;

    /// Returns all the global warnings
    fn global_warnings(&self) -> Vec<Warning>;
}

/// Types that implement this trait can generate
/// a collection controll factors used to analyze the
/// movement execution, such as distances or angles
/// between joints.
pub trait GenControlFactors {
    fn control_factors(&self) -> ControlFactorMap;
}

struct WarningProgress {
    warning: Warning,
    threshold: f32,
    progress: f32,
}

pub struct MotionAnalyzer<E> {
    state_id: StateId,
    warnings_progress: Vec<WarningProgress>,
    exercise: E,
}

#[derive(Debug)]
pub struct ProgresState {
    current_state: StateId,
    warnings: Vec<Warning>,
    events: Vec<Event>,
}

impl<E> MotionAnalyzer<E>
where
    E: Exercise,
{
    pub fn new(exercise: E) -> Self {
        let initial_state = exercise.initial_state();
        let warnings_progress = exercise
            .state_warnings(&initial_state)
            .into_iter()
            .map(|w| WarningProgress {
                warning: w,
                threshold: 100.0,
                progress: 0.0,
            })
            .collect();

        Self {
            state_id: initial_state,
            warnings_progress,
            exercise,
        }
    }

    fn change_current_state(&mut self, state: StateId) {
        self.state_id = state;
        self.warnings_progress = self
            .exercise
            .state_warnings(&self.state_id)
            .into_iter()
            .map(|w| WarningProgress {
                warning: w,
                threshold: 100.0,
                progress: 0.0,
            })
            .collect();
    }

    pub fn progress<G>(&mut self, deltatime: f32, input: G) -> ProgresState
    where
        G: GenControlFactors,
    {
        let mut result_events: Vec<Event> = vec![];

        // Get new data, create control factors, check all transition, add to duration if all
        // conditions are satisfied.

        // Check all transitions
        let cfs = input.control_factors();
        let transitions_to_check = self.exercise.state_transitions(&self.state_id);
        for trans in &transitions_to_check {
            // Check all conditions for this transition
            let mut can_follow = true;
            for condition in &trans.conditions {
                can_follow &= condition.is_valid(&cfs);
            }

            // emit all events and change state is possible
            if can_follow {
                self.change_current_state(trans.to.clone());
                result_events = trans.emit.clone();
                break;
            }
        }

        // Check all warnings
        let mut result_warnings = vec![];
        let warnings_to_check = &mut self.warnings_progress;
        for warn in warnings_to_check {
            warn.progress = if warn.warning.is_valid(&cfs) {
                warn.progress + deltatime
            } else {
                0.0
            };

            // Add valid warning to output
            if warn.progress >= warn.threshold {
                result_warnings.push(warn.warning.clone());
            }
        }

        ProgresState {
            current_state: self.state_id.clone(),
            warnings: result_warnings,
            events: result_events,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone, Serialize, Deserialize)]
    struct JsonExercise {
        states: Vec<String>,
        initial_state: String,
        state_warnings: HashMap<StateId, Vec<Warning>>,
        state_transitions: HashMap<StateId, Vec<Transition>>,
        global_warnings: Option<Vec<Warning>>,
    }

    impl Exercise for JsonExercise {
        fn states(&self) -> Vec<String> {
            self.states.clone()
        }

        fn initial_state(&self) -> String {
            self.initial_state.clone()
        }

        fn state_transitions(&self, state: &String) -> Vec<Transition> {
            if let Some(transitions) = self.state_transitions.get(state) {
                transitions.clone()
            } else {
                vec![]
            }
        }

        fn state_warnings(&self, state: &StateId) -> Vec<Warning> {
            if let Some(warnings) = self.state_warnings.get(state) {
                warnings.clone()
            } else {
                vec![]
            }
        }

        fn global_warnings(&self) -> Vec<Warning> {
            todo!()
        }
    }

    struct SampleExercise;
    impl Exercise for SampleExercise {
        /// All the states of this exercise
        fn states(&self) -> Vec<String> {
            vec![
                "setup".into(),
                "high_distance".into(),
                "small_distance".into(),
            ]
        }

        /// Get the initial state
        fn initial_state(&self) -> String {
            "setup".into()
        }

        fn state_warnings(&self, state: &StateId) -> Vec<Warning> {
            match state.as_str() {
                "setup" => vec![Warning {
                    name: "incorrect_initial_state_1".into(),
                    description: "blah blah blah".into(),
                    condition: MappedCondition {
                        control_factor: "feet_distance".into(),
                        condition: Condition::InRange {
                            range: (-10.0..10.0),
                        },
                    },
                }],
                "small_distance" => vec![Warning {
                    name: "feet_too_clone_for_confort".into(),
                    description: "blah blah blah".into(),
                    condition: MappedCondition {
                        control_factor: "feet_distance".into(),
                        condition: Condition::InRange { range: (-6.0..6.0) },
                    },
                }],
                "high_distance" => vec![Warning {
                    name: "feet_too_far_away".into(),
                    description: "blah blah blah".into(),
                    condition: MappedCondition {
                        control_factor: "feet_distance".into(),
                        condition: Condition::NotInRange {
                            range: (-12.0..12.0),
                        },
                    },
                }],
                _ => unimplemented!(),
            }
        }

        /// All the transitions from each state
        fn state_transitions(&self, state: &String) -> Vec<Transition> {
            match state.as_ref() {
                "setup" => vec![
                    // This requires the user to keep the initial position for some time
                    Transition {
                        to: "high_distance".into(),
                        emit: vec![],
                        conditions: vec![
                            // Feet must be separated
                            MappedCondition {
                                control_factor: "feet_distance".into(),
                                condition: Condition::NotInRange {
                                    range: (-10.0..10.0),
                                },
                            },
                        ],
                    },
                ],
                "high_distance" => vec![
                    // This requires the user to move togheter the feet
                    Transition {
                        to: "small_distance".into(),
                        emit: vec![],
                        conditions: vec![
                            // Feet must be close
                            MappedCondition {
                                control_factor: "feet_distance".into(),
                                condition: Condition::InRange {
                                    range: (-10.0..10.0),
                                },
                            },
                        ],
                    },
                ],
                "small_distance" => vec![
                    // This requires the feet to move away from eachother
                    Transition {
                        to: "high_distance".into(),
                        emit: vec![Event::RepetitionComplete],
                        conditions: vec![
                            // Feet must be far away
                            MappedCondition {
                                control_factor: "feet_distance".into(),
                                condition: Condition::NotInRange {
                                    range: (-10.0..10.0),
                                },
                            },
                        ],
                    },
                ],
                _ => unimplemented!(),
            }
        }

        fn global_warnings(&self) -> Vec<Warning> {
            vec![]
        }
    }

    type TestPoseData = f32;
    impl GenControlFactors for TestPoseData {
        fn control_factors(&self) -> ControlFactorMap {
            HashMap::from([("feet_distance".into(), *self)])
        }
    }

    #[test]
    fn serialize() {
        let base_exercise = SampleExercise;
        let json_exercise = JsonExercise {
            states: base_exercise.states(),
            initial_state: base_exercise.initial_state(),
            global_warnings: None,
            state_transitions: base_exercise
                .states()
                .iter()
                .map(|s| (s.clone(), base_exercise.state_transitions(s)))
                .collect(),
            state_warnings: base_exercise
                .states()
                .iter()
                .map(|s| (s.clone(), base_exercise.state_warnings(s)))
                .collect(),
        };

        let data = serde_yaml::to_string(&json_exercise).unwrap();
        println!("{}", data);
        std::fs::write("exercise.yaml", data).unwrap();

        check_engine(json_exercise);
    }

    #[test]
    fn deserialize() {
        let string = fs::read_to_string("exercise.yaml").unwrap();
        let exercise = serde_yaml::from_str::<JsonExercise>(&string).unwrap();
        print!("{:?}", exercise);
        check_engine(exercise);
    }

    fn check_engine<E: Exercise>(ex: E) {
        let poses = vec![
            3.0, -1.0, 5.0, 8.0, // During setup, not ready
            11.0, 11.5, 12.0, // During setup, becomming ready
            8.0, 5.0, // To small distance
            14.0, 16.0, // To big distance
        ];

        let mut analyzer = MotionAnalyzer::new(ex);
        for pose in poses {
            let deltatime = 100.0;
            let progress = analyzer.progress(deltatime, pose);

            println!("=====================================");
            println!("current feet_distance: {}", pose);
            println!("{:?}", progress);
        }
    }

    #[test]
    fn it_works_tm() {
        check_engine(SampleExercise);
    }
}
