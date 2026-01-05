use std::{marker::PhantomData, pin::Pin, sync::Arc};

use mlua::{FromLua, FromLuaMulti, IntoLua, IntoLuaMulti, Lua, Value};

use crate::{
    evaluator::{TableError, TableValue, interaction::CellInfo},
    table::{cell::CellPos, slice::SlicePos},
};

type TableLuaBoxFuture<'a, V> = Pin<Box<dyn Future<Output = mlua::Result<V>> + Send + Sync + 'a>>;
type TableBoxFn<'a, T, V> = Box<dyn Fn(Lua, T) -> TableLuaBoxFuture<'a, V> + Send + Sync + 'a>;

fn get<'a>(info: &'a CellInfo<'a>) -> TableBoxFn<'a, CellPos, TableValue> {
    Box::new(move |_lua, pos: CellPos| Box::pin(async move { Ok(info.get(pos).await.into()) }))
}
fn pos<'a>(info: &'a CellInfo<'a>) -> TableBoxFn<'a, (), (usize, usize)> {
    Box::new(move |_lua, _| {
        Box::pin({
            let pos = info.pos();
            async move { Ok((pos.x, pos.y)) }
        })
    })
}

unsafe fn trust_me_bro(info: &CellInfo) -> &'static CellInfo<'static> {
    // clippy warns about 2 identical casts here but his fix suggestion doesn't work
    #[allow(clippy::unnecessary_cast)]
    unsafe {
        &*(info as *const _ as *const CellInfo<'static>)
    }
}

pub struct CellEvaluator<'a> {
    lua: Lua,
    info: &'static CellInfo<'static>,
    _phantom_info: PhantomData<&'a CellInfo<'a>>,
}

impl<'a> CellEvaluator<'a> {
    fn new(info: &'a CellInfo<'a>, lua: Lua) -> Self {
        let info = unsafe { trust_me_bro(info) };

        Self {
            lua,
            info,
            _phantom_info: PhantomData,
        }
    }
    fn add_global_fn<T: FromLuaMulti + 'static, V: IntoLuaMulti + 'static>(
        &mut self,
        name: &str,
        f: impl Fn(&'static CellInfo<'static>) -> TableBoxFn<'static, T, V>,
    ) {
        let f = self.lua.create_async_function(f(self.info)).unwrap();
        self.lua.globals().set(name, f).unwrap();
    }

    async fn evaluate(&mut self, source: &str) -> mlua::Result<TableValue> {
        let chunk = self.lua.load(source);
        chunk.eval_async::<TableValue>().await
    }
}

pub async fn evaluate<'a>(source: &str, info: &'a CellInfo<'a>) -> TableValue {
    let lua = Lua::new();
    lua.load(include_str!("../prelude.lua"))
        .exec()
        .expect("Prelude is valid and known at compile time");
    let mut ev = CellEvaluator::new(info, lua);

    ev.add_global_fn("POS", pos);
    ev.add_global_fn("GET", get);

    let res = ev.evaluate(source).await;

    res.unwrap_or_else(|err| TableValue::Err(TableError::LuaError(Arc::new(err))))
}

impl FromLua for TableValue {
    fn from_lua(value: mlua::Value, _lua: &Lua) -> mlua::Result<Self> {
        use mlua::Value::{Integer, Number};
        match value {
            Number(n) => Ok(TableValue::Number(n)),
            Integer(n) => Ok(TableValue::Number(n as f64)),
            _ => match value.to_string() {
                Ok(s) => Ok(TableValue::from_stringable(s)),
                Err(e) => Ok(TableValue::lua_error(e)),
            },
        }
    }
}

impl IntoLua for TableValue {
    fn into_lua(self, lua: &Lua) -> mlua::Result<mlua::Value> {
        match self {
            Self::Empty => mlua::Nil.into_lua(lua),
            Self::Text(s) => s.to_string().into_lua(lua),
            Self::Err(TableError::LuaError(le)) => le.as_ref().to_owned().into_lua(lua),
            Self::Number(value) => Ok(value.into_lua(lua).expect("Failed to conver f64 to lua")),
            Self::Err(e) => e.to_string().into_lua(lua),
        }
    }
}

fn try_lua_to_usize(value: &mlua::Value) -> Option<usize> {
    Some(match value {
        Value::Integer(x) if *x >= 0 => *x as usize,
        Value::Number(x) if x.is_normal() && !x.is_sign_negative() => x.next_down() as usize,
        _ => return None,
    })
}

impl FromLuaMulti for CellPos {
    fn from_lua_multi(values: mlua::MultiValue, _lua: &Lua) -> mlua::Result<Self> {
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
                    Value::Table(t) => {
                        let Ok(x) = t
                            .get("x")
                            .or_else(|_| t.get("col"))
                            .or_else(|_| t.get("column"))
                            .or_else(|_| t.get(1))
                        else {
                            return err;
                        };

                        let Ok(y) = t.get("y").or_else(|_| t.get("row")).or_else(|_| t.get(2))
                        else {
                            return err;
                        };
                        let (Some(x), Some(y)) = (try_lua_to_usize(&x), try_lua_to_usize(&y))
                        else {
                            return err;
                        };
                        CellPos::from((x, y))
                    }
                    Value::String(s) => {
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
                let (Some(x), Some(y)) = (try_lua_to_usize(&x), try_lua_to_usize(&y)) else {
                    return err;
                };
                CellPos::from((x, y))
            }
        };

        Ok(pos)
    }
}

impl FromLua for SlicePos {
    fn from_lua(value: mlua::Value, _lua: &Lua) -> mlua::Result<Self> {
        let err = Err(mlua::Error::FromLuaConversionError {
            from: "",
            to: "SlicePos".into(),
            message: Some(
                "CellPos can be created from a string in format {CellPos}_{CellPos}".into(),
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
