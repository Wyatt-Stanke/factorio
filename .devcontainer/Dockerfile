FROM mcr.microsoft.com/devcontainers/universal:5@sha256:87508bfb3a85b7c13e45739b8fc2ea98a832f28500031d77e400eda8ccedd474

RUN apt-get update
RUN apt-get install -y \
    build-essential \
    curl

RUN apt-get update

ENV CARGO_HOME="/usr/local/cargo"
ENV RUSTUP_HOME="/usr/local/rustup"

RUN curl https://sh.rustup.rs -sSf | bash -s -- -y -c rust-src
ENV PATH="${CARGO_HOME}/bin:${PATH}"

RUN echo 'export RUSTUP_HOME="/usr/local/rustup"' >> /etc/bash.bashrc
RUN echo 'export CARGO_HOME="/usr/local/cargo"' >> /etc/bash.bashrc
RUN echo 'export PATH="${CARGO_HOME}/bin:${PATH}"' >> /etc/bash.bashrc

RUN chmod -R a+w ${CARGO_HOME}

RUN curl -L --proto '=https' --tlsv1.2 -sSf https://raw.githubusercontent.com/cargo-bins/cargo-binstall/main/install-from-binstall-release.sh | bash
RUN cargo binstall cargo-nextest --secure