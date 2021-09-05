FROM nvcr.io/nvidia/deepstream:5.1-21.02-devel as build
# Install dependencies
RUN apt-get update && apt-get install -y \
    # rust
    build-essential \
    curl \
    # gstreamer-rs dependencies
    libgstreamer1.0-dev libgstreamer-plugins-base1.0-dev \
    gstreamer1.0-plugins-base gstreamer1.0-plugins-good \
    gstreamer1.0-plugins-bad gstreamer1.0-plugins-ugly \
    gstreamer1.0-libav libgstrtspserver-1.0-dev \
    # nvmsgconv
    libglib2.0-dev \
    libjson-glib-dev uuid-dev \
    # others
    wget

# Install Rust
RUN curl https://sh.rustup.rs -sSf | bash -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"

# Download models
WORKDIR /models
RUN wget https://github.com/onnx/models/raw/master/vision/object_detection_segmentation/tiny-yolov2/model/tinyyolov2-8.onnx

WORKDIR /usr/src/deepstream-rs

# Copy our manifests
COPY dummy.rs .
COPY Cargo.toml .

# Build only the dependencies to cache them
RUN sed -i 's#src/main.rs#dummy.rs#' Cargo.toml &&\
    cargo build --release &&\
    sed -i 's#dummy.rs#src/main.rs#' Cargo.toml

# Copy commons
COPY includes includes

# Build custom gst-plugins
COPY gst-plugins gst-plugins
RUN cd gst-plugins/gst-nvobjconv &&\
    make &&\
    make install

# Build custom libs
COPY libs libs
RUN cd libs/nvmsgconv && make && make install
RUN cd libs/libnvdsinfer_custom_bbox_tiny_yolo && make && make install

# Copy source code
COPY ./src ./src

# Check code with clippy
RUN cargo clippy -- -D warnings

# Build for release
RUN cargo install --path .

FROM nvcr.io/nvidia/deepstream:5.1-21.02-base
WORKDIR /usr/src/deepstream-rs

COPY --from=build /usr/src/deepstream-rs/target/release/deepstream-rs .
COPY --from=build /opt/nvidia/deepstream/deepstream-5.1/lib /opt/nvidia/deepstream/deepstream-5.1/lib
COPY --from=build /models /models

# Copy configurations
COPY ./config ./config

CMD ["./deepstream-rs"]
