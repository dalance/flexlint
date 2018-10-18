use failure::{Error, ResultExt};
use lint::{Checked, CheckedState};
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use term::{self, color};

// -------------------------------------------------------------------------------------------------
// Printer
// -------------------------------------------------------------------------------------------------

static CHAR_CR: u8 = 0x0d;
static CHAR_LF: u8 = 0x0a;

pub struct Printer;

impl Printer {
    pub fn print(checked: Vec<Checked>, verbose: bool) -> Result<bool, Error> {
        let mut all_pass = true;
        let mut term = term::stdout().ok_or(format_err!("failed to open terminal"))?;

        let mut map: HashMap<PathBuf, Vec<Checked>> = HashMap::new();
        for c in checked {
            if map.contains_key(&c.path) {
                map.get_mut(&c.path).unwrap().push(c);
            } else {
                map.insert(c.path.clone(), vec![c]);
            }
        }

        for (k, mut v) in map {
            v.sort_unstable_by_key(|x| x.pos);

            let mut f = File::open(&k)
                .with_context(|_| format!("failed to open: '{}'", k.to_string_lossy()))?;
            let mut s = String::new();
            let _ = f.read_to_string(&mut s);

            let mut pos = 0;
            let mut column = 1;
            let mut last_lf = 0;
            while pos < s.len() {
                if s.as_bytes()[pos] == CHAR_LF {
                    column += 1;
                    last_lf = pos;
                }
                pos += 1;

                for checked in &v {
                    if checked.pos == pos {
                        if checked.state != CheckedState::Fail && !verbose {
                            continue;
                        }

                        let row = pos - last_lf;
                        let mut next_crlf = pos;
                        while next_crlf < s.len() {
                            if s.as_bytes()[next_crlf] == CHAR_CR
                                || s.as_bytes()[next_crlf] == CHAR_LF
                            {
                                break;
                            }
                            next_crlf += 1;
                        }

                        match checked.state {
                            CheckedState::Pass => {
                                let _ = term.fg(color::BRIGHT_GREEN);
                                write!(term, "Pass");
                            }
                            CheckedState::Fail => {
                                all_pass = false;
                                let _ = term.fg(color::BRIGHT_RED);
                                write!(term, "Fail");
                            }
                            CheckedState::Skip => {
                                let _ = term.fg(color::BRIGHT_MAGENTA);
                                write!(term, "Skip");
                            }
                        }

                        let _ = term.fg(color::BRIGHT_BLUE);
                        write!(term, "\t{}:{}:{}", k.to_string_lossy(), column, row);

                        let _ = term.fg(color::WHITE);
                        write!(
                            term,
                            "\t{}",
                            String::from_utf8_lossy(&s.as_bytes()[pos..next_crlf])
                        );

                        let _ = term.fg(color::BRIGHT_YELLOW);
                        write!(term, "\thint: {}\n", checked.hint);
                        let _ = term.reset();
                    }
                }
            }
        }
        Ok(all_pass)
    }
}

// -------------------------------------------------------------------------------------------------
// Test
// -------------------------------------------------------------------------------------------------
