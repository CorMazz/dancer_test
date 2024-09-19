# Todo List

## Required TODO
- Add HTMX functionality to everything
- Send emails to testees once their tests are graded
- Add the "failing score label" functionality (as well as an explanation to the user)
- Add test config to allow point values to be shown on questions or not
- Add test config to allow on the fly grading to be shown while proctoring or not


## Additional Possible Functionality
- Add "most recent testee" to the dashboard
- Add dynamic form validation to leader/follower test pages
- Add dynamic form validation to the sign up/login pages
- Add pagination/active search to the test summary list page

## Pretty TODO
- Make my errors not just display unformatted HTML cause I'm lazy...

## Security TODO
- Serverside disable form submission for the demo mode
    - Frontend the submit button can be re-enabled by the user and used to submit forms. This is a security vulnerability.


## Would be Nice TODO
- Refactor the error handling to use the anyhow crate
- Separate the test parsing into a separate crate that I can provide as an executable so that other people can work on creating valid tests


## Last Thing I Was Doing
Working on the test summaries.
In insert test metadata in the save test to database add failure reason and is_passing