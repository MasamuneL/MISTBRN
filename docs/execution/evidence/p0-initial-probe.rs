//! Disposable P0 probe: compile/load evidence only, no gameplay changes.
use horizon_client::mod_api::Hooks;

pub fn hooks() -> Box<dyn Hooks> {
    eprintln!("ALLOMANCY_PROBE_LOADED v0.0.1");
    Box::new(Probe)
}

struct Probe;
impl Hooks for Probe {}
