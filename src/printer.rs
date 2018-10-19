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
    pub fn print(checked: Vec<Checked>, simple: bool, verbose: bool) -> Result<bool, Error> {
        let path_checked = Printer::collect_by_path(checked);

        let all_pass = if simple {
            Printer::print_simple(&path_checked, verbose)?
        } else {
            Printer::print_pretty(&path_checked, verbose)?
        };

        Ok(all_pass)
    }

    fn collect_by_path(checked: Vec<Checked>) -> Vec<(PathBuf, Vec<Checked>)> {
        let mut map: HashMap<PathBuf, Vec<Checked>> = HashMap::new();
        let mut key = Vec::new();
        for c in checked {
            if map.contains_key(&c.path) {
                map.get_mut(&c.path).unwrap().push(c);
            } else {
                key.push(c.path.clone());
                map.insert(c.path.clone(), vec![c]);
            }
        }

        key.sort_unstable();
        let mut ret = Vec::new();

        for k in key {
            let mut v = map.remove(&k).unwrap();
            v.sort_unstable_by_key(|x| x.beg);
            ret.push((k, v));
        }

        ret
    }

    fn print_simple(
        path_checked: &Vec<(PathBuf, Vec<Checked>)>,
        verbose: bool,
    ) -> Result<bool, Error> {
        let mut all_pass = true;
        let mut term = term::stdout().ok_or(format_err!("failed to open terminal"))?;

        for (path, checked) in path_checked {
            let mut f = File::open(&path)
                .with_context(|_| format!("failed to open: '{}'", path.to_string_lossy()))?;
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

                for checked in checked.iter() {
                    if checked.beg == pos {
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
                        write!(term, "\t{}:{}:{}", path.to_string_lossy(), column, row);

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

    fn print_pretty(
        path_checked: &Vec<(PathBuf, Vec<Checked>)>,
        verbose: bool,
    ) -> Result<bool, Error> {
        let mut all_pass = true;
        let mut term = term::stdout().ok_or(format_err!("failed to open terminal"))?;

        for (path, checked) in path_checked {
            let mut f = File::open(&path)
                .with_context(|_| format!("failed to open: '{}'", path.to_string_lossy()))?;
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

                for checked in checked.iter() {
                    if checked.beg == pos {
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

                        let column_len = format!("{}", column).len();

                        let _ = term.fg(color::BRIGHT_WHITE);
                        write!(term, ": {}\n", checked.name);

                        let _ = term.fg(color::BRIGHT_BLUE);
                        write!(term, "   -->");

                        let _ = term.fg(color::WHITE);
                        write!(term, " {}:{}:{}\n", path.to_string_lossy(), column, row);

                        let _ = term.fg(color::BRIGHT_BLUE);
                        write!(term, "{}|\n", " ".repeat(column_len + 1));

                        let _ = term.fg(color::BRIGHT_BLUE);
                        write!(term, "{} |", column);

                        let _ = term.fg(color::WHITE);
                        write!(
                            term,
                            " {}\n",
                            String::from_utf8_lossy(&s.as_bytes()[last_lf + 1..next_crlf])
                        );

                        let _ = term.fg(color::BRIGHT_BLUE);
                        write!(term, "{}|", " ".repeat(column_len + 1));

                        let _ = term.fg(color::BRIGHT_YELLOW);
                        write!(
                            term,
                            " {}{}",
                            " ".repeat(pos - last_lf - 1),
                            "^".repeat(checked.end - checked.beg)
                        );

                        if checked.state == CheckedState::Fail {
                            let _ = term.fg(color::BRIGHT_YELLOW);
                            write!(term, " hint: {}\n\n", checked.hint);
                        } else {
                            write!(term, "\n\n");
                        }

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
