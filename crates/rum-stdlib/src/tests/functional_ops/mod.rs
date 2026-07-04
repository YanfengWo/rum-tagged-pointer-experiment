use crate::install_core;
use rum_runtime::Session;

fn core_session() -> Session {
    let mut session = Session::new();
    install_core(&mut session);
    session
}

mod incidental_apply_map;
mod incidental_callable;
mod iteration;
mod targeted_map;
