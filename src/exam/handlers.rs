use chrono::{Local, NaiveDateTime};
use sqlx::{query, Error, PgPool};
use strum_macros::Display;
use std::{collections::HashMap, fmt::Display};
use crate::exam::models::{
    BonusPointName, GradedBonusPoint, GradedPattern, GradedTechnique, GradedTest, PatternName, ScoringCategoryName, TechniqueName, TechniqueScoringHeaderName, TestSummary, TestType, Testee
};

use super::models::TestTemplate;

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

// -------------------------------------------------------------------------------------------------------------------------------------------------------
// Parse Test
// -------------------------------------------------------------------------------------------------------------------------------------------------------

/// Parses the test form data, which should have a format of a hashmap more or less like this
///
/// first_name: Corrado
/// last_name: Mazzarelli
/// email: corrado@mazzarelli.biz
/// pattern--starter_step--scoring_category--footwork--max_score--3: 3
/// pattern--starter_step--scoring_category--timing--max_score--1: 0
/// pattern--left_side_pass_from_closed--scoring_category--footwork--max_score--3: 2
/// pattern--left_side_pass_from_closed--scoring_category--timing--max_score--1: 0
/// pattern--sugar_tuck--scoring_category--footwork--max_score--3: 1
/// pattern--sugar_tuck--scoring_category--timing--max_score--1: 0
/// pattern--cutoff_whip--scoring_category--footwork--max_score--3: 0
/// pattern--cutoff_whip--scoring_category--timing--max_score--1: 1
/// pattern--left_side_pass--scoring_category--footwork--max_score--3: 0
/// pattern--left_side_pass--scoring_category--timing--max_score--1: 1
/// pattern--whip--scoring_category--footwork--max_score--3: 0
/// pattern--whip--scoring_category--timing--max_score--1: 1
/// pattern--sugar_push--scoring_category--footwork--max_score--3: 0
/// pattern--sugar_push--scoring_category--timing--max_score--1: 1
/// pattern--spinning_side_pass--scoring_category--footwork--max_score--3: 0
/// pattern--spinning_side_pass--scoring_category--timing--max_score--1: 1
/// pattern--right_side_pass--scoring_category--footwork--max_score--3: 1
/// pattern--right_side_pass--scoring_category--timing--max_score--1: 0
/// pattern--basket_whip--scoring_category--footwork--max_score--3: 2
/// pattern--basket_whip--scoring_category--timing--max_score--1: 0
/// pattern--free_spin--scoring_category--footwork--max_score--3: 3
/// pattern--free_spin--scoring_category--timing--max_score--1: 0
/// technique--body_lead--max_score--8: Consistent >90%--8
/// technique--post--max_score--6: Present 75%--4
/// technique--strong_frame--max_score--6: Occasional 50%--2
/// technique--closed_connection--max_score--4: Lacking 25%--0
/// technique--connection_transition--max_score--4: Missing <10%--0
/// technique--on_time--max_score--8: Lacking 25%--0
/// technique--move_off_slot--max_score--4: Occasional 50%--0
/// technique--safe--max_score--8: Present 75%--0
/// technique--body_angle--max_score--3: Over-Angled--2
/// technique--prep--max_score--3: Overkill Prep--0
/// bonus--no_thumbs: 1
/// bonus--clear_turn_signal: 1
/// bonus--swung_triple: 4
/// 
pub fn parse_test_form_data(test: HashMap<String, String>, test_type: TestType, test_template: TestTemplate) -> GradedTest {
    let mut pattern_scores = Vec::new();
    let mut technique_scores = Vec::new();
    let mut bonus_scores = Vec::new();
    let mut user_info = HashMap::new();

    for (key, value) in test.iter() {
        if key.starts_with("pattern--") {
            // Extract pattern score
            let parts: Vec<&str> = key.split("--").collect();

            match parts.len() {
                6 => {
                    match (parts[1].parse::<PatternName>(), parts[3].parse::<ScoringCategoryName>(), parts[5].parse::<u32>()) {
                        (Ok(pattern), Ok(category), Ok(max_score)) => {
                            match value.parse::<u32>() {
                                Ok(score) => pattern_scores.push(GradedPattern {
                                    // id: None,
                                    // test_id: None,
                                    pattern: pattern,
                                    category: category,
                                    score: score,
                                    max_score: max_score,
                                }),
                                Err(_) => eprintln!("Failed to parse score from value '{}'", value),
                            }
                        }
                        (Err(e), _, _) => eprintln!("Failed to parse pattern name from key '{}': {:?}", key, e),
                        (_, Err(e), _) => eprintln!("Failed to parse category name from key '{}': {:?}", key, e),
                        (_, _, Err(e)) => eprintln!("Failed to parse max_score from key '{}': {:?}", key, e),
                    }
                }
                _ => eprintln!("The key '{}' should be formatted as follows pattern--pattern_name--scoring_category--scoring_category_name--max_score--num", key),
            }
        } else if key.starts_with("technique--") {
            // Extract technique score
            let key_parts: Vec<&str> = key.split("--").collect();
            let value_parts: Vec<&str> = value.split("--").collect();

            match (key_parts.len(), value_parts.len()) {
                (4, 2) => {
                    match (key_parts[1].parse::<TechniqueName>(), key_parts[3].parse::<u32>(), value_parts[0].parse::<TechniqueScoringHeaderName>(), value_parts[1].parse::<u32>()) {
                        (Ok(technique), Ok(max_score), Ok(score_header), Ok(score)) => {
                            technique_scores.push(GradedTechnique {
                                // id: None,
                                // test_id: None,
                                technique: technique,
                                score_header:score_header,
                                score: score,
                                max_score: max_score
                            });
                        }
                        (Err(e), _,  _, _) => eprintln!("Failed to parse technique name from key '{}': {:?}", key, e),
                        (_, Err(e), _,  _) => eprintln!("Failed to parse max_score from key '{}': {:?}", key, e),
                        (_, _, Err(e), _) => eprintln!("Failed to parse scoring header from value '{}': {:?}", value, e),
                        (_, _, _, Err(e)) => eprintln!("Failed to parse score from value '{}': {:?}", value, e),
                    }
                }
                _ => eprintln!("The key '{}' and value '{}' should be formatted as follows technique--technique_name--max_score--num: technique_scoring_header--score", key, value),
            }
        } else if key.starts_with("bonus--") {
            // Extract bonus score
            let parts: Vec<&str> = key.split("--").collect();

            match parts.len() {
                2 => {
                    match (parts[1].parse::<BonusPointName>(), value.parse::<u32>()) {
                        (Ok(name), Ok(score)) => {
                            bonus_scores.push(GradedBonusPoint {
                                // id: None,
                                // test_id: None,
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
            id: -1,
            first_name,
            last_name,
            email,
        },
        // TODO: This should probably be refactored to propagate an error
        _ => {
            eprintln!("Missing user information. Please ensure 'first_name', 'last_name', and 'email' are provided.");
            Testee {
                id: -1,
                first_name: String::new(),
                last_name: String::new(),
                email: String::new(),
            }
        }
    };

    GradedTest::new(
        testee,
        Local::now().naive_utc(),
        test_type,
        test_template.passing_score,
        pattern_scores,
        technique_scores,
        bonus_scores,
    )
    
}

// -------------------------------------------------------------------------------------------------------------------------------------------------------
// Save Test to Database
// -------------------------------------------------------------------------------------------------------------------------------------------------------


pub async fn save_test_to_database(
    pool: &PgPool,
    graded_test: GradedTest
) -> Result<(), TestError> {
    // Insert the testee or get the testee ID if the testee already exists
    let testee = create_testee(
        pool, 
        &graded_test.testee.first_name, 
        &graded_test.testee.last_name,
        &graded_test.testee.email,
    ).await?;

    // Insert a new test record
    let test_id = match sqlx::query!(
        "INSERT INTO tests (testee_id, role, test_date, score, max_score, passing_score)
        VALUES ($1, $2, $3, $4, $5, $6)
        RETURNING id",
        testee.id,
        graded_test.test_type.to_string(),
        graded_test.test_date,
        graded_test.score as i32,
        graded_test.max_score as i32,
        graded_test.passing_score as i32,
    )
    .fetch_one(pool)
    .await {
        Ok(record) => record.id,
        Err(e) => return Err(e.into()),
    };

    // Insert pattern scores
    for pattern in graded_test.patterns {
        sqlx::query!(
            "INSERT INTO patterns (test_id, pattern, category, score, max_score)
            VALUES ($1, $2, $3, $4, $5)",
            test_id,
            pattern.pattern.to_string(),
            pattern.category.to_string(),
            pattern.score as i32,
            pattern.max_score as i32,
        )
        .execute(pool)
        .await?;
    }

    // Insert technique scores
    for technique in graded_test.techniques {
        sqlx::query!(
            "INSERT INTO techniques (test_id, technique, score, score_header, max_score)
            VALUES ($1, $2, $3, $4, $5)",
            test_id,
            technique.technique.to_string(),
            technique.score as i32,
            technique.score_header.to_string(),
            technique.max_score as i32,
        )
        .execute(pool)
        .await?;
    }

    // Insert bonus scores
    for bonus in graded_test.bonuses {
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

/// This function parses test results from the DB into GradedPattern, GradedTechnique, and GradedBonus objects
/// Thus, old pattern names, technique names, and other enum variants CANNOT be retired so long 
/// as they may exist within the database. 
/// This is where some refactoring work could be done. The parsing into the enum types is fragile since
/// I manually defined the to_str and serialize traits using strum. Some of them require snake case,
/// others don't.
pub async fn fetch_test_results_by_id(
    pool: &PgPool, 
    test_id: i32
) -> Result<Option<GradedTest>, TestError> {
    // Grab the testee corresponding to a given test
    let testee = match sqlx::query_as!(
        Testee,
        "SELECT id, first_name, last_name, email FROM testees WHERE id = (
            SELECT testee_id FROM tests WHERE id = $1
        )",
        test_id
    )
    .fetch_optional(pool)
    .await? {
        Some(testee) => testee,
        None => return Ok(None)
    };

    let raw_test_info = sqlx::query!(
        "SELECT role, test_date, score, max_score, passing_score FROM tests WHERE id = $1",
        test_id
    )
    .fetch_one(pool)
    .await?;

    // println!("{}", raw_test_info.role);

    let test_type = raw_test_info.role.parse::<TestType>()?;

    let raw_patterns = sqlx::query!(
        "SELECT pattern, category, score, max_score FROM patterns WHERE test_id = $1",
        test_id
    )
    .fetch_all(pool)
    .await?;

    // println!("{}", raw_patterns[0].pattern.to_lowercase().replace(" ", "_"));
    // println!("{}", raw_patterns[0].category.to_lowercase().replace(" ", "_"));

    let patterns: Vec<GradedPattern> = raw_patterns
    .into_iter()
    .map(|raw_pattern|
        Ok(GradedPattern {
            pattern: raw_pattern.pattern.to_lowercase().replace(" ", "_").parse::<PatternName>()?,
            category: raw_pattern.category.to_lowercase().replace(" ", "_").parse::<ScoringCategoryName>()?,
            score: raw_pattern.score as u32,
            max_score: raw_pattern.max_score as u32,
        })
    ).collect::<Result<Vec<GradedPattern>, strum::ParseError>>()?;

    let raw_techniques = sqlx::query!(
        "SELECT technique, score, score_header, max_score FROM techniques WHERE test_id = $1",
        test_id
    )
    .fetch_all(pool)
    .await?;

    // println!("{}", raw_techniques[0].technique.to_lowercase().replace(" ", "_"));
    // println!("{}", raw_techniques[0].score_header);

    let techniques: Vec<GradedTechnique> = raw_techniques
    .into_iter()
    .map(|raw_technique| 
        Ok(GradedTechnique {
            technique: raw_technique.technique.to_lowercase().replace(" ", "_").parse::<TechniqueName>()?,
            score: raw_technique.score as u32,
            max_score: raw_technique.max_score as u32,
            score_header: raw_technique.score_header.parse::<TechniqueScoringHeaderName>()?,
        })
    ).collect::<Result<Vec<GradedTechnique>, strum::ParseError>>()?;

    let raw_bonuses = sqlx::query!(
        "SELECT name, score FROM bonus_points WHERE test_id = $1",
        test_id
    )
    .fetch_all(pool)
    .await?;

    // println!("{}", raw_bonuses[0].name.to_lowercase().replace(" ", "_"));


    let bonuses: Vec<GradedBonusPoint> = raw_bonuses
    .into_iter()
    .map(|raw_bonus| 
        Ok(GradedBonusPoint {
            name: raw_bonus.name.to_lowercase().replace(" ", "_").parse::<BonusPointName>()?,
            score: raw_bonus.score as u32,
        })
    ).collect::<Result<Vec<GradedBonusPoint>, strum::ParseError>>()?;

    Ok(Some(GradedTest {
        testee: testee,
        test_date: raw_test_info.test_date,
        test_type: test_type,
        patterns: patterns,
        techniques: techniques,
        bonuses: bonuses,
        score: raw_test_info.score as u32,
        max_score: raw_test_info.max_score as u32,
        passing_score: raw_test_info.passing_score as u32,
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
) -> Result<Option<Vec<TestSummary>>, TestError> {

    let testee = match fetch_testee_by_id(pool, testee_id).await? {
        Some(data) => data,
        None => return Ok(None),
    };

    // By virtue of there being a testee, they have taken at least one test.
    Ok(Some(sqlx::query_as!(
        TestSummary,
        "
        SELECT id, role, test_date, score, passing_score, max_score 
        FROM tests 
        WHERE testee_id = $1
        ORDER BY test_date DESC
        ",
        testee.id
    )
    .fetch_all(pool)
    .await?))

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

pub async fn enqueue_testee(pool: &PgPool, testee_id: i32, role: &str) -> Result<(), TestError> {
    sqlx::query!(
        "INSERT INTO queue (testee_id, role)
        VALUES ($1, $2)
        ON CONFLICT (testee_id, role) DO NOTHING",
        testee_id,
        role
    )
    .execute(pool)
    .await
    .map_err(TestError::from)?;

    Ok(())
}

// -------------------------------------------------------------------------------------------------------------------------------------------------------
// Dequeue Testee 
// -------------------------------------------------------------------------------------------------------------------------------------------------------

/// Remove and return the next person on the queue plus the role of their desired test
/// If a testee_id is given, remove that person, (or throw an error if not found)
pub async fn dequeue_testee(
    pool: &PgPool,
    testee_id: Option<i32>,
) -> Result<Option<(Testee, String)>, TestError> {
    // Determine which testee_id to use: provided one or first in the queue
    let (testee_id, role) = if let Some(id) = testee_id {
        // If a testee_id is provided, use it
        match sqlx::query!(
            "DELETE FROM queue WHERE testee_id = $1 RETURNING role",
            id
        )
        .fetch_optional(pool)
        .await? {
        Some(role) => (id, role.role),
        None => return Ok(None)    
        }
    } else {
        // Otherwise, dequeue the first person in the queue
        match sqlx::query!(
            "DELETE FROM queue WHERE testee_id = (
                SELECT testee_id FROM queue ORDER BY added_at LIMIT 1
            ) RETURNING testee_id, role"
        )
        .fetch_optional(pool)
        .await? {
        Some(result ) => (result.testee_id, result.role),
        None => return Ok(None)
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

    Ok(Some((testee, role)))
}

// -------------------------------------------------------------------------------------------------------------------------------------------------------
// Get Queue
// -------------------------------------------------------------------------------------------------------------------------------------------------------

pub async fn retrieve_queue(pool: &PgPool) -> Result<Vec<(Testee, String)>, TestError> {
    let rows = sqlx::query!(
        "
        SELECT 
            t.id, t.first_name, t.last_name, t.email, q.role 
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

    // Map the result into a Vec of (Testee, role) tuples
    let queue = rows.into_iter().map(|row| {
        (
            Testee {
                id: row.id,
                first_name: row.first_name,
                last_name: row.last_name,
                email: row.email,
            },
            row.role,
        )
    }).collect();

    Ok(queue)
}