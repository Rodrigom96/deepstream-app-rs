docker run --gpus all -it --rm \
    -e DISPLAY=$DISPLAY -v /tmp/.X11-unix/:/tmp/.X11-unix:ro \
    deepstream-rs