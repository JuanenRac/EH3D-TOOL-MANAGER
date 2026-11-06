// EH3D-TOOL-MANAGER - the tool crib of the EH3D Fabrication Suite.
// Copyright (C) 2026 JuanenRac (Electro Hobby 3D). GPL-3.0-or-later.
#![forbid(unsafe_code)]
//! Tools (drills, end mills, V-bits, engravers, lasers, nozzles) with their geometry and their wear, stored as JSON or CSV, and the feeds and speeds to start from. With the `gui`
//! feature it also has the window (`window` holds its state, `gui` draws it).

pub mod calc;
pub mod csv;
#[cfg(feature = "gui")]
pub mod gui;
pub mod lang;
pub mod model;
pub mod settings;
pub mod shot;
pub mod store;
pub mod window;
