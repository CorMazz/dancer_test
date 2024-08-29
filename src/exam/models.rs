use std::{collections::HashMap};
use askama::Template;
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use strum_macros::{Display, EnumString};
use crate::filters;

// #######################################################################################################################################################
// #######################################################################################################################################################
// Declare Structs/Enums Used to Define the Test
// #######################################################################################################################################################
// #######################################################################################################################################################

// Using structs/enums makes the compiler ensure that all future uses are spelled correctly, as opposed to just strings. 
// Need to implement fmt::Display and FromStr for all of these because the html takes the names of the enums and turns them into strings as the id of the
// html elements, but the element IDs can't have spaces. 
// NONE OF THESE STRINGS SHOULD HAVE -- IN THEM, THAT IS RESERVED AS A DELIMITER WITHIN THE HTML

// -------------------------------------------------------------------------------------------------------------------------------------------------------
// Bonus Point Name
// -------------------------------------------------------------------------------------------------------------------------------------------------------

#[derive(Debug, EnumString, Display, Serialize, Deserialize)]
#[strum(ascii_case_insensitive)]
pub enum BonusPointName {

    // Leads
    #[strum(serialize = "clear_turn_signal", to_string = "Clear Turn Signal")]
    ClearTurnSignal,

    #[strum(serialize = "no_thumbs", to_string = "No Thumbs")]
    NoThumbs,

    // Both
    #[strum(serialize = "swung_triple", to_string = "Swung Triple")]
    SwungTriple,

    // Follows
    #[strum(serialize = "no_biceps", to_string = "No Biceps")]
    NoBiceps,

    #[strum(serialize = "spotting", to_string = "Spotting")]
    Spotting,

    #[strum(serialize = "connection_in_motion", to_string = "Connection in Motion")]
    ConnectionInMotion,
}


// -------------------------------------------------------------------------------------------------------------------------------------------------------
// Test Definition Bonus Item
// -------------------------------------------------------------------------------------------------------------------------------------------------------

/// A struct to define something that will give you bonus points on the test
pub struct TestDefinitionBonusItem {
    pub label: BonusPointName,
    pub points: u32,
}

// -------------------------------------------------------------------------------------------------------------------------------------------------------
// Technique Name
// -------------------------------------------------------------------------------------------------------------------------------------------------------

/// Old pattern names, technique names, and other enum variants CANNOT be retired so long 
/// as they may exist within the database. 
#[derive(Debug, EnumString, Display, Serialize, Deserialize)]
pub enum TechniqueName {
    // Leads
    #[strum(serialize = "body_lead", to_string = "Body Lead")]
    BodyLead,

    #[strum(serialize = "closed_connection", to_string = "Closed Connection")]
    ClosedConnection,

    #[strum(serialize = "on_time", to_string = "On Time")]
    OnTime,

    #[strum(serialize = "move_off_slot", to_string = "Move Off Slot")]
    MoveOffSlot,

    #[strum(serialize = "safe", to_string = "Safe")]
    Safe,

    #[strum(serialize = "prep", to_string = "Prep")]
    Prep,

    // Both

    #[strum(serialize = "post", to_string = "Post")]
    Post,

    #[strum(serialize = "strong_frame", to_string = "Strong Frame")]
    StrongFrame,

    #[strum(serialize = "connection_transition", to_string = "Connection Transition")]
    ConnectionTransition,

    #[strum(serialize = "body_angle", to_string = "Body Angle")]
    BodyAngle,

    // Follows

    #[strum(serialize = "anchor_in_3rd_position", to_string = "Anchor in 3rd Position")]
    AnchorIn3rdPosition,

    #[strum(serialize = "stretch", to_string = "Stretch")]
    Stretch,

    #[strum(serialize = "variable_speed", to_string = "Variable Speed")]
    VariableSpeed,

    #[strum(serialize = "connection_hierarchy", to_string = "Connection Hierarchy")]
    ConnectionHierarchy,

    #[strum(serialize = "directionality", to_string = "Directionality")]
    Directionality,

    #[strum(serialize = "kissing_connection", to_string = "Kissing Connection")]
    KissingConnection,

    #[strum(serialize = "spins", to_string = "Spins")]
    Spins,

    #[strum(serialize = "personal_safety", to_string = "Personal Safety")]
    PersonalSafety,

    #[strum(serialize = "partner_safety", to_string = "Partner Safety")]
    PartnerSafety,
}

// -------------------------------------------------------------------------------------------------------------------------------------------------------
// Technique Scoring Header Name
// -------------------------------------------------------------------------------------------------------------------------------------------------------

/// This one doesn't get converted to snake case and isn't used as an html value so can have ><
#[derive(Debug, EnumString, Display, Serialize, Deserialize)]
pub enum TechniqueScoringHeaderName {
    // For most techniques
    #[strum(serialize = "Consistent >90%")]
    Consistent90,

    #[strum(serialize = "Present 75%")]
    Present75,

    #[strum(serialize = "Occasional 50%")]
    Occasional50,

    #[strum(serialize = "Lacking 25%")]
    Lacking25,

    #[strum(serialize = "Missing <10%")]
    Missing10,

    // For level of angle technique

    #[strum(serialize = "Perpendicular")]
    Perpendicular,

    #[strum(serialize = "Over-Angled")]
    OverAngled,

    #[strum(serialize = "Angled")]
    Angled,

    #[strum(serialize = "Under-Angled")]
    UnderAngled,

    #[strum(serialize = "Flat")]
    Flat,

    // For level of prep technique

    #[strum(serialize = "Overkill Prep")]
    OverkillPrep,

    #[strum(serialize = "Over-Prepped")]
    Overprepped,

    #[strum(serialize = "Prepped")]
    Prepped,

    #[strum(serialize = "Under-Prepped")]
    UnderPrepped,

    #[strum(serialize = "No Prep")]
    NoPrep,

}
// -------------------------------------------------------------------------------------------------------------------------------------------------------
// Test Definition Technique
// -------------------------------------------------------------------------------------------------------------------------------------------------------

pub struct TestDefinitionTechnique {
    pub name: TechniqueName,
    pub subtext: &'static str,
    pub points: Vec<u32>,
    pub antithesis: &'static str,
    pub max_score: u32,
}
impl TestDefinitionTechnique {
    pub fn new(name: TechniqueName, subtext: &'static str, points: Vec<u32>, antithesis: &'static str) -> Self {
        let max_score = points.iter().copied().max().unwrap_or(0);
        Self {
            name,
            subtext,
            points,
            antithesis,
            max_score,
        }
    }
}

// -------------------------------------------------------------------------------------------------------------------------------------------------------
// Test Definition Technique Group
// -------------------------------------------------------------------------------------------------------------------------------------------------------

// Both vecs should have the same length 
pub struct TestDefinitionTechniqueGroup {
    pub technique_headers: Vec<TechniqueScoringHeaderName>,
    pub techniques: Vec<TestDefinitionTechnique>,
}

// -------------------------------------------------------------------------------------------------------------------------------------------------------
// Scoring Category Name
// -------------------------------------------------------------------------------------------------------------------------------------------------------

/// Old pattern names, technique names, and other enum variants CANNOT be retired so long 
/// as they may exist within the database. 
#[derive(Debug, EnumString, Display, Serialize, Deserialize)]
pub enum ScoringCategoryName {

    // Both

    #[strum(serialize = "footwork", to_string = "Footwork")]
    Footwork,

    #[strum(serialize = "timing", to_string = "Timing")]
    Timing,

    // Follows

    #[strum(serialize = "shaping", to_string = "Shaping")]
    Shaping
}

// -------------------------------------------------------------------------------------------------------------------------------------------------------
// Pattern Scoring Category
// -------------------------------------------------------------------------------------------------------------------------------------------------------

pub struct TestDefinitionPatternScoringCategory {
    pub name: ScoringCategoryName,
    pub headers: Vec<&'static str>,
    pub points: Vec<u32>,
    pub max_score: u32
}
impl TestDefinitionPatternScoringCategory {
    pub fn new(name: ScoringCategoryName, headers: Vec<&'static str>, points: Vec<u32>) -> Self {
        let max_score = points.iter().copied().max().unwrap_or(0);

        Self {
            name,
            headers,
            points,
            max_score,
        }
    }
}

// -------------------------------------------------------------------------------------------------------------------------------------------------------
// Pattern Name
// -------------------------------------------------------------------------------------------------------------------------------------------------------

/// Old pattern names, technique names, and other enum variants CANNOT be retired so long 
/// as they may exist within the database. 
#[derive(Debug, EnumString, Display, Serialize, Deserialize)]
pub enum PatternName {

    // Leads

    #[strum(serialize = "starter_step", to_string = "Starter Step")]
    StarterStep,

    #[strum(serialize = "left_side_pass_from_closed", to_string = "Left Side Pass from Closed")]
    LeftSidePassFromClosed,

    #[strum(serialize = "sugar_tuck", to_string = "Sugar Tuck")]
    SugarTuck,

    #[strum(serialize = "cutoff_whip", to_string = "Cutoff Whip")]
    CutoffWhip,

    #[strum(serialize = "basket_whip", to_string = "Basket Whip")]
    BasketWhip,

    #[strum(serialize = "free_spin", to_string = "Free Spin")]
    FreeSpin,

    // Both

    #[strum(serialize = "left_side_pass", to_string = "Left Side Pass")]
    LeftSidePass,

    #[strum(serialize = "right_side_pass", to_string = "Right Side Pass")]
    RightSidePass,
    
    #[strum(serialize = "whip", to_string = "Whip")]
    Whip,

    #[strum(serialize = "spinning_side_pass", to_string = "Spinning Side Pass")]
    SpinningSidePass,

    #[strum(serialize = "sugar_push", to_string = "Sugar Push")]
    SugarPush,

}

// -------------------------------------------------------------------------------------------------------------------------------------------------------
// Test Template
// -------------------------------------------------------------------------------------------------------------------------------------------------------

#[derive(Template)]
#[template(path = "./primary_templates/dancer_test.html")] 
pub struct TestTemplate {
    pub test_type: TestType,
    pub passing_score: u32,
    pub patterns: Vec<PatternName>,
    pub pattern_scoring_categories: Vec<TestDefinitionPatternScoringCategory>,
    pub technique_groups: Vec<TestDefinitionTechniqueGroup>,
    pub bonus_items: Vec<TestDefinitionBonusItem>,
    pub is_demo_mode: bool,
}

// #######################################################################################################################################################
// #######################################################################################################################################################
// Declare Structs/Enums Used to Record Test Results
// #######################################################################################################################################################
// #######################################################################################################################################################
// Using structs/enums makes the compiler ensure that all future uses are spelled correctly, as opposed to just strings. 

#[derive(Debug, sqlx::FromRow, Serialize, Deserialize)]
pub struct GradedPattern {
    // pub id: Option<i32>,  // Optional ID field
    // pub test_id: Option<i32>,  // Optional test ID field
    pub pattern: PatternName,
    pub category: ScoringCategoryName,
    pub score: u32,
    pub max_score: u32,
}

#[derive(Debug, sqlx::FromRow, Serialize, Deserialize)]
pub struct GradedTechnique {
    // pub id: Option<i32>,  // Optional ID field
    // pub test_id: Option<i32>,  // Optional test ID field
    pub technique: TechniqueName,
    pub score: u32,
    pub score_header: TechniqueScoringHeaderName,
    pub max_score: u32,
}

#[derive(Debug, sqlx::FromRow, Serialize, Deserialize)]
pub struct GradedBonusPoint {
    // pub id: Option<i32>,  // Optional ID field
    // pub test_id: Option<i32>,  // Optional test ID field
    pub name: BonusPointName,
    pub score: u32,
}

#[derive(Debug, sqlx::FromRow, Serialize, Deserialize)]
pub struct Testee {
    pub id: i32,  
    pub first_name: String,
    pub last_name: String,
    pub email: String,
}

impl From<HashMap<String, String>> for Testee {
    fn from(mut map: HashMap<String, String>) -> Self {
        let first_name = map.remove("first_name").unwrap_or_default();
        let last_name = map.remove("last_name").unwrap_or_default();
        let email = map.remove("email").unwrap_or_default();

        Testee {
            id: -1,  // ID is None when parsed from form data
            first_name,
            last_name,
            email,
        }
    }
}

// Didn't use SQLX custom types because they required hoops to jump through for compile time type checking to work
#[derive(Debug, EnumString, Display, Serialize, Deserialize)]
#[strum(serialize_all = "snake_case")]
pub enum TestType {
    Leader,
    Follower,
}


#[derive(Debug, Serialize, Deserialize)]
pub struct GradedTest {
    pub testee: Testee,
    pub test_date: NaiveDateTime,
    pub test_type: TestType,
    pub score: u32,
    pub max_score: u32,
    pub passing_score: u32,
    pub patterns: Vec<GradedPattern>,
    pub techniques: Vec<GradedTechnique>,
    pub bonuses: Vec<GradedBonusPoint>,
}

impl GradedTest {
    pub fn new(
        testee: Testee,
        test_date: NaiveDateTime,
        test_type: TestType,
        passing_score: u32,
        patterns: Vec<GradedPattern>,
        techniques: Vec<GradedTechnique>,
        bonuses: Vec<GradedBonusPoint>,
    ) -> Self {
        let mut score = 0;
        let mut max_score = 0;

        for pattern in &patterns {
            score += pattern.score;
            max_score += pattern.max_score;
        }

        for technique in &techniques {
            score += technique.score;
            max_score += technique.max_score;
        }

        for bonus in &bonuses {
            score += bonus.score;
        }

        Self {
            testee,
            test_date,
            test_type,
            score,
            max_score,
            passing_score,
            patterns,
            techniques,
            bonuses,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TestSummary {
    pub id: i32,
    pub test_date: NaiveDateTime,
    pub role: String,  // This probably should've been labeled test_type, but I'm lazy here...
    pub score: i32,
    pub max_score: i32,
    pub passing_score: i32,
}

// #######################################################################################################################################################
// #######################################################################################################################################################
// Functions to Generate the Tests and Parse Test Results
// #######################################################################################################################################################
// #######################################################################################################################################################

// -------------------------------------------------------------------------------------------------------------------------------------------------------
// Leader Test
// -------------------------------------------------------------------------------------------------------------------------------------------------------

pub fn generate_leader_test(is_demo_mode: bool) -> TestTemplate {
    TestTemplate {
        is_demo_mode: is_demo_mode,
        test_type: TestType::Leader,
        passing_score: 60,
        patterns:  vec![
            PatternName::StarterStep,
            PatternName::LeftSidePassFromClosed,
            PatternName::SugarTuck,
            PatternName::CutoffWhip,
            PatternName::LeftSidePass,
            PatternName::Whip,
            PatternName::SugarPush,
            PatternName::SpinningSidePass,
            PatternName::RightSidePass,
            PatternName::BasketWhip,
            PatternName::FreeSpin,
        ],
        pattern_scoring_categories: vec![
            TestDefinitionPatternScoringCategory::new(
                ScoringCategoryName::Footwork,
                vec!["Perfect", "Variation?", "Right Concept", "Nope"],
                vec![3, 2, 1, 0],
            ),
            TestDefinitionPatternScoringCategory::new(
                ScoringCategoryName::Timing,
                vec!["On", "Off"],
                vec![1, 0],
            ),
        ],

        technique_groups: vec![
            TestDefinitionTechniqueGroup {
                technique_headers: vec![
                    TechniqueScoringHeaderName::Consistent90,
                    TechniqueScoringHeaderName::Present75,
                    TechniqueScoringHeaderName::Occasional50,
                    TechniqueScoringHeaderName::Lacking25,
                    TechniqueScoringHeaderName::Missing10,
                ],
                techniques: vec![
                    TestDefinitionTechnique::new(
                        TechniqueName::BodyLead,
                        "(Week 1)",
                        vec![8, 6, 0, 0, 0],
                        "Arm Lead",
                    ),
                    TestDefinitionTechnique::new(
                        TechniqueName::Post,
                        "(Week 1)",
                        vec![6, 4, 0, 0, 0],
                        "Floating Anchor",
                    ),
                    TestDefinitionTechnique::new(
                        TechniqueName::StrongFrame,
                        "(Week 2)",
                        vec![6, 4, 2, 0, 0],
                        "Weak Frame",
                    ),
                    TestDefinitionTechnique::new(
                        TechniqueName::ClosedConnection,
                        "(Week 3/4)",
                        vec![4, 3, 2, 0, 0],
                        "Free Hand Only",
                    ),
                    TestDefinitionTechnique::new(
                        TechniqueName::ConnectionTransition,
                        "(Week 2 - Dimmer Switch)",
                        vec![4, 3, 2, 0, 0],
                        "Brick Wall (Toggle Switch)",
                    ),
                    TestDefinitionTechnique::new(
                        TechniqueName::OnTime,
                        "",
                        vec![8, 6, 0, 0 ,0], 
                        "Off Time",
                    ),
                    TestDefinitionTechnique::new(
                        TechniqueName::MoveOffSlot,
                        "",
                        vec![4, 3, 0, 0, 0],
                        "In the Way",
                    ),
                    TestDefinitionTechnique::new(
                        TechniqueName::Safe,
                        "",
                        vec![8, 0, 0, 0, 0],
                        "Unsafe",
                    ),
                ],
            },
            TestDefinitionTechniqueGroup {
                technique_headers: vec![
                    TechniqueScoringHeaderName::Perpendicular,
                    TechniqueScoringHeaderName::OverAngled,
                    TechniqueScoringHeaderName::Angled,
                    TechniqueScoringHeaderName::UnderAngled,
                    TechniqueScoringHeaderName::Flat,
                ],
                techniques: vec![
                    TestDefinitionTechnique::new(
                        TechniqueName::BodyAngle,
                        "(Week 2)",
                        vec![0, 2, 3, 2, 0],
                        "",
                    ),
                ],
            },
            TestDefinitionTechniqueGroup {
                technique_headers: vec![
                    TechniqueScoringHeaderName::OverkillPrep,
                    TechniqueScoringHeaderName::Overprepped,
                    TechniqueScoringHeaderName::Prepped,
                    TechniqueScoringHeaderName::UnderPrepped,
                    TechniqueScoringHeaderName::NoPrep,
                ],
                techniques: vec![
                    TestDefinitionTechnique::new(
                        TechniqueName::Prep,
                        "(Week 3)",
                        vec![0, 2, 3, 0, 0],
                        "",
                    ),
                ],
            },
        ],
        bonus_items: vec![
            TestDefinitionBonusItem {
                label: BonusPointName::NoThumbs,
                points: 1,
            },
            TestDefinitionBonusItem {
                label: BonusPointName::ClearTurnSignal,
                points: 1,
            },
            TestDefinitionBonusItem {
                label: BonusPointName::SwungTriple,
                points: 4,
            },
        ],
    }
}

// -------------------------------------------------------------------------------------------------------------------------------------------------------
// Leader Test
// -------------------------------------------------------------------------------------------------------------------------------------------------------

pub fn generate_follower_test(is_demo_mode: bool) -> TestTemplate {
    TestTemplate {
        is_demo_mode: is_demo_mode,
        test_type: TestType::Follower,
        passing_score: 60,
        patterns:  vec![
            PatternName::LeftSidePass,
            PatternName::RightSidePass,
            PatternName::Whip,
            PatternName::SpinningSidePass,
            PatternName::SugarPush,
        ],
        pattern_scoring_categories: vec![
            TestDefinitionPatternScoringCategory::new(
                ScoringCategoryName::Footwork,
                vec!["Perfect", "Right Concept", "Nope"],
                vec![2, 1, 0],
            ),
            TestDefinitionPatternScoringCategory::new(
                ScoringCategoryName::Timing,
                vec!["On", "Off"],
                vec![1, 0],
            ),
            TestDefinitionPatternScoringCategory::new(
                ScoringCategoryName::Shaping,
                vec!["Shaped", "Flat"],
                vec![1, 0],
            ),
        ],
        technique_groups: vec![
            TestDefinitionTechniqueGroup {
                technique_headers: vec![
                    TechniqueScoringHeaderName::Consistent90,
                    TechniqueScoringHeaderName::Present75,
                    TechniqueScoringHeaderName::Occasional50,
                    TechniqueScoringHeaderName::Lacking25,
                    TechniqueScoringHeaderName::Missing10,
                ],
                techniques: vec![
                    TestDefinitionTechnique::new(
                        TechniqueName::StrongFrame,
                        "(Week 2)",
                        vec![8, 5, 0, 0, 0],
                        "Weak Frame",
                    ),
                    TestDefinitionTechnique::new(
                        TechniqueName::Post,
                        "(Week 2)",
                        vec![6, 4, 2, 0, 0],
                        "Floating Anchor",
                    ),
                    TestDefinitionTechnique::new(
                        TechniqueName::AnchorIn3rdPosition,
                        "(Week 2)",
                        vec![8, 0, 0, 0, 0],
                        "Rock Step",
                    ),
                    TestDefinitionTechnique::new(
                        TechniqueName::Stretch,
                        "Week 1/2)",
                        vec![6, 4, 2, 0, 0],
                        "Over Eager",
                    ),
                    TestDefinitionTechnique::new(
                        TechniqueName::VariableSpeed,
                        "(Week 1)",
                        vec![4, 3, 2, 1, 0],
                        "Monotonous",
                    ),
                    TestDefinitionTechnique::new(
                        TechniqueName::ConnectionTransition,
                        "(Week 2 - Dimmer Switch)",
                        vec![8, 5, 0, 0, 0],
                        "Brick Wall (Toggle Switch)",
                    ),
                    TestDefinitionTechnique::new(
                        TechniqueName::ConnectionHierarchy,
                        "(Week 4)",
                        vec![6, 4, 2, 0, 0],
                        "Panic",
                    ),
                    TestDefinitionTechnique::new(
                        TechniqueName::Directionality,
                        "(Week 1/3)",
                        vec![6, 4, 0, 0, 0],
                        "Directionless",
                    ),
                    TestDefinitionTechnique::new(
                        TechniqueName::KissingConnection,
                        "(Week 3)",
                        vec![6, 4, 2, 0, 0],
                        "Disconnected",
                    ),
                    TestDefinitionTechnique::new(
                        TechniqueName::Spins,
                        "(Week 3)",
                        vec![4, 3, 0, 0, 0],
                        "Unbalanced",
                    ),
                    TestDefinitionTechnique::new(
                        TechniqueName::PersonalSafety,
                        "",
                        vec![8, 5, 0, 0, 0],
                        "Masochist",
                    ),
                    TestDefinitionTechnique::new(
                        TechniqueName::PartnerSafety,
                        "",
                        vec![8, 0, 0, 0, 0],
                        "Abusive",
                    ),
                ],
            },
            TestDefinitionTechniqueGroup {
                technique_headers: vec![
                    TechniqueScoringHeaderName::Perpendicular,
                    TechniqueScoringHeaderName::OverAngled,
                    TechniqueScoringHeaderName::Angled,
                    TechniqueScoringHeaderName::UnderAngled,
                    TechniqueScoringHeaderName::Flat,
                ],
                techniques: vec![
                    TestDefinitionTechnique::new(
                        TechniqueName::BodyAngle,
                        "(Week 2)",
                        vec![0, 2, 3, 2, 0],
                        "",
                    ),
                ],
            },
        ],
        bonus_items: vec![
            TestDefinitionBonusItem {
                label: BonusPointName::NoBiceps,
                points: 1,
            },
            TestDefinitionBonusItem {
                label: BonusPointName::Spotting,
                points: 1,
            },
            TestDefinitionBonusItem {
                label: BonusPointName::ConnectionInMotion,
                points: 2,
            },
            TestDefinitionBonusItem {
                label: BonusPointName::SwungTriple,
                points: 4,
            },
        ],
    }
}


