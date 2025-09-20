use std::path::Path;
#[cfg(feature = "analyzers")]
use tokei::{Config, Languages};

/// Compute total LOC and per-language breakdown.
pub fn compute_loc_breakdown(root: &Path) -> Option<(i64, Vec<(String, i64)>)> {
    #[cfg(feature = "analyzers")]
    {
        let mut languages = Languages::new();
        let config = Config::default();
        languages.get_statistics(&[root], &[], &config);
        let total = languages.total().code as i64;
        let breakdown = languages
            .iter()
            .map(|(lang, stats)| (lang.to_string(), stats.code as i64))
            .collect::<Vec<_>>();
        return Some((total, breakdown));
    }
    #[allow(unreachable_code)]
    None
}

/// Convenience to only return total LOC.
pub fn compute_loc(root: &Path) -> Option<i64> {
    compute_loc_breakdown(root).map(|(total, _)| total)
}
