use super::CellRange;

impl mlua::FromLua for CellRange {
    fn from_lua(value: mlua::Value, _lua: &mlua::Lua) -> mlua::Result<Self> {
        let err = Err(mlua::Error::FromLuaConversionError {
            from: "",
            to: "CellRange".into(),
            message: Some(
                "CellRange can be created from a string in format {CellPos}_{CellPos}".into(),
            ),
        });

        let mlua::Value::String(pos) = value else {
            return err;
        };
        let Ok(pos) = pos.to_str() else { return err };
        let Ok(pos) = pos.parse() else { return err };

        Ok(pos)
    }
}

impl mlua::UserData for CellRange {}
