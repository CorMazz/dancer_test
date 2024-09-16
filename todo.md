# Todo List

## Required TODO
- Add HTMX functionality to everything
- Send emails to testees once their tests are graded
- Fix the test max score calculation when I have wifi

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
Add this from to_be_deleted into the test_section.html value="{{ technique_group.technique_headers[loop.index0] }}--{{ point }}" 
