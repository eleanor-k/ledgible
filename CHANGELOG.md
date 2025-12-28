# Changelog

## Version 0.4.0

- Add `--check` option for verifying journal formatting, printing a diff for
any discrepancies
- Remove trailing whitespace at end of journal
- Detect comment blocks and comments starting with `#`
- Remove explicit `panic!()`s from `lib.rs`
- Add that --inplace can be dangerous to help message
- Preserve comment delimiter
- Move types to `types.rs`
- Remove leading empty lines
- Change whitespace check for statuses to match file specification

## Version 0.3.0

- Rename `format.rs` to `lib.rs`
- Use `.tmp` file extension instead of `.old` for temp file when writing in
place
- Check for indentation instead of token count to determine if line is posting

## Version 0.2.0

- Remove extraneous whitespace from ledger
- Add support for posting status

## Version 0.1.1

- Fix semicolons being deleted

## Version 0.1.0

- Initial release
