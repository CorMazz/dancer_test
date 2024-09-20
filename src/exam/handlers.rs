use chrono::Local;
use sqlx::{Error, PgPool};
use std::{collections::HashMap, fs::File, io::Read};
use crate::exam::models::{
    AchievedScoreLabel, BonusItem, Competency, FailingScoreLabels, Metadata, ScoringCategory, Test, TestDefinitionYaml, TestSection, FullTestSummary, TestTable, Testee, TestGradeSummary, TestConfig
};





// -------------------------------------------------------------------------------------------------------------------------------------------------------
// Custom Error Enum
// -------------------------------------------------------------------------------------------------------------------------------------------------------

#[derive(Debug)]
pub enum TestError {
    InternalServerError(String)
}

impl From<sqlx::Error> for TestError {
    fn from(error: sqlx::Error) -> Self {
        TestError::InternalServerError(error.to_string())
    }
}

impl From<strum::ParseError> for TestError {
    fn from(error: strum::ParseError) -> Self {
        TestError::InternalServerError(error.to_string())
    }
}

impl From<serde_json::Error> for TestError {
    fn from(error: serde_json::Error) -> Self {
        TestError::InternalServerError(error.to_string())
    }
}


// -------------------------------------------------------------------------------------------------------------------------------------------------------
// Parse Test from YAML
// -------------------------------------------------------------------------------------------------------------------------------------------------------

/// Read the test definition from a .yaml file
pub fn parse_test_definition(file_path: &str) -> Result<TestDefinitionYaml, serde_yaml::Error> {
    let mut file = File::open(file_path).expect(&format!("couldn't open file: {}", file_path));
    let mut yaml_string = String::new();
    file.read_to_string(&mut yaml_string).expect(&format!("Couldn't read file '{}' to string. This should work...", file_path));
    serde_yaml::from_str(&yaml_string)
}


// -------------------------------------------------------------------------------------------------------------------------------------------------------
// Parse Test
// -------------------------------------------------------------------------------------------------------------------------------------------------------

/// Parses the test form data, which should have a format of a hashmap more or less like this
/// 
/// Takes in a test_template which it will then mutate, adding the results so that it is graded.
pub fn parse_test_form_data(test: HashMap<String, String>, mut test_template: Test) -> Result<Test, TestError> {

    let mut user_info = HashMap::new();

    // Sort the keys so that the graded test gets reconstructed in the same order as the test definition
    let mut sorted_keys: Vec<&String> = test.keys().collect();
    sorted_keys.sort(); 

    for key in sorted_keys {
        let value = &test[key];

        // Build the hash map with all of the graded items
        if key.starts_with("table_index") {
            let key_parts: Vec<&str> = key.split("---").collect();
            let value_parts: Vec<&str> = value.split("---").collect();

            match (key_parts.len(), value_parts.len()) {
                (8, 4) => {
                    match (
                        key_parts[1].parse::<usize>(), 
                        key_parts[3].parse::<usize>(), 
                        key_parts[5].parse::<usize>(), 
                        key_parts[7].parse::<usize>(), 
                        value_parts[1].parse::<usize>(), 
                        value_parts[3].parse::<i32>()
                    ) {
                        (Ok(table_index), Ok(section_index), Ok(item_index), Ok(scoring_category_index), Ok(scoring_category_label_index), Ok(points)) => {

                            let scoring_category_name = test_template.tables[table_index]
                                .sections[section_index]
                                .scoring_categories[scoring_category_index].name.clone();

                            let label = test_template.tables[table_index]
                                .sections[section_index]
                                .scoring_categories[scoring_category_index]
                                .values[scoring_category_label_index]
                                .clone();

                            if let Some(item) = test_template.tables[table_index]
                            .sections[section_index]
                            .competencies
                            .get_mut(item_index)
                        {
                            item.achieved_scores.get_or_insert_with(Vec::new).push(points);
                                               
                            item.achieved_score_labels
                                .get_or_insert_with(Vec::new)
                                .push(AchievedScoreLabel {
                                     scoring_category_name,
                                     value: label, 
                                    });
                        }
                        },

                        (Err(e), _, _, _, _, _) => eprintln!("Failed to parse table index key '{}': {:?}", key, e),
                        (_, Err(e), _, _, _, _) => eprintln!("Failed to parse section index from key '{}': {:?}", key, e),
                        (_, _, Err(e), _, _, _) => eprintln!("Failed to parse item index from key'{}': {:?}", key, e),
                        (_, _, _, Err(e), _, _) => eprintln!("Failed to parse scoring category index from key '{}': {:?}", key, e),
                        (_, _, _, _, Err(e), _) => eprintln!("Failed to parse scoring category label index from value '{}': {:?}", value, e),
                        (_, _, _, _, _, Err(e)) => eprintln!("Failed to parse score from value '{}': {:?}", value, e),
                    }
                }
                _ => eprintln!("The key '{}' and value '{}' should be formatted as follows 'table_index---0---section_index---0---item_index---0---scoring_category_index---1': 'scoring_category_value_index---0---points---1'", key, value),
            }
        } else if key.starts_with("bonus_index") {
            if let Some(bonus_items) = &mut test_template.bonus_items {
                let key_parts: Vec<&str> = key.split("---").collect();
                match key_parts.len() {
                    2 => {
                        match (key_parts[1].parse::<usize>(), value.parse::<i64>()) {
                            (Ok(bonus_index), Ok(_)) => {
                                bonus_items[bonus_index].achieved.insert(true);
                            },
                            (Err(e), _) => eprintln!("Failed to parse bonus index from key '{}': {:?}", key, e),
                            (_, Err(e)) => eprintln!("Failed to parse points from value '{}': {:?}", value, e),
                        }
                    }
                    _ => eprintln!("The key '{}' should be formatted as 'bonus_index---<index>', but got '{}'", key, key),
                }
            }
        } else {
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
        _ => {
           return Err(TestError::InternalServerError("Missing user information. Please ensure 'first_name', 'last_name', and 'email' are provided.".to_string()));
        }
    };

    // Assign the testee
    test_template.metadata.testee = Some(testee);

    // Grade the test
    if let Err(e) = test_template.grade() {
        return Err(TestError::InternalServerError(e)); // Return the error
    }

    println!("Graded Test:\n{:#?}", test_template);
    Ok(test_template)

}

//         if key.starts_with("pattern--") {
//             // Extract pattern score
//             let parts: Vec<&str> = key.split("--").collect();

//             match parts.len() {
//                 6 => {
//                     match (parts[1].parse::<PatternName>(), parts[3].parse::<ScoringCategoryName>(), parts[5].parse::<u32>()) {
//                         (Ok(pattern), Ok(category), Ok(max_score)) => {
//                             match value.parse::<u32>() {
//                                 Ok(score) => pattern_scores.push(GradedPattern {
//                                     // id: None,
//                                     // test_id: None,
//                                     pattern: pattern,
//                                     category: category,
//                                     score: score,
//                                     max_score: max_score,
//                                 }),
//                                 Err(_) => eprintln!("Failed to parse score from value '{}'", value),
//                             }
//                         } 
//                         (Err(e), _, _) => eprintln!("Failed to parse pattern name from key '{}': {:?}", key, e),
//                         (_, Err(e), _) => eprintln!("Failed to parse category name from key '{}': {:?}", key, e),
//                         (_, _, Err(e)) => eprintln!("Failed to parse max_score from key '{}': {:?}", key, e),
//                     }
//                 }
//                 _ => eprintln!("The key '{}' should be formatted as follows pattern--pattern_name--scoring_category--scoring_category_name--max_score--num", key),
//             }
//         } else if key.starts_with("technique--") {
//             // Extract technique score
//             let key_parts: Vec<&str> = key.split("--").collect();
//             let value_parts: Vec<&str> = value.split("--").collect();

//             match (key_parts.len(), value_parts.len()) {
//                 (4, 2) => {
//                     match (key_parts[1].parse::<TechniqueName>(), key_parts[3].parse::<u32>(), value_parts[0].parse::<TechniqueScoringHeaderName>(), value_parts[1].parse::<u32>()) {
//                         (Ok(technique), Ok(max_score), Ok(score_header), Ok(score)) => {
//                             technique_scores.push(GradedTechnique {
//                                 // id: None,
//                                 // test_id: None,
//                                 technique: technique,
//                                 score_header:score_header,
//                                 score: score,
//                                 max_score: max_score
//                             });
//                         }
//                         (Err(e), _,  _, _) => eprintln!("Failed to parse technique name from key '{}': {:?}", key, e),
//                         (_, Err(e), _,  _) => eprintln!("Failed to parse max_score from key '{}': {:?}", key, e),
//                         (_, _, Err(e), _) => eprintln!("Failed to parse scoring header from value '{}': {:?}", value, e),
//                         (_, _, _, Err(e)) => eprintln!("Failed to parse score from value '{}': {:?}", value, e),
//                     }
//                 }
//                 _ => eprintln!("The key '{}' and value '{}' should be formatted as follows technique--technique_name--max_score--num: technique_scoring_header--score", key, value),
//             }
//         } else if key.starts_with("bonus--") {
//             // Extract bonus score
//             let parts: Vec<&str> = key.split("--").collect();

//             match parts.len() {
//                 2 => {
//                     match (parts[1].parse::<BonusPointName>(), value.parse::<u32>()) {
//                         (Ok(name), Ok(score)) => {
//                             bonus_scores.push(GradedBonusPoint {
//                                 // id: None,
//                                 // test_id: None,
//                                 name,
//                                 score,
//                             });
//                         }
//                         (Err(e), _) => eprintln!("Failed to parse bonus point name from key '{}': {:?}", key, e),
//                         (_, Err(e)) => eprintln!("Failed to parse bonus score from value '{}': {}", value, e),
//                     }
//                 }
//                 _ => eprintln!("The key '{}' should be formatted as follows bonus--bonus_point_name: score", key),
//             }
//         } else {
//             // Assume user info
//             user_info.insert(key.clone(), value.clone());
//         }
//     }

//     // Construct the GradedTestee instance from the user_info hashmap
//     let testee: Testee = match (
//         user_info.get("first_name").cloned(),
//         user_info.get("last_name").cloned(),
//         user_info.get("email").cloned()
//     ) {
//         (Some(first_name), Some(last_name), Some(email)) => Testee {
//             id: -1,
//             first_name,
//             last_name,
//             email,
//         },
//         // TODO: This should probably be refactored to propagate an error
//         _ => {
//             eprintln!("Missing user information. Please ensure 'first_name', 'last_name', and 'email' are provided.");
//             Testee {
//                 id: -1,
//                 first_name: String::new(),
//                 last_name: String::new(),
//                 email: String::new(),
//             }
//         }
//     };

//     GradedTest::new(
//         testee,
//         Local::now().naive_utc(),
//         test_type,
//         test_template.passing_score,
//         pattern_scores,
//         technique_scores,
//         bonus_scores,
//     )
    
// }

// -------------------------------------------------------------------------------------------------------------------------------------------------------
// Save Test to Database
// -------------------------------------------------------------------------------------------------------------------------------------------------------

/// Assumes that the graded_test has metadata with a testee object or the code panics.
pub async fn save_test_to_database(
    pool: &PgPool,
    graded_test: Test,
) -> Result<(), TestError> {
    // Insert the testee or get the testee ID if the testee already exists
    let testee = create_testee(
        pool, 
        &graded_test.metadata.testee.clone().expect("If this error was thrown, the invariant in the docstring of save_test_to_database was violated.").first_name, 
        &graded_test.metadata.testee.clone().expect("If this error was thrown, the invariant in the docstring of save_test_to_database was violated.").last_name,
        &graded_test.metadata.testee.clone().expect("If this error was thrown, the invariant in the docstring of save_test_to_database was violated.").email,
    ).await?;

    // Insert a new test record
    let test_id = sqlx::query!(
        "INSERT INTO tests DEFAULT VALUES RETURNING id"
    )
    .fetch_one(pool)
    .await?
    .id;

    // Insert test metadata
    sqlx::query!(
        "INSERT INTO test_metadata (test_id, test_name, minimum_percent, max_score, achieved_score, testee_id, test_date, is_passing, failure_explanation)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)",
        test_id,
        graded_test.metadata.test_name,
        graded_test.metadata.minimum_percent,
        graded_test.metadata.max_score,
        graded_test.metadata.achieved_score,
        testee.id,
        Local::now().naive_utc(),
        graded_test.metadata.is_passing,
        graded_test.metadata.failure_explanation.as_deref()
    ).execute(pool)
    .await?;

    // Insert test tables, sections, scoring categories, and competencies
    for table in graded_test.tables {
        let table_id = sqlx::query!(
            "INSERT INTO test_tables (test_id)
            VALUES ($1)
            RETURNING (id)",
            test_id
        ).fetch_one(pool)
        .await?
        .id;

        for section in table.sections {
            let section_id = sqlx::query!(
                "INSERT INTO test_sections (table_id, name)
                VALUES ($1, $2)
                RETURNING (id)",
                table_id,
                section.name
            ).fetch_one(pool)
            .await?
            .id;

            for scoring_category in section.scoring_categories {
                sqlx::query!(
                    "INSERT INTO scoring_categories (section_id, name, values)
                    VALUES ($1, $2, $3)",
                    section_id,
                    &scoring_category.name,
                    &scoring_category.values,
                ).execute(pool)
                .await?;
            };

            for competency in section.competencies {
                sqlx::query!(
                    "INSERT INTO competencies (section_id, name, scores, subtext, antithesis, achieved_scores, achieved_score_labels, failing_score_labels)
                    VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
                section_id,
                &competency.name,
                &serde_json::to_value(&competency.scores)?, // Convert Vec<Vec<i64>> to JSON
                competency.subtext.as_deref(),
                competency.antithesis.as_deref(),
                &serde_json::to_value(competency.achieved_scores)?, // Convert Option<Vec<i64>> to JSON
                &serde_json::to_value(competency.achieved_score_labels)?,// Convert Option<Vec<String>> to JSON
                &serde_json::to_value(&competency.failing_score_labels)?, // Convert Option<Vec<FailingScoreLabels>> to JSON
                ).execute(pool)
                .await?;
            };
        };
    };

    if let Some(bonus_items) = graded_test.bonus_items {
        for bonus in bonus_items {
            sqlx::query!(
                "INSERT INTO bonus_items (test_id, name, score, achieved)
                VALUES ($1, $2, $3, $4)",
                test_id,
                bonus.name.to_string(),
                bonus.score,
                bonus.achieved.unwrap_or(false)
            )
            .execute(pool)
            .await?;
        }
    };


    Ok(())
}


// -------------------------------------------------------------------------------------------------------------------------------------------------------
// Search for Testee
// -------------------------------------------------------------------------------------------------------------------------------------------------------

/// Searches for a testee by matching the query string to the first name, last name, or email 
/// using trigram similarity metrics. Returns at most 50 results. 
pub async fn search_for_testee(
    query: String,
    pool: &PgPool,
) -> Result<Option<Vec<Testee>>, Error> {

    let results = sqlx::query_as::<_, Testee>(
        "
        SELECT id, first_name, last_name, email
        FROM testees
        WHERE first_name % $1
           OR last_name % $1
           OR email % $1
        ORDER BY
           GREATEST(similarity(first_name, $1),
                    similarity(last_name, $1),
                    similarity(email, $1)) DESC
        LIMIT 50
        "
    )
    .bind(query)
    .fetch_all(pool)
    .await;

    match results {
        Ok(data) => return Ok(Some(data)),
        Err(e) => match e {
            sqlx::Error::RowNotFound => return Ok(None),
            _ => return Err(e)
        }
    }

}

// -------------------------------------------------------------------------------------------------------------------------------------------------------
// Fetch Test Results
// -------------------------------------------------------------------------------------------------------------------------------------------------------

pub async fn fetch_test_results_by_id(pool: &PgPool, test_id: i32) -> Result<Option<Test>, TestError> {
    // Fetch test metadata
    let raw_metadata = match sqlx::query!(
        r#"
        SELECT test_id, test_name, minimum_percent, max_score, achieved_score, testee_id, test_date, is_passing, failure_explanation
        FROM test_metadata
        WHERE test_id = $1
        "#,
        test_id
    )
    .fetch_optional(pool)
    .await? {
        Some(data) => data,
        None => return Ok(None)
    };

    // Fetch the testee
    let testee = sqlx::query_as!(
    Testee,
    "SELECT id, first_name, last_name, email FROM testees WHERE id = $1",
    raw_metadata.testee_id
    )
    .fetch_one(pool)
    .await?;

    // Create the metadata object
    let metadata: Metadata = Metadata {
        test_id: Some(test_id),
        test_name: raw_metadata.test_name,
        minimum_percent: raw_metadata.minimum_percent,
        max_score: raw_metadata.max_score,
        achieved_score: Some(raw_metadata.achieved_score),
        testee: Some(testee),
        test_date: Some(raw_metadata.test_date),
        is_graded: Some(()),
        is_passing: Some(raw_metadata.is_passing),
        failure_explanation: raw_metadata.failure_explanation,
        config_settings: TestConfig {live_grading: true, show_point_values:true}, // must be true to show results on graded test page
    };

    // Fetch test tables
    let table_ids: Vec<i32> = sqlx::query!(
        "SELECT id FROM test_tables WHERE test_id = $1
        ORDER BY id ASC",
        test_id
    )
    .fetch_all(pool)
    .await?
    .into_iter()
    .map(|record| record.id)
    .collect();

    let mut test_tables: Vec<TestTable> = Vec::new();

    for table_id in table_ids {
        // Fetch sections for each table
        let sections = sqlx::query!(
            "SELECT id, name FROM test_sections WHERE table_id = $1
            ORDER BY id ASC",
            table_id
        )
        .fetch_all(pool)
        .await?;

        let mut test_sections: Vec<TestSection> = Vec::new();

        for section in sections {
            // Fetch scoring categories for each section
            let scoring_categories = sqlx::query!(
                r#"
                SELECT id, section_id, name, values
                FROM scoring_categories
                WHERE section_id = $1
                ORDER BY id ASC
                "#,
                section.id
            )
            .fetch_all(pool)
            .await?
            .into_iter()
            .map(|record| ScoringCategory {
                section_id: Some(section.id),
                name: record.name,
                values: record.values
            })
            .collect();

            // Fetch competencies for each section
            let raw_competencies = sqlx::query!(
                r#"
                SELECT id, section_id, name, scores, subtext, antithesis, achieved_scores, achieved_score_labels, failing_score_labels
                FROM competencies
                WHERE section_id = $1
                ORDER BY id ASC
                "#,
                section.id
            )
            .fetch_all(pool)
            .await?;

            let mut competency_vec: Vec<Competency> = Vec::new();
            for raw_competency in raw_competencies {
                let scores: Vec<Vec<i32>> = serde_json::from_value(raw_competency.scores)?;
                let achieved_scores: Option<Vec<i32>> = serde_json::from_value(raw_competency.achieved_scores)?;
                let achieved_score_labels: Option<Vec<AchievedScoreLabel>> = serde_json::from_value(raw_competency.achieved_score_labels)?;
                let failing_score_labels: Option<Vec<FailingScoreLabels>> = serde_json::from_value(raw_competency.failing_score_labels)?;

                competency_vec.push(Competency {
                    section_id: Some(raw_competency.section_id),
                    name: raw_competency.name,
                    scores,
                    subtext: raw_competency.subtext,
                    antithesis: raw_competency.antithesis,
                    achieved_scores,
                    achieved_score_labels,
                    failing_score_labels,
                });
            }

            test_sections.push(TestSection {
                table_id: Some(table_id),
                name: section.name,
                scoring_categories,
                competencies: competency_vec,
            });
        }

        test_tables.push(TestTable {
            test_id: Some(test_id),
            table_id: Some(table_id),
            sections: test_sections,
        });
    }

    // Fetch bonus items
    let bonus_items = sqlx::query!(
        "SELECT id, test_id, name, score, achieved FROM bonus_items WHERE test_id = $1
        ORDER BY id ASC",
        test_id
    )
    .fetch_all(pool)
    .await?;

    let bonus_items = bonus_items
        .into_iter()
        .map(|bonus| BonusItem {
            test_id: Some(test_id),
            name: bonus.name,
            score: bonus.score,
            achieved: Some(bonus.achieved),
        })
        .collect::<Vec<_>>();

    // Construct the Test object
    Ok(Some(Test {
        metadata,
        tables: test_tables,
        bonus_items: (!bonus_items.is_empty()).then_some(bonus_items),
    }))
}


// -------------------------------------------------------------------------------------------------------------------------------------------------------
// Fetch Testee Tests
// -------------------------------------------------------------------------------------------------------------------------------------------------------

/// Grab all the tests that a given testee has taken, by their testee id. 
/// By virtue of there being a testee, they have taken at least one test.
pub async fn fetch_testee_tests_by_id(
    pool: &PgPool, 
    testee_id: i32
) -> Result<Option<Vec<FullTestSummary>>, TestError> {

    let testee = match fetch_testee_by_id(pool, testee_id).await? {
        Some(data) => data,
        None => return Ok(None),
    };

    // By virtue of there being a testee, they have taken at least one test.
    Ok(Some(sqlx::query!(
        "
        SELECT test_id, test_name, test_date, achieved_score, minimum_percent, max_score, is_passing, failure_explanation 
        FROM test_metadata 
        WHERE testee_id = $1
        ORDER BY test_date DESC
        ",
        testee.id
    )
    .fetch_all(pool)
    .await?
    .into_iter()
    .map(|record| FullTestSummary {
        test_id: record.test_id, 
        test_date: record.test_date,
        test_name: record.test_name,
        grade_summary: TestGradeSummary {
            achieved_score: record.achieved_score,
            achieved_percent: record.achieved_score as f32 / record.max_score as f32,
            max_score: record.max_score,
            minimum_percent: record.minimum_percent,
            is_passing: record.is_passing,
            failure_explanation: record.failure_explanation
        }
    }
    ).collect()))

}

// -------------------------------------------------------------------------------------------------------------------------------------------------------
// Fetch Testee by ID
// -------------------------------------------------------------------------------------------------------------------------------------------------------

/// Fetch a testee from the testees table by their id
pub async fn fetch_testee_by_id(pool: &PgPool, testee_id: i32) -> Result<Option<Testee>, TestError> {
    let testee = sqlx::query_as!(
        Testee,
        "SELECT id, first_name, last_name, email FROM testees WHERE id = $1",
        testee_id
    )
    .fetch_optional(pool)
    .await?;
    
    Ok(testee)
}

// -------------------------------------------------------------------------------------------------------------------------------------------------------
// Create Testee
// -------------------------------------------------------------------------------------------------------------------------------------------------------

/// Returns a testee that 100% has an ID.
pub async fn create_testee(pool: &PgPool, first_name: &str, last_name: &str, email: &str) -> Result<Testee, TestError> {
    sqlx::query_as!(
        Testee,
        "INSERT INTO testees (first_name, last_name, email)
        VALUES ($1, $2, $3)
        ON CONFLICT (email) DO UPDATE SET first_name = $1, last_name = $2
        RETURNING id, first_name, last_name, email",
        first_name,
        last_name,
        email
    )
    .fetch_one(pool)
    .await
    .map_err(TestError::from)
}

// -------------------------------------------------------------------------------------------------------------------------------------------------------
// Enqueue Testee
// -------------------------------------------------------------------------------------------------------------------------------------------------------

pub async fn enqueue_testee(pool: &PgPool, testee_id: i32, test_definition_index: i32) -> Result<(), TestError> {
    sqlx::query!(
        "INSERT INTO queue (testee_id, test_definition_index)
        VALUES ($1, $2)
        ON CONFLICT (testee_id, test_definition_index) DO NOTHING",
        testee_id,
        test_definition_index,
    )
    .execute(pool)
    .await
    .map_err(TestError::from)?;

    Ok(())
}

// -------------------------------------------------------------------------------------------------------------------------------------------------------
// Dequeue Testee 
// -------------------------------------------------------------------------------------------------------------------------------------------------------

/// Remove and return the next person on the queue plus the test_definition_index of their desired test.
/// If a testee_id is given, remove that person, (or throw an error if not found)
pub async fn dequeue_testee(
    pool: &PgPool,
    testee_id: Option<i32>,
    test_definition_index: Option<i32>,
) -> Result<Option<(Testee, i32)>, TestError> {

    // Handle different cases based on the presence of testee_id and test_definition_index
    let (testee_id, test_definition_index) = match (testee_id, test_definition_index) {
        (Some(id), Some(r)) => {
            // Both testee_id and test_definition_index are provided; delete the specific entry
            match sqlx::query!(
                "DELETE FROM queue WHERE testee_id = $1 AND test_definition_index = $2 RETURNING test_definition_index",
                id, r
            )
            .fetch_optional(pool)
            .await? {
                Some(result) => (id, result.test_definition_index),
                None => return Ok(None),
            }
        }
        (Some(id), None) => {
            // Only testee_id is provided; delete the oldest entry for that testee_id
            match sqlx::query!(
                "DELETE FROM queue WHERE ctid = (
                    SELECT ctid FROM queue WHERE testee_id = $1 ORDER BY added_at LIMIT 1
                ) RETURNING test_definition_index",
                id
            )
            .fetch_optional(pool)
            .await? {
                Some(result) => (id, result.test_definition_index),
                None => return Ok(None),
            }
        }
        (None, None) => {
            // Neither testee_id nor test_definition_index is provided; delete the oldest queue item
            match sqlx::query!(
                "DELETE FROM queue WHERE ctid = (
                    SELECT ctid FROM queue ORDER BY added_at LIMIT 1
                ) RETURNING testee_id, test_definition_index"
            )
            .fetch_optional(pool)
            .await? {
                Some(result) => (result.testee_id, result.test_definition_index),
                None => return Ok(None),
            }
        }
        (None, Some(_)) => {
            // Only test_definition_index is provided, which is an invalid case
            return Err(TestError::InternalServerError("Test index specified without testee_id when trying to dequeue.".into()));
        }
    };

    // Fetch the testee details
    let testee = sqlx::query_as!(
        Testee,
        "SELECT id, first_name, last_name, email FROM testees WHERE id = $1",
        testee_id
    )
    .fetch_one(pool)
    .await?;

    Ok(Some((testee, test_definition_index)))
}

// -------------------------------------------------------------------------------------------------------------------------------------------------------
// Get Queue
// -------------------------------------------------------------------------------------------------------------------------------------------------------

pub async fn retrieve_queue(pool: &PgPool) -> Result<Vec<(Testee, i32)>, TestError> {
    let rows = sqlx::query!(
        "
        SELECT 
            t.id, t.first_name, t.last_name, t.email, q.test_definition_index
        FROM 
            queue q
        JOIN 
            testees t 
        ON 
            q.testee_id = t.id
        ORDER BY 
            q.added_at
        "
    )
    .fetch_all(pool)
    .await?;

    // Map the result into a Vec of (Testee, test_name) tuples
    let queue = rows.into_iter().map(|row| {
        (
            Testee {
                id: Some(row.id),
                first_name: row.first_name,
                last_name: row.last_name,
                email: row.email,
            },
            row.test_definition_index,
        )
    }).collect();

    Ok(queue)
}