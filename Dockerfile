FROM quay.io/pypa/manylinux2010_x86_64 as builder
RUN /opt/python/cp39-cp39/bin/pip3.9 install maturin virtualenv
RUN curl --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"

FROM builder
WORKDIR /build
COPY . .
RUN cargo --version
RUN /opt/python/cp39-cp39/bin/maturin build --release --strip --manylinux 2010 -i /opt/python/cp39-cp39/bin/python3.9 --cargo-extra-args="--features extension-module"
