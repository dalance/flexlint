use failure::{Error, ResultExt};
use glob::glob;
use regex::Regex;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};

// -------------------------------------------------------------------------------------------------
// RuleSet
// -------------------------------------------------------------------------------------------------

#[derive(Deserialize, Debug)]
pub struct RuleSet {
    pub rules: Vec<Rule>,
}

impl RuleSet {
    #[cfg_attr(tarpaulin, skip)]
    pub fn check(&self) -> Result<Vec<Checked>, Error> {
        let mut ret = Vec::new();
        for rule in &self.rules {
            ret.append(&mut rule.check()?);
        }
        Ok(ret)
    }
}

// -------------------------------------------------------------------------------------------------
// Rule
// -------------------------------------------------------------------------------------------------

#[derive(Deserialize, Debug)]
pub struct Rule {
    pub name: String,

    #[serde(with = "serde_regex")]
    pub pattern: Regex,

    #[serde(with = "serde_option_regex", default)]
    pub required: Option<Regex>,

    #[serde(with = "serde_option_regex", default)]
    pub forbidden: Option<Regex>,

    #[serde(with = "serde_option_regex", default)]
    pub ignore: Option<Regex>,

    pub hint: String,

    pub globs: Vec<String>,
}

mod serde_regex {
    use regex::Regex;
    use serde::{self, Deserialize, Deserializer};

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Regex, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let r = Regex::new(&s).map_err(serde::de::Error::custom)?;
        Ok(r)
    }
}

mod serde_option_regex {
    use regex::Regex;
    use serde::{self, Deserialize, Deserializer};

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<Regex>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let r = Regex::new(&s).map_err(serde::de::Error::custom)?;
        Ok(Some(r))
    }
}

impl Rule {
    #[cfg_attr(tarpaulin, skip)]
    pub fn check(&self) -> Result<Vec<Checked>, Error> {
        let mut ret = Vec::new();
        for g in &self.globs {
            for entry in glob(&g).with_context(|_| format!("failed to parse glob: '{}'", g))? {
                let entry = entry?;
                let mut f = File::open(&entry)
                    .with_context(|_| format!("failed to open: '{}'", entry.to_string_lossy()))?;
                let mut s = String::new();
                let _ = f.read_to_string(&mut s);

                let ignore = self.gen_ignore(&s);
                let mut checked = self.gen_checked(&entry, &s, &ignore);

                ret.append(&mut checked);
            }
        }
        Ok(ret)
    }

    fn gen_ignore(&self, src: &str) -> Vec<(usize, usize)> {
        let mut ret = Vec::new();
        if let Some(ref ignore) = self.ignore {
            for m in ignore.find_iter(&src) {
                ret.push((m.start(), m.end()));
            }
        }
        ret
    }

    fn gen_checked(&self, entry: &Path, src: &str, ignore: &[(usize, usize)]) -> Vec<Checked> {
        let mut ret = Vec::new();
        for m in self.pattern.find_iter(&src) {
            let pat_start = m.start();
            let pat_end = m.end();
            let mut pass = true;
            let mut skip = false;

            for (beg, end) in ignore {
                if *beg <= pat_start && pat_start < *end {
                    skip = true;
                }
            }

            if !skip {
                if let Some(ref required) = self.required {
                    pass &= match required.find_at(&src, pat_start) {
                        Some(x) => x.start() == pat_start,
                        None => false,
                    };
                }

                if let Some(ref forbidden) = self.forbidden {
                    pass &= match forbidden.find_at(&src, pat_start) {
                        Some(x) => x.start() != pat_start,
                        None => true,
                    };
                }
            }

            let state = if skip {
                CheckedState::Skip
            } else if pass {
                CheckedState::Pass
            } else {
                CheckedState::Fail
            };

            let checked = Checked {
                path: entry.to_path_buf(),
                beg: pat_start,
                end: pat_end,
                state,
                name: self.name.clone(),
                hint: self.hint.clone(),
            };

            ret.push(checked);
        }
        ret
    }
}

// -------------------------------------------------------------------------------------------------
// Checked
// -------------------------------------------------------------------------------------------------

#[derive(Debug)]
pub struct Checked {
    pub path: PathBuf,
    pub beg: usize,
    pub end: usize,
    pub state: CheckedState,
    pub name: String,
    pub hint: String,
}

#[derive(Debug, PartialEq)]
pub enum CheckedState {
    Pass,
    Fail,
    Skip,
}

// -------------------------------------------------------------------------------------------------
// Test
// -------------------------------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use toml;

    static TOML_SAMPLE: &'static str = r#"
[[rules]]
name      = "aaa"
pattern   = 'bbb'
required  = 'ccc'
forbidden = 'ddd'
ignore    = 'eee'
hint      = "fff"
globs     = ["ggg"]
        "#;

    static C_RULE: &'static str = r#"
[[rules]]
name      = "'if' with brace"
pattern   = '(?m)(^|[\t ])if\s'
forbidden = '(?m)(^|[\t ])if\s[^;{]*$'
ignore    = '(/\*/?([^/]|[^*]/)*\*/)|(//.*\n)'
hint      = "multiline 'if' must have brace"
globs     = ["**/*.c", "**/*.cpp"]
        "#;

    static C_SRC: &'static str = r#"
int test() {
    int hoge = 0;

    if ( hoge )
        return 1;

    if ( hoge ) return 1;

    if ( hoge ) {
        return 1;
    }

    // if ( hoge )
    //     return 1;

    /*
    if ( hoge )
        return 1;
    */
}
        "#;

    static VERILOG_RULE: &'static str = r#"
[[rules]]
name     = "'if' with 'begin'"
pattern  = '(?m)(^|[\t ])if\s'
required = '(?m)(^|[\t ])if\s*\([^)]*\)\s*begin'
ignore   = '(/\*/?([^/]|[^*]/)*\*/)|(//.*\n)'
hint     = "'if' statement must have 'begin'"
globs    = ["**/*.v", "**/*.sv"]
        "#;

    static VERILOG_SRC: &'static str = r#"
module test ();

    wire clk;
    wire rst;

    reg reg1;
    always @ ( posedge clk ) begin
        reg1 <= 0;
    end

    reg reg2;
    always_ff @ ( posedge clk or negedge rst ) begin
        if ( rst )
            reg2 <= 0;
        else
            reg2 <= 1;
    end

    reg reg3;
    always_comb begin
        reg3 = reg0 | reg1;
    end

endmodule
        "#;

    #[test]
    fn test_deserialize_ruleset() {
        let rule: RuleSet = toml::from_str(&TOML_SAMPLE).unwrap();
        assert_eq!(rule.rules[0].name, "aaa");
        assert_eq!(
            format!("{:?}", rule.rules[0].pattern),
            format!("{:?}", Regex::new("bbb").unwrap())
        );
        assert_eq!(
            format!("{:?}", rule.rules[0].required),
            format!("{:?}", Some(Regex::new("ccc").unwrap()))
        );
        assert_eq!(
            format!("{:?}", rule.rules[0].forbidden),
            format!("{:?}", Some(Regex::new("ddd").unwrap()))
        );
        assert_eq!(
            format!("{:?}", rule.rules[0].ignore),
            format!("{:?}", Some(Regex::new("eee").unwrap()))
        );
        assert_eq!(rule.rules[0].hint, "fff");
        assert_eq!(rule.rules[0].globs[0], "ggg");
    }

    #[test]
    fn test_gen_ignore() {
        let rule: RuleSet = toml::from_str(&C_RULE).unwrap();
        let ignore = rule.rules[0].gen_ignore(&C_SRC);
        assert_eq!(ignore.len(), 3);
        assert_eq!(ignore[0], (142, 157));
        assert_eq!(ignore[1], (161, 178));
        assert_eq!(ignore[2], (183, 226));
    }

    #[test]
    fn test_gen_checked_with_required() {
        let rule: RuleSet = toml::from_str(&VERILOG_RULE).unwrap();
        let ignore = rule.rules[0].gen_ignore(&VERILOG_SRC);
        let checked = rule.rules[0].gen_checked(&PathBuf::from(""), &VERILOG_SRC, &ignore);
        assert_eq!(checked.len(), 1);
        assert_eq!(checked[0].state, CheckedState::Fail);
        assert_eq!(checked[0].beg, 198);
        assert_eq!(checked[0].end, 202);
    }

    #[test]
    fn test_gen_checked_with_forbidden() {
        let rule: RuleSet = toml::from_str(&C_RULE).unwrap();
        let ignore = rule.rules[0].gen_ignore(&C_SRC);
        let checked = rule.rules[0].gen_checked(&PathBuf::from(""), &C_SRC, &ignore);
        assert_eq!(checked.len(), 5);
        assert_eq!(checked[0].state, CheckedState::Fail);
        assert_eq!(checked[0].beg, 36);
        assert_eq!(checked[0].end, 40);
        assert_eq!(checked[1].state, CheckedState::Pass);
        assert_eq!(checked[1].beg, 71);
        assert_eq!(checked[1].end, 75);
        assert_eq!(checked[2].state, CheckedState::Pass);
        assert_eq!(checked[2].beg, 98);
        assert_eq!(checked[2].end, 102);
        assert_eq!(checked[3].state, CheckedState::Skip);
        assert_eq!(checked[3].beg, 144);
        assert_eq!(checked[3].end, 148);
        assert_eq!(checked[4].state, CheckedState::Skip);
        assert_eq!(checked[4].beg, 189);
        assert_eq!(checked[4].end, 193);
    }
}
