FROM nvcr.io/nvidia/deepstream:5.1-21.02-base as base

# Install dependencies
RUN apt-get update && apt-get install -y \
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

FROM base
WORKDIR /usr/src/deepstream-rs

# Copy our manifests
COPY dummy.rs .
COPY Cargo.toml .

# Build only the dependencies to cache them
RUN sed -i 's#src/main.rs#dummy.rs#' Cargo.toml
RUN cargo build --release
RUN sed -i 's#dummy.rs#src/main.rs#' Cargo.toml

# Copy source code
COPY ./src ./src

# Check code with clippy
RUN cargo clippy -- -D warnings

# Build for release
RUN cargo install --path .

CMD ["deepstream-rs"]