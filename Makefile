DOCKER_TAG = deepstream-rs
COMMAND_TEST = cargo test

build:
	docker build -t $(DOCKER_TAG) .

run:
	docker run --gpus all -it --rm \
    -e DISPLAY=${DISPLAY} -v /tmp/.X11-unix/:/tmp/.X11-unix:ro \
    -e MY_LOG_LEVEL=debug -e MY_LOG_STYLE=always \
    $(DOCKER_TAG)

test:
	docker run -it --rm $(DOCKER_TAG) $(COMMAND_TEST)