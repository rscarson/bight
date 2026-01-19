use mlua::{FromLuaMulti as _, IntoLua as _};

use crate::table::CellRange;

use super::CellPos;

impl mlua::FromLuaMulti for CellPos {
    fn from_lua_multi(values: mlua::MultiValue, _lua: &mlua::Lua) -> mlua::Result<Self> {
        fn try_lua_to_isize(value: &mlua::Value) -> Option<isize> {
            Some(match value {
                mlua::Value::Integer(x) => (*x).try_into().unwrap(),
                mlua::Value::Number(x) if x.is_normal() => *x as isize,
                _ => return None,
            })
        }

        const ERROR_MESSAGE: &str = "CellPos can be created from a string in format [A-Za-z]+[0-9]+, 2 non-negative numbers, or a table with x, col, column or 1st element for x coordinate and y, row, or 2nd element for y coordinate";
        let err = Err(mlua::Error::FromLuaConversionError {
            from: "",
            to: "CellPos".into(),
            message: Some(ERROR_MESSAGE.into()),
        });

        let pos = match values.len() {
            0 => return err,
            1 => {
                let value = values.into_iter().next().unwrap();
                match value {
                    mlua::Value::Table(t) => {
                        let Ok(x) = t
                            .get("x")
                            .or_else(|_| t.get("col"))
                            .or_else(|_| t.get("column"))
                            .or_else(|_| t.get(1))
                        else {
                            log::trace!("could find x value");
                            return err;
                        };

                        let Ok(y) = t.get("y").or_else(|_| t.get("row")).or_else(|_| t.get(2))
                        else {
                            log::trace!("could find y value");
                            return err;
                        };
                        CellPos::from((x, y))
                    }
                    mlua::Value::String(s) => {
                        let Ok(pos) = s.to_str() else { return err };
                        let Ok(pos) = pos.parse::<CellPos>() else {
                            return err;
                        };
                        pos
                    }
                    _ => return err,
                }
            }
            2.. => {
                let mut iter = values.into_iter();
                let (x, y) = (iter.next().unwrap(), iter.next().unwrap());
                let (Some(x), Some(y)) = (try_lua_to_isize(&x), try_lua_to_isize(&y)) else {
                    return err;
                };
                CellPos::from((x, y))
            }
        };

        Ok(pos)
    }
}

impl mlua::UserData for CellPos {
    fn add_methods<M: mlua::UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method("val", |lua, this, ()| {
            let get_fn: mlua::Function = lua.globals().get("GET")?;
            get_fn.call::<mlua::Value>((this.x, this.y))
        });

        methods.add_meta_method("__index", |lua, &this, val: String| {
            let pos =
                CellPos::from_lua_multi(mlua::MultiValue::from_vec(vec![val.into_lua(lua)?]), lua)?;
            Ok(CellRange::new_limits(this, pos))
        });
    }

    fn add_fields<F: mlua::UserDataFields<Self>>(fields: &mut F) {
        fields.add_field_method_get("x", |_lua, this| Ok(this.x));
        fields.add_field_method_get("y", |_lua, this| Ok(this.y));
    }
}
