FROM phusion/baseimage:0.10.2 

RUN apt update
RUN apt-get install -y ssh git python3-pip curl cmake pkg-config libssl-dev git clang 
RUN curl https://sh.rustup.rs -sSf | sh -s -- -y && \
	export PATH="$PATH:$HOME/.cargo/bin" && \
	rustup toolchain install nightly && \
	rustup target add wasm32-unknown-unknown --toolchain nightly && \
	cargo install --git https://github.com/alexcrichton/wasm-gc && \
	rustup default nightly && \
	rustup default stable
ENV PATH="/root/.cargo/bin:$PATH"
ADD . /polpatrol/
WORKDIR /polpatrol
RUN git clone https://github.com/paritytech/polkadot.git polkadot_mod
RUN cd /polpatrol/polkadot_mod && git checkout d517dbeb1d27b8068952e086c9ae472d49b707bd && git apply ../polkadot.diff
RUN git clone https://github.com/paritytech/substrate.git substrate_code_mod
RUN cd /polpatrol/substrate_code_mod && git checkout 73104d3ae1ec061c4efd981a83cdd09104ba159f && git apply ../substrate.diff
RUN cargo build --release
ENTRYPOINT ["/polpatrol/target/release/polpatrol"]
