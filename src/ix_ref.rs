//! The `ix://<org>/<repo>/<path>` reference shape, parsed in one place for
//! Filament edge refs (FR-045) and relationship targets (FR-076).

/// A parsed `ix://<org>/<repo>/<path>` reference: `org`, `repo`, and the
/// first `path` segment are non-empty; `path` may hold further segments.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct IxRef<'a> {
    package: &'a str,
    org: &'a str,
    repo: &'a str,
    path: &'a str,
}

impl<'a> IxRef<'a> {
    /// Parse `value`; `None` when it is not an `ix://` reference with an
    /// org, a repo, and a non-empty first path segment.
    pub(crate) fn parse(value: &'a str) -> Option<Self> {
        let rest = value.strip_prefix("ix://")?;
        let (org, after_org) = rest.split_once('/')?;
        let (repo, path) = after_org.split_once('/')?;
        let first = path.split('/').next().unwrap_or_default();
        if org.is_empty() || repo.is_empty() || first.is_empty() {
            return None;
        }
        Some(Self {
            package: rest.get(..org.len() + 1 + repo.len())?,
            org,
            repo,
            path,
        })
    }

    /// `<org>/<repo>`.
    pub(crate) fn package(&self) -> &'a str {
        self.package
    }

    pub(crate) fn org(&self) -> &'a str {
        self.org
    }

    pub(crate) fn repo(&self) -> &'a str {
        self.repo
    }

    /// Everything after `<org>/<repo>/`.
    pub(crate) fn path(&self) -> &'a str {
        self.path
    }

    /// The last `/`-segment of the path: the artifact id an FR-026
    /// reference resolves to.
    pub(crate) fn last_segment(&self) -> &'a str {
        self.path.rsplit('/').next().unwrap_or(self.path)
    }
}

#[cfg(test)]
mod tests {
    use super::IxRef;

    #[test]
    fn parses_org_repo_and_path() {
        let r = IxRef::parse("ix://agent-ix/quoin/spec/FR-001").unwrap();
        assert_eq!(
            (r.org(), r.repo(), r.package(), r.path(), r.last_segment()),
            (
                "agent-ix",
                "quoin",
                "agent-ix/quoin",
                "spec/FR-001",
                "FR-001"
            )
        );
    }

    #[test]
    fn refuses_missing_or_empty_segments() {
        for bad in [
            "agent-ix/quoin/FR-001",
            "ix://agent-ix",
            "ix://agent-ix/quoin",
            "ix://agent-ix/quoin/",
            "ix:///quoin/FR-001",
            "ix://agent-ix//FR-001",
        ] {
            assert!(IxRef::parse(bad).is_none(), "{bad}");
        }
    }
}
