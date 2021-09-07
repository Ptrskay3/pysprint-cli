#![warn(rust_2018_idioms)]
#![feature(toowned_clone_into)]
#![warn(clippy::pedantic, clippy::cargo, clippy::nursery)]
#![allow(
    clippy::let_underscore_drop,
    clippy::missing_panics_doc,
    clippy::implicit_clone,
    clippy::must_use_candidate,
    clippy::missing_errors_doc,
    clippy::used_underscore_binding,
    clippy::cast_precision_loss,
    clippy::module_name_repetitions,
    clippy::unnecessary_wraps,
    clippy::too_many_lines,
    clippy::if_not_else,
    clippy::similar_names,
    clippy::cognitive_complexity
)]

pub mod app;
pub mod audit;
pub mod codegen;
pub mod deserialize;
pub mod io;
pub mod parser;
pub mod python;
pub mod statistics;
pub mod utils;
pub mod watch;
