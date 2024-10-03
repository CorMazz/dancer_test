# Todo List

## Required TODO
- Add HTMX functionality to everything
- Send emails to testees once their tests are graded
- Figure out how to export the passing/failing test results so front of house can confirm class attendance



## Additional Possible Functionality
- Add "most recent testee" to the dashboard
- Add dynamic form validation to the sign up/login pages
- Add pagination/active search to the test summary list page

## Pretty TODO
- Make my errors not just display unformatted HTML cause I'm lazy...

## Security TODO
- Serverside disable form submission for the demo mode
    - Frontend the submit button can be re-enabled by the user and used to submit forms. This is a security vulnerability.
- Add functionality to require a licensing code to enter the queue

- Test this in the validate_failing_header_labels
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

## Would be Nice TODO
- Refactor the error handling to use the anyhow crate (I started this, it removes some of the nuance of my error handling...)
- Separate the test parsing into a separate crate that I can provide as an executable so that other people can work on creating valid tests
- Go through and make composable classes to remove a lot of the Option<> types on my Test class that makes the logic a little funky. This would enable me to have all the funky logic in one place as we move from one class type to another. (4 hours wasted here already...)
- Add a command upon the server restarting to clear the queue. 
- Add a command to clear the queue after a certain amount of time.
- Put emails on the test summaries page since the email is the primary key for a testee
- Add an option to not send the users an email after taking a test
- Add an option to manually send a user an email


## Last Thing I Was Doing

