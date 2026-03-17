use std::io::{self, BufRead, Write};

use rum_runtime::Session;

pub struct Repl<R, W> {
    input: R,
    output: W,
    prompt: String,
}

impl<R, W> Repl<R, W>
where
    R: BufRead,
    W: Write,
{
    pub fn new(input: R, output: W) -> Self {
        Self {
            input,
            output,
            prompt: "rum> ".to_string(),
        }
    }

    pub fn run(&mut self, session: &mut Session) -> io::Result<()> {
        loop {
            self.output.write_all(self.prompt.as_bytes())?;
            self.output.flush()?;

            let mut line = String::new();
            let bytes_read = self.input.read_line(&mut line)?;

            if bytes_read == 0 {
                writeln!(self.output)?;
                return Ok(());
            }

            let line = line.trim();

            if line.is_empty() {
                continue;
            }

            if self.try_handle_command(session, line)? {
                return Ok(());
            }

            match session.eval_text(line) {
                Ok(outcome) => writeln!(self.output, "{}", outcome.value)?,
                Err(err) => writeln!(self.output, "! {err}")?,
            }
        }
    }

    fn try_handle_command(&mut self, session: &Session, line: &str) -> io::Result<bool> {
        match line {
            ":quit" | ":q" => Ok(true),
            ":help" => {
                writeln!(self.output, ":help           Show REPL commands.")?;
                writeln!(self.output, ":inspect NAME   Show basic symbol metadata.")?;
                writeln!(self.output, ":quit           Exit the REPL.")?;
                Ok(false)
            }
            _ if line.starts_with(":inspect ") => {
                let name = line.trim_start_matches(":inspect ").trim();

                if let Some(info) = session.inspect(name) {
                    writeln!(
                        self.output,
                        "{} :: {}",
                        info.qualified_name,
                        info.summary.unwrap_or_default()
                    )?;
                } else {
                    writeln!(self.output, "! unknown symbol `{name}`")?;
                }

                Ok(false)
            }
            _ => Ok(false),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use rum_runtime::Session;

    use super::Repl;

    #[test]
    fn repl_handles_help_and_quit() {
        let input = Cursor::new(":help\n:quit\n");
        let mut output = Vec::new();
        let mut repl = Repl::new(input, &mut output);
        let mut session = Session::new();

        repl.run(&mut session).unwrap();

        let rendered = String::from_utf8(output).unwrap();
        assert!(rendered.contains(":inspect NAME"));
    }
}
