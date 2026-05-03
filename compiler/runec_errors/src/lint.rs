pub enum LintAdm {      // full name: LintAdmissibility
    Allow,
    Deny,
    Warn,
}

const KNOWN_LINTS: &[&str] = &[
    "unknown_lint"
];

/// Returns `true` if the given string contains in KNOWN_LINTS.
pub fn is_lint_known(s: &str) -> bool {
    KNOWN_LINTS.contains(&s)
}

#[derive(Debug)]
pub struct Lint<'diag>(&'diag str);

impl<'diag> Lint<'diag> {
    /// Creates a new `Lint` from a string slice.
    pub const fn from_str(s: &'diag str) -> Self {
        Lint(s)
    }

    /// Returns `true` if this lint matches the given string.
    ///
    /// # Examples
    /// ```
    /// let lint = crate::runec_errors::lint::Lint::from_str("lint_type");
    /// assert!(lint.is("lint_type"));
    /// assert!(!lint.is("not_lint_type"));
    /// ```
    pub fn is(&self, s: &str) -> bool {
        self.as_str() == s
    }

    /// Returns `true` if this lint matches the given string.
    ///
    /// # Examples
    /// ```
    /// let lint = crate::runec_errors::lint::Lint::from_str("lint_type");
    /// assert!(lint.contains_in(&["lint_type", "other_lint_type"]));
    /// assert!(!lint.contains_in(&["not_lint_type", "other_lint_type"]));
    /// ```
    pub fn contains_in(&self, slice: &[&str]) -> bool {
        slice.iter().any(|s| self.is(s))
    }

    /// Returns the inner string slice representing the lint.
    ///
    /// # Examples
    /// ```
    /// let lint = crate::runec_errors::lint::Lint::from_str("lint_type");
    /// assert_eq!(lint.as_str(), "lint_type");
    /// ```
    pub const fn as_str(&self) -> &'diag str {
        self.0
    }
}
