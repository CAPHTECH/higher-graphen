use super::CliError;
use std::ffi::OsString;

pub(super) fn required_segment(
    args: &mut impl Iterator<Item = OsString>,
    expected: &'static str,
) -> Result<OsString, CliError> {
    args.next()
        .ok_or_else(|| CliError::usage(format!("missing command segment {expected:?}")))
}
