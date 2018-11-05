use colored::*;
use failure::{Error, ResultExt};
use lint::{Checked, CheckedState};
use std::cmp;
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use term::{self, color, StdoutTerminal};

// -------------------------------------------------------------------------------------------------
// Color
// -------------------------------------------------------------------------------------------------

#[derive(PartialEq)]
#[allow(dead_code)]
enum Color {
    Black,
    Red,
    Green,
    Yellow,
    Blue,
    Magenta,
    Cyan,
    White,
    BrightBlack,
    BrightRed,
    BrightGreen,
    BrightYellow,
    BrightBlue,
    BrightMagenta,
    BrightCyan,
    BrightWhite,
    Reset,
}

// -------------------------------------------------------------------------------------------------
// Printer
// -------------------------------------------------------------------------------------------------

static CHAR_CR: u8 = 0x0d;
static CHAR_LF: u8 = 0x0a;

pub struct Printer {
    term: Option<Box<StdoutTerminal>>,
}

impl Printer {
    #[cfg_attr(tarpaulin, skip)]
    pub fn new() -> Printer {
        Printer {
            term: term::stdout(),
        }
    }

    #[cfg_attr(tarpaulin, skip)]
    pub fn print(
        &mut self,
        checked: Vec<Checked>,
        simple: bool,
        verbose: bool,
    ) -> Result<bool, Error> {
        let path_checked = Printer::collect_by_path(checked);

        let all_pass = if simple {
            self.print_simple(&path_checked, verbose)?
        } else {
            self.print_pretty(&path_checked, verbose)?
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

    #[cfg_attr(tarpaulin, skip)]
    fn write(&mut self, dat: &str, color: Color) {
        if let Some(ref mut term) = self.term {
            let term_color = match color {
                Color::Black => color::BLACK,
                Color::Red => color::RED,
                Color::Green => color::GREEN,
                Color::Yellow => color::YELLOW,
                Color::Blue => color::BLUE,
                Color::Magenta => color::MAGENTA,
                Color::Cyan => color::CYAN,
                Color::White => color::WHITE,
                Color::BrightBlack => color::BRIGHT_BLACK,
                Color::BrightRed => color::BRIGHT_RED,
                Color::BrightGreen => color::BRIGHT_GREEN,
                Color::BrightYellow => color::BRIGHT_YELLOW,
                Color::BrightBlue => color::BRIGHT_BLUE,
                Color::BrightMagenta => color::BRIGHT_MAGENTA,
                Color::BrightCyan => color::BRIGHT_CYAN,
                Color::BrightWhite => color::BRIGHT_WHITE,
                Color::Reset => color::BLACK,
            };
            if color == Color::Reset {
                let _ = term.reset();
            } else {
                let _ = term.fg(term_color);
            }
            write!(term, "{}", dat);
        } else {
            let colored = match color {
                Color::Black => dat.black(),
                Color::Red => dat.red(),
                Color::Green => dat.green(),
                Color::Yellow => dat.yellow(),
                Color::Blue => dat.blue(),
                Color::Magenta => dat.magenta(),
                Color::Cyan => dat.cyan(),
                Color::White => dat.white(),
                Color::BrightBlack => dat.bright_black(),
                Color::BrightRed => dat.bright_red(),
                Color::BrightGreen => dat.bright_green(),
                Color::BrightYellow => dat.bright_yellow(),
                Color::BrightBlue => dat.bright_blue(),
                Color::BrightMagenta => dat.bright_magenta(),
                Color::BrightCyan => dat.bright_cyan(),
                Color::BrightWhite => dat.bright_white(),
                Color::Reset => dat.clear(),
            };
            print!("{}", colored);
        }
    }

    #[cfg_attr(tarpaulin, skip)]
    fn print_simple(
        &mut self,
        path_checked: &[(PathBuf, Vec<Checked>)],
        verbose: bool,
    ) -> Result<bool, Error> {
        let mut all_pass = true;

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
                                self.write("Pass", Color::BrightGreen);
                            }
                            CheckedState::Fail => {
                                all_pass = false;
                                self.write("Fail", Color::BrightRed);
                            }
                            CheckedState::Skip => {
                                self.write("Skip", Color::BrightMagenta);
                            }
                        }

                        self.write(
                            &format!("\t{}:{}:{}", path.to_string_lossy(), column, row),
                            Color::BrightBlue,
                        );

                        self.write(
                            &format!(
                                "\t{}",
                                String::from_utf8_lossy(&s.as_bytes()[pos..next_crlf])
                            ),
                            Color::White,
                        );

                        self.write(&format!("\thint: {}\n", checked.hint), Color::BrightYellow);

                        self.write("", Color::Reset);
                    }
                }
            }
        }
        Ok(all_pass)
    }

    #[cfg_attr(tarpaulin, skip)]
    fn print_pretty(
        &mut self,
        path_checked: &[(PathBuf, Vec<Checked>)],
        verbose: bool,
    ) -> Result<bool, Error> {
        let mut all_pass = true;

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
                                self.write("Pass", Color::BrightGreen);
                            }
                            CheckedState::Fail => {
                                all_pass = false;
                                self.write("Fail", Color::BrightRed);
                            }
                            CheckedState::Skip => {
                                self.write("Skip", Color::BrightMagenta);
                            }
                        }

                        let column_len = format!("{}", column).len();

                        self.write(&format!(": {}\n", checked.name), Color::BrightWhite);

                        self.write("   -->", Color::BrightBlue);

                        self.write(
                            &format!(" {}:{}:{}\n", path.to_string_lossy(), column, row),
                            Color::White,
                        );

                        self.write(
                            &format!("{}|\n", " ".repeat(column_len + 1)),
                            Color::BrightBlue,
                        );

                        self.write(&format!("{} |", column), Color::BrightBlue);

                        self.write(
                            &format!(
                                " {}\n",
                                String::from_utf8_lossy(&s.as_bytes()[last_lf + 1..next_crlf])
                            ),
                            Color::White,
                        );

                        self.write(
                            &format!("{}|", " ".repeat(column_len + 1)),
                            Color::BrightBlue,
                        );

                        self.write(
                            &format!(
                                " {}{}",
                                " ".repeat(pos - last_lf - 1),
                                "^".repeat(cmp::min(checked.end, next_crlf) - checked.beg)
                            ),
                            Color::BrightYellow,
                        );

                        if checked.state == CheckedState::Fail {
                            self.write(
                                &format!(" hint: {}\n\n", checked.hint),
                                Color::BrightYellow,
                            );
                        } else {
                            self.write("\n\n", Color::BrightYellow);
                        }

                        self.write("", Color::Reset);
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_collect_by_path() {
        let mut checked = Vec::new();

        checked.push(Checked {
            path: PathBuf::from("bbb"),
            beg: 100,
            end: 200,
            state: CheckedState::Pass,
            name: String::from(""),
            hint: String::from(""),
        });

        checked.push(Checked {
            path: PathBuf::from("aaa"),
            beg: 10,
            end: 20,
            state: CheckedState::Pass,
            name: String::from(""),
            hint: String::from(""),
        });

        checked.push(Checked {
            path: PathBuf::from("aaa"),
            beg: 0,
            end: 10,
            state: CheckedState::Pass,
            name: String::from(""),
            hint: String::from(""),
        });

        checked.push(Checked {
            path: PathBuf::from("bbb"),
            beg: 20,
            end: 30,
            state: CheckedState::Pass,
            name: String::from(""),
            hint: String::from(""),
        });

        let path_checked = Printer::collect_by_path(checked);

        assert_eq!(format!("{}", path_checked[0].0.to_string_lossy()), "aaa");
        assert_eq!(format!("{}", path_checked[1].0.to_string_lossy()), "bbb");
        assert_eq!(path_checked[0].1[0].beg, 0);
        assert_eq!(path_checked[0].1[1].beg, 10);
        assert_eq!(path_checked[1].1[0].beg, 20);
        assert_eq!(path_checked[1].1[1].beg, 100);
    }
}
