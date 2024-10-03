# Dancexam

Dancexam is an innovative platform designed to streamline the process of dance exams and evaluations. It allows instructors, students, and examiners to interact seamlessly, track progress, and provide feedback in a structured manner. The platform supports customized dance profiles, exam scheduling, automated grading, and detailed performance analysis.

## Features

- **Custom Dance Profiles**: Create profiles for dancers with detailed information including skill levels, training history, and personal goals.
- **Exam Scheduling**: Easily set up and manage dance exams day-of with a queue system.
- **Automated Grading**: Streamline the grading process with automated scoring based on predefined criteria, ensuring fair and consistent evaluations.
- **Email Grade Results**: Email students their results so that they get immediate feedback and insights into their dancing.
- **Performance Analysis**: Detailed reports provide insights into strengths, weaknesses, and areas for improvement, helping dancers enhance their skills.
- **Responsive Design**: Optimized for both desktop and mobile devices, ensuring a smooth experience across all platforms.

## Getting Started

### Prerequisites

This application was developed in a dev container. All requisite files are contained in their to create your dev environment, provided you have Docker and VSCode. 

## Usage

- **Creating a Dance Exam**: Dance exams are defined with the test_definitions.yaml file, which is parsed upon server initialization. Any number of tests can be created at once. 
- **Grading**: During or after the exam, use the grading interface to provide scores based on performance. The system will automatically calculate the overall score and generate feedback.
- **Viewing Results**: Dancers can check their emails to see performance reports and track their progress.

## License

This project is licensed under the GPLv3.0 License.

## Contact

For more information, visit [corradomazzarelli.com](https://www.corradomazzarelli.com).

## Acknowledgements

- Thanks to all contributors and users for their support and feedback.
- Special thanks to the Greenville Westies for their consistent feedback and requests to make this a better app. 


### Note from Cory

This application is containerized, and was also developed in a dev container. Everything within the .devcontainer folder defines the configuration
for the dev container, and the other docker files out here define how to create the production containers. 

#### Developer Notes

Tailwind must be rebuilt everytime you make changes to the html classes. That can be done with the tailwindcss executable in the tailwind folder

./tailwind/tailwindcss -i ./static/css/input.css -o ./static/css/output.css -c ./tailwind/tailwind.config.js

If you want to automagically recompile your Rust executable and rebuild your css everytime you save a file, you can run this command.

cargo watch -s './tailwind/tailwindcss -i ./static/css/input.css -o ./static/css/output.css -c ./tailwind/tailwind.config.js && cargo sqlx prepare && cargo run' --ignore *css* --ignore .sqlx --ignore main.rs --why

For some reason the cargo sqlx prepare command changes the permissions on the main.rs file, which was causing cargo watch to fire repeatedly.
The cargo sqlx prepare command allows sqlx to compile even when the database is offline. 

To build the two environments locally:

`
ENV_FILE=.env_prod DOCKER_PORT_MAPPING=7000 SERVER_PORT=8000 PG_ADMIN_DOCKER_PORT_MAPPING=7001 docker-compose -p dancexam-prod up --build
ENV_FILE=.env_demo DOCKER_PORT_MAPPING=7002 SERVER_PORT=8000 PG_ADMIN_DOCKER_PORT_MAPPING=7003 docker-compose -p dancexam-demo up --build
`

=======

### Deploying The Application

This app has a CI/CD pipeline. Any push to master rebuilds the docker container with the new source code and if the build is successful, pulls the new image and runs it on the Linode server hosting the current instances of the app.