DOCKER_TAG = deepstream-rs
COMMAND_TEST = cargo test

build:
	docker build -t $(DOCKER_TAG) .

run:
	docker-compose up app

test:
	docker run -it --rm $(DOCKER_TAG) $(COMMAND_TEST)

bash:
	docker-compose run app bash
