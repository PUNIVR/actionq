use actionq_common::*;
use mlua::prelude::*;

#[derive(Debug, Clone)]
pub struct LuaVec2(pub glam::Vec2);

#[derive(Debug, Clone)]
pub struct LuaVec3(pub glam::Vec3);

pub type LuaSkeleton2D = Skeleton<LuaVec2>;
pub type LuaSkeleton3D = Skeleton<LuaVec3>;

pub type LuaSkeletonMap3D = SkeletonMap<LuaVec3>;

/// Convert a Vec2 into a LuaVec2. 
/// Necessary to implement traits on the Vec2 struct from glam.
impl Into<LuaVec2> for glam::Vec2 {
    fn into(self) -> LuaVec2 {
        LuaVec2(self)
    }
}

/// Convert a LuaVec2 into a Vec2
impl Into<glam::Vec2> for LuaVec2 {
    fn into(self) -> glam::Vec2 {
        self.0
    }
}

/// Convert LuaVec2 to Lua table
impl IntoLua for LuaVec2 {
    fn into_lua(self, lua: &Lua) -> LuaResult<LuaValue> {
        let result = lua.create_table()?;
        result.set("x", self.0.x)?;
        result.set("y", self.0.y)?;
        Ok(LuaValue::Table(result))
    }
}

/// Convert Lua table into a LuaVec2
impl FromLua for LuaVec2 {
    fn from_lua(value: LuaValue, _: &Lua) -> LuaResult<Self> {
        //println!("{:?}", value);
        if let LuaValue::Table(t) = value {
            return Ok(LuaVec2(glam::Vec2::new(
                t.get("x")?, t.get("y")?)))
        }
        unimplemented!()
    }
}

/// Convert a Vec3 into a LuaVec3. 
/// Necessary to implement traits on the Vec3 struct from glam.
impl Into<LuaVec3> for glam::Vec3 {
    fn into(self) -> LuaVec3 {
        LuaVec3(self)
    }
}

/// Convert a LuaVec3 into a Vec3
impl Into<glam::Vec3> for LuaVec3 {
    fn into(self) -> glam::Vec3 {
        self.0
    }
}

/// Convert LuaVec3 to Lua table
impl IntoLua for LuaVec3 {
    fn into_lua(self, lua: &Lua) -> LuaResult<LuaValue> {
        let result = lua.create_table()?;
        result.set("x", self.0.x)?;
        result.set("y", self.0.y)?;
        result.set("z", self.0.z)?;
        Ok(LuaValue::Table(result))
    }
}

/// Convert Lua table into a LuaVec3
impl FromLua for LuaVec3 {
    fn from_lua(value: LuaValue, _: &Lua) -> LuaResult<Self> {
        if let LuaValue::Table(t) = value {
            return Ok(LuaVec3(glam::Vec3::new(
                t.get("x")?, t.get("y")?, t.get("z")?)))
        }
        unimplemented!()
    }
}

