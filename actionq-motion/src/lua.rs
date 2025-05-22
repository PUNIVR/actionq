use crate::common::*;
use mlua::prelude::*;

/// Add all helper functions related to FSM behaviour
pub fn add_functions_control(ctx: &Lua) -> LuaResult<()> {
    // Stay on same state, optionally supports state metadata
    ctx.globals().set(
        "stay",
        ctx.create_function(|lua, metadata: Option<LuaTable>| {
            let result = lua.create_table()?; // Create basic table
            result.set("next_state", LuaValue::Nil)?;
            result.set("metadata", metadata)?;
            Ok(result)
        })?,
    )?;

    // Change state, first argument is the next state, optionally supports metadata
    ctx.globals().set(
        "step",
        ctx.create_function(|lua, (next_state, metadata): (String, Option<LuaTable>)| {
            //println!("next_state: {}", next_state);
            let result = lua.create_table()?; // Create basic table
            result.set("next_state", next_state)?;
            result.set("metadata", metadata)?;
            Ok(result)
        })?,
    )?;

    Ok(())
}

/// Add all helper functions related to Vec3
pub fn add_functions_vec3(ctx: &Lua) -> LuaResult<()> {
    // Normalize
    ctx.globals().set(
        "normv3",
        ctx.create_function(|lua, v: LuaValue| {
            let v: glam::Vec3 = lua.from_value(v)?;
            lua.to_value(&v.normalize())
        })?,
    )?;

    // Cross
    ctx.globals().set(
        "crossv3",
        ctx.create_function(|lua, (a, b): (LuaValue, LuaValue)| {
            let a: glam::Vec3 = lua.from_value(a)?;
            let b: glam::Vec3 = lua.from_value(b)?;
            lua.to_value(&a.cross(b))
        })?,
    )?;

    // Dot
    ctx.globals().set(
        "dotv3",
        ctx.create_function(|lua, (a, b): (LuaValue, LuaValue)| {
            let a: glam::Vec3 = lua.from_value(a)?;
            let b: glam::Vec3 = lua.from_value(b)?;
            lua.to_value(&a.dot(b))
        })?,
    )?;

    // Mul with float
    ctx.globals().set(
        "mulfv3",
        ctx.create_function(|lua, (a, b): (LuaValue, f32)| {
            let a: glam::Vec3 = lua.from_value(a)?;
            lua.to_value((a * b).as_ref())
        })?,
    )?;

    // Mul
    ctx.globals().set(
        "mulv3",
        ctx.create_function(|lua, (a, b): (LuaValue, LuaValue)| {
            let a: glam::Vec3 = lua.from_value(a)?;
            let b: glam::Vec3 = lua.from_value(b)?;
            lua.to_value((a * b).as_ref())
        })?,
    )?;

    // Subtract
    ctx.globals().set(
        "subv3",
        ctx.create_function(|lua, (a, b): (LuaValue, LuaValue)| {
            let a: glam::Vec3 = lua.from_value(a)?;
            let b: glam::Vec3 = lua.from_value(b)?;
            lua.to_value((a - b).as_ref())
        })?,
    )?;

    // Mid
    ctx.globals().set(
        "midv3",
        ctx.create_function(|lua, (a, b): (LuaValue, LuaValue)| {
            let a: glam::Vec3 = lua.from_value(a)?;
            let b: glam::Vec3 = lua.from_value(b)?;
            lua.to_value(((a + b) * 0.5).as_ref())
        })?,
    )?;

    // Simple angle between vectors
    ctx.globals().set(
        "anglev3",
        ctx.create_function(|lua, (a, b): (LuaValue, LuaValue)| {
            let a: glam::Vec3 = lua.from_value(a)?;
            let b: glam::Vec3 = lua.from_value(b)?;
            let angle = a.dot(b).acos();
            Ok(angle.to_degrees())
        })?,
    )?;

    // Simple signed-angle between vectors
    ctx.globals().set(
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
    ctx.globals().set(
        "projv3",
        ctx.create_function(|lua, (v, n): (LuaValue, LuaValue)| {
            let v: glam::Vec3 = lua.from_value(v)?;
            let n: glam::Vec3 = lua.from_value(n)?;
            lua.to_value((v - v.dot(n) * n).normalize().as_ref())
        })?,
    )?;

    // Crate the local reference system (sagittal, frontal and transverse normal vectors) of the body
    // It requires the shouldes and hips keypoints
    ctx.globals().set(
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
    ctx.globals().set(
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
    ctx.globals().set(
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

    Ok(())
}
