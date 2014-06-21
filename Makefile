all: rustbot

rustbot:
	rustc rustbot.rs -o rustbot

clean:
	rm rustbot rustbot-test -f

remake: clean rustbot

test:
	rustc rustbot.rs --test -o rustbot-test
	./rustbot-test
