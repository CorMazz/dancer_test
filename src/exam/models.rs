use std::{collections::HashMap, fmt, str::FromStr};
use askama::Template;
use serde::Serialize;
use sqlx::PgPool;
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

#[derive(Debug)]
pub enum BonusPointName {
    // For Leads
    NoThumbs,
    ClearTurnSignal,
    SwungTriple,
}
impl fmt::Display for BonusPointName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            BonusPointName::ClearTurnSignal => "Clear Turn Signal",
            BonusPointName::NoThumbs => "No Thumbs",
            BonusPointName::SwungTriple => "Swung Triple",
        };
        write!(f, "{}", name)
    }
}
impl FromStr for BonusPointName {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "clear_turn_signal" => Ok(BonusPointName::ClearTurnSignal),
            "no_thumbs" => Ok(BonusPointName::NoThumbs),
            "swung_triple" => Ok(BonusPointName::SwungTriple),
            _ => Err(()),
        }
    }
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

#[derive(Debug)]
pub enum TechniqueName {
    // For Leads
    BodyLead,
    Post,
    StrongFrame,
    ClosedConnection,
    ConnectionTransition,
    OnTime,
    MoveOffSlot,
    Safe,
}
impl fmt::Display for TechniqueName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            TechniqueName::BodyLead => "Body Lead",
            TechniqueName::Post => "Post",
            TechniqueName::StrongFrame => "Strong Frame",
            TechniqueName::ClosedConnection => "Closed Connection",
            TechniqueName::ConnectionTransition => "Connection Transition",
            TechniqueName::OnTime => "On Time",
            TechniqueName::MoveOffSlot => "Move Off Slot",
            TechniqueName::Safe => "Safe",
        };
        write!(f, "{}", name)
    }
}
impl FromStr for TechniqueName {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "body_lead" => Ok(TechniqueName::BodyLead),
            "post" => Ok(TechniqueName::Post),
            "strong_frame" => Ok(TechniqueName::StrongFrame),
            "closed_connection" => Ok(TechniqueName::ClosedConnection),
            "connection_transition" => Ok(TechniqueName::ConnectionTransition),
            "on_time" => Ok(TechniqueName::OnTime),
            "move_off_slot" => Ok(TechniqueName::MoveOffSlot),
            "safe" => Ok(TechniqueName::Safe),
            _ => Err(()),
        }
    }
}

// -------------------------------------------------------------------------------------------------------------------------------------------------------
// Technique Scoring Header Name
// -------------------------------------------------------------------------------------------------------------------------------------------------------

#[derive(Debug)]
pub enum TechniqueScoringHeaderName {
    Consistent90,
    Present75,
    Occasional50,
    Lacking25,
    Missing0,
}
impl fmt::Display for TechniqueScoringHeaderName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            TechniqueScoringHeaderName::Consistent90 => "Consistent 90%",
            TechniqueScoringHeaderName::Present75 => "Present 75%",
            TechniqueScoringHeaderName::Occasional50 => "Occasional 50%",
            TechniqueScoringHeaderName::Lacking25 => "Lacking 25%",
            TechniqueScoringHeaderName::Missing0 => "Missing 0%",
        };
        write!(f, "{}", name)
    }
}
impl FromStr for TechniqueScoringHeaderName {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Consistent 90%" => Ok(TechniqueScoringHeaderName::Consistent90),
            "Present 75%" => Ok(TechniqueScoringHeaderName::Present75),
            "Occasional 50%" => Ok(TechniqueScoringHeaderName::Occasional50),
            "Lacking 25%" => Ok(TechniqueScoringHeaderName::Lacking25),
            "Missing 0%" => Ok(TechniqueScoringHeaderName::Missing0),
            _ => Err(()),
        }
    }
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

#[derive(Debug)]
pub enum ScoringCategoryName {
    Footwork,
    Timing,
}
impl fmt::Display for ScoringCategoryName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            ScoringCategoryName::Footwork => "Footwork",
            ScoringCategoryName::Timing => "Timing",
        };
        write!(f, "{}", name)
    }
}
impl FromStr for ScoringCategoryName {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "footwork" => Ok(ScoringCategoryName::Footwork),
            "timing" => Ok(ScoringCategoryName::Timing),
            _ => Err(()),
        }
    }
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

#[derive(Debug)]
pub enum PatternName {
    StarterStep,
    LeftSidePassFromClosed,
    SugarTuck,
    CutoffWhip,
    LeftSidePass,
    Whip,
    SugarPush,
    SpinningSidePass,
    RightSidePass,
    BasketWhip,
    FreeSpin,
}
impl fmt::Display for PatternName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            PatternName::StarterStep => "Starter Step",
            PatternName::LeftSidePassFromClosed => "Left Side Pass from Closed",
            PatternName::SugarTuck => "Sugar Tuck",
            PatternName::CutoffWhip => "Cutoff Whip",
            PatternName::LeftSidePass => "Left Side Pass",
            PatternName::Whip => "Whip",
            PatternName::SugarPush => "Sugar Push",
            PatternName::SpinningSidePass => "Spinning Side Pass",
            PatternName::RightSidePass => "Right Side Pass",
            PatternName::BasketWhip => "Basket Whip",
            PatternName::FreeSpin => "Free Spin",
        };
        write!(f, "{}", name)
    }
}
impl FromStr for PatternName {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "starter_step" => Ok(PatternName::StarterStep),
            "left_side_pass_from_closed" => Ok(PatternName::LeftSidePassFromClosed),
            "sugar_tuck" => Ok(PatternName::SugarTuck),
            "cutoff_whip" => Ok(PatternName::CutoffWhip),
            "left_side_pass" => Ok(PatternName::LeftSidePass),
            "whip" => Ok(PatternName::Whip),
            "sugar_push" => Ok(PatternName::SugarPush),
            "spinning_side_pass" => Ok(PatternName::SpinningSidePass),
            "right_side_pass" => Ok(PatternName::RightSidePass),
            "basket_whip" => Ok(PatternName::BasketWhip),
            "free_spin" => Ok(PatternName::FreeSpin),
            _ => Err(()),
        }
    }
}

// -------------------------------------------------------------------------------------------------------------------------------------------------------
// Test Template
// -------------------------------------------------------------------------------------------------------------------------------------------------------

#[derive(Template)]
#[template(path = "./primary_templates/dancer_test.html")] 
pub struct TestTemplate {
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

#[derive(Debug, Serialize)]
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
            TechniqueScoringHeaderName::Missing0,
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
        test_type,
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