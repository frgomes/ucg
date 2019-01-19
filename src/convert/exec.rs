// Copyright 2017 Jeremy Wall <jeremy@marzhillstudios.com>
//
//  Licensed under the Apache License, Version 2.0 (the "License");
//  you may not use this file except in compliance with the License.
//  You may obtain a copy of the License at
//
//      http://www.apache.org/licenses/LICENSE-2.0
//
//  Unless required by applicable law or agreed to in writing, software
//  distributed under the License is distributed on an "AS IS" BASIS,
//  WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
//  See the License for the specific language governing permissions and
//  limitations under the License.

//! Contains code for converting a UCG Val into an executable script output target.
use std;
use std::io::{Cursor, Write};
use std::rc::Rc;

use crate::ast::{Position, PositionedItem};
use crate::build::Val;
use crate::build::Val::Tuple;
use crate::convert;
use crate::convert::traits::{Converter, Result};
use crate::error::BuildError;
use crate::error::ErrorType;

pub struct ExecConverter {}

// let exec = {
// env = [],
// command = "",
// args = [],
// };
impl ExecConverter {
    pub fn new() -> Self {
        ExecConverter {}
    }

    #[allow(unused_assignments)]
    fn write(&self, v: &Val, w: &mut Write) -> Result {
        // We always expect the Val to be a Tuple.
        if let &Tuple(ref fields) = v {
            // We expect no more than three fields in our exec tuple.
            if fields.len() > 3 {
                return Err(Box::new(BuildError::new(
                    "Exec tuples must have no more than 3 fields",
                    ErrorType::TypeFail,
                    Position::new(0, 0, 0),
                )));
            }
            let mut env: Option<&Vec<(PositionedItem<String>, Rc<Val>)>> = None;
            let mut command: Option<&str> = None;
            let mut args: Option<&Vec<Rc<Val>>> = None;
            for &(ref name, ref val) in fields.iter() {
                // We require a command field in our exec tuple.
                if name.val == "command" {
                    if command.is_some() {
                        return Err(Box::new(BuildError::new(
                            "There can only be one command field in an exec tuple",
                            ErrorType::TypeFail,
                            name.pos.clone(),
                        )));
                    }
                    if let &Val::Str(ref s) = val.as_ref() {
                        command = Some(s);
                        continue;
                    }
                    return Err(Box::new(BuildError::new(
                        "The command field of an exec tuple must be a string",
                        ErrorType::TypeFail,
                        name.pos.clone(),
                    )));
                }
                // We optionally allow an env field in our exec tuple.
                if name.val == "env" {
                    if let &Val::Tuple(ref l) = val.as_ref() {
                        if env.is_some() {
                            return Err(Box::new(BuildError::new(
                                "There can only be one env field in an exec tuple",
                                ErrorType::TypeFail,
                                name.pos.clone(),
                            )));
                        }
                        env = Some(l);
                        continue;
                    }
                    return Err(Box::new(BuildError::new(
                        "The env field of an exec tuple must be a list",
                        ErrorType::TypeFail,
                        name.pos.clone(),
                    )));
                }
                // We optionally allow an args field in our exec tuple.
                if name.val == "args" {
                    if let &Val::List(ref l) = val.as_ref() {
                        if args.is_some() {
                            return Err(Box::new(BuildError::new(
                                "There can only be one args field of an exec tuple",
                                ErrorType::TypeFail,
                                name.pos.clone(),
                            )));
                        }
                        args = Some(l);
                        continue;
                    }
                    return Err(Box::new(BuildError::new(
                        "The args field of an exec tuple must be a list",
                        ErrorType::TypeFail,
                        name.pos.clone(),
                    )));
                }
            }
            if command.is_none() {
                return Err(Box::new(BuildError::new(
                    "An exec tuple must have a command field",
                    ErrorType::TypeFail,
                    Position::new(0, 0, 0),
                )));
            }
            // Okay if we have made it this far then we are ready to start creating our script.
            let mut script = Cursor::new(vec![]);
            // 1. First the script prefix line.
            write!(script, "#!/usr/bin/env bash\n")?;
            // 2. then some initial setup. for bash hygiene.
            write!(script, "# Turn on unofficial Bash-Strict-Mode\n")?;
            write!(script, "set -euo pipefail\n")?;
            // 3. Then assign our environment variables
            if let Some(env_list) = env {
                for &(ref name, ref v) in env_list.iter() {
                    // We only allow string fields in our env tuple.
                    if let &Val::Str(ref s) = v.as_ref() {
                        write!(script, "{}=\"{}\"\n", name.val, s)?;
                        continue;
                    }
                    return Err(Box::new(BuildError::new(
                        "The env fields of an exec tuple must contain only string values",
                        ErrorType::TypeFail,
                        name.pos.clone(),
                    )));
                }
            }
            write!(script, "\n")?;
            // TODO(jwall): Should Flag converter have a strict mode?
            let flag_converter = convert::flags::FlagConverter::new();
            // 4. Then construct our command line. (be sure to use exec)
            write!(script, "exec {} ", command.unwrap())?;
            if let Some(arg_list) = args {
                for v in arg_list.iter() {
                    // We only allow tuples or strings in our args list.
                    match v.as_ref() {
                        &Val::Str(ref s) => {
                            write!(script, "{} ", s)?;
                        }
                        &Val::Tuple(_) => flag_converter.convert(v.clone(), &mut script)?,
                        _ => {
                            return Err(Box::new(BuildError::new(
                                "Exec args must be a list of strings or tuples of strings.",
                                ErrorType::TypeFail,
                                Position::new(0, 0, 0),
                            )));
                        }
                    }
                }
            }
            // Put cursor to the beginning of our script so when we copy
            // we copy the whole thing.
            script.set_position(0);
            std::io::copy(&mut script, w)?;
            return Ok(());
        }

        Err(Box::new(BuildError::new(
            "Exec outputs must be of type Tuple",
            ErrorType::TypeFail,
            Position::new(0, 0, 0),
        )))
    }
}

impl Converter for ExecConverter {
    fn convert(&self, v: Rc<Val>, mut w: &mut Write) -> Result {
        self.write(&v, &mut w)
    }

    fn file_ext(&self) -> String {
        String::from("sh")
    }

    fn description(&self) -> String {
        "Convert ucg Vals into an bash script with \nenvironment variables set and command line arguments sent..".to_string()
    }
}

#[cfg(test)]
mod exec_test {
    use super::*;
    use crate::build::assets::MemoryCache;
    use crate::build::FileBuilder;
    use crate::convert::traits::Converter;

    use std;
    use std::cell::RefCell;
    use std::io::Cursor;

    #[test]
    fn convert_just_command_test() {
        let i_paths = Vec::new();
        let cache = Rc::new(RefCell::new(MemoryCache::new()));
        let mut b = FileBuilder::new(std::env::current_dir().unwrap(), &i_paths, cache);
        let conv = ExecConverter::new();
        b.eval_string(
            "let script = {
            command = \"/bin/echo\",
        };",
        )
        .unwrap();
        let result = b.get_out_by_name("script").unwrap();
        let mut expected = "#!/usr/bin/env bash\n".to_string();
        expected.push_str("# Turn on unofficial Bash-Strict-Mode\n");
        expected.push_str("set -euo pipefail\n\n");
        expected.push_str("exec /bin/echo ");
        let mut buf = Cursor::new(vec![]);
        conv.convert(result, &mut buf).unwrap();
        assert_eq!(String::from_utf8_lossy(&buf.into_inner()), expected);
    }

    #[test]
    fn convert_command_with_env_test() {
        let i_paths = Vec::new();
        let cache = Rc::new(RefCell::new(MemoryCache::new()));
        let mut b = FileBuilder::new(std::env::current_dir().unwrap(), &i_paths, cache);
        let conv = ExecConverter::new();
        b.eval_string(
            "let script = {
            command = \"/bin/echo\",
            env = {
                foo = \"bar\",
                quux = \"baz\",
            },
        };",
        )
        .unwrap();
        let result = b.get_out_by_name("script").unwrap();
        let mut expected = "#!/usr/bin/env bash\n".to_string();
        expected.push_str("# Turn on unofficial Bash-Strict-Mode\n");
        expected.push_str("set -euo pipefail\n");
        expected.push_str("foo=\"bar\"\n");
        expected.push_str("quux=\"baz\"\n");
        expected.push_str("\n");
        expected.push_str("exec /bin/echo ");
        let mut buf = Cursor::new(vec![]);
        conv.convert(result, &mut buf).unwrap();
        assert_eq!(String::from_utf8_lossy(&buf.into_inner()), expected);
    }

    #[test]
    fn convert_command_with_arg_test() {
        let i_paths = Vec::new();
        let cache = Rc::new(RefCell::new(MemoryCache::new()));
        let mut b = FileBuilder::new(std::env::current_dir().unwrap(), &i_paths, cache);
        let conv = ExecConverter::new();
        b.eval_string(
            "let script = {
            command = \"/bin/echo\",
            env = {
                foo = \"bar\",
                quux = \"baz\",
            },
            args = [
                \"subcommand\",
                {flag1 = 1},
            ],
        };",
        )
        .unwrap();
        let result = b.get_out_by_name("script").unwrap();
        let mut expected = "#!/usr/bin/env bash\n".to_string();
        expected.push_str("# Turn on unofficial Bash-Strict-Mode\n");
        expected.push_str("set -euo pipefail\n");
        expected.push_str("foo=\"bar\"\n");
        expected.push_str("quux=\"baz\"\n");
        expected.push_str("\n");
        expected.push_str("exec /bin/echo subcommand --flag1 1 ");
        let mut buf = Cursor::new(vec![]);
        conv.convert(result, &mut buf).unwrap();
        assert_eq!(String::from_utf8_lossy(&buf.into_inner()), expected);
    }
}
