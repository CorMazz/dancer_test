# Todo List

## Required TODO





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


## Would be Nice TODO
- Refactor the error handling to use the anyhow crate (I started this, it removes some of the nuance of my error handling...)
- Separate the test parsing into a separate crate that I can provide as an executable so that other people can work on creating valid tests
- Go through and make composable classes to remove a lot of the Option<> types on my Test class that makes the logic a little funky. This would enable me to have all the funky logic in one place as we move from one class type to another. (4 hours wasted here already...)
- Add a command upon the server restarting to clear the queue. 
- Add a command to clear the queue after a certain amount of time.
- Put emails on the test summaries page since the email is the primary key for a testee
- Add an option to manually send a user an email
- Figure out how to export the passing/failing test results so front of house can confirm class attendance
- Add pagination for the testee search and the broad result search
- Add the ability to change the name and email for a testee after they get created (actually, the name of a testee already gets changed if a different name for an email is submitted)
- Add the ability to change what testee a test belongs to. 


## Last Thing I Was Doing
When no results are available, ensure that it tells users that no results match the given filters.
Otherwise, test that results page and then make it pretty.