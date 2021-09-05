# deepstream-rs

## Requirements
- CUDA >= 11.1
- [nvida-docker](https://github.com/NVIDIA/nvidia-docker)

## Usage

### Build container
To build docker image
```sh
make build
```
### Run container
Allow external applications to connect to the host's X display
```
xhost +
```
Run docker container
```
make run
```

## References
- https://gitlab.freedesktop.org/gstreamer/gstreamer-rs/-/tree/master/examples
- https://github.com/NVIDIA-AI-IOT/deepstream_python_apps
- https://github.com/mdegans/deepstream-5.1
- https://github.com/alexcrichton/rust-ffi-examples
- https://github.com/thatbrguy/Deep-Stream-ONNX