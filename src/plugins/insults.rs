extern crate time;
extern crate mi_insults;

use std::rand::{ mod, TaskRng };

use irc::{ IrcWriter, IrcCommand, BotInfo, Plugin, IrcPrivMsg };
use util;

use self::mi_insults::Insults as MiInsults;

pub struct Insults<'a> {
    insults: MiInsults,
    /// (last insult sent, retort we want)
    last: Option<(&'a str, &'a str)>,
    rng: TaskRng,
}

impl<'a> Insults<'a> {
    pub fn new() -> Insults<'a> {
        Insults {
            // FIXME not sure how to handle this in a generic way?
            // For now just symlinked on my machine.
            insults: MiInsults::new("insults.json"),
            last: None,
            rng: rand::task_rng(),
        }
    }

    /// Check if the corresponding text is a retort to the last sent insult.
    fn is_retort(&self, retort: &str) -> bool {
        match self.last {
            Some((_, wanted)) => {
                retort.trim() == wanted
            },
            None => false,
        }
    }

    /// Return a random monkey island insult.
    fn insult(&'a mut self) -> &'a str {
        let insult = self.insults.rand_insult(&mut self.rng)[];
        // Unwrap is ok as they are always in a pair
        let retort = self.insults.retort(insult).unwrap();
        self.last = Some((insult, retort));
        insult
    }

    /// Retort to a string, or fail message.
    fn retort(&'a mut self, insult: &str) -> &'a str {
        self.insults.retort_or_rand_fail(insult, &mut self.rng)
    }
}

impl<'a> Plugin for Insults<'a> {
    fn privmsg(&mut self, msg: &IrcPrivMsg, writer: &IrcWriter, _: &BotInfo) {
        if self.is_retort(msg.txt[]) {
            let response = self.insults.rand_failed_retort(&mut self.rng);
            writer.msg(msg.channel[], response);
        }
    }

    fn cmd(&mut self, cmd: &IrcCommand, writer: &IrcWriter, _info: &BotInfo) {
        let args = util::join(&cmd.args, " ");
        match cmd.name {
            "insult" => writer.msg(cmd.channel[], self.insult()[]),
            "retort" => writer.msg(cmd.channel[], self.retort(args[])[]),
            _ => {},
        }
    }
}

