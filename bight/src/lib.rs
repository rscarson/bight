//! `bight` is an Excel-like spreadsheet engine and editor. It can be used as a standalone TUI editor
//! (WIP, many features are missing), a [neovim plugin](https://github.com/WASDetchan/bight.nvim),
//! or as a library.
//!
//! Bight defines Table traits and data types to work with tables in an abstract
//! way (see the [`table`] module). Using those traits, bight implements an
//! [`evaluator::EvaluatorTable`]. It's the main type of the bight engine. It can be used to
//! asyncronously evaluate cells with formulas written in Lua languange in Excel-like manner (the
//! cell source starting with '=' is interpreted like a formula) (more thorough usage docs are available
//! in the nvim plugin documentation).
//!
//! The tables may be saved and loaded from files, evaluated
//! cells may be exported as csv. Bight also provides some simple terminal (including keybindings) and clipboard
//! abstrations.

pub mod app;
pub mod callback;
pub mod clipboard;
pub mod editor;
pub mod evaluator;
pub mod file;
pub mod key;
pub mod table;
pub mod term;
