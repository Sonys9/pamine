//! Lua-facing wrappers for the rewrite types.
//!
//! Scripts see two things:
//!
//! - `Injection` as a userdata with read-only fields (`changeable`, `depth`,
//!   `bytes`, `len`) and a few methods (`is_empty`, `is_exhausted`,
//!   `push_frame`, `clear`). `bytes` is exposed as a Lua string, not as a
//!   sequence table, so binary frames survive the round trip unchanged.
//! - `update` as a plain table of constructors (`none`, `change`, `inject`,
//!   `both`, `injection`). It is a table rather than a userdata on purpose:
//!   `update.both(a, b)` cannot accidentally pass `update` as `self`, which is
//!   exactly the mistake that `engine.register(...)` used to make.
//!
//! `Update` itself is never handed to Lua; scripts build a table with the
//! `update.*` helpers and the engine converts it back through [`FromLua`].

use std::sync::atomic::Ordering;

use mlua::{IntoLua, prelude::*};

use crate::{
    connections::{Injection, Update},
    packets::UnsureThreshold,
};

/// The `update` global: a namespace of constructors for [`Update`] tables.
pub struct UpdateBindings;

impl UpdateBindings {
    /// A table with both fields set to `nil`, i.e. "leave the packet alone".
    fn none(lua: &Lua) -> LuaResult<LuaTable> {
        let table = lua.create_table()?;
        table.set("change_with", None::<Injection>)?;
        table.set("inject", None::<Injection>)?;
        Ok(table)
    }
}

/// Exposes the compression threshold as `{ threshold = <int> | nil }`.
///
/// `exists` is read *before* `value`, matching the store order in
/// [`UnsureThreshold::set`](crate::packets::UnsureThreshold::set). Reading
/// `value` first would let a concurrent `set` produce a table that claims
/// compression is on while carrying the stale `0` threshold.
impl IntoLua for &UnsureThreshold {
    fn into_lua(self, lua: &Lua) -> LuaResult<LuaValue> {
        let table = lua.create_table()?;
        let threshold = if self.exists.load(Ordering::Acquire) {
            Some(self.value.load(Ordering::Acquire))
        } else {
            None
        };
        table.set("threshold", threshold)?;
        Ok(LuaValue::Table(table))
    }
}

/// `Injection` is a userdata, so `IntoLua` comes from mlua's blanket impl.
/// `FromLua` has to be written by hand: it borrows and clones, which lets a
/// script pass the same injection to several `update.*` calls.
impl FromLua for Injection {
    fn from_lua(value: LuaValue, _lua: &Lua) -> LuaResult<Self> {
        match value {
            LuaValue::UserData(ud) => Ok(ud.borrow::<Injection>()?.clone()),
            other => Err(LuaError::runtime(format!(
                "expected an Injection, got {}",
                other.type_name()
            ))),
        }
    }
}

impl LuaUserData for Injection {
    fn add_fields<F: LuaUserDataFields<Self>>(fields: &mut F) {
        fields.add_field_method_get("changeable", |_, this| Ok(this.changeable));
        fields.add_field_method_get("depth", |_, this| Ok(this.depth));
        fields.add_field_method_get("len", |_, this| Ok(this.bytes.len()));
        // A Lua string keeps arbitrary bytes intact; a `Vec<u8>` would become a
        // sequence table of numbers and lose the frame on the way back.
        fields.add_field_method_get("bytes", |lua, this| lua.create_string(&this.bytes));
    }

    fn add_methods<M: LuaUserDataMethods<Self>>(methods: &mut M) {
        methods.add_method("is_empty", |_, this, ()| Ok(this.is_empty()));
        methods.add_method("is_exhausted", |_, this, ()| Ok(this.is_exhausted()));

        // Appends an already-framed packet. Framing needs the threshold, which
        // lives on the Rust side, so scripts hand over complete frames.
        methods.add_method_mut("push_frame", |_, this, frame: LuaString| {
            this.bytes.extend_from_slice(&frame.as_bytes());
            Ok(())
        });

        methods.add_method_mut("clear", |_, this, ()| {
            this.bytes.clear();
            Ok(())
        });
    }
}

/// Builds the `update` table. Plain functions, no `self`, no colon trap.
impl IntoLua for UpdateBindings {
    fn into_lua(self, lua: &Lua) -> LuaResult<LuaValue> {
        let table = lua.create_table()?;

        table.set(
            "none",
            lua.create_function(|lua, ()| UpdateBindings::none(lua))?,
        )?;

        table.set(
            "change",
            lua.create_function(|lua, change_with: Injection| {
                let table = UpdateBindings::none(lua)?;
                table.set("change_with", change_with)?;
                Ok(table)
            })?,
        )?;

        table.set(
            "inject",
            lua.create_function(|lua, inject: Injection| {
                let table = UpdateBindings::none(lua)?;
                table.set("inject", inject)?;
                Ok(table)
            })?,
        )?;

        table.set(
            "both",
            lua.create_function(
                |lua, (change_with, inject): (Injection, Injection)| {
                    let table = UpdateBindings::none(lua)?;
                    table.set("change_with", change_with)?;
                    table.set("inject", inject)?;
                    Ok(table)
                },
            )?,
        )?;

        table.set(
            "injection",
            lua.create_function(|_, changeable: bool| Ok(Injection::empty(changeable)))?,
        )?;

        Ok(LuaValue::Table(table))
    }
}

/// Converts the table built by `update.*` back into an [`Update`].
impl FromLua for Update {
    fn from_lua(value: LuaValue, _lua: &Lua) -> LuaResult<Self> {
        let table = value
            .as_table()
            .ok_or_else(|| LuaError::runtime(format!("expected an Update table, got {}", value.type_name())))?;

        Ok(Update {
            change_with: table.get::<Option<Injection>>("change_with")?,
            inject: table.get::<Option<Injection>>("inject")?,
        })
    }
}
