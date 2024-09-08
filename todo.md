# Todo List

## Required TODO
- Add HTMX functionality to everything
- Send emails to testees once their tests are graded

## Additional Possible Functionality
- Add "most recent testee" to the dashboard
- Add a testee queue
- Add dynamic grading to leader/follower test pages
- Add dynamic form validation to leader/follower test pages
- Add dynamic form validation to the sign up/login pages
- Add pagination/active search to the test summary list page
- Add organizations to the users so that different clubs can use this

## Pretty TODO
- Make my errors not just display unformatted HTML cause I'm lazy...

## Security TODO
- Serverside disable form submission for the demo mode
    - Frontend the submit button can be re-enabled by the user and used to submit forms. This is a security vulnerability.

## Would be Nice TODO
- Refactor the test definition to parse a leader_test.yaml and follower_test.yaml at runtime (so that any community can easily define their own test without needing to recompile)
- Refactor the error handling to use the anyhow crate
