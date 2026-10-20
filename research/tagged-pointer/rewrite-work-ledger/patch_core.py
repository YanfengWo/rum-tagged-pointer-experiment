"""Add exact work events to core owners only in diagnostic source copies."""
from pathlib import Path


def instrument(root: Path):
    path = root / "crates/rum-core/src/expr.rs"
    value = path.read_text(encoding="utf-8")

    def change(before, after):
        nonlocal value
        assert value.count(before) == 1, before
        value = value.replace(before, after)

    for signature in ["    pub fn into_owned(self) -> Expr {", "    pub fn into_vec(self) -> Vec<Expr> {",
                      "    pub fn unwrap_shared_args(args: SharedArgs) -> Vec<Expr> {"]:
        change(signature, "    #[track_caller]\n" + signature)
    for marker, kind in [("record_into_owned(false);", "into-owned-unique"),
                         ("record_into_owned(true);", "into-owned-copy")]:
        change(marker, f'crate::rewrite_work_ledger::event("{kind}", std::panic::Location::caller().file(), 1);\n                ' + marker)
    for marker, kind in [("record_into_vec(false);", "into-vec-unique"), ("record_into_vec(true);", "into-vec-copy")]:
        change(marker, f'crate::rewrite_work_ledger::event("{kind}", std::panic::Location::caller().file(), args.len());\n                ' + marker)
    change("    pub fn new(args: Vec<Expr>) -> Self {\n        let shared = Self(Arc::new(args));",
           '''    pub fn new(args: Vec<Expr>) -> Self {
        crate::rewrite_work_ledger::event("shared-args-new", "length-slots", args.len());
        crate::rewrite_work_ledger::event("shared-args-new", "capacity-slots", args.capacity());
        let shared = Self(Arc::new(args));''')
    change("    pub fn new(expr: Expr) -> Self {\n        let shared = Self(Arc::new(expr));",
           '''    pub fn new(expr: Expr) -> Self {
        crate::rewrite_work_ledger::event("shared-expr-new", "owner", 1);
        let shared = Self(Arc::new(expr));''')
    path.write_text(value, encoding="utf-8", newline="\n")
    lib = root / "crates/rum-core/src/lib.rs"
    lib.write_text(lib.read_text(encoding="utf-8") + "\n#[doc(hidden)]\npub mod rewrite_work_ledger;\n", encoding="utf-8", newline="\n")
