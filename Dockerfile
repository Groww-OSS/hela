# Use the latest Rust image as the base
FROM rust:latest

# Install a specific stable Rust version (1.83.0)
RUN rustup install 1.83.0 && \
    rustup default 1.83.0

# Verify the installed Rust version
RUN rustc --version && cargo --version

# Explicitly define HOME to avoid warnings
ENV HOME=/root

# Set up the project files
COPY Cargo.toml /hela/Cargo.toml
COPY Cargo.lock /hela/Cargo.lock
COPY src /hela/src

# Set the working directory
WORKDIR /hela

# Build the project with the specified Rust version
RUN cargo build --release

# Move the binary to a global path and clean up
RUN mv /hela/target/release/Hela /usr/local/bin/hela && \
    rm -rf /hela

# Update the package list and upgrade the system
RUN apt-get update && \
    apt-get -y upgrade && \
    apt-get install -y --no-install-recommends \
    tzdata \
    software-properties-common \
    python3-pip \
    default-jdk \
    npm \
    maven \
    curl \
    wget \
    python3-venv && \
    apt-get clean && rm -rf /var/lib/apt/lists/*

# Configure timezone to avoid interactive prompts
ENV DEBIAN_FRONTEND=noninteractive
ENV TZ=Europe/London

# Install Semgrep
RUN pip3 install semgrep --break-system-packages

# Upgrade Node.js using npm
RUN npm install -g n && n stable

# Install Go and set the PATH
RUN wget https://go.dev/dl/go1.21.9.linux-amd64.tar.gz && \
    tar -C /usr/local -xzf go1.21.9.linux-amd64.tar.gz && \
    rm go1.21.9.linux-amd64.tar.gz
ENV GOPATH=$HOME/go
ENV PATH=$PATH:/usr/local/go/bin:$GOPATH/bin

# Install TruffleHog
RUN curl -sSfL https://raw.githubusercontent.com/trufflesecurity/trufflehog/main/scripts/install.sh | sh -s -- -b /usr/local/bin

# Install OSV-Scanner
RUN go install github.com/google/osv-scanner/cmd/osv-scanner@v1

# Install CycloneDX tools and pnpm
RUN npm install -g @cyclonedx/cdxgen pnpm

# Export the FETCH_LICENSE variable (if required by your application)
ENV FETCH_LICENSE=true

# Set the entry point to run Hela
ENTRYPOINT ["hela"]
