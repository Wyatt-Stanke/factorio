FROM mcr.microsoft.com/devcontainers/universal:5@sha256:feeaa66d9deb342b5a5a9fdcf8afef18689cd284359a67a62289a77772573337

RUN apt-get update
RUN apt-get install -y \
    build-essential \
    curl

RUN apt-get update

ENV CARGO_HOME="/usr/local/cargo"
ENV RUSTUP_HOME="/usr/local/rustup"

RUN curl https://sh.rustup.rs -sSf | bash -s -- -y
ENV PATH="${CARGO_HOME}/bin:${PATH}"

RUN echo 'export RUSTUP_HOME="/usr/local/rustup"' >> /etc/bash.bashrc
RUN echo 'export CARGO_HOME="/usr/local/cargo"' >> /etc/bash.bashrc
RUN echo 'export PATH="${CARGO_HOME}/bin:${PATH}"' >> /etc/bash.bashrc

RUN curl -L --proto '=https' --tlsv1.2 -sSf https://raw.githubusercontent.com/cargo-bins/cargo-binstall/main/install-from-binstall-release.sh | bash
RUN cargo binstall cargo-nextest --secure