use std::{cell::RefCell, pin::Pin};

use mlua::{FromLua, IntoLua, Lua};

use crate::{
    evaluator::{TableError, TableValue, interaction::CellInfo},
    pool,
    table::cell::CellPos,
};

struct LuaVM {
    lua: mlua::Lua,
}

struct LuaManager {}

impl pool::Manager for LuaManager {
    type Type = LuaVM;
    fn create(&self) -> Self::Type {
        let lua = mlua::Lua::new();

        lua.load(include_str!("../prelude.lua"))
            .exec()
            .expect("Prelude is valid and known at compile time");

        let convert_pos = lua.create_function(|_, val: CellPos| Ok(val)).unwrap();
        lua.globals().set("CELL_POS", convert_pos).unwrap();

        LuaVM { lua }
    }
}

thread_local! {
    static LUA_POOL : RefCell<pool::Pool<LuaVM, LuaManager>> =
        const { RefCell::new(pool::Pool::new(LuaManager {})) };
}

#[cfg(not(feature = "multi-thread"))]
type TableLuaBoxFuture<'a, V> = Pin<Box<dyn Future<Output = mlua::Result<V>> + 'a>>;
#[cfg(not(feature = "multi-thread"))]
type TableBoxFn<'a, T, V> = Box<dyn Fn(Lua, T) -> TableLuaBoxFuture<'a, V> + 'a>;

#[cfg(feature = "multi-thread")]
type TableLuaBoxFuture<'a, V> = Pin<Box<dyn Future<Output = mlua::Result<V>> + Send + Sync + 'a>>;
#[cfg(feature = "multi-thread")]
type TableBoxFn<'a, T, V> = Box<dyn Fn(Lua, T) -> TableLuaBoxFuture<'a, V> + Send + Sync + 'a>;

fn get<'a>(info: &'a CellInfo<'a>) -> TableBoxFn<'a, CellPos, TableValue> {
    Box::new(move |_lua, pos: CellPos| Box::pin(async move { Ok(info.get(pos).await.into()) }))
}

fn pos<'a>(info: &'a CellInfo<'a>) -> TableBoxFn<'a, (), CellPos> {
    Box::new(move |_lua, _| Box::pin(async move { Ok(info.pos()) }))
}

pub async fn evaluate<'a>(source: &str, info: &'a CellInfo<'a>) -> TableValue {
    // Acquire a VM from thread-local pool
    let lua_vm = LUA_POOL.with_borrow_mut(|pool| pool.get());
    let lua = &lua_vm.lua;

    // Safety: info is only used for its original lifetime (this is a lie). All references to info are deleted in
    // this function.
    // 2 globals (THIS_POS and GET are removed explicitly, everything else is cleared beacuse the
    //   chunk cannot modify global environment
    let info = unsafe {
        #[allow(
            clippy::unnecessary_cast,
            reason = "clippy warns about 2 identical casts here but his fix suggestion doesn't work"
        )]
        &*(info as *const _ as *const CellInfo<'static>)
    };

    lua.globals()
        .set("THIS_POS", lua.create_async_function(pos(info)).unwrap())
        .unwrap();

    lua.globals()
        .set("GET", lua.create_async_function(get(info)).unwrap())
        .unwrap();

    let env = lua.create_table().unwrap();

    let env_meta = lua.create_table().unwrap();

    {
        let env = env.clone();
        let globals = lua.globals().clone();
        env_meta
            .set(
                mlua::MetaMethod::Index.name(),
                lua.create_function(move |_, (_, index): (mlua::Value, String)| {
                    if index == "_G" {
                        Ok(mlua::Value::Table(env.clone()))
                    } else {
                        globals.get::<mlua::Value>(index)
                    }
                })
                .unwrap(),
            )
            .unwrap();
    }

    env.set_metatable(Some(env_meta)).unwrap();

    let chunk = lua.load(source).set_environment(env);
    let res = chunk.eval_async::<TableValue>().await;

    // Clean up VM and return to the pool
    lua.globals().set("THIS_POS", mlua::Value::Nil).unwrap();
    lua.globals().set("GET", mlua::Value::Nil).unwrap();
    LUA_POOL.with_borrow_mut(|pool| pool.put(lua_vm));

    res.unwrap_or_else(TableValue::lua_error)
}

impl FromLua for TableValue {
    fn from_lua(value: mlua::Value, _lua: &Lua) -> mlua::Result<Self> {
        use mlua::Value::{Error, Integer, Nil, Number};
        match value {
            Nil => Ok(TableValue::Empty),
            Number(n) => Ok(TableValue::Number(n)),
            Integer(n) => Ok(TableValue::Number(n as f64)),
            Error(e) => Ok(TableValue::lua_error(*e)),
            _ => match value.to_string() {
                Ok(s) => Ok(TableValue::from_text(s)),
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
            Self::Number(value) => Ok(value.into_lua(lua).expect("Failed to conver f64 to lua")),
            Self::Err(e) => match e {
                TableError::LuaError(e) => Err((*e).clone()),
                TableError::OtherError(e) => Err(mlua::Error::ExternalError(e)),
            },
        }
    }
}
