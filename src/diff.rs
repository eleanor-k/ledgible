/*
 * Copyright 2025 Eleanor Kelley <me at eleanorkelley dot com>
 * Copyright 2016-2021 The Rust Project Developers
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 * http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express
 * or implied. See the License for the specific language governing
 * permissions and limitations under the License.
 */

use std::collections::VecDeque;
use std::io;
use std::io::Write;

#[derive(Debug, PartialEq)]
pub(crate) enum DiffLine {
    Context(String),
    Expected(String),
    Resulting(String),
}

#[derive(Debug, PartialEq)]
pub(crate) struct Mismatch {
    /// The line number in the formatted version.
    pub(crate) line_number: u32,
    /// The line number in the original version.
    pub(crate) line_number_orig: u32,
    /// The set of lines (context and old/new) in the mismatch.
    pub(crate) lines: Vec<DiffLine>,
}

impl Mismatch {
    fn new(line_number: u32, line_number_orig: u32) -> Mismatch {
        Mismatch {
            line_number,
            line_number_orig,
            lines: Vec::new(),
        }
    }
}

// This struct handles writing output to stdout and abstracts away the logic
// of printing in color, if it's possible in the executing environment.
pub(crate) struct OutputWriter {
    terminal: Option<Box<dyn term::Terminal<Output = io::Stdout>>>,
}

impl OutputWriter {
    // Create a new OutputWriter instance based on the caller's preference
    // for colorized output and the capabilities of the terminal.
    pub(crate) fn new() -> Self {
        if let Some(t) = term::stdout()
            && t.supports_color()
        {
            return OutputWriter { terminal: Some(t) };
        }
        OutputWriter { terminal: None }
    }

    // Write output in the optionally specified color. The output is written
    // in the specified color if this OutputWriter instance contains a
    // Terminal in its `terminal` field.
    pub(crate) fn writeln(&mut self, msg: &str, color: Option<term::color::Color>) {
        match &mut self.terminal {
            Some(t) => {
                if let Some(color) = color {
                    t.fg(color).unwrap();
                }
                writeln!(t, "{msg}").unwrap();
                if color.is_some() {
                    t.reset().unwrap();
                }
            }
            None => println!("{msg}"),
        }
    }
}

// Produces a diff between the expected output and actual output of rustfmt.
pub(crate) fn make_diff(expected: &str, actual: &str, context_size: usize) -> Vec<Mismatch> {
    let mut line_number = 1;
    let mut line_number_orig = 1;
    let mut context_queue: VecDeque<&str> = VecDeque::with_capacity(context_size);
    let mut lines_since_mismatch = context_size + 1;
    let mut results = Vec::new();
    let mut mismatch = Mismatch::new(0, 0);

    for result in diff::lines(expected, actual) {
        match result {
            diff::Result::Left(str) => {
                if lines_since_mismatch >= context_size && lines_since_mismatch > 0 {
                    results.push(mismatch);
                    mismatch = Mismatch::new(
                        line_number - context_queue.len() as u32,
                        line_number_orig - context_queue.len() as u32,
                    );
                }

                while let Some(line) = context_queue.pop_front() {
                    mismatch.lines.push(DiffLine::Context(line.to_owned()));
                }

                mismatch.lines.push(DiffLine::Resulting(str.to_owned()));
                line_number_orig += 1;
                lines_since_mismatch = 0;
            }
            diff::Result::Right(str) => {
                if lines_since_mismatch >= context_size && lines_since_mismatch > 0 {
                    results.push(mismatch);
                    mismatch = Mismatch::new(
                        line_number - context_queue.len() as u32,
                        line_number_orig - context_queue.len() as u32,
                    );
                }

                while let Some(line) = context_queue.pop_front() {
                    mismatch.lines.push(DiffLine::Context(line.to_owned()));
                }

                mismatch.lines.push(DiffLine::Expected(str.to_owned()));
                line_number += 1;
                lines_since_mismatch = 0;
            }
            diff::Result::Both(str, _) => {
                if context_queue.len() >= context_size {
                    let _ = context_queue.pop_front();
                }

                if lines_since_mismatch < context_size {
                    mismatch.lines.push(DiffLine::Context(str.to_owned()));
                } else if context_size > 0 {
                    context_queue.push_back(str);
                }

                line_number += 1;
                line_number_orig += 1;
                lines_since_mismatch += 1;
            }
        }
    }

    results.push(mismatch);
    results.remove(0);

    results
}

pub(crate) fn print_diff<F>(diff: Vec<Mismatch>, get_section_title: F)
where
    F: Fn(u32) -> String,
{
    let mut writer = OutputWriter::new();

    for mismatch in diff {
        writer.writeln(&get_section_title(mismatch.line_number_orig), None);

        for line in mismatch.lines {
            match line {
                DiffLine::Context(str) => writer.writeln(&format!(" {str}"), None),
                DiffLine::Expected(str) => {
                    writer.writeln(&format!("+{str}"), Some(term::color::GREEN))
                }

                DiffLine::Resulting(str) => {
                    writer.writeln(&format!("-{str}"), Some(term::color::RED))
                }
            }
        }
    }
}
