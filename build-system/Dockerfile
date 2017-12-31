FROM ubuntu
RUN apt-get update
RUN apt-get install curl -y
RUN apt-get install make -y
RUN apt-get install llvm -y
RUN apt-get install clang -y
RUN apt-get install nasm -y
RUN apt-get install xorriso -y
RUN apt-get install grub -y
RUN apt-get install grub-pc-bin -y
RUN curl https://sh.rustup.rs -sSf | sh -s -- -y
ENV PATH="$PATH:/root/.cargo/bin"
RUN rustup toolchain install nightly-2017-12-31
RUN rustup default nightly-2017-12-31
RUN cargo install xargo
RUN rustup component add rust-src
VOLUME /project
WORKDIR /project
