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

cargo watch -s './tailwind/tailwindcss -i ./static/css/input.css -o ./static/css/output.css -c ./tailwind/tailwind.config.js && cargo run' --ignore *css*

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