use serde::{Serialize, Deserialize};

use videopose::{FrameData, Framebuffer};
use motion::{
    MotionAnalyzer, 
    Exercise, 
    Transition, 
    Warning, 
    StateId, 
    GenControlFactors, 
    ControlFactorMap, 
    MappedCondition, 
    Condition,
    ProgresState,
    Event
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonState {
    /// Unique name
    name: String,
    /// All transitions to other states
    transitions: Vec<Transition>,
    /// Warnings active only in this state
    warnings: Vec<Warning>,
}

/// An exercise specified using json
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonExercise {
    /// All the possible states
    states: Vec<JsonState>,
    /// Initial state the analyzer will start in
    initial_state: String,
    /// Global warnings active in all states
    warnings: Vec<Warning>,
}

impl JsonExercise {
    
    /// A very simple example exercise 
    pub fn simple() -> Self {
        Self {
            states: vec![
                JsonState {
                    name: "start".into(),
                    warnings: vec![],
                    transitions: vec![
                        Transition {
                            to: "down".into(),
                            emit: vec![],
                            conditions: vec![
                                motion::MappedCondition {
                                    control_factor: "arm_angle_l".into(),
                                    condition: motion::Condition::InRange {
                                        range: (0.0..30.0),
                                    },
                                },
                                motion::MappedCondition {
                                    control_factor: "arm_angle_r".into(),
                                    condition: motion::Condition::InRange {
                                        range: (0.0..30.0),
                                    },
                                },
                            ],
                        },
                    ],
                },
                JsonState {
                    name: "up".into(),
                    warnings: vec![],
                    transitions: vec![
                        Transition {
                            to: "down".into(),
                            emit: vec![motion::Event::RepetitionComplete],
                            conditions: vec![
                                motion::MappedCondition {
                                    control_factor: "arm_angle_l".into(),
                                    condition: motion::Condition::InRange {
                                        range: (0.0..30.0),
                                    },
                                },
                                motion::MappedCondition {
                                    control_factor: "arm_angle_r".into(),
                                    condition: motion::Condition::InRange {
                                        range: (0.0..30.0),
                                    },
                                },
                            ],
                        },
                    ],
                },
                JsonState {
                    name: "down".into(),
                    warnings: vec![],
                    transitions: vec![
                        motion::Transition {
                            to: "up".into(),
                            emit: vec![],
                            conditions: vec![
                                motion::MappedCondition {
                                    control_factor: "arm_angle_l".into(),
                                    condition: motion::Condition::InRange {
                                        range: (45.0..90.0),
                                    },
                                },
                                motion::MappedCondition {
                                    control_factor: "arm_angle_r".into(),
                                    condition: motion::Condition::InRange {
                                        range: (45.0..90.0),
                                    },
                                },
                            ],
                        },
                    ],
                },
            ],
            initial_state: "start".into(),
            warnings: vec![]
        }
    }

    pub fn from_file<S: AsRef<str>>(filepath: S) -> Self {
        let string = std::fs::read_to_string(filepath.as_ref()).unwrap();
        Self::from_str(string)
    }

    pub fn from_str<S: AsRef<str>>(string: S) -> Self {
        serde_json::from_str::<Self>(string.as_ref()).unwrap()
    }

    pub fn to_string(&self) -> String {
        serde_json::to_string(&self)
            .expect("unable to serialize fsm into string")
    }
}

impl Exercise for JsonExercise {

    fn states(&self) -> Vec<String> {
        self.states.iter()
            .map(|s| s.name.clone())
            .collect()
    }

    fn initial_state(&self) -> String {
        self.initial_state.clone()
    }

    fn state_transitions(&self, state: &String) -> Vec<Transition> {
        self.states.iter()
            .filter(|s| s.name == *state)
            .flat_map(|s| s.transitions.clone())
            .collect()
    }

    fn state_warnings(&self, state: &StateId) -> Vec<Warning> {
        self.states.iter()
            .filter(|s| s.name == *state)
            .flat_map(|s| s.warnings.clone())
            .collect()
    }

    fn global_warnings(&self) -> Vec<Warning> {
        self.warnings.clone()
    }
}