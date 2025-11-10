/*
 * ledgible - Formatter for hledger journals
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

use std::io::{BufRead, BufReader};

fn main() -> std::io::Result<()> {
    let mut lines = BufReader::new(std::fs::File::open(match std::env::var("LEDGER_FILE") {
        Ok(val) => val,
        Err(_) => String::from("~/.hledger.journal"),
    })?)
    .lines();

    while let Some(line) = lines.next() {
        let line = line?;
        let line: Vec<&str> = line.split("  ").filter(|x| !x.is_empty()).collect();
        for item in line {
            if item.starts_with(";") {
                break;
            }
            print!("{item} / ");
        }
        println!();
    }

    Ok(())
}
