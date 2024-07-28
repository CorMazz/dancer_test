use std::{collections::HashMap, fmt, str::FromStr};
use askama::Template;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
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

#[derive(Debug, EnumString, Display)]
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

#[derive(Debug, EnumString, Display)]
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

    // Both

    #[strum(serialize = "post", to_string = "Post")]
    Post,

    #[strum(serialize = "strong_frame", to_string = "Strong Frame")]
    StrongFrame,

    #[strum(serialize = "connection_transition", to_string = "Connection Transition")]
    ConnectionTransition,

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
#[derive(Debug, EnumString, Display)]
pub enum TechniqueScoringHeaderName {
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
}
// -------------------------------------------------------------------------------------------------------------------------------------------------------
// Test Definition Technique
// -------------------------------------------------------------------------------------------------------------------------------------------------------

pub struct TestDefinitionTechnique {
    pub name: TechniqueName,
    pub subtext: &'static str,
    pub points: Vec<u32>,
    pub antithesis: &'static str,
}

// -------------------------------------------------------------------------------------------------------------------------------------------------------
// Scoring Category Name
// -------------------------------------------------------------------------------------------------------------------------------------------------------

#[derive(Debug, EnumString, Display)]
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
    pub points: Vec<u32>,
}

// -------------------------------------------------------------------------------------------------------------------------------------------------------
// Pattern Name
// -------------------------------------------------------------------------------------------------------------------------------------------------------

#[derive(Debug, EnumString, Display)]
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
    pub patterns: Vec<PatternName>,
    pub pattern_scoring_categories: Vec<TestDefinitionPatternScoringCategory>,
    pub technique_headers: Vec<TechniqueScoringHeaderName>,
    pub techniques: Vec<TestDefinitionTechnique>,
    pub bonus_items: Vec<TestDefinitionBonusItem>,
}

// #######################################################################################################################################################
// #######################################################################################################################################################
// Declare Structs/Enums Used to Record Test Results
// #######################################################################################################################################################
// #######################################################################################################################################################
// Using structs/enums makes the compiler ensure that all future uses are spelled correctly, as opposed to just strings. 

#[derive(Debug, sqlx::FromRow)]
pub struct GradedPattern {
    pub id: Option<i32>,  // Optional ID field
    pub test_id: Option<i32>,  // Optional test ID field
    pub pattern: PatternName,
    pub category: ScoringCategoryName,
    pub score: u32,
}

#[derive(Debug, sqlx::FromRow)]
pub struct GradedTechnique {
    pub id: Option<i32>,  // Optional ID field
    pub test_id: Option<i32>,  // Optional test ID field
    pub technique: TechniqueName,
    pub score: u32,
    pub score_header: TechniqueScoringHeaderName,
}

#[derive(Debug, sqlx::FromRow)]
pub struct GradedBonusPoint {
    pub id: Option<i32>,  // Optional ID field
    pub test_id: Option<i32>,  // Optional test ID field
    pub name: BonusPointName,
    pub score: u32,
}

#[derive(Debug, sqlx::FromRow)]
pub struct Testee {
    pub id: Option<i32>,  // Optional ID field
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
            id: None,  // ID is None when parsed from form data
            first_name,
            last_name,
            email,
        }
    }
}

// Didn't use SQLX custom types because they required hoops to jump through for compile time type checking to work
#[derive(Debug, EnumString, Display)]
#[strum(serialize_all = "snake_case")]
pub enum TestType {
    Leader,
    Follower,
}


// #######################################################################################################################################################
// #######################################################################################################################################################
// Functions to Generate the Tests and Parse Test Results
// #######################################################################################################################################################
// #######################################################################################################################################################

// -------------------------------------------------------------------------------------------------------------------------------------------------------
// Leader Test
// -------------------------------------------------------------------------------------------------------------------------------------------------------

pub fn generate_leader_test() -> TestTemplate {
    TestTemplate {
        test_type: TestType::Leader,
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
            TestDefinitionPatternScoringCategory {
                name: ScoringCategoryName::Footwork,
                points: vec![3, 2, 1, 0],
            },
            TestDefinitionPatternScoringCategory {
                name: ScoringCategoryName::Timing,
                points: vec![1, 0],
            },
        ],

        technique_headers: vec![
            TechniqueScoringHeaderName::Consistent90,
            TechniqueScoringHeaderName::Present75,
            TechniqueScoringHeaderName::Occasional50,
            TechniqueScoringHeaderName::Lacking25,
            TechniqueScoringHeaderName::Missing10,
        ],
        techniques: vec![
            TestDefinitionTechnique {
                name: TechniqueName::BodyLead,
                subtext: "(Week 1)",
                points: vec![8, 6, 0, 0, 0],
                antithesis: "Arm Lead",
            },
            TestDefinitionTechnique {
                name: TechniqueName::Post,
                subtext: "(Week 1)",
                points: vec![6, 4, 0, 0, 0],
                antithesis: "Floating Anchor",
            },
            TestDefinitionTechnique {
                name: TechniqueName::StrongFrame,
                subtext: "(Week 2)",
                points: vec![6, 4, 2, 0, 0],
                antithesis: "Weak Frame",
            },
            TestDefinitionTechnique {
                name: TechniqueName::ClosedConnection,
                subtext: "(Week 3/4)",
                points: vec![4, 3, 2, 0, 0],
                antithesis: "Free Hand Only",
            },
            TestDefinitionTechnique {
                name: TechniqueName::ConnectionTransition,
                subtext: "(Week 2 - Dimmer Switch)",
                points: vec![4, 3, 2, 0, 0],
                antithesis: "Brick Wall (Toggle Switch)",
            },
            TestDefinitionTechnique {
                name: TechniqueName::OnTime,
                subtext: "",
                points: vec![8, 6, 0, 0 ,0], 
                antithesis: "Off Time",
            },
            TestDefinitionTechnique {
                name: TechniqueName::MoveOffSlot,
                subtext: "",
                points: vec![4, 3, 0, 0, 0],
                antithesis: "In the Way",
            },
            TestDefinitionTechnique {
                name: TechniqueName::Safe,
                subtext: "",
                points: vec![8, 0, 0, 0, 0],
                antithesis: "Unsafe",
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

pub fn generate_follower_test() -> TestTemplate {
    TestTemplate {
        test_type: TestType::Follower,
        patterns:  vec![
            PatternName::LeftSidePass,
            PatternName::RightSidePass,
            PatternName::Whip,
            PatternName::SpinningSidePass,
            PatternName::SugarPush,
        ],
        pattern_scoring_categories: vec![
            TestDefinitionPatternScoringCategory {
                name: ScoringCategoryName::Footwork,
                points: vec![2, 1, 0],
            },
            TestDefinitionPatternScoringCategory {
                name: ScoringCategoryName::Timing,
                points: vec![1, 0],
            },
            TestDefinitionPatternScoringCategory {
                name: ScoringCategoryName::Shaping,
                points: vec![1, 0],
            },
        ],

        technique_headers: vec![
            TechniqueScoringHeaderName::Consistent90,
            TechniqueScoringHeaderName::Present75,
            TechniqueScoringHeaderName::Occasional50,
            TechniqueScoringHeaderName::Lacking25,
            TechniqueScoringHeaderName::Missing10,
        ],
        techniques: vec![
            TestDefinitionTechnique {
                name: TechniqueName::StrongFrame,
                subtext: "(Week 2)",
                points: vec![8, 5, 0, 0, 0],
                antithesis: "Weak Frame",
            },
            TestDefinitionTechnique {
                name: TechniqueName::Post,
                subtext: "(Week 2)",
                points: vec![6, 4, 2, 0, 0],
                antithesis: "Floating Anchor",
            },
            TestDefinitionTechnique {
                name: TechniqueName::AnchorIn3rdPosition,
                subtext: "(Week 2)",
                points: vec![8, 0, 0, 0, 0],
                antithesis: "Rock Step",
            },
            TestDefinitionTechnique {
                name: TechniqueName::Stretch,
                subtext: "(Week 1/2)",
                points: vec![6, 4, 2, 0, 0],
                antithesis: "Over Eager",
            },
            TestDefinitionTechnique {
                name: TechniqueName::VariableSpeed,
                subtext: "(Week 1)",
                points: vec![4, 3, 2, 1, 0],
                antithesis: "Monotonous",
            },
            TestDefinitionTechnique {
                name: TechniqueName::ConnectionTransition,
                subtext: "(Week 2 - Dimmer Switch)",
                points: vec![8, 5, 0, 0, 0],
                antithesis: "Brick Wall (Toggle Switch)",
            },
            TestDefinitionTechnique {
                name: TechniqueName::ConnectionHierarchy,
                subtext: "(Week 4)",
                points: vec![6, 4, 2, 0, 0],
                antithesis: "Panic",
            },
            TestDefinitionTechnique {
                name: TechniqueName::Directionality,
                subtext: "(Week 1/3)",
                points: vec![6, 4, 0, 0, 0],
                antithesis: "Directionless",
            },
            TestDefinitionTechnique {
                name: TechniqueName::KissingConnection,
                subtext: "(Week 3)",
                points: vec![6, 4, 2, 0, 0],
                antithesis: "Disconnected",
            },
            TestDefinitionTechnique {
                name: TechniqueName::Spins,
                subtext: "(Week 3)",
                points: vec![4, 3, 0, 0, 0],
                antithesis: "Unbalanced",
            },
            TestDefinitionTechnique {
                name: TechniqueName::PersonalSafety,
                subtext: "",
                points: vec![8, 5, 0, 0, 0],
                antithesis: "Masochist",
            },
            TestDefinitionTechnique {
                name: TechniqueName::PartnerSafety,
                subtext: "",
                points: vec![8, 0, 0, 0, 0],
                antithesis: "Abusive",
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


// -------------------------------------------------------------------------------------------------------------------------------------------------------
// Parse Test
// -------------------------------------------------------------------------------------------------------------------------------------------------------

/// Parses the test form data, which should have a format of a hashmap more or less like this
///
/// first_name: Corrado
/// last_name: Mazzarelli
/// email: corrado@mazzarelli.biz
/// pattern--starter_step--scoring_category--footwork: 3
/// pattern--starter_step--scoring_category--timing: 1
/// pattern--left_side_pass_from_closed--scoring_category--footwork: 2
/// pattern--left_side_pass_from_closed--scoring_category--timing: 1
/// pattern--sugar_tuck--scoring_category--footwork: 1
/// pattern--sugar_tuck--scoring_category--timing: 1
/// pattern--cutoff_whip--scoring_category--footwork: 0
/// pattern--cutoff_whip--scoring_category--timing: 1
/// pattern--left_side_pass--scoring_category--footwork: 1
/// pattern--left_side_pass--scoring_category--timing: 0
/// pattern--whip--scoring_category--footwork: 2
/// pattern--whip--scoring_category--timing: 0
/// pattern--sugar_push--scoring_category--footwork: 3
/// pattern--sugar_push--scoring_category--timing: 0
/// pattern--spinning_side_pass--scoring_category--footwork: 2
/// pattern--spinning_side_pass--scoring_category--timing: 0
/// pattern--right_side_pass--scoring_category--footwork: 1
/// pattern--right_side_pass--scoring_category--timing: 1
/// pattern--basket_whip--scoring_category--footwork: 0
/// pattern--basket_whip--scoring_category--timing: 1
/// pattern--free_spin--scoring_category--footwork: 1
/// pattern--free_spin--scoring_category--timing: 1
/// technique--body_lead: Consistent 90%--8
/// technique--post: Present 75%--4
/// technique--strong_frame: Occasional 50%--2
/// technique--closed_connection: Lacking 25%--0
/// technique--connection_transition: Missing 0%--0
/// technique--on_time: Lacking 25%--0
/// technique--move_off_slot: Occasional 50%--0
/// technique--safe: Present 75%--0
/// bonus--no_thumbs: 1
/// bonus--clear_turn_signal: 1
/// bonus--swung_triple: 4
/// 
pub fn parse_test_form_data(test: HashMap<String, String>) -> (
    Vec<GradedPattern>,
    Vec<GradedTechnique>,
    Vec<GradedBonusPoint>,
    Testee,
) {
    let mut pattern_scores = Vec::new();
    let mut technique_scores = Vec::new();
    let mut bonus_scores = Vec::new();
    let mut user_info = HashMap::new();

    for (key, value) in test.iter() {
        if key.starts_with("pattern--") {
            // Extract pattern score
            let parts: Vec<&str> = key.split("--").collect();

            match parts.len() {
                4 => {
                    match (parts[1].parse::<PatternName>(), parts[3].parse::<ScoringCategoryName>()) {
                        (Ok(pattern), Ok(category)) => {
                            match value.parse::<u32>() {
                                Ok(score) => pattern_scores.push(GradedPattern {
                                    id: None,
                                    test_id: None,
                                    pattern,
                                    category,
                                    score,
                                }),
                                Err(_) => eprintln!("Failed to parse score from value '{}'", value),
                            }
                        }
                        (Err(e), _) => eprintln!("Failed to parse pattern name from key '{}': {:?}", key, e),
                        (_, Err(e)) => eprintln!("Failed to parse category name from key '{}': {:?}", key, e),
                    }
                }
                _ => eprintln!("The key '{}' should be formatted as follows pattern--pattern_name--scoring_category--scoring_category_name", key),
            }
        } else if key.starts_with("technique--") {
            // Extract technique score
            let key_parts: Vec<&str> = key.split("--").collect();
            let value_parts: Vec<&str> = value.split("--").collect();

            match (key_parts.len(), value_parts.len()) {
                (2, 2) => {
                    match (key_parts[1].parse::<TechniqueName>(), value_parts[0].parse::<TechniqueScoringHeaderName>(), value_parts[1].parse::<u32>()) {
                        (Ok(technique), Ok(score_header), Ok(score)) => {
                            technique_scores.push(GradedTechnique {
                                id: None,
                                test_id: None,
                                technique,
                                score_header,
                                score,
                            });
                        }
                        (Err(e), _, _) => eprintln!("Failed to parse technique name from key '{}': {:?}", key, e),
                        (_, Err(e), _) => eprintln!("Failed to parse scoring header from value '{}': {:?}", value, e),
                        (_, _, Err(e)) => eprintln!("Failed to parse score from value '{}': {:?}", value, e),
                    }
                }
                _ => eprintln!("The key '{}' and value '{}' should be formatted as follows technique--technique_name: technique_scoring_header--score", key, value),
            }
        } else if key.starts_with("bonus--") {
            // Extract bonus score
            let parts: Vec<&str> = key.split("--").collect();

            match parts.len() {
                2 => {
                    match (parts[1].parse::<BonusPointName>(), value.parse::<u32>()) {
                        (Ok(name), Ok(score)) => {
                            bonus_scores.push(GradedBonusPoint {
                                id: None,
                                test_id: None,
                                name,
                                score,
                            });
                        }
                        (Err(e), _) => eprintln!("Failed to parse bonus point name from key '{}': {:?}", key, e),
                        (_, Err(e)) => eprintln!("Failed to parse bonus score from value '{}': {}", value, e),
                    }
                }
                _ => eprintln!("The key '{}' should be formatted as follows bonus--bonus_point_name: score", key),
            }
        } else {
            // Assume user info
            user_info.insert(key.clone(), value.clone());
        }
    }

    // Construct the GradedTestee instance from the user_info hashmap
    let testee: Testee = match (
        user_info.get("first_name").cloned(),
        user_info.get("last_name").cloned(),
        user_info.get("email").cloned()
    ) {
        (Some(first_name), Some(last_name), Some(email)) => Testee {
            id: None,
            first_name,
            last_name,
            email,
        },
        // TODO: This should probably be refactored to propagate an error
        _ => {
            eprintln!("Missing user information. Please ensure 'first_name', 'last_name', and 'email' are provided.");
            Testee {
                id: None,
                first_name: String::new(),
                last_name: String::new(),
                email: String::new(),
            }
        }
    };

    (pattern_scores, technique_scores, bonus_scores, testee)
}

// -------------------------------------------------------------------------------------------------------------------------------------------------------
// Save Test to Database
// -------------------------------------------------------------------------------------------------------------------------------------------------------


pub async fn save_test_to_database(
    pool: &PgPool,
    testee: Testee,
    test_type: TestType,
    pattern_scores: Vec<GradedPattern>,
    technique_scores: Vec<GradedTechnique>,
    bonus_scores: Vec<GradedBonusPoint>,
) -> Result<(), sqlx::Error> {
    // Insert the testee or get the testee ID if the testee already exists
    let testee_id = match sqlx::query!(
        "INSERT INTO testees (first_name, last_name, email)
        VALUES ($1, $2, $3)
        ON CONFLICT (email) DO UPDATE SET first_name = $1, last_name = $2
        RETURNING id",
        testee.first_name,
        testee.last_name,
        testee.email
    )
    .fetch_one(pool)
    .await {
        Ok(record) => record.id,
        Err(e) => return Err(e),
    };


    // Insert a new test record
    let test_id = match sqlx::query!(
        "INSERT INTO tests (testee_id, role)
        VALUES ($1, $2)
        RETURNING id",
        testee_id,
        test_type.to_string(),
    )
    .fetch_one(pool)
    .await {
        Ok(record) => record.id,
        Err(e) => return Err(e),
    };

    // Insert pattern scores
    for pattern in pattern_scores {
        sqlx::query!(
            "INSERT INTO patterns (test_id, pattern, category, score)
            VALUES ($1, $2, $3, $4)",
            test_id,
            pattern.pattern.to_string(),
            pattern.category.to_string(),
            pattern.score as i32
        )
        .execute(pool)
        .await?;
    }

    // Insert technique scores
    for technique in technique_scores {
        sqlx::query!(
            "INSERT INTO techniques (test_id, technique, score, score_header)
            VALUES ($1, $2, $3, $4)",
            test_id,
            technique.technique.to_string(),
            technique.score as i32,
            technique.score_header.to_string()
        )
        .execute(pool)
        .await?;
    }

    // Insert bonus scores
    for bonus in bonus_scores {
        sqlx::query!(
            "INSERT INTO bonus_points (test_id, name, score)
            VALUES ($1, $2, $3)",
            test_id,
            bonus.name.to_string(),
            bonus.score as i32
        )
        .execute(pool)
        .await?;
    }

    Ok(())
}