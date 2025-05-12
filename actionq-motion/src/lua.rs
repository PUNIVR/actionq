use mlua::prelude::*;
use crate::common::*;

/// Add all helper functions related to FSM behaviour
pub fn add_functions_control(ctx: &Lua) -> LuaResult<()> {

    // Stay on same state, optionally supports state metadata
    ctx.globals().set("stay", 
        ctx.create_function(|lua, metadata: Option<LuaTable>| {
            let result = lua.create_table()?; // Create basic table
            result.set("next_state", LuaValue::Nil)?;
            result.set("metadata", metadata)?;
            Ok(result)
        })?
    )?;

    // Change state, first argument is the next state, optionally supports metadata
    ctx.globals().set("step", 
        ctx.create_function(|lua, (next_state, metadata): (String, Option<LuaTable>) | {
            //println!("next_state: {}", next_state);
            let result = lua.create_table()?; // Create basic table
            result.set("next_state", next_state)?;
            result.set("metadata", metadata)?;
            Ok(result)
        })?
    )?;

    Ok(())
}

/// Add all helper functions related to Vec2
pub fn add_functions_vec2(ctx: &Lua) -> LuaResult<()> {

    // Inner angle
    ctx.globals().set("inner_angle",
        ctx.create_function(|_, (a, z, b): (LuaVec2, LuaVec2, LuaVec2)| {

            let z2a = a.0 - z.0;
            let z2b = b.0 - z.0;

            let dot = z2a.x * z2b.x + z2a.y * z2b.y;
            let det = z2a.x * z2b.y - z2a.y * z2b.x;

            Ok(det.atan2(dot))
        })?
    )?;

    // Inner angle aligned
    ctx.globals().set("inner_angle_aligned",
        ctx.create_function(|_, (a, z, b): (LuaVec2, LuaVec2, LuaVec2)| {
            
            let a2z = z.0 - a.0;
            let z2b = b.0 - z.0;

            let k = a2z.dot(z2b)/(a2z.length()*z2b.length());
            let r = k.acos().to_degrees();

            Ok(r)
        })?
    )?;

    // Angle compared to a reference axix
    ctx.globals().set("inner_angle_aligned_axis",
        ctx.create_function(|_, (axis, a, b): (LuaVec2, LuaVec2, LuaVec2)| {
            let axis = axis.0;
            let d = b.0 - a.0;
            let k = axis.dot(d)/(axis.length()*d.length());
            Ok(k.acos().to_degrees())
        })?
    )?;

    Ok(())
}

/// Add all helper functions related to Vec3
pub fn add_functions_vec3(ctx: &Lua) -> LuaResult<()> {

    // Normalize
    ctx.globals().set("normv3",
        ctx.create_function(|_, v: LuaVec3| {
            Ok(LuaVec3(v.0.normalize()))
        })?
    )?;

    // Cross
    ctx.globals().set("crossv3",
        ctx.create_function(|_, (a, b): (LuaVec3, LuaVec3)| {
            Ok(LuaVec3(a.0.cross(b.0)))
        })?
    )?;

    // Dot
    ctx.globals().set("dotv3",
        ctx.create_function(|_, (a, b): (LuaVec3, LuaVec3)| {
            Ok(a.0.dot(b.0))
        })?
    )?;

    // Mul with float
    ctx.globals().set("mulfv3",
        ctx.create_function(|_, (a, b): (LuaVec3, f32)| {
            Ok(LuaVec3(a.0 * b))
        })?
    )?;

    // Mul
    ctx.globals().set("mulv3",
        ctx.create_function(|_, (a, b): (LuaVec3, LuaVec3)| {
            Ok(LuaVec3(a.0 * b.0))
        })?
    )?;


    // Subtract
    ctx.globals().set("subv3",
        ctx.create_function(|_, (a, b): (LuaVec3, LuaVec3)| {
            Ok(LuaVec3(a.0 - b.0))
        })?
    )?;

    // Mid
    ctx.globals().set("midv3",
        ctx.create_function(|_, (a, b): (LuaVec3, LuaVec3)| {
            Ok(LuaVec3((a.0 + b.0) * 0.5))
        })?
    )?;

    // Simple angle between vectors
    ctx.globals().set("anglev3",
        ctx.create_function(|_, (a, b): (LuaVec3, LuaVec3)| {
            let angle = a.0.dot(b.0).acos();
            Ok(angle.to_degrees())
        })?
    )?;

    // Simple signed-angle between vectors
    ctx.globals().set("sanglev3",
        ctx.create_function(|_, (a, b, n): (LuaVec3, LuaVec3, LuaVec3)| {
            let angle = a.0.dot(b.0).acos();
            let sign = (n.0.dot(a.0.cross(b.0))).signum();
            Ok(sign * angle.to_degrees())
        })?
    )?;

    // Project a vector into a plane defined by a normal
    // NOTE: all input must be nornalized
    ctx.globals().set("projv3",
        ctx.create_function(|_, (v, n): (LuaVec3, LuaVec3)| {
            Ok(LuaVec3((v.0 - v.0.dot(n.0) * n.0).normalize()))
        })?
    )?;

    // Crate the local reference system (sagittal, frontal and transverse normal vectors) of the body
    // It requires the shouldes and hips keypoints 
    ctx.globals().set("body_planes",
        ctx.create_function(|lua, (ls, rs, lh, rh): (LuaVec3, LuaVec3, LuaVec3, LuaVec3)| {

            // Sagittal plane normal
            let sagittal = (ls.0 - rs.0).normalize();

            // Transverse plane normal
            let ms = rs.0.midpoint(ls.0);
            let mh = rh.0.midpoint(lh.0);
            let transverse = (mh - ms).normalize();

            // Frontal plane
            let frontal = sagittal.cross(transverse).normalize();

            let result = lua.create_table()?;
            result.set("sagittal", LuaVec3(sagittal))?;
            result.set("frontal", LuaVec3(frontal))?;
            result.set("transverse", LuaVec3(transverse))?;
            Ok(result)
        })?
    )?;


    // Inner angle without reference plane
    ctx.globals().set("inner_angle_3d",
        ctx.create_function(|_, (a, m, b): (LuaVec3, LuaVec3, LuaVec3)| {

            let ma = (a.0 - m.0).normalize(); // m -> a
            let mb = (b.0 - m.0).normalize(); // m -> b

            let dot = ma.dot(mb);
            let crs = ma.cross(mb);
            let axs = crs.normalize();
            let angle = crs.length().atan2(dot);

            Ok(angle.to_degrees())
        })?
    )?;

    // Inner angle with reference plane
    ctx.globals().set("inner_angle_3d_aligned",
        ctx.create_function(|_, (a, m, b, normal): (LuaVec3, LuaVec3, LuaVec3, LuaVec3)| {

            let normal = normal.0;
            let ma = a.0 - m.0; // m -> a
            let mb = b.0 - m.0; // m -> b

            let dot = ma.dot(mb).clamp(-1.0, 1.0);
            let angle = dot.acos(); // [0, pi]

            let cross = ma.cross(mb);
            let sign = if normal.dot(cross) < 0.0 { -1.0 } else { 1.0 };
            let signed_angle = angle * sign;

            Ok(signed_angle.to_degrees())
        })?
    )?;

    Ok(())
}
