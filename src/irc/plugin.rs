use irc::{ IrcWriter, IrcCommand, BotInfo };

pub trait Plugin {
    // list of cmds the plugin listens to

    //pub fn privmsg(msg: &IrcPrivMsg, writer: &IrcWriter, info: &BotInfo) {
    fn privmsg(&mut self) {
        println!("Plugin::privmsg");
    }

    fn cmd(&mut self, _cmd: &IrcCommand, _writer: &IrcWriter, _info: &BotInfo) {
        println!("Plugin::cmd");
    }
}

