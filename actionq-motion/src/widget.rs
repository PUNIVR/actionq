use mlua::prelude::*;
use crate::common::*;

/// Custom widget to draw on screen over the video stream.
/// Used to help the patient reach the exercise goal.
#[derive(Debug, Clone)]
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
        to: glam::Vec2
    },
    /// Vertical line
    VLine { x: f32 },
    /// Horizontal line
    HLine { y: f32 }
}

/// Create a Widget from a Lua table
impl FromLua for Widget {
    fn from_lua(value: LuaValue, _: &Lua) -> LuaResult<Self> {
        if let LuaValue::Table(t) = value {
            let widget_type: String = t.get("widget")?;
            return Ok(match widget_type.as_str() {
                "circle" => {

                    // From LuaVec2 to Vec2 with optionally None
                    let position: LuaVec2 = t.get("position")?;
                    
                    // BUG: this is Nil
                    //let text_offset: LuaVec2 = t.get("text_offset")
                    //    .unwrap_or(LuaVec2(Vec2::new(0.0, 0.0)));

                    let text_offset = glam::Vec2::new(0.0, 0.0);

                    Widget::Circle {
                        position: position.0,
                        text_offset: text_offset,
                        text: t.get("text").ok(),
                    }
                },
                "segment" => {
                    
                    let from: LuaVec2 = t.get("from")?;
                    let to: LuaVec2 = t.get("to")?;
                    
                    Widget::Segment { 
                        from: from.0, 
                        to: to.0 
                    }
                },
                "hline" => Widget::HLine { y: t.get("y")? },
                "vline" => Widget::VLine { x: t.get("x")? },
                _ => unimplemented!()
            })
        }
        unimplemented!()
    }
}