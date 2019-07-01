// Copyright 2019 Jeremy Wall
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.
use std::collections::{BTreeMap, BTreeSet};
use std::iter::FromIterator;

use super::{Error, Value};

#[derive(Clone, PartialEq, Debug)]
pub enum Bindings {
    Sealed(BTreeMap<String, Value>),
    Open(BTreeMap<String, Value>),
}
use Bindings::{Open, Sealed};

impl Bindings {
    pub fn new() -> Self {
        Sealed(BTreeMap::new())
    }

    pub fn into_open(self) -> Self {
        match self {
            Open(flds) => Open(flds),
            Sealed(flds) => Open(flds),
        }
    }

    pub fn into_sealed(self) -> Self {
        match self {
            Open(flds) => Sealed(flds),
            Sealed(flds) => Sealed(flds),
        }
    }

    pub fn get(&self, name: &str) -> Option<&Value> {
        match self {
            Open(flds) => flds.get(name),
            Sealed(flds) => flds.get(name),
        }
    }

    pub fn add(&mut self, name: String, val: Value) {
        match self {
            Sealed(flds) => flds.insert(name, val),
            Open(flds) => flds.insert(name, val),
        };
    }

    pub fn symbol_list(&self) -> Vec<&String> {
        match self {
            Sealed(flds) => flds.keys().collect(),
            Open(flds) => flds.keys().collect(),
        }
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct Stack {
    curr: Bindings,
    prev: Vec<Bindings>,
}

impl Stack {
    pub fn new() -> Self {
        Stack {
            curr: Bindings::new(),
            prev: Vec::new(),
        }
    }

    pub fn get(&self, name: &str) -> Option<&Value> {
        match &self.curr {
            Sealed(flds) => flds.get(name),
            Open(flds) => {
                if let Some(v) = flds.get(name) {
                    return Some(v);
                } else {
                    for b in self.prev.iter() {
                        match b {
                            Sealed(bflds) => {
                                if let Some(v) = bflds.get(name) {
                                    return Some(v);
                                }
                                return None;
                            }
                            Open(bflds) => {
                                if let Some(v) = bflds.get(name) {
                                    return Some(v);
                                }
                            }
                        }
                    }
                }
                return None;
            }
        }
    }

    pub fn push(&mut self) {
        let mut nb = Bindings::new();
        std::mem::swap(&mut nb, &mut self.curr);
        self.prev.push(nb);
    }

    pub fn to_open(&mut self) {
        let mut tmp = Bindings::new();
        std::mem::swap(&mut tmp, &mut self.curr);
        tmp = tmp.into_open();
        std::mem::swap(&mut tmp, &mut self.curr);
    }

    pub fn pop(&mut self) -> Result<(), Error> {
        if let Some(parent) = self.prev.pop() {
            self.curr = parent;
            Ok(())
        } else {
            dbg!(Err(Error {}))
        }
    }

    pub fn add(&mut self, name: String, val: Value) {
        self.curr.add(name, val);
    }

    pub fn symbol_list(&self) -> Vec<&String> {
        match &self.curr {
            Sealed(flds) => flds.keys().collect(),
            Open(flds) => {
                let mut keys = BTreeSet::new();
                keys.extend(flds.keys());
                for b in self.prev.iter() {
                    match b {
                        Sealed(bflds) => {
                            keys.extend(bflds.keys());
                            return Vec::from_iter(keys.iter().cloned());
                        }
                        Open(bflds) => {
                            keys.extend(bflds.keys());
                        }
                    }
                }
                return Vec::from_iter(keys.iter().cloned());
            }
        }
    }

    pub fn snapshot(&self) -> Self {
        let curr = self.curr.clone();
        match curr {
            Sealed(_) => Self {
                curr: curr,
                prev: Vec::new(),
            },
            Open(_) => {
                let mut prev = Vec::new();
                for b in self.prev.iter() {
                    match b {
                        Sealed(_) => {
                            prev.push(b.clone());
                            break;
                        }
                        Open(_) => prev.push(b.clone()),
                    }
                }
                Self { curr, prev }
            }
        }
    }
}