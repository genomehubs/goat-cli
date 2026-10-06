use clap::parser::MatchesError;
use std::any::Any;

/// Lenient argument access for [`clap::ArgMatches`].
///
/// Several code paths (e.g. [`crate::report::report::Report::new`] and
/// [`crate::utils::cli_matches::process_cli_args`]) are shared between
/// subcommands which do not all define the same arguments. `get_one` panics
/// on an undefined argument id in debug builds, so use `opt_one` instead,
/// which treats an argument the subcommand does not define as absent.
pub trait ArgMatchesExt {
    /// Like [`clap::ArgMatches::get_one`], but returns `None` if this
    /// subcommand does not define `id`.
    ///
    /// Still panics if `id` is defined with a different type to `T`, as
    /// that is a programming error.
    fn opt_one<T: Any + Clone + Send + Sync + 'static>(&self, id: &str) -> Option<&T>;

    /// Returns `true` only if the `SetTrue` flag `id` is defined and set.
    fn flag(&self, id: &str) -> bool {
        self.opt_one::<bool>(id).copied().unwrap_or(false)
    }
}

impl ArgMatchesExt for clap::ArgMatches {
    fn opt_one<T: Any + Clone + Send + Sync + 'static>(&self, id: &str) -> Option<&T> {
        match self.try_get_one::<T>(id) {
            Ok(value) => value,
            Err(MatchesError::UnknownArgument { .. }) => None,
            Err(e) => panic!("Mismatch between definition and access of `{}`. {}", id, e),
        }
    }
}
