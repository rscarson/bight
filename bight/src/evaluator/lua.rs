use std::{marker::PhantomData, pin::Pin, sync::Arc};

use mlua::{FromLua, FromLuaMulti, IntoLua, IntoLuaMulti, Lua};

use crate::{
    evaluator::{TableError, TableValue, interaction::CellInfo},
    table::cell::CellPos,
};

type TableLuaBoxFuture<'a, V> = Pin<Box<dyn Future<Output = mlua::Result<V>> + Send + Sync + 'a>>;
type TableBoxFn<'a, T, V> = Box<dyn Fn(Lua, T) -> TableLuaBoxFuture<'a, V> + Send + Sync + 'a>;

fn get<'a>(info: &'a CellInfo<'a>) -> TableBoxFn<'a, CellPos, TableValue> {
    Box::new(move |_lua, pos: CellPos| Box::pin(async move { Ok(info.get(pos).await.into()) }))
}

fn pos<'a>(info: &'a CellInfo<'a>) -> TableBoxFn<'a, (), CellPos> {
    Box::new(move |_lua, _| Box::pin(async move { Ok(info.pos()) }))
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

    let convert_pos = lua.create_function(|_, val: CellPos| Ok(val)).unwrap();

    lua.globals().set("CELL_POS", convert_pos).unwrap();

    let mut ev = CellEvaluator::new(info, lua);

    ev.add_global_fn("THIS_POS", pos);
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
