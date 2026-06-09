use rum_difftest::oracle::OracleKernel;

pub fn print_oracle_header(lane: &str, oracle: &OracleKernel) {
    println!("{lane} oracle: {}", oracle.executable().display());
    println!(
        "{lane} oracle timeout seconds: {}",
        oracle.timeout().as_secs()
    );
    println!(
        "{lane} oracle output limit bytes per stream: {}",
        oracle.output_limit_bytes()
    );
    println!("{lane} oracle version policy: user-provided/unpinned");
    match oracle.version() {
        Ok(version) if !version.is_empty() => {
            println!("{lane} oracle version: {version}");
        }
        Ok(_) => {
            println!("{lane} oracle version: <unavailable: empty version output>");
        }
        Err(error) => {
            println!("{lane} oracle version: <unavailable: {error}>");
        }
    }
}
