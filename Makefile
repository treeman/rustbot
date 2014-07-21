OUT_DIR = target
MKDIR_P = mkdir -p

.PHONY: directories

all: directories rustbot

rustbot:
	rustc src/main.rs -o target/rustbot

clean:
	rm target -fr

remake: clean target/rustbot

test: directories
	rustc src/main.rs --test -o target/rustbot-test
	./target/rustbot-test

directories: ${OUT_DIR}

${OUT_DIR}:
	${MKDIR_P} ${OUT_DIR}
