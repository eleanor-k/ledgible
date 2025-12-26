/*
 * ledgible - Formatter for ledger and hledger journals
 * Copyright (C) 2025  Eleanor Kelley <me at eleanorkelley dot com>
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program.  If not, see <http://www.gnu.org/licenses/>.
 */

mod diff;

use crate::diff::{make_diff, print_diff};
use clap::{Arg, command, error::ErrorKind};
use std::{
    fs::write,
    io::{Read, stdin},
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = command!()
        .arg(Arg::new("input").value_name("FILE").help("Input journal"))
        .arg(
            Arg::new("output")
                .short('o')
                .long("output")
                .value_name("FILE")
                .help("Write formatted journal to file"),
        )
        .arg(
            Arg::new("overwrite")
                .short('i')
                .long("inplace")
                .requires("input")
                .conflicts_with("output")
                .action(clap::ArgAction::SetTrue)
                .help("Overwrite input file"),
        )
        .arg(
            Arg::new("env")
                .short('e')
                .long("env")
                .conflicts_with("input")
                .action(clap::ArgAction::SetTrue)
                .help("Read journal from $LEDGER_FILE"),
        )
        .arg(
            Arg::new("check")
                .short('c')
                .long("check")
                .conflicts_with("overwrite")
                .action(clap::ArgAction::SetTrue)
                .help("Check whether journal is formatted properly"),
        );
    let matches = cmd.get_matches_mut();

    // Set `input` to correct file
    let input = match matches.get_flag("env") {
        true => match std::env::var("LEDGER_FILE") {
            Ok(file) => Some(&file.clone()),
            Err(_) => panic!("Error reading $LEDGER_FILE"),
        },
        false => matches.get_one::<String>("input"),
    };

    // Read appropriate input into `ledger`
    let mut ledger = String::new();
    match input {
        Some(file) => {
            ledger = match std::fs::read_to_string(file) {
                Ok(file) => file,
                Err(_) => cmd.error(ErrorKind::Io, "Cannot read file").exit(),
            }
        }
        None => {
            let _ = stdin().read_to_string(&mut ledger).unwrap();
        }
    }

    let mut buffer = String::new();

    ledgible::format(&mut buffer, &ledger)?;

    if matches.get_flag("check") {
        let mismatch = make_diff(&ledger, &buffer, 3);
        std::process::exit(match mismatch.is_empty() {
            true => 0,
            false => {
                print_diff(mismatch, |n| format!("Diff at line {}:", n));
                1
            }
        })
    }

    // Write output
    match matches.get_one::<String>("output") {
        Some(file) => write(file, buffer)?,
        None => match matches.get_flag("overwrite") {
            true => {
                let tempfile = format!("{}.tmp", input.unwrap());
                write(&tempfile, buffer)?;
                std::fs::rename(tempfile, input.unwrap())?
            }
            false => print!("{buffer}"),
        },
    };

    Ok(())
}
