docker run --gpus all -it --rm \
    -e DISPLAY=$DISPLAY -v /tmp/.X11-unix/:/tmp/.X11-unix:ro \
    -e MY_LOG_LEVEL=debug -e MY_LOG_STYLE=always \
    deepstream-rs