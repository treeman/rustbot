all: rustbot

rustbot:
	rustc src/rustbot.rs -o rustbot

clean:
	rm rustbot rustbot-test -f

remake: clean rustbot

test:
	rustc src/rustbot.rs --test -o rustbot-test
	./rustbot-test
