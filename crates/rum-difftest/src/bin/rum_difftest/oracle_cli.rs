use rum_difftest::oracle::OracleKernel;

pub fn print_oracle_header(lane: &str, oracle: &OracleKernel) {
    println!("{lane} oracle: {}", oracle.executable().display());
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
