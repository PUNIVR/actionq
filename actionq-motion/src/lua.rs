use mlua::prelude::*;
use crate::widget::*;

/// Add all helper functions related to FSM behaviour
pub fn control_module(ctx: &Lua) -> LuaResult<LuaTable> {
    let state = ctx.create_table()?;

    // Stay on same state, optionally supports state metadata
    state.set(
        "stay",
        ctx.create_function(|lua, metadata: Option<LuaTable>| {
            let result = lua.create_table()?; // Create basic table
            result.set("next_state", LuaValue::Nil)?;
            result.set("metadata", metadata)?;
            Ok(result)
        })?,
    )?;

    // Change state, first argument is the next state, optionally supports metadata
    state.set(
        "step",
        ctx.create_function(|lua, (next_state, metadata): (String, Option<LuaTable>)| {
            //println!("next_state: {}", next_state);
            let result = lua.create_table()?; // Create basic table
            result.set("next_state", next_state)?;
            result.set("metadata", metadata)?;
            Ok(result)
        })?,
    )?;

    Ok(state)
}

/// Add all function used to draw widgets
pub fn draw_module(ctx: &Lua) -> LuaResult<LuaTable> {
    let draw = ctx.create_table()?;

    draw.set("circle",
        ctx.create_function(|lua, (pos, text): (LuaValue, Option<String>)| {
            let buffer: LuaTable = lua.globals().get("_widgets_buffer")?;
            buffer.push(lua.to_value(&Widget::Circle {
                text_offset: glam::Vec2::new(0.0, 0.0),
                position: lua.from_value(pos)?,
                text,
            })?)?;
            Ok(())
        })?
    )?;

    draw.set("segment",
        ctx.create_function(|lua, (from, to): (LuaValue, LuaValue)| {
            let buffer: LuaTable = lua.globals().get("_widgets_buffer")?;
            buffer.push(lua.to_value(&Widget::Segment {
                from: lua.from_value(from)?,
                to: lua.from_value(to)?,
            })?)?;
            Ok(())
        })?
    )?;

    draw.set("arc",
        ctx.create_function(|lua, (center, radius, angle, delta): (LuaValue, f32, f32, f32)| {
            let buffer: LuaTable = lua.globals().get("_widgets_buffer")?;
            buffer.push(lua.to_value(&Widget::Arc {
                center: lua.from_value(center)?,
                radius,
                angle,
                delta
            })?)?;
            Ok(())
        })?
    )?;

    draw.set("vline",
        ctx.create_function(|lua, x: f32| {
            let buffer: LuaTable = lua.globals().get("_widgets_buffer")?;
            buffer.push(lua.to_value(&Widget::VLine { x })?)?;
            Ok(())
        })?
    )?;

    draw.set("hline",
        ctx.create_function(|lua, y: f32| {
            let buffer: LuaTable = lua.globals().get("_widgets_buffer")?;
            buffer.push(lua.to_value(&Widget::HLine { y })?)?;
            Ok(())
        })?
    )?;

    Ok(draw)
}

/// Add all helper functions related to Vec3
pub fn math_module(ctx: &Lua) -> LuaResult<LuaTable> {
    let math = ctx.create_table()?;

    // Normalize
    math.set(
        "normv3",
        ctx.create_function(|lua, v: LuaValue| {
            let v: glam::Vec3 = lua.from_value(v)?;
            lua.to_value(&v.normalize())
        })?,
    )?;

    // Cross
    math.set(
        "crossv3",
        ctx.create_function(|lua, (a, b): (LuaValue, LuaValue)| {
            let a: glam::Vec3 = lua.from_value(a)?;
            let b: glam::Vec3 = lua.from_value(b)?;
            lua.to_value(&a.cross(b))
        })?,
    )?;

    // Dot
    math.set(
        "dotv3",
        ctx.create_function(|lua, (a, b): (LuaValue, LuaValue)| {
            let a: glam::Vec3 = lua.from_value(a)?;
            let b: glam::Vec3 = lua.from_value(b)?;
            lua.to_value(&a.dot(b))
        })?,
    )?;

    // Mul with float
    math.set(
        "mulfv3",
        ctx.create_function(|lua, (a, b): (LuaValue, f32)| {
            let a: glam::Vec3 = lua.from_value(a)?;
            lua.to_value((a * b).as_ref())
        })?,
    )?;

    // Mul
    math.set(
        "mulv3",
        ctx.create_function(|lua, (a, b): (LuaValue, LuaValue)| {
            let a: glam::Vec3 = lua.from_value(a)?;
            let b: glam::Vec3 = lua.from_value(b)?;
            lua.to_value((a * b).as_ref())
        })?,
    )?;

    // Subtract
    math.set(
        "subv3",
        ctx.create_function(|lua, (a, b): (LuaValue, LuaValue)| {
            let a: glam::Vec3 = lua.from_value(a)?;
            let b: glam::Vec3 = lua.from_value(b)?;
            lua.to_value((a - b).as_ref())
        })?,
    )?;

    // Mid
    math.set(
        "midv3",
        ctx.create_function(|lua, (a, b): (LuaValue, LuaValue)| {
            let a: glam::Vec3 = lua.from_value(a)?;
            let b: glam::Vec3 = lua.from_value(b)?;
            lua.to_value(((a + b) * 0.5).as_ref())
        })?,
    )?;
    math.set(
        "midv2",
        ctx.create_function(|lua, (a, b): (LuaValue, LuaValue)| {
            let a: glam::Vec2 = lua.from_value(a)?;
            let b: glam::Vec2 = lua.from_value(b)?;
            lua.to_value(((a + b) * 0.5).as_ref())
        })?,
    )?;

    // Simple angle between vectors
    math.set(
        "anglev3",
        ctx.create_function(|lua, (a, b): (LuaValue, LuaValue)| {
            let a: glam::Vec3 = lua.from_value(a)?;
            let b: glam::Vec3 = lua.from_value(b)?;
            let angle = a.dot(b).acos();
            Ok(angle.to_degrees())
        })?,
    )?;

    // Simple signed-angle between vectors
    math.set(
        "sanglev3",
        ctx.create_function(|lua, (a, b, n): (LuaValue, LuaValue, LuaValue)| {
            let a: glam::Vec3 = lua.from_value(a)?;
            let b: glam::Vec3 = lua.from_value(b)?;
            let n: glam::Vec3 = lua.from_value(n)?;

            let angle = a.dot(b).acos();
            let sign = (n.dot(a.cross(b))).signum();
            Ok(sign * angle.to_degrees())
        })?,
    )?;

    // Project a vector into a plane defined by a normal
    // NOTE: all input must be nornalized
    math.set(
        "projv3",
        ctx.create_function(|lua, (v, n): (LuaValue, LuaValue)| {
            let v: glam::Vec3 = lua.from_value(v)?;
            let n: glam::Vec3 = lua.from_value(n)?;
            lua.to_value((v - v.dot(n) * n).normalize().as_ref())
        })?,
    )?;

    // Crate the local reference system (sagittal, frontal and transverse normal vectors) of the body
    // It requires the shouldes and hips keypoints
    math.set(
        "body_planes",
        ctx.create_function(
            |lua, (ls, rs, lh, rh): (LuaValue, LuaValue, LuaValue, LuaValue)| {
                let ls: glam::Vec3 = lua.from_value(ls)?;
                let rs: glam::Vec3 = lua.from_value(rs)?;
                let lh: glam::Vec3 = lua.from_value(lh)?;
                let rh: glam::Vec3 = lua.from_value(rh)?;

                // Sagittal plane normal
                let sagittal = (ls - rs).normalize();

                // Transverse plane normal
                let ms = rs.midpoint(ls);
                let mh = rh.midpoint(lh);
                let transverse = (mh - ms).normalize();

                // Frontal plane
                let frontal = sagittal.cross(transverse).normalize();

                let result = lua.create_table()?;
                result.set("sagittal", lua.to_value(&sagittal)?)?;
                result.set("frontal", lua.to_value(&frontal)?)?;
                result.set("transverse", lua.to_value(&transverse)?)?;
                Ok(result)
            },
        )?,
    )?;

    // Inner angle without reference plane
    math.set(
        "inner_angle_3d",
        ctx.create_function(|lua, (a, m, b): (LuaValue, LuaValue, LuaValue)| {
            let a: glam::Vec3 = lua.from_value(a)?;
            let m: glam::Vec3 = lua.from_value(m)?;
            let b: glam::Vec3 = lua.from_value(b)?;
            let ma = (a - m).normalize(); // m -> a
            let mb = (b - m).normalize(); // m -> b

            let dot = ma.dot(mb);
            let crs = ma.cross(mb);
            let angle = crs.length().atan2(dot);

            Ok(angle.to_degrees())
        })?,
    )?;

    // Inner angle with reference plane
    math.set(
        "inner_angle_3d_aligned",
        ctx.create_function(
            |lua, (a, m, b, n): (LuaValue, LuaValue, LuaValue, LuaValue)| {
                let a: glam::Vec3 = lua.from_value(a)?;
                let m: glam::Vec3 = lua.from_value(m)?;
                let b: glam::Vec3 = lua.from_value(b)?;
                let n: glam::Vec3 = lua.from_value(n)?;
                let ma = a - m; // m -> a
                let mb = b - m; // m -> b

                let dot = ma.dot(mb).clamp(-1.0, 1.0);
                let angle = dot.acos(); // [0, pi]

                let cross = ma.cross(mb);
                let sign = if n.dot(cross) < 0.0 { -1.0 } else { 1.0 };
                let signed_angle = angle * sign;

                Ok(signed_angle.to_degrees())
            },
        )?,
    )?;

    Ok(math)
}
