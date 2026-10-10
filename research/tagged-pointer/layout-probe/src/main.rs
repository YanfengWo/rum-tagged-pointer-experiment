//! External measurement probe: the original rum source is unchanged.
use rum_core::expr::SymbolText;
use rum_core::{Atom, Expr, IntValue, Rational, SharedArgs, SharedExpr, Symbol, WeakSharedArgs};
use std::mem::{align_of, size_of};
use std::sync::Arc;

fn row<T>(name: &str, comma: bool) {
    println!(
        "    {{\"type\":\"{name}\",\"bytes\":{},\"alignment\":{}}}{}",
        size_of::<T>(),
        align_of::<T>(),
        if comma { "," } else { "" }
    );
}

fn main() {
    println!(
        "{{\n  \"target\":\"{}-{}\",\n  \"types\":[",
        std::env::consts::ARCH,
        std::env::consts::OS
    );
    row::<usize>("usize", true);
    row::<IntValue>("IntValue", true);
    row::<Rational>("Rational", true);
    row::<Symbol>("Symbol", true);
    row::<SymbolText>("SymbolText", true);
    row::<Atom>("Atom", true);
    row::<Expr>("Expr", true);
    row::<SharedExpr>("SharedExpr", true);
    row::<SharedArgs>("SharedArgs", true);
    row::<WeakSharedArgs>("WeakSharedArgs", true);
    row::<Option<SharedExpr>>("Option<SharedExpr>", true);
    row::<Option<SharedArgs>>("Option<SharedArgs>", true);
    row::<Arc<Expr>>("Arc<Expr>", true);
    row::<Arc<Vec<Expr>>>("Arc<Vec<Expr>>", true);
    row::<Vec<Expr>>("Vec<Expr>", false);
    println!("  ]\n}}");
}
