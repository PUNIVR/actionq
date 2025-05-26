use serde::{Deserialize, Serialize};
use mlua::prelude::*;

/// Custom widget to draw on screen over the video stream.
/// Used to help the patient reach the exercise goal.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "widget")]
pub enum Widget {
    /// Small circle
    Circle {
        /// What text to show near or inside the circle
        text: Option<String>,
        /// At what offset from the position the text should be rendered
        text_offset: glam::Vec2,
        /// At what position of the screen the circle should be rendered
        position: glam::Vec2,
    },
    /// Line segment from A to B
    Segment {
        /// At what position of the screen the segment starts
        from: glam::Vec2,
        /// At what position of the screen the segment ends
        to: glam::Vec2,
    },
    /// A circular segment
    Arc {
        /// Center of the arc
        center: glam::Vec2,
        /// Distance from the center
        radius: f32,
        /// Starting angle anti-clockwise
        angle: f32,
        /// Delta angle anti-clockwise
        delta: f32,
    },
    /// Vertical line
    VLine { x: f32 },
    /// Horizontal line
    HLine { y: f32 },
}

/// Create a Widget from a Lua table
impl FromLua for Widget {
    fn from_lua(value: LuaValue, lua: &Lua) -> LuaResult<Self> {
        if let LuaValue::Table(t) = value {
            let widget_type: String = t.get("widget")?;
            return Ok(match widget_type.as_str() {
                "circle" => {
                    let text_offset = glam::Vec2::new(0.0, 0.0);
                    Widget::Circle {
                        position: lua.from_value(t.get("position")?)?,
                        text_offset: text_offset,
                        text: t.get("text")?,
                    }
                }
                "segment" => {
                    Widget::Segment {
                        from: lua.from_value(t.get("from")?)?,
                        to: lua.from_value(t.get("to")?)?,
                    }
                }
                "hline" => Widget::HLine { y: t.get("y")? },
                "vline" => Widget::VLine { x: t.get("x")? },
                _ => unimplemented!(),
            });
        }
        unimplemented!()
    }
}

