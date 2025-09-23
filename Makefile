all: basino rust #nim rust

clean:
	cd basino && make clean && cd .. && cd rust-basino && cargo clean && cd .. && cd basino_atmega328p && rm -f basino_atmega328p && cd ..

basino:
	cd basino && make && cd ..

rust:
	make -C basino
	cd rust-basino && cargo clean && mkdir -p target/debug/deps && ln -sf ../../../../basino/libbasino.a target/debug/deps && cargo build & cd ..

nim:
	make -C basino
	cd basino_atmega328p && ln -sf ../basino/libbasino.a . && ratel build && cd ..

test:
	make -C basino
	cd rust-basino && cargo run -r --features test-base,test-stack,test-queue && cd ..
