FROM nvcr.io/nvidia/deepstream:6.4-samples-multiarch as build
# Install dependencies
RUN apt-get update && apt-get install -y \
    # rust
    build-essential \
    curl \
    # gstreamer-rs dependencies
    libgstreamer1.0-dev libgstreamer-plugins-base1.0-dev \
    gstreamer1.0-plugins-base gstreamer1.0-plugins-good \
    gstreamer1.0-plugins-bad \
    gstreamer1.0-libav libgstrtspserver-1.0-dev \
    # build cuda (nvcc)
    cuda-toolkit \
    # others
    wget \
    && apt remove -y gstreamer1.0-plugins-ugly

# Install Rust
RUN curl https://sh.rustup.rs -sSf | bash -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"

# Download models
WORKDIR /models
RUN wget https://github.com/Megvii-BaseDetection/YOLOX/releases/download/0.1.1rc0/yolox_s.onnx

WORKDIR /usr/src/deepstream-rs

# Copy our manifests
COPY libs libs
COPY deepstream-sys deepstream-sys
COPY deepstream deepstream
COPY dummy.rs .
COPY Cargo.toml .

# Build only the dependencies to cache them
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    sed -i 's#src/main.rs#dummy.rs#' Cargo.toml &&\
    cargo build --release &&\
    sed -i 's#dummy.rs#src/main.rs#' Cargo.toml

# Copy commons
COPY includes includes

# Build custom libs
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    cd libs/gst-nvobjconv && make && make install
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    cd libs/nvmsgconv && make && make install
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    cd libs/nvdsinfer_custom_impl_yolox && make && make install

# Copy source code
COPY ./src ./src

# Check code with clippy
#RUN cargo clippy -- -D warnings

# Build for release
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    cargo install --path .

FROM nvcr.io/nvidia/deepstream:6.4-samples-multiarch

ENV LD_LIBRARY_PATH="${LD_LIBRARY_PATH}:/opt/nvidia/deepstream/deepstream/lib"

WORKDIR /usr/src/deepstream-rs

RUN /opt/nvidia/deepstream/deepstream/user_additional_install.sh \
    && apt remove -y gstreamer1.0-plugins-ugly

COPY --from=build /usr/src/deepstream-rs/target/release/deepstream-rs .
COPY --from=build /opt/nvidia/deepstream/deepstream/lib /opt/nvidia/deepstream/deepstream/lib
COPY --from=build /models /models

# Copy configurations
COPY ./config ./config

CMD ["./deepstream-rs"]
