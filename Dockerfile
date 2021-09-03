FROM nvcr.io/nvidia/deepstream:5.1-21.02-samples as base

# Install dependencies
RUN apt-get update && apt-get install -y \
    # rust
    build-essential \
    curl \
    # gstreamer-rs dependencies
    libgstreamer1.0-dev libgstreamer-plugins-base1.0-dev \
    gstreamer1.0-plugins-base gstreamer1.0-plugins-good \
    gstreamer1.0-plugins-bad gstreamer1.0-plugins-ugly \
    gstreamer1.0-libav libgstrtspserver-1.0-dev

# Install Rust
RUN curl https://sh.rustup.rs -sSf | bash -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"

FROM base as build
WORKDIR /usr/src/deepstream-rs

# Copy our manifests
COPY dummy.rs .
COPY Cargo.toml .

# Build only the dependencies to cache them
RUN sed -i 's#src/main.rs#dummy.rs#' Cargo.toml &&\
    cargo build --release &&\
    sed -i 's#dummy.rs#src/main.rs#' Cargo.toml

# Build custom gst-plugins
COPY gst-plugins gst-plugins
RUN cd gst-plugins/gst-nvobjconv &&\
    make &&\
    make install

# Copy source code
COPY ./src ./src

# Check code with clippy
RUN cargo clippy -- -D warnings

# Build for release
RUN cargo install --path .

FROM base
WORKDIR /usr/src/deepstream-rs

COPY --from=build /usr/src/deepstream-rs/target/release/deepstream-rs .
COPY --from=build /opt/nvidia/deepstream/deepstream-5.1/lib/gst-plugins /opt/nvidia/deepstream/deepstream-5.1/lib/gst-plugins

# Copy configurations
COPY ./config ./config

CMD ["./deepstream-rs"]
