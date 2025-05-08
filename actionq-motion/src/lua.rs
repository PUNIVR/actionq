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