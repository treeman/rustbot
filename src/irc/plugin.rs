use irc::{ IrcWriter, IrcCommand, BotInfo, IrcPrivMsg };

pub trait Plugin {
    // TODO
    // * list of cmds the plugin listens to
    // * help for all cmds
    // * raw hooks
    // * irc code hooks

    fn privmsg(&mut self, _msg: &IrcPrivMsg, _writer: &IrcWriter, _info: &BotInfo) { }

    fn cmd(&mut self, _cmd: &IrcCommand, _writer: &IrcWriter, _info: &BotInfo) { }
}

