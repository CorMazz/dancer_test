
/// A struct to define something that will give you bonus points on the test
pub struct BonusItem {
    pub label: &'static str,
    pub points: usize,
}

pub struct Technique {
    pub name: &'static str,
    pub subtext: &'static str,
    pub points: Vec<usize>,
    pub antithesis: &'static str,
}

pub struct PatternScoringCategory {
    pub name: &'static str,
    pub points: Vec<usize>,
}
