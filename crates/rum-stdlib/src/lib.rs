mod catalog;

use rum_runtime::Session;

pub fn install_core(session: &mut Session) {
    for spec in catalog::core_builtins() {
        session.register_builtin(*spec);
    }
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
