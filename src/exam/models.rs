use std::{array, collections::{HashMap, HashSet}};
use askama::Template;
use axum::{http::header, routing::head};
use chrono::NaiveDateTime;
use serde::{Deserialize, Deserializer, Serialize};

use strum_macros::{Display, EnumString};
use crate::filters;


// #######################################################################################################################################################
// #######################################################################################################################################################
// Declare Structs/Enums Used to Define the Test
// #######################################################################################################################################################
// #######################################################################################################################################################

#[derive(Debug, Serialize, Deserialize, Clone)]

pub struct TestDefinitionYaml {
    pub tests: Vec<Test>
}

/// A test object -- can be graded or ungraded, and is used to store the 
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Test {
    pub metadata: Metadata,
    pub tables: Vec<TestTable>,
    pub bonus_items: Option<Vec<BonusItem>>,
}

impl Test {
    /// Iterates over each competency scores lists and calculates the max possible score, not including bonus points. 
    fn calculate_max_score(&self) -> i32 {
        self.tables.iter()
            .flat_map(|table| table.sections.iter()) // Flatten the list of lists
            .flat_map(|section| section.competencies.iter()) // Flatten to items to be graded
            .map(|item| {
                item.scores.iter()
                    .flat_map(|score| score.iter())
                    .max()
                    .cloned()
                    .unwrap_or(0) // Get the max score for this item, or 0 if no scores
            })
            .sum() // Sum the max scores of all items
    }
    

    /// Ensures the score labels are correct, ensures that failing score labels are correct, ensures that antitheses are only present
    /// for single scoring category questions, and ensures that the max score is properly documented in the metadata. 
    /// This violates parse, don't validate, and if this method is not called it is technically possible to have an invalid test
    /// definition, but I'm going to be real, the serde documentation was a huge PITA to figure out the parse don't validate and I'm the
    /// only one using this so just remember to call validate the 2 times you ever deserialize a test from yaml. 
    pub fn validate(&self) -> Result<(), String> {
        for table in &self.tables {
            for section in &table.sections {
                validate_score_labels(&section.competencies, &section.scoring_categories, &self.metadata.test_name)?;

                validate_failing_score_labels(&section.competencies, &section.scoring_categories, &self.metadata.test_name)?;

                validate_antitheses(&section.competencies, &self.metadata.test_name)?;
            }
        }

        if self.calculate_max_score() != self.metadata.max_score {
            return Err(format!(
                "The test metadata for the test named {} indicates a max score of {} when the actual max score (without bonus points) is {}.",
                self.metadata.test_name, self.metadata.max_score, self.calculate_max_score()
            ))
        }
            
        Ok(())
    }

    pub fn grade(& mut self) -> Result<(i32, bool, Option<Vec<String>>), String> {
        let mut total_score: i32= 0;
        let mut is_passing: bool = true;
        let mut failure_explanation: Vec<String> = Vec::new();

        for table in &self.tables {
            for section in &table.sections {

                for competency in &section.competencies {
                    let competency_score: i32 = match &competency.achieved_scores {
                        Some(scores) => scores.iter().sum(),
                        None => return Err(format!("Missing scores for competency '{}' when grading the test.", competency.name)),
                    };

                    total_score += competency_score;

                    // Check to see if a competency is failing and if it is, set the test to failing
                    if let Some(failing_score_labels_items) = &competency.failing_score_labels {

                        // Create a hashmap of the header labels so that we can correspond failing score labels on the graded item to the true header labels
                        let mut achieved_scoring_category_hm: HashMap<String, String> = HashMap::new();
                        for achieved_score_label in competency.achieved_score_labels.clone().ok_or(
                            format!("Missing score labels for competency '{}' when grading the test.", competency.name),
                        )? {
                            achieved_scoring_category_hm.insert(achieved_score_label.scoring_category_name.clone(), achieved_score_label.value.clone());
                        };

                        for failing_score_label in failing_score_labels_items {
                            let achieved_score_label_value = achieved_scoring_category_hm
                                .get(&failing_score_label.scoring_category_name)
                                .ok_or(format!("Failing score label '{}' for competency '{}' does not match the achieved scoring category names for that section: {:?} (meaning your test definition was invalid).",
                                    failing_score_label.scoring_category_name, competency.name, achieved_scoring_category_hm.keys())
                                )?;

                            if failing_score_label.values.contains(&achieved_score_label_value) {
                                is_passing = false;
                                failure_explanation.push(
                                    format!("Competency {} is failing because a label of {} was achieved, and the labels '{}' fail the test.",
                                    competency.name, achieved_score_label_value, failing_score_label.values.join(", "))
                                );
                            }
                        };
                    };
                };
            };
        }

        self.metadata.achieved_score = Some(total_score);
        self.metadata.is_passing = Some(is_passing);
        self.metadata.failure_explanation = (!failure_explanation.is_empty()).then_some(failure_explanation.clone());
        self.metadata.is_graded = Some(());

        // Return the total score for the test
        Ok((total_score, is_passing, (!failure_explanation.is_empty()).then_some(failure_explanation)))
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TestTable {
    pub test_id: Option<i32>,
    pub table_id: Option<i32>,
    pub sections: Vec<TestSection>
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TestSection {
    pub table_id: Option<i32>,
    pub name: String,
    pub scoring_categories: Vec<ScoringCategory>,
    pub competencies: Vec<Competency>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BonusItem {
    pub test_id: Option<i32>,
    pub name: String,
    pub score: i32,
    pub achieved: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Metadata {
    pub test_id: Option<i32>,
    pub test_name: String,
    pub minimum_percent: f32,
    pub max_score: i32,
    pub achieved_score: Option<i32>,
    pub testee: Option<Testee>,
    pub test_date: Option<NaiveDateTime>,
    pub is_graded: Option<()>, // An option being used as a bool. So that serde_yaml parses the data
    pub is_passing: Option<bool>,
    pub failure_explanation: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ScoringCategory {
    pub section_id: Option<i32>,
    pub name: String,
    pub values: Vec<String>,
}

/// This is used to hold the score labels that cause a failure
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FailingScoreLabels {
    pub scoring_category_name: String,
    pub values: Vec<String>, 
}

/// This is used to hold the score label that the proctor gave for a competency during a test.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AchievedScoreLabel {
    pub scoring_category_name: String,
    pub value: String, 
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Competency {
    pub section_id: Option<i32>,
    pub name: String,
    pub scores: Vec<Vec<i32>>,
    pub subtext: Option<String>,
    pub failing_score_labels: Option<Vec<FailingScoreLabels>>,
    pub antithesis: Option<String>,
    pub achieved_scores: Option<Vec<i32>>,
    pub achieved_score_labels: Option<Vec<AchievedScoreLabel>>
}


/// When given the list of GradedItems and the list of HeaderLabels corresponding to a TestSection, will
/// validate that the GradedItems have scores that line up with the number of HeaderLabels in the TestSection. 
/// IE, in the following yaml ensures that there is only one scores list in the graded item named "Body Lead"
/// since there is only one header label, and ensures that the length of that scores list is 5 since there are 
/// 5 values within the header label. 
///   - section_name: "Technique Scoring"
///      scoring_categories:
///      - name: ""
///        values: ["Consistent >90%", "Present 75%", "Occasional 50%", "Lacking 25%", "Missing <10%"]
///      graded_items: 
///        - name: "Body Lead"
///          subtext: "(Week 1)"
///          scores: 
///            - [8, 6, 0, 0, 0]
fn validate_score_labels(graded_items: &[Competency], score_labels: &[ScoringCategory], test_name: &String) -> Result<(), String> {
    
    // Check to ensure that each item has one list of scores per header label.
    let expected_number_of_scores_lists = score_labels.len();
    for item in graded_items {
        if item.scores.len() != expected_number_of_scores_lists {
            return Err(format!(
                "On the test named '{},' graded item '{}' has a number of lists of scores ({}) that does not correspond to the number of scoring categories. ({})",
                test_name, item.name, item.scores.len(), score_labels.len()
            ))
        }
    }

    // Check to ensure that each item's list of scores is the same length as the corresponding list of header labels.
    for (i, score_label) in score_labels.into_iter().enumerate() {
        let expected_number_of_scores = score_label.values.len();
        for item in graded_items {
            if item.scores[i].len() != expected_number_of_scores {
                return Err(format!(
                    "On the test named '{},' the graded item named '{}' has a score list at index {} of length {} that does not correspond to the number of score labels ({}) for the scoring category at index {}.",
                    test_name, item.name, i, item.scores[i].len(), expected_number_of_scores, i
                ))
            }
        }
    }
    Ok(())
}

/// Checks to ensure that all of the failing score labels for the graded items correspond to actual header values.
/// IE, in the following yaml, checks to ensure that the failing score labels for the starter step correspond
/// to actual header section labels and that the values correspond to the values. So it matches the string footwork
/// to footwork and makes sure "Nope" is inside the list of scoring_categories values. 
/// scoring_categories:
/// - name: "Footwork"
///   values: ["Perfect", "Variation?", "Right Concept", "Nope"]
/// - name: "Timing"
///   values: ["On", "Off"]
/// graded_items:
/// - name: "Starter Step"
///   scores: 
///     - [3, 2, 1, 0]
///     - [1, 0]
///   failing_score_labels: 
///     - name: "Footwork"
///       values: ["Nope"]
fn validate_failing_score_labels(graded_items: &[Competency], score_labels: &[ScoringCategory], test_name: &String) -> Result<(), String> {

    // Create a hashmap of the header labels so that we can correspond failing score labels on the graded item to the true header labels
    let mut score_label_hm: HashMap<String, Vec<String>> = HashMap::new();
    for score_label in score_labels {
        if let Some(duplicate_name) = score_label_hm.insert(score_label.name.clone(), score_label.values.clone()) {
            return Err(format!(
                "On the test named '{},' the scoring category name '{:#?}' is not unique within its section.",
                test_name, duplicate_name
            ))
        };
    }

    for item in graded_items {
        match &item.failing_score_labels {
            // Has failing score labels
            Some(labels) => for label in labels {

                match score_label_hm.get(&label.scoring_category_name) {
                    // The failing score label corresponds to a section (ie, the footwork section)
                    Some(valid_failing_score_labels) => for failing_score_label in &label.values {
                        if !valid_failing_score_labels.contains(&failing_score_label) {
                            return Err(format!(
                                "On the test named '{},' the graded item named '{}' has a failing score label '{}' that does not correspond to any of the score labels ({:#?}) in the scoring category named '{}'.",
                                test_name, item.name, failing_score_label, valid_failing_score_labels, label.scoring_category_name
                            ))
                        }
                    },
                    // The failing score label does not correspond to a valid section
                    None => return Err(format!(
                        "On the test named '{},' the graded item named '{}' has failing score labels '{:#?}' under the scoring category '{}' that does not correspond to any of the valid scoring category labels ({:#?}).",
                        test_name, item.name, label.values, label.scoring_category_name, score_label_hm.keys()
                    ))
                }
            }
            // Does not have failing score labels
            None => continue
        }
    }
    Ok(())
}

/// Ensures that if there is more than one scoring category for an competency (which can be checked by checking the length of the
/// vec of scores) that the item does not have an antithesis. 
fn validate_antitheses(graded_items: &[Competency], test_name: &String) -> Result<(), String> {
    for item in graded_items {
        match &item.antithesis {
            Some(antithesis) => if item.scores.len() > 1 {return Err(format!(
                "On the test named '{},' the competency named '{}' has an antithesis {} which is not supported when there is more than one scoring category for that item.",
                test_name, item.name, antithesis
            ))}
            None => continue
        }
    }
    Ok(())
}

// // #######################################################################################################################################################
// // #######################################################################################################################################################
// // Declare Structs/Enums Used to Grade the Test
// // #######################################################################################################################################################
// // #######################################################################################################################################################

// // There is code duplication here but Rust doesn't have inheritance. I would rather deal with the duplication
// // than having to make achieved scores option types in the original test definition

// #[derive(Debug, Serialize, Deserialize, Clone)]
// pub struct GradedTestDefinition {
//     pub metadata: GradedMetadata,
//     pub tables: Vec<GradedTestTable>,
//     pub bonus_items: Option<Vec<GradedBonusItem>>,
// }

// #[derive(Debug, Serialize, Deserialize, Clone)]
// pub struct GradedMetadata {
//     pub test_name: String,
//     pub test_type: TestType,
//     pub minimum_percent: f64,
//     pub max_score: u64,
//     pub achieved_score: u64,
// }

// #[derive(Debug, Serialize, Deserialize, Clone)]
// pub struct GradedTestTable {
//     sections: Vec<GradedTestSection>
// }

// #[derive(Debug, Serialize, Deserialize, Clone)]
// pub struct GradedTestSection {
//     pub name: String,
//     pub scoring_categories: Vec<ScoringCategory>,
//     pub graded_items: Vec<GradedItemToBeGraded>
// }

// #[derive(Debug, Serialize, Deserialize, Clone)]
// pub struct BonusItem {
//     pub original: BonusItem,
//     pub achieved_score: u64,
// }

// #[derive(Debug, Serialize, Deserialize, Clone)]
// pub struct GradedItemToBeGraded {
//     pub original: ItemToBeGraded,
//     pub achieved_scores: Vec<u64>,
//     pub score_labels: Vec<String>,
//     pub is_failing: bool,
// }


// // #######################################################################################################################################################
// // #######################################################################################################################################################
// // Declare Structs/Enums Used to Define the Test
// // #######################################################################################################################################################
// // #######################################################################################################################################################

// // Using structs/enums makes the compiler ensure that all future uses are spelled correctly, as opposed to just strings. 
// // Need to implement fmt::Display and FromStr for all of these because the html takes the names of the enums and turns them into strings as the id of the
// // html elements, but the element IDs can't have spaces. 
// // NONE OF THESE STRINGS SHOULD HAVE -- IN THEM, THAT IS RESERVED AS A DELIMITER WITHIN THE HTML

// // -------------------------------------------------------------------------------------------------------------------------------------------------------
// // Bonus Point Name
// // -------------------------------------------------------------------------------------------------------------------------------------------------------

// #[derive(Debug, EnumString, Display, Serialize, Deserialize)]
// #[strum(ascii_case_insensitive)]
// pub enum BonusPointName {

//     // Leads
//     #[strum(serialize = "clear_turn_signal", to_string = "Clear Turn Signal")]
//     ClearTurnSignal,

//     #[strum(serialize = "no_thumbs", to_string = "No Thumbs")]
//     NoThumbs,

//     // Both
//     #[strum(serialize = "swung_triple", to_string = "Swung Triple")]
//     SwungTriple,

//     // Follows
//     #[strum(serialize = "no_biceps", to_string = "No Biceps")]
//     NoBiceps,

//     #[strum(serialize = "spotting", to_string = "Spotting")]
//     Spotting,

//     #[strum(serialize = "connection_in_motion", to_string = "Connection in Motion")]
//     ConnectionInMotion,
// }


// // -------------------------------------------------------------------------------------------------------------------------------------------------------
// // Test Definition Bonus Item
// // -------------------------------------------------------------------------------------------------------------------------------------------------------

// /// A struct to define something that will give you bonus points on the test
// pub struct TestDefinitionBonusItem {
//     pub label: BonusPointName,
//     pub points: u32,
// }

// // -------------------------------------------------------------------------------------------------------------------------------------------------------
// // Technique Name
// // -------------------------------------------------------------------------------------------------------------------------------------------------------

// /// Old pattern names, technique names, and other enum variants CANNOT be retired so long 
// /// as they may exist within the database. 
// #[derive(Debug, EnumString, Display, Serialize, Deserialize)]
// pub enum TechniqueName {
//     // Leads
//     #[strum(serialize = "body_lead", to_string = "Body Lead")]
//     BodyLead,

//     #[strum(serialize = "closed_connection", to_string = "Closed Connection")]
//     ClosedConnection,

//     #[strum(serialize = "on_time", to_string = "On Time")]
//     OnTime,

//     #[strum(serialize = "move_off_slot", to_string = "Move Off Slot")]
//     MoveOffSlot,

//     #[strum(serialize = "safe", to_string = "Safe")]
//     Safe,

//     #[strum(serialize = "prep", to_string = "Prep")]
//     Prep,

//     // Both

//     #[strum(serialize = "post", to_string = "Post")]
//     Post,

//     #[strum(serialize = "strong_frame", to_string = "Strong Frame")]
//     StrongFrame,

//     #[strum(serialize = "connection_transition", to_string = "Connection Transition")]
//     ConnectionTransition,

//     #[strum(serialize = "body_angle", to_string = "Body Angle")]
//     BodyAngle,

//     // Follows

//     #[strum(serialize = "anchor_in_3rd_position", to_string = "Anchor in 3rd Position")]
//     AnchorIn3rdPosition,

//     #[strum(serialize = "stretch", to_string = "Stretch")]
//     Stretch,

//     #[strum(serialize = "variable_speed", to_string = "Variable Speed")]
//     VariableSpeed,

//     #[strum(serialize = "connection_hierarchy", to_string = "Connection Hierarchy")]
//     ConnectionHierarchy,

//     #[strum(serialize = "directionality", to_string = "Directionality")]
//     Directionality,

//     #[strum(serialize = "kissing_connection", to_string = "Kissing Connection")]
//     KissingConnection,

//     #[strum(serialize = "spins", to_string = "Spins")]
//     Spins,

//     #[strum(serialize = "personal_safety", to_string = "Personal Safety")]
//     PersonalSafety,

//     #[strum(serialize = "partner_safety", to_string = "Partner Safety")]
//     PartnerSafety,
// }

// // -------------------------------------------------------------------------------------------------------------------------------------------------------
// // Technique Scoring Header Name
// // -------------------------------------------------------------------------------------------------------------------------------------------------------

// /// This one doesn't get converted to snake case and isn't used as an html value so can have ><
// #[derive(Debug, EnumString, Display, Serialize, Deserialize)]
// pub enum TechniqueScoringHeaderName {
//     // For most techniques
//     #[strum(serialize = "Consistent >90%")]
//     Consistent90,

//     #[strum(serialize = "Present 75%")]
//     Present75,

//     #[strum(serialize = "Occasional 50%")]
//     Occasional50,

//     #[strum(serialize = "Lacking 25%")]
//     Lacking25,

//     #[strum(serialize = "Missing <10%")]
//     Missing10,

//     // For level of angle technique

//     #[strum(serialize = "Perpendicular")]
//     Perpendicular,

//     #[strum(serialize = "Over-Angled")]
//     OverAngled,

//     #[strum(serialize = "Angled")]
//     Angled,

//     #[strum(serialize = "Under-Angled")]
//     UnderAngled,

//     #[strum(serialize = "Flat")]
//     Flat,

//     // For level of prep technique

//     #[strum(serialize = "Overkill Prep")]
//     OverkillPrep,

//     #[strum(serialize = "Over-Prepped")]
//     Overprepped,

//     #[strum(serialize = "Prepped")]
//     Prepped,

//     #[strum(serialize = "Under-Prepped")]
//     UnderPrepped,

//     #[strum(serialize = "No Prep")]
//     NoPrep,

// }
// // -------------------------------------------------------------------------------------------------------------------------------------------------------
// // Test Definition Technique
// // -------------------------------------------------------------------------------------------------------------------------------------------------------

// pub struct TestDefinitionTechnique {
//     pub name: TechniqueName,
//     pub subtext: &'static str,
//     pub points: Vec<u32>,
//     pub antithesis: &'static str,
//     pub max_score: u32,
// }
// impl TestDefinitionTechnique {
//     pub fn new(name: TechniqueName, subtext: &'static str, points: Vec<u32>, antithesis: &'static str) -> Self {
//         let max_score = points.iter().copied().max().unwrap_or(0);
//         Self {
//             name,
//             subtext,
//             points,
//             antithesis,
//             max_score,
//         }
//     }
// }

// // -------------------------------------------------------------------------------------------------------------------------------------------------------
// // Test Definition Technique Group
// // -------------------------------------------------------------------------------------------------------------------------------------------------------

// // Both vecs should have the same length 
// pub struct TestDefinitionTechniqueGroup {
//     pub technique_headers: Vec<TechniqueScoringHeaderName>,
//     pub techniques: Vec<TestDefinitionTechnique>,
// }

// // -------------------------------------------------------------------------------------------------------------------------------------------------------
// // Scoring Category Name
// // -------------------------------------------------------------------------------------------------------------------------------------------------------

// /// Old pattern names, technique names, and other enum variants CANNOT be retired so long 
// /// as they may exist within the database. 
// #[derive(Debug, EnumString, Display, Serialize, Deserialize)]
// pub enum ScoringCategoryName {

//     // Both

//     #[strum(serialize = "footwork", to_string = "Footwork")]
//     Footwork,

//     #[strum(serialize = "timing", to_string = "Timing")]
//     Timing,

//     // Follows

//     #[strum(serialize = "shaping", to_string = "Shaping")]
//     Shaping
// }

// // -------------------------------------------------------------------------------------------------------------------------------------------------------
// // Pattern Scoring Category
// // -------------------------------------------------------------------------------------------------------------------------------------------------------

// pub struct TestDefinitionPatternScoringCategory {
//     pub name: ScoringCategoryName,
//     pub headers: Vec<&'static str>,
//     pub points: Vec<u32>,
//     pub max_score: u32
// }
// impl TestDefinitionPatternScoringCategory {
//     pub fn new(name: ScoringCategoryName, headers: Vec<&'static str>, points: Vec<u32>) -> Self {
//         let max_score = points.iter().copied().max().unwrap_or(0);

//         Self {
//             name,
//             headers,
//             points,
//             max_score,
//         }
//     }
// }

// // -------------------------------------------------------------------------------------------------------------------------------------------------------
// // Pattern Name
// // -------------------------------------------------------------------------------------------------------------------------------------------------------

// /// Old pattern names, technique names, and other enum variants CANNOT be retired so long 
// /// as they may exist within the database. 
// #[derive(Debug, EnumString, Display, Serialize, Deserialize)]
// pub enum PatternName {

//     // Leads

//     #[strum(serialize = "starter_step", to_string = "Starter Step")]
//     StarterStep,

//     #[strum(serialize = "left_side_pass_from_closed", to_string = "Left Side Pass from Closed")]
//     LeftSidePassFromClosed,

//     #[strum(serialize = "sugar_tuck", to_string = "Sugar Tuck")]
//     SugarTuck,

//     #[strum(serialize = "cutoff_whip", to_string = "Cutoff Whip")]
//     CutoffWhip,

//     #[strum(serialize = "basket_whip", to_string = "Basket Whip")]
//     BasketWhip,

//     #[strum(serialize = "free_spin", to_string = "Free Spin")]
//     FreeSpin,

//     // Both

//     #[strum(serialize = "left_side_pass", to_string = "Left Side Pass")]
//     LeftSidePass,

//     #[strum(serialize = "right_side_pass", to_string = "Right Side Pass")]
//     RightSidePass,
    
//     #[strum(serialize = "whip", to_string = "Whip")]
//     Whip,

//     #[strum(serialize = "spinning_side_pass", to_string = "Spinning Side Pass")]
//     SpinningSidePass,

//     #[strum(serialize = "sugar_push", to_string = "Sugar Push")]
//     SugarPush,

// }

// // -------------------------------------------------------------------------------------------------------------------------------------------------------
// // Test Definition
// // -------------------------------------------------------------------------------------------------------------------------------------------------------

// pub struct TestDefinition {
//     pub test_type: TestType,
//     pub passing_score: u32,
//     pub patterns: Vec<PatternName>,
//     pub pattern_scoring_categories: Vec<TestDefinitionPatternScoringCategory>,
//     pub technique_groups: Vec<TestDefinitionTechniqueGroup>,
//     pub bonus_items: Vec<TestDefinitionBonusItem>,
// }

// // #######################################################################################################################################################
// // #######################################################################################################################################################
// // Declare Structs/Enums Used to Record Test Results
// // #######################################################################################################################################################
// // #######################################################################################################################################################
// // Using structs/enums makes the compiler ensure that all future uses are spelled correctly, as opposed to just strings. 

// #[derive(Debug, sqlx::FromRow, Serialize, Deserialize)]
// pub struct GradedPattern {
//     // pub id: Option<i32>,  // Optional ID field
//     // pub test_id: Option<i32>,  // Optional test ID field
//     pub pattern: PatternName,
//     pub category: ScoringCategoryName,
//     pub score: u32,
//     pub max_score: u32,
// }

// #[derive(Debug, sqlx::FromRow, Serialize, Deserialize)]
// pub struct GradedTechnique {
//     // pub id: Option<i32>,  // Optional ID field
//     // pub test_id: Option<i32>,  // Optional test ID field
//     pub technique: TechniqueName,
//     pub score: u32,
//     pub score_header: TechniqueScoringHeaderName,
//     pub max_score: u32,
// }

// #[derive(Debug, sqlx::FromRow, Serialize, Deserialize)]
// pub struct GradedBonusPoint {
//     // pub id: Option<i32>,  // Optional ID field
//     // pub test_id: Option<i32>,  // Optional test ID field
//     pub name: BonusPointName,
//     pub score: u32,
// }

#[derive(Debug, Clone, sqlx::FromRow, Serialize, Deserialize)]
pub struct Testee {
    pub id: Option<i32>,  
    pub first_name: String,
    pub last_name: String,
    pub email: String,
}

// impl From<HashMap<String, String>> for Testee {
//     fn from(mut map: HashMap<String, String>) -> Self {
//         let first_name = map.remove("first_name").unwrap_or_default();
//         let last_name = map.remove("last_name").unwrap_or_default();
//         let email = map.remove("email").unwrap_or_default();

//         Testee {
//             id: None,  // ID is None when parsed from form data
//             first_name,
//             last_name,
//             email,
//         }
//     }
// }

// #[derive(Debug, Serialize, Deserialize)]
// pub struct GradedTest {
//     pub testee: Testee,
//     pub test_date: NaiveDateTime,
//     pub test_type: TestType,
//     pub score: u32,
//     pub max_score: u32,
//     pub passing_score: u32,
//     pub patterns: Vec<GradedPattern>,
//     pub techniques: Vec<GradedTechnique>,
//     pub bonuses: Vec<GradedBonusPoint>,
// }

// impl GradedTest {
//     pub fn new(
//         testee: Testee,
//         test_date: NaiveDateTime,
//         test_type: TestType,
//         passing_score: u32,
//         patterns: Vec<GradedPattern>,
//         techniques: Vec<GradedTechnique>,
//         bonuses: Vec<GradedBonusPoint>,
//     ) -> Self {
//         let mut score = 0;
//         let mut max_score = 0;

//         for pattern in &patterns {
//             score += pattern.score;
//             max_score += pattern.max_score;
//         }

//         for technique in &techniques {
//             score += technique.score;
//             max_score += technique.max_score;
//         }

//         for bonus in &bonuses {
//             score += bonus.score;
//         }

//         Self {
//             testee,
//             test_date,
//             test_type,
//             score,
//             max_score,
//             passing_score,
//             patterns,
//             techniques,
//             bonuses,
//         }
//     }
// }

#[derive(Debug, Serialize, Deserialize)]
/// Passing may be failed even if the achieved percent is above the minimum percent if a competency with a failing score label was graded as failing. 
pub struct TestSummary {
    pub test_id: i32,
    pub test_date: NaiveDateTime,
    pub test_name: String,  // This probably should've been labeled test_type, but I'm lazy here...
    pub achieved_score: i32,
    pub achieved_percent: f32,
    pub max_score: i32,
    pub minimum_percent: f32,
    pub is_passing: bool,
    pub failure_explanation: Option<Vec<String>>,
}

// // #######################################################################################################################################################
// // #######################################################################################################################################################
// // Functions to Generate the Tests and Parse Test Results
// // #######################################################################################################################################################
// // #######################################################################################################################################################

// // -------------------------------------------------------------------------------------------------------------------------------------------------------
// // Leader Test
// // -------------------------------------------------------------------------------------------------------------------------------------------------------

// pub fn generate_leader_test() -> TestDefinition {
//     TestDefinition {
//         test_type: TestType::Leader,
//         passing_score: 60,
//         patterns:  vec![
//             PatternName::StarterStep,
//             PatternName::LeftSidePassFromClosed,
//             PatternName::SugarTuck,
//             PatternName::CutoffWhip,
//             PatternName::LeftSidePass,
//             PatternName::Whip,
//             PatternName::SugarPush,
//             PatternName::SpinningSidePass,
//             PatternName::RightSidePass,
//             PatternName::BasketWhip,
//             PatternName::FreeSpin,
//         ],
//         pattern_scoring_categories: vec![
//             TestDefinitionPatternScoringCategory::new(
//                 ScoringCategoryName::Footwork,
//                 vec!["Perfect", "Variation?", "Right Concept", "Nope"],
//                 vec![3, 2, 1, 0],
//             ),
//             TestDefinitionPatternScoringCategory::new(
//                 ScoringCategoryName::Timing,
//                 vec!["On", "Off"],
//                 vec![1, 0],
//             ),
//         ],

//         technique_groups: vec![
//             TestDefinitionTechniqueGroup {
//                 technique_headers: vec![
//                     TechniqueScoringHeaderName::Consistent90,
//                     TechniqueScoringHeaderName::Present75,
//                     TechniqueScoringHeaderName::Occasional50,
//                     TechniqueScoringHeaderName::Lacking25,
//                     TechniqueScoringHeaderName::Missing10,
//                 ],
//                 techniques: vec![
//                     TestDefinitionTechnique::new(
//                         TechniqueName::BodyLead,
//                         "(Week 1)",
//                         vec![8, 6, 0, 0, 0],
//                         "Arm Lead",
//                     ),
//                     TestDefinitionTechnique::new(
//                         TechniqueName::Post,
//                         "(Week 1)",
//                         vec![6, 4, 0, 0, 0],
//                         "Floating Anchor",
//                     ),
//                     TestDefinitionTechnique::new(
//                         TechniqueName::StrongFrame,
//                         "(Week 2)",
//                         vec![6, 4, 2, 0, 0],
//                         "Weak Frame",
//                     ),
//                     TestDefinitionTechnique::new(
//                         TechniqueName::ClosedConnection,
//                         "(Week 3/4)",
//                         vec![4, 3, 2, 0, 0],
//                         "Free Hand Only",
//                     ),
//                     TestDefinitionTechnique::new(
//                         TechniqueName::ConnectionTransition,
//                         "(Week 2 - Dimmer Switch)",
//                         vec![4, 3, 2, 0, 0],
//                         "Brick Wall (Toggle Switch)",
//                     ),
//                     TestDefinitionTechnique::new(
//                         TechniqueName::OnTime,
//                         "",
//                         vec![8, 6, 0, 0 ,0], 
//                         "Off Time",
//                     ),
//                     TestDefinitionTechnique::new(
//                         TechniqueName::MoveOffSlot,
//                         "",
//                         vec![4, 3, 0, 0, 0],
//                         "In the Way",
//                     ),
//                     TestDefinitionTechnique::new(
//                         TechniqueName::Safe,
//                         "",
//                         vec![8, 0, 0, 0, 0],
//                         "Unsafe",
//                     ),
//                 ],
//             },
//             TestDefinitionTechniqueGroup {
//                 technique_headers: vec![
//                     TechniqueScoringHeaderName::Perpendicular,
//                     TechniqueScoringHeaderName::OverAngled,
//                     TechniqueScoringHeaderName::Angled,
//                     TechniqueScoringHeaderName::UnderAngled,
//                     TechniqueScoringHeaderName::Flat,
//                 ],
//                 techniques: vec![
//                     TestDefinitionTechnique::new(
//                         TechniqueName::BodyAngle,
//                         "(Week 2)",
//                         vec![0, 2, 3, 2, 0],
//                         "",
//                     ),
//                 ],
//             },
//             TestDefinitionTechniqueGroup {
//                 technique_headers: vec![
//                     TechniqueScoringHeaderName::OverkillPrep,
//                     TechniqueScoringHeaderName::Overprepped,
//                     TechniqueScoringHeaderName::Prepped,
//                     TechniqueScoringHeaderName::UnderPrepped,
//                     TechniqueScoringHeaderName::NoPrep,
//                 ],
//                 techniques: vec![
//                     TestDefinitionTechnique::new(
//                         TechniqueName::Prep,
//                         "(Week 3)",
//                         vec![0, 2, 3, 0, 0],
//                         "",
//                     ),
//                 ],
//             },
//         ],
//         bonus_items: vec![
//             TestDefinitionBonusItem {
//                 label: BonusPointName::NoThumbs,
//                 points: 1,
//             },
//             TestDefinitionBonusItem {
//                 label: BonusPointName::ClearTurnSignal,
//                 points: 1,
//             },
//             TestDefinitionBonusItem {
//                 label: BonusPointName::SwungTriple,
//                 points: 4,
//             },
//         ],
//     }
// }

// // -------------------------------------------------------------------------------------------------------------------------------------------------------
// // Leader Test
// // -------------------------------------------------------------------------------------------------------------------------------------------------------

// pub fn generate_follower_test() -> TestDefinition {
//     TestDefinition {
//         test_type: TestType::Follower,
//         passing_score: 60,
//         patterns:  vec![
//             PatternName::LeftSidePass,
//             PatternName::RightSidePass,
//             PatternName::Whip,
//             PatternName::SpinningSidePass,
//             PatternName::SugarPush,
//         ],
//         pattern_scoring_categories: vec![
//             TestDefinitionPatternScoringCategory::new(
//                 ScoringCategoryName::Footwork,
//                 vec!["Perfect", "Right Concept", "Nope"],
//                 vec![2, 1, 0],
//             ),
//             TestDefinitionPatternScoringCategory::new(
//                 ScoringCategoryName::Timing,
//                 vec!["On", "Off"],
//                 vec![1, 0],
//             ),
//             TestDefinitionPatternScoringCategory::new(
//                 ScoringCategoryName::Shaping,
//                 vec!["Shaped", "Flat"],
//                 vec![1, 0],
//             ),
//         ],
//         technique_groups: vec![
//             TestDefinitionTechniqueGroup {
//                 technique_headers: vec![
//                     TechniqueScoringHeaderName::Consistent90,
//                     TechniqueScoringHeaderName::Present75,
//                     TechniqueScoringHeaderName::Occasional50,
//                     TechniqueScoringHeaderName::Lacking25,
//                     TechniqueScoringHeaderName::Missing10,
//                 ],
//                 techniques: vec![
//                     TestDefinitionTechnique::new(
//                         TechniqueName::StrongFrame,
//                         "(Week 2)",
//                         vec![8, 5, 0, 0, 0],
//                         "Weak Frame",
//                     ),
//                     TestDefinitionTechnique::new(
//                         TechniqueName::Post,
//                         "(Week 2)",
//                         vec![6, 4, 2, 0, 0],
//                         "Floating Anchor",
//                     ),
//                     TestDefinitionTechnique::new(
//                         TechniqueName::AnchorIn3rdPosition,
//                         "(Week 2)",
//                         vec![8, 0, 0, 0, 0],
//                         "Rock Step",
//                     ),
//                     TestDefinitionTechnique::new(
//                         TechniqueName::Stretch,
//                         "Week 1/2)",
//                         vec![6, 4, 2, 0, 0],
//                         "Over Eager",
//                     ),
//                     TestDefinitionTechnique::new(
//                         TechniqueName::VariableSpeed,
//                         "(Week 1)",
//                         vec![4, 3, 2, 1, 0],
//                         "Monotonous",
//                     ),
//                     TestDefinitionTechnique::new(
//                         TechniqueName::ConnectionTransition,
//                         "(Week 2 - Dimmer Switch)",
//                         vec![8, 5, 0, 0, 0],
//                         "Brick Wall (Toggle Switch)",
//                     ),
//                     TestDefinitionTechnique::new(
//                         TechniqueName::ConnectionHierarchy,
//                         "(Week 4)",
//                         vec![6, 4, 2, 0, 0],
//                         "Panic",
//                     ),
//                     TestDefinitionTechnique::new(
//                         TechniqueName::Directionality,
//                         "(Week 1/3)",
//                         vec![6, 4, 0, 0, 0],
//                         "Directionless",
//                     ),
//                     TestDefinitionTechnique::new(
//                         TechniqueName::KissingConnection,
//                         "(Week 3)",
//                         vec![6, 4, 2, 0, 0],
//                         "Disconnected",
//                     ),
//                     TestDefinitionTechnique::new(
//                         TechniqueName::Spins,
//                         "(Week 3)",
//                         vec![4, 3, 0, 0, 0],
//                         "Unbalanced",
//                     ),
//                     TestDefinitionTechnique::new(
//                         TechniqueName::PersonalSafety,
//                         "",
//                         vec![8, 5, 0, 0, 0],
//                         "Masochist",
//                     ),
//                     TestDefinitionTechnique::new(
//                         TechniqueName::PartnerSafety,
//                         "",
//                         vec![8, 0, 0, 0, 0],
//                         "Abusive",
//                     ),
//                 ],
//             },
//             TestDefinitionTechniqueGroup {
//                 technique_headers: vec![
//                     TechniqueScoringHeaderName::Perpendicular,
//                     TechniqueScoringHeaderName::OverAngled,
//                     TechniqueScoringHeaderName::Angled,
//                     TechniqueScoringHeaderName::UnderAngled,
//                     TechniqueScoringHeaderName::Flat,
//                 ],
//                 techniques: vec![
//                     TestDefinitionTechnique::new(
//                         TechniqueName::BodyAngle,
//                         "(Week 2)",
//                         vec![0, 2, 3, 2, 0],
//                         "",
//                     ),
//                 ],
//             },
//         ],
//         bonus_items: vec![
//             TestDefinitionBonusItem {
//                 label: BonusPointName::NoBiceps,
//                 points: 1,
//             },
//             TestDefinitionBonusItem {
//                 label: BonusPointName::Spotting,
//                 points: 1,
//             },
//             TestDefinitionBonusItem {
//                 label: BonusPointName::ConnectionInMotion,
//                 points: 2,
//             },
//             TestDefinitionBonusItem {
//                 label: BonusPointName::SwungTriple,
//                 points: 4,
//             },
//         ],
//     }
// }


