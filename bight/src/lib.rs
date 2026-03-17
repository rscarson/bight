//! `bight` is an Excel-like spreadsheet engine.
//! It has a standalone TUI editor (bight_tui in this repository) (mainly for quick testing, many features are missing),
//! and a [neovim plugin](https://github.com/WASDetchan/bight.nvim) (the recommended way to use),
//!
//! Bight defines Table traits and data types to work with tables in an abstract
//! way (see the [`table`] module). Using those traits, bight implements an
//! [`evaluator::EvaluatorTable`]. It's the main type of the bight engine. It can be used to
//! asyncronously evaluate cells with formulas written in Lua languange in Excel-like manner (the
//! cell source starting with '=' is interpreted like a formula) (more thorough usage docs are available
//! in the nvim plugin documentation).
//!
//! The tables may be saved and loaded from files, evaluated
//! cells may be exported as csv (see the [`mod@file`] module). Bight also provides some simple terminal (including keybindings) and clipboard
//! abstrations.

pub mod clipboard;
pub mod evaluator;
pub mod file;
pub mod sync;
pub mod table;

#[cfg(feature = "plot")]
pub mod plot;

#[cfg(test)]
pub mod test_util;
