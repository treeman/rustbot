use std::io;

use irc::*;
use super::CMD_PREFIX;

// If we shall continue the stdin loop or not.
enum StdinControl {
    Quit,
    Continue
}

// Read input from stdin.
pub fn reader(writer: IrcWriter) {
    println!("Spawning stdin reader");
    for line in io::stdin().lines() {
        // FIXME prettier...
        let s : String = line.unwrap();
        let x = s[].trim();

        match Command::new(x, CMD_PREFIX) {
            Some(cmd) => {
                match stdin_cmd(&cmd, &writer) {
                    StdinControl::Quit => break,
                    _ => (),
                }
            },
            None => (),
        }
    }
    println!("Quitting stdin reader");
}

// We can do some rudimentary things from the commandline.
fn stdin_cmd(cmd: &Command, writer: &IrcWriter) -> StdinControl {
    match cmd.name {
        "quit" => {
            writer.quit("Gone for repairs");
            return StdinControl::Quit;
        },
        "echo" => {
            let rest = cmd.args.connect(" ");
            writer.output(rest);
        },
        "say" => {
            if cmd.args.len() > 1 {
                let chan = cmd.args[0];
                let rest = cmd.args.slice_from(1).connect(" ");
                writer.msg(chan, rest[]);
            }
            else {
                // <receiver> can be either a channel or a user nick
                println!("Usage: .say <receiver> text to send");
            }
        },
        _ => (),
    }
    StdinControl::Continue // Don't quit by default
}
