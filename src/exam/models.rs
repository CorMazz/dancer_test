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

#[macro_export]
/// Used to create the structs that define the leader/follower test items and test templates (for Askama)
macro_rules! define_test_structs {
    ($name:ident) => {
        pub struct $name {
            pub patterns: Vec<&'static str>,
            pub pattern_scoring_categories: Vec<PatternScoringCategory>,
            pub technique_headers: Vec<&'static str>,
            pub techniques: Vec<Technique>,
            pub bonus_items: Vec<BonusItem>,
        }
    };
}

struct ScoringCategory {
    score: usize,
    category: String,
}

struct Pattern {
    pattern_name: String,
    scoring_categories: Vec<ScoringCategory>,
}


// #######################################################################################################################################################
// Leader Test
// #######################################################################################################################################################

define_test_structs!(LeaderTest);

pub fn generate_leader_test() -> LeaderTest {

    // DO NOT USE -- IN ANY OF THESE STRINGS, IT IS RESERVED AS A DELIMITER 
    LeaderTest {
        patterns:  vec![
            "Starter Step",
            "Left Side Pass from Closed",
            "Sugar Tuck",
            "Cutoff Whip",
            "Left Side Pass",
            "Whip",
            "Sugar Push",
            "Spinning Side Pass",
            "Right Side Pass",
            "Basket Whip",
            "Free Spin",
        ],
        pattern_scoring_categories: vec![
            PatternScoringCategory {
                name: "Footwork",
                points: vec![3, 2, 1, 0],
            },
            PatternScoringCategory {
                name: "Timing",
                points: vec![1, 0],
            },
        ],

        technique_headers: vec![
            "Consistent 90%",
            "Present 75%",
            "Occasional 50%",
            "Lacking 25%",
            "Missing 0%",
        ],
        techniques: vec![
            Technique {
                name: "Body Lead",
                subtext: "(Week 1)",
                points: vec![8, 6, 0, 0, 0],
                antithesis: "Arm Lead",
            },
            Technique {
                name: "Post",
                subtext: "(Week 1)",
                points: vec![6, 4, 0, 0, 0],
                antithesis: "Floating Anchor",
            },
            Technique {
                name: "Strong Frame",
                subtext: "(Week 2)",
                points: vec![6, 4, 2, 0, 0],
                antithesis: "Weak Frame",
            },
            Technique {
                name: "Closed Connection",
                subtext: "(Week 3/4)",
                points: vec![4, 3, 2, 0, 0],
                antithesis: "Free Hand Only",
            },
            Technique {
                name: "Connection Transition",
                subtext: "(Week 2 - Dimmer Switch)",
                points: vec![4, 3, 2, 0, 0],
                antithesis: "Brick Wall (Toggle Switch)",
            },
            Technique {
                name: "On Time",
                subtext: "",
                points: vec![8, 6, 0, 0 ,0], 
                antithesis: "Off Time",
            },
            Technique {
                name: "Move Off Slot",
                subtext: "",
                points: vec![4, 3, 0, 0, 0],
                antithesis: "In the Way",
            },
            Technique {
                name: "Safe",
                subtext: "",
                points: vec![8, 0, 0, 0, 0],
                antithesis: "Unsafe",
            },
        ],

        bonus_items: vec![
            BonusItem {
                label: "No Thumbs",
                points: 1,
            },
            BonusItem {
                label: "Clear Turn Signal",
                points: 1,
            },
            BonusItem {
                label: "Swung Triple",
                points: 4,
            },
        ],
    }
}