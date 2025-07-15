//! Environment and configuration utilities.

use std::env;
use std::ffi::OsString;

// I'm not sure all of this logic really makes sense -- some of it may be
// specific to my own personal preferences -- but let's use this until
// someone complains.
//
// In the Ruby tool, I do, in fact, force "RS" if --oneline is selected,
// similarly to what I do here, so perhaps the logic following the
// retrieval of $LESS should simply be
//
//     let less = if *oneline { "RS" } else { less };
//
// However, since I send ANSI color codes whenever we are hooked up to a
// tty, I definitely want "R" to be included, so if I instead respect
// the user's possible absence of "R", I should make sure I only send
// ANSI color codes when "R" is included in $LESS.
//
// Specifically, the Ruby tool includes this code (spread around the
// codebase, but listed here contiguously for clarity):
//
//    ENV['LESS'] = 'RS' if options[:oneline]
//    ENV['LESS'] = 'FSRX' unless ENV['LESS']
//
// Oy vey.
//
// Also, I should test this with various values of $LESS. For example,
// my $LESS is simply set to "R", but I should test output when the
// default option of "FSRX is used.

/// Returns an appropriate vector of environment variables to pass to the pager.
///
/// By default, this is `FSRX`, unless the user has defined `$LESS` in the
/// environment. However, because text is printed in color, `R` is always
/// included regardless of the value of `$LESS` (it is appended to `$LESS` if
/// not already present), and when output is printed to oneline (via the
/// `--oneline` option), `S` is appended to `$LESS` if not already present.
///
/// This ensures that output is pleasant for the user, regardless of the
/// definition of `$LESS`.
///
/// # Examples
///
/// `pager_env` will return a default value if `$LESS` is not set:
///
/// ```
/// use usaidwat::conf::pager_env;
/// # use temp_env::with_var_unset;
/// # with_var_unset("LESS", || {
/// let less = pager_env(&false).into_iter().nth(0).unwrap().into();
/// assert_eq!(less, "LESS=FSRX");
/// # });
/// ```
///
/// It will include `S` if `oneline` is `true`:
///
/// ```
/// use usaidwat::conf::pager_env;
/// # use temp_env::with_var_unset;
/// # with_var_unset("LESS", || {
/// let less = pager_env(&true).into_iter().nth(0).unwrap().into();
/// assert_eq!(less, "LESS=FSRX");
/// # });
/// ```
///
/// In this example, `$LESS` was set to `SX`, but `R` will be appended anyway:
///
/// ```
/// use usaidwat::conf::pager_env;
/// # use temp_env::with_var;
/// # with_var("LESS", Some("SX"), || {
/// let less = pager_env(&false).into_iter().nth(0).unwrap().into();
/// assert_eq!(less, "LESS=SXR");
/// # });
/// ```
///
/// In this example, `$LESS` was set to `RSX`. Note that `R` is still included,
/// but `$LESS` was not altered since `R` was already in it:
///
/// ```
/// use usaidwat::conf::pager_env;
/// # use temp_env::with_var;
/// # with_var("LESS", Some("RSX"), || {
/// let less = pager_env(&false).into_iter().nth(0).unwrap().into();
/// assert_eq!(less, "LESS=RSX");
/// # });
/// ```
///
/// In this example, `$LESS` was set to `R`. Because the `oneline` option is
/// `true`, `S` is also appended:
///
/// ```
/// use usaidwat::conf::pager_env;
/// # use temp_env::with_var;
/// # with_var("LESS", Some("R"), || {
/// let less = pager_env(&true).into_iter().nth(0).unwrap().into();
/// assert_eq!(less, "LESS=RS");
/// # });
/// ```
///
/// In this example, `$LESS` was set to `SR`. Because the `oneline` option is
/// `true`, `S` is still included, but because it is already present, the
/// value of `$LESS` does not change:
///
/// ```
/// use usaidwat::conf::pager_env;
/// # use temp_env::with_var;
/// # with_var("LESS", Some("SR"), || {
/// let less = pager_env(&true).into_iter().nth(0).unwrap().into();
/// assert_eq!(less, "LESS=SR");
/// # });
/// ```
pub fn pager_env(oneline: &bool) -> impl IntoIterator<Item = impl Into<OsString>> {
    // Get the value of $LESS, defaulting to "FSRX" if $LESS is unset.
    let less = env::var_os("LESS").unwrap_or(
        "FSRX"
            .parse()
            .expect("could not parse 'FSRX' into OsString"),
    );
    let less = less.to_string_lossy();

    // Always interpret ANSI color escape sequences.
    let less = if !less.contains("R") {
        less + "R"
    } else {
        less
    };

    // When printing to one line, really print to one line, and force scrolling
    // to the right if lines are too long.
    let less = if *oneline && !less.contains("S") {
        less + "S"
    } else {
        less
    };

    vec![format!("LESS={less}")]
}
