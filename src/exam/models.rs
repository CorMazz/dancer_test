use std::collections::HashMap;
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use uuid::Uuid;



// #######################################################################################################################################################
// #######################################################################################################################################################
// Declare Structs/Enums Used to Define the Test
// #######################################################################################################################################################
// #######################################################################################################################################################

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct TestDefinitionYaml {
    pub tests: Vec<UngradedTest>
}

pub enum Test {
    Ungraded(UngradedTest),
    Graded(GradedTest),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct UngradedTest {
    pub metadata: UngradedMetadata,
    pub tables: Vec<UngradedTestTable>,
    pub bonus_items: Option<Vec<UngradedBonusItem>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GradedTest {
    pub metadata: GradedMetadata,
    pub tables: Vec<GradedTestTable>,
    pub bonus_items: Option<Vec<GradedBonusItem>>,
}

impl UngradedTest {
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

        Ok(())
    }
}
//     pub fn grade(& mut self) -> Result<(i32, bool, Option<Vec<String>>), String> {
//         let mut total_score: i32= 0;
//         let mut is_passing: bool = true;
//         let mut failure_explanation: Vec<String> = Vec::new();

//         for table in &self.tables {
//             for section in &table.sections {

//                 for competency in &section.competencies {
//                     let competency_score: i32 = match &competency.achieved_scores {
//                         Some(scores) => scores.iter().sum(),
//                         None => return Err(format!("Missing scores for competency '{}' when grading the test.", competency.name)),
//                     };

//                     total_score += competency_score;

//                     // Check to see if a competency is failing and if it is, set the test to failing
//                     if let Some(failing_score_labels_items) = &competency.failing_score_labels {

//                         // Create a hashmap of the header labels so that we can correspond failing score labels on the graded item to the true header labels
//                         let mut achieved_scoring_category_hm: HashMap<String, String> = HashMap::new();
//                         for achieved_score_label in competency.achieved_score_labels.clone().ok_or(
//                             format!("Missing score labels for competency '{}' when grading the test.", competency.name),
//                         )? {
//                             achieved_scoring_category_hm.insert(achieved_score_label.scoring_category_name.clone(), achieved_score_label.value.clone());
//                         };

//                         for failing_score_label in failing_score_labels_items {
//                             let achieved_score_label_value = achieved_scoring_category_hm
//                                 .get(&failing_score_label.scoring_category_name)
//                                 .ok_or(format!("Failing score label '{}' for competency '{}' does not match the achieved scoring category names for that section: {:?} (meaning your test definition was invalid).",
//                                     failing_score_label.scoring_category_name, competency.name, achieved_scoring_category_hm.keys())
//                                 )?;

//                             if failing_score_label.values.contains(&achieved_score_label_value) {
//                                 // TODO refactor this to give the raw parts of the string instead of the formatted string
//                                 let explanation = if section.scoring_categories.len() > 1 {
//                                     format!(
//                                         "Competency '{}' is failing because a label of '{}' was achieved for the '{}' category, and the label(s) '{}' fail the test.",
//                                         competency.name,
//                                         achieved_score_label_value,
//                                         &failing_score_label.scoring_category_name,
//                                         failing_score_label.values.join(", ")
//                                     )
//                                 } else {
//                                     format!(
//                                         "Competency '{}' is failing because a label of '{}' was achieved, and the label(s) '{}' fail the test.",
//                                         competency.name,
//                                         achieved_score_label_value,
//                                         failing_score_label.values.join(", ")
//                                     )
//                                 };
                                
//                                 is_passing = false;
//                                 failure_explanation.push(explanation);
//                             }
//                         };
//                     };
//                 };
//             };
//         }

//         // Check if the achieved percent is above the minimum percent
//         if ((total_score as f32) / (self.metadata.max_score as f32)) < self.metadata.minimum_percent {
//             is_passing = false;
//             failure_explanation.push(format!("Your score of {:.1}% is lower than the minimum passing score of {:.1}%.",
//             ((total_score as f32) / (self.metadata.max_score as f32)) * 100.0, self.metadata.minimum_percent * 100.0 ));
//         }

//         self.metadata.achieved_score = Some(total_score);
//         self.metadata.is_passing = Some(is_passing);
//         self.metadata.failure_explanation = (!failure_explanation.is_empty()).then_some(failure_explanation.clone());
//         self.metadata.is_graded = Some(());

//         // Return the total score for the test
//         Ok((total_score, is_passing, (!failure_explanation.is_empty()).then_some(failure_explanation)))
//     }


//     pub fn full_summary(&self) -> Result<FullTestSummary, String> {
        
//         let grade_summary = self.grade_summary()?;

//         Ok(FullTestSummary {
//             test_id: self.metadata.test_id.ok_or("No test id on this test to summarize. Has this test been graded yet?")?, 
//             test_date: self.metadata.test_date.ok_or("No test date on this test to summarize. Has this test been graded?")?,
//             test_name: self.metadata.test_name.clone(),
//             proctor: self.metadata.proctor.clone().ok_or("No proctor on this test to summarize. Has this test been graded?")?,
//             grade_summary
//         })
//     }

//     /// Used to just provide the grade information on a graded test. Call the grade method first.
//     pub fn grade_summary(&self) -> Result<TestGradeSummary, String> {
//         // Check that the test is graded
//         self.metadata.is_graded.ok_or("Cannot give a grade summary on an ungraded test.".to_string())?;

//         Ok(TestGradeSummary {
//             achieved_score: self.metadata.achieved_score.ok_or("Invariant that graded tests all have an achieved score violated in get_test_results fn")?,
//             achieved_percent: self.metadata.achieved_score.ok_or("Invariant that graded tests all have an achieved score violated in get_test_results fn")? as f32 / self.metadata.max_score as f32,
//             max_score: self.metadata.max_score,
//             minimum_percent: self.metadata.minimum_percent,
//             is_passing: self.metadata.is_passing.ok_or("Invariant that graded tests all have is_passing violated in get_test_results fn")?,
//             failure_explanation: self.metadata.failure_explanation.clone()
//         })
//     }
// }

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UngradedTestTable {
    pub sections: Vec<UngradedTestSection>
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GradedTestTable {
    pub test_id: Uuid,
    pub table_id: Uuid,
    pub ungraded_test_table: UngradedTestTable,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UngradedTestSection {
    pub name: String,
    pub scoring_categories: Vec<UngradedScoringCategory>,
    pub competencies: Vec<UngradedCompetency>,
}


#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GradedTestSection {
    pub table_id: Uuid,
    pub ungraded_test_section: UngradedTestSection,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UngradedBonusItem {
    pub name: String,
    pub score: i32,
}

impl UngradedBonusItem {
    pub fn grade(self, achieved: bool, test_id: Uuid) -> GradedBonusItem {
        GradedBonusItem {
            test_id,
            achieved,
            ungraded_bonus_item: self
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GradedBonusItem {
    pub test_id: Uuid,
    pub achieved: bool,
    ungraded_bonus_item: UngradedBonusItem,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UngradedMetadata {
    pub test_name: String,
    pub minimum_percent: f32,
    pub config_settings: TestConfig,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GradedMetadata {
    pub test_id: Uuid,
    pub max_score: i32,
    pub achieved_score: i32,
    pub testee: Testee,
    pub test_date: NaiveDateTime,
    pub is_passing: bool,
    pub proctor: Proctor,
    pub failure_explanation: Option<Vec<String>>,
    pub ungraded_metadata: UngradedMetadata,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TestConfig {
    pub live_grading: bool,
    pub show_point_values: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UngradedScoringCategory {
    pub name: String,
    pub values: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GradedScoringCategory {
    pub section_id: Uuid,
    pub ungraded_scoring_category: UngradedScoringCategory,
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
pub struct UngradedCompetency {
    pub name: String,
    pub scores: Vec<Vec<i32>>,
    pub subtext: Option<String>,
    pub failing_score_labels: Option<Vec<FailingScoreLabels>>,
    pub antithesis: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GradedCompetency {
    pub competency_id: Uuid,
    pub section_id: Uuid,
    pub achieved_scores: Vec<i32>,
    pub achieved_score_labels: Vec<AchievedScoreLabel>,
    pub ungraded_compentency: UngradedCompetency
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
fn validate_score_labels(competencies: &[UngradedCompetency], scoring_categories: &[UngradedScoringCategory], test_name: &String) -> Result<(), String> {
    
    // Check to ensure that each item has one list of scores per header label.
    let expected_number_of_scores_lists = scoring_categories.len();
    for item in competencies {
        if item.scores.len() != expected_number_of_scores_lists {
            return Err(format!(
                "On the test named '{},' graded item '{}' has a number of lists of scores ({}) that does not correspond to the number of scoring categories. ({})",
                test_name, item.name, item.scores.len(), scoring_categories.len()
            ))
        }
    }

    // Check to ensure that each item's list of scores is the same length as the corresponding list of header labels.
    for (i, score_label) in scoring_categories.into_iter().enumerate() {
        let expected_number_of_scores = score_label.values.len();
        for item in competencies {
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
fn validate_failing_score_labels(competencies: &[UngradedCompetency], scoring_categories: &[UngradedScoringCategory], test_name: &String) -> Result<(), String> {

    // Create a hashmap of the header labels so that we can correspond failing score labels on the graded item to the true header labels
    let mut score_label_hm: HashMap<String, Vec<String>> = HashMap::new();
    for score_label in scoring_categories {
        if let Some(duplicate_name) = score_label_hm.insert(score_label.name.clone(), score_label.values.clone()) {
            return Err(format!(
                "On the test named '{},' the scoring category name '{:#?}' is not unique within its section.",
                test_name, duplicate_name
            ))
        };
    }

    for item in competencies {
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
fn validate_antitheses(competencies: &[UngradedCompetency], test_name: &String) -> Result<(), String> {
    for item in competencies {
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



#[derive(Debug, Clone, sqlx::FromRow, Serialize, Deserialize)]
pub struct Testee {
    pub id: Option<Uuid>,  
    pub first_name: String,
    pub last_name: String,
    pub email: String,
}

#[derive(Debug, Serialize, Deserialize)]
/// Passing may be failed even if the achieved percent is above the minimum percent if a competency with a failing score label was graded as failing. 
pub struct FullTestSummary {
    pub test_id: Uuid,
    pub test_date: NaiveDateTime,
    pub test_name: String,  // This probably should've been labeled test_type, but I'm lazy here...
    pub proctor: Proctor,
    pub grade_summary: TestGradeSummary,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
/// Passing may be failed even if the achieved percent is above the minimum percent if a competency with a failing score label was graded as failing. 
pub struct TestGradeSummary {
    pub achieved_score: i32,
    pub achieved_percent: f32,
    pub max_score: i32,
    pub minimum_percent: f32,
    pub is_passing: bool,
    pub failure_explanation: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Proctor {
    pub id: Uuid,
    pub first_name: String,
    pub last_name: String,
}




