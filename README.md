# deepstream-rs

## Requirements
- CUDA >= 11.1
- [nvida-docker](https://github.com/NVIDIA/nvidia-docker)

## Usage

### Build container
To build docker image
```sh
sh build.sh
```
### Run container
Allow external applications to connect to the host's X display
```
xhost +
```
Run docker container
```
sh run.sh
```

## References
- https://gitlab.freedesktop.org/gstreamer/gstreamer-rs/-/tree/master/examples
- https://github.com/NVIDIA-AI-IOT/deepstream_python_apps