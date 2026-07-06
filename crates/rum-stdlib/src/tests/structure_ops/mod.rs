use crate::install_core;
use rum_runtime::Session;

fn core_session() -> Session {
    let mut session = Session::new();
    install_core(&mut session);
    session
}

mod collection_stats;
mod dedup;
mod join;
mod list_plumbing;
mod ordering_sets;
mod reordering;
mod selectors;
mod sequence;
mod surface;
