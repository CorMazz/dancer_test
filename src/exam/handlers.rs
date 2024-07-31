use chrono::{Local, NaiveDateTime};
use sqlx::PgPool;
use std::collections::HashMap;
use crate::exam::models::{
    GradedPattern,
    GradedTechnique,
    GradedBonusPoint,
    Testee,
    PatternName,
    ScoringCategoryName,
    TechniqueName,
    TechniqueScoringHeaderName,
    BonusPointName,
    TestType,
    GradedTest,
};

// -------------------------------------------------------------------------------------------------------------------------------------------------------
// Custom Error Enum
// -------------------------------------------------------------------------------------------------------------------------------------------------------

#[derive(Debug)]
enum TestError {
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
pub fn parse_test_form_data(test: HashMap<String, String>, test_type: TestType) -> GradedTest {
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
                                    // id: None,
                                    // test_id: None,
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
                                // id: None,
                                // test_id: None,
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

    GradedTest {
        testee: testee,
        test_date: Local::now().naive_utc(),
        test_type: test_type,
        patterns: pattern_scores,
        techniques: technique_scores,
        bonuses: bonus_scores,
    }
    
}

// -------------------------------------------------------------------------------------------------------------------------------------------------------
// Save Test to Database
// -------------------------------------------------------------------------------------------------------------------------------------------------------


pub async fn save_test_to_database(
    pool: &PgPool,
    graded_test: GradedTest
) -> Result<(), TestError> {
    // Insert the testee or get the testee ID if the testee already exists
    let testee_id = match sqlx::query!(
        "INSERT INTO testees (first_name, last_name, email)
        VALUES ($1, $2, $3)
        ON CONFLICT (email) DO UPDATE SET first_name = $1, last_name = $2
        RETURNING id",
        graded_test.testee.first_name,
        graded_test.testee.last_name,
        graded_test.testee.email
    )
    .fetch_one(pool)
    .await {
        Ok(record) => record.id,
        Err(e) => return Err(e.into()),
    };


    // Insert a new test record
    let test_id = match sqlx::query!(
        "INSERT INTO tests (testee_id, role)
        VALUES ($1, $2)
        RETURNING id",
        testee_id,
        graded_test.test_type.to_string(),
    )
    .fetch_one(pool)
    .await {
        Ok(record) => record.id,
        Err(e) => return Err(e.into()),
    };

    // Insert pattern scores
    for pattern in graded_test.patterns {
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
    for technique in graded_test.techniques {
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
// Fetch Test Results
// -------------------------------------------------------------------------------------------------------------------------------------------------------

/// This function parses test results into GradedPattern, GradedTechnique, and GradedBonus objects
/// Thus, old pattern names, technique names, and other enum variants CANNOT be retired so long 
/// as they may exist within the database. 
async fn fetch_test_results(pool: &PgPool, test_id: i32) -> Result<GradedTest, TestError> {
    let testee = sqlx::query_as!(
        Testee,
        "SELECT * FROM testees WHERE id = (
            SELECT testee_id FROM tests WHERE id = $1
        )",
        test_id
    )
    .fetch_one(pool)
    .await?;

    let raw_test_info = sqlx::query!(
        "SELECT role, test_date FROM tests WHERE id = $1",
        test_id
    )
    .fetch_one(pool)
    .await?;

    let test_type = raw_test_info.role.parse::<TestType>()?;
    
    // Assuming the `test_date` field is returned as a string. Adjust parsing based on the actual data type.
    let test_date = raw_test_info.test_date
        .parse::<NaiveDateTime>()
        .map_err(|e| TestError::InternalServerError(e.to_string()))?;

    let raw_patterns = sqlx::query!(
        "SELECT pattern, category, score FROM patterns WHERE test_id = $1",
        test_id
    )
    .fetch_all(pool)
    .await?;

    let patterns: Vec<GradedPattern> = raw_patterns
    .into_iter()
    .map(|raw_pattern|
        Ok(GradedPattern {
            pattern: raw_pattern.pattern.parse::<PatternName>()?,
            category: raw_pattern.category.parse::<ScoringCategoryName>()?,
            score: raw_pattern.score as u32,
        })
    ).collect::<Result<Vec<GradedPattern>, strum::ParseError>>()?;

    let raw_techniques = sqlx::query!(
        "SELECT technique, score, score_header FROM techniques WHERE test_id = $1",
        test_id
    )
    .fetch_all(pool)
    .await?;

    let techniques: Vec<GradedTechnique> = raw_techniques
    .into_iter()
    .map(|raw_technique| 
        Ok(GradedTechnique {
            technique: raw_technique.technique.parse::<TechniqueName>()?,
            score: raw_technique.score as u32,
            score_header: raw_technique.score_header.parse::<TechniqueScoringHeaderName>()?,
        })
    ).collect::<Result<Vec<GradedTechnique>, strum::ParseError>>()?;

    let raw_bonuses = sqlx::query!(
        "SELECT name, score FROM bonus_points WHERE test_id = $1",
        test_id
    )
    .fetch_all(pool)
    .await?;

    let bonuses: Vec<GradedBonusPoint> = raw_bonuses
    .into_iter()
    .map(|raw_bonus| 
        Ok(GradedBonusPoint {
            name: raw_bonus.name.parse::<BonusPointName>()?,
            score: raw_bonus.score as u32,
        })
    ).collect::<Result<Vec<GradedBonusPoint>, strum::ParseError>>()?;

    Ok(GradedTest {
        testee: testee,
        test_date: test_date,
        test_type: test_type,
        patterns: patterns,
        techniques: techniques,
        bonuses: bonuses,
    })
}