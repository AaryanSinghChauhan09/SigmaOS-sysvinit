//! Options model.

#[derive(Debug, Clone, PartialEq)]
pub enum Mode { Default, Service(String, u32), Start(String), Stop(String), Runlevel(u32), List }

#[derive(Debug, Clone)]
pub struct Options { pub mode: Mode }
impl Default for Options { fn default() -> Self { Self { mode: Mode::Default } } }
#[cfg(test)] mod tests { use super::*; #[test] fn o() { assert!(matches!(Options::default().mode, Mode::Default)); } }
