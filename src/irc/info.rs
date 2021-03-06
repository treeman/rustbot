use irc::config::IrcConfig;

// Information about our bot.
// Will get shared with callbacks.
pub struct BotInfo<'a> {
    pub nick: &'a str,
    pub descr: &'a str,
    pub channels: Vec<&'a str>,
    pub cmd_prefix: char,
}

impl<'a> BotInfo<'a> {
    pub fn new(conf: &IrcConfig<'a>) -> BotInfo<'a> {
        BotInfo {
            nick: conf.nick,
            descr: conf.descr,
            channels: conf.channels.clone(),
            cmd_prefix: conf.cmd_prefix,
        }
    }
}

