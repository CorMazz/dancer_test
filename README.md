### Building and running your application

When you're ready, start your application by running:
`docker compose up --build`.

Your application will be available at http://localhost:(port) where port is specified in the compose.yaml file.


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

ENV_FILE=.env_prod DOCKER_PORT_MAPPING=7000 SERVER_PORT=8000 PG_ADMIN_DOCKER_PORT_MAPPING=7001 docker-compose -p dancexam-prod up
ENV_FILE=.env_demo DOCKER_PORT_MAPPING=7002 SERVER_PORT=8000 PG_ADMIN_DOCKER_PORT_MAPPING=7003 docker-compose -p dancexam-demo up

=======

### Deploying your application to the cloud

First, build your image, e.g.: `docker build -t myapp .`.
If your cloud uses a different CPU architecture than your development
machine (e.g., you are on a Mac M1 and your cloud provider is amd64),
you'll want to build the image for that platform, e.g.:
`docker build --platform=linux/amd64 -t myapp .`.

Then, push it to your registry, e.g. `docker push myregistry.com/myapp`.

Consult Docker's [getting started](https://docs.docker.com/go/get-started-sharing/)
docs for more detail on building and pushing.

### References
* [Docker's Rust guide](https://docs.docker.com/language/rust/)