use rum_runtime::{Session, core_builtin_specs};

pub fn install_core(session: &mut Session) {
    for spec in core_builtin_specs() {
        session.register_builtin(*spec);
    }
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
