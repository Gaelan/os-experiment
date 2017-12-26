FROM ubuntu
RUN apt-get update
RUN apt-get install curl -y
RUN apt-get install make -y
RUN apt-get install llvm -y
RUN apt-get install clang -y
RUN apt-get install nasm -y
RUN apt-get install grub -y
RUN curl https://sh.rustup.rs -sSf | sh -s -- -y
# RUN printf "\n# Rust Binaries\nexport PATH=$PATH:$HOME/.cargo/bin\n" >> $HOME/.bashrc
ENV PATH="$PATH:/root/.cargo/bin"
VOLUME /project
WORKDIR /project
