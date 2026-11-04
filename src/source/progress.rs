const PERCENT: f32 = 100.0;

/// How far along a package manager says it is in `line`: the last `NN%`, else
/// the last `n/m` step (as `(2/5) installing …` or `Installing 2/5…`).
pub fn progress_of(line: &str) -> Option<f32> {
    percent(line).or_else(|| step(line))
}

fn percent(line: &str) -> Option<f32> {
    line.rmatch_indices('%').find_map(|(end, _)| {
        let digits = line[..end].rsplit(|c: char| !c.is_ascii_digit()).next()?;
        let value: f32 = digits.parse().ok()?;
        (value <= PERCENT).then_some(value / PERCENT)
    })
}

fn step(line: &str) -> Option<f32> {
    line.split_whitespace().rev().find_map(|token| {
        let token = token.trim_matches(|c: char| !c.is_ascii_digit());
        let (done, total) = token.split_once('/')?;
        let (done, total): (u32, u32) = (done.parse().ok()?, total.parse().ok()?);
        (total > 0 && (1..=total).contains(&done)).then(|| (done - 1) as f32 / total as f32)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_percentages_and_steps() {
        assert_eq!(progress_of("downloading firefox  45%"), Some(0.45));
        assert_eq!(progress_of("(3/4) installing vlc"), Some(0.5));
        assert_eq!(progress_of("Installing 1/2… 100%"), Some(1.0));
        assert_eq!(progress_of("resolving dependencies..."), None);
        assert_eq!(progress_of("/usr/share/a/b"), None);
        assert_eq!(progress_of("(0/0)"), None);
        assert_eq!(progress_of("150%"), None);
    }
}
