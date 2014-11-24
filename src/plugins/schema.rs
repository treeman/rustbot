extern crate time;
extern crate timeedit;

use std::time::Duration;

use util;
use irc::{ IrcWriter, IrcCommand, BotInfo, Plugin };

pub struct Schema;

impl Schema {
    fn find_schema(&self, args: &Vec<&str>) -> String {
        let base = "https://se.timeedit.net/web/liu/db1/schema";

        let s = util::join(args, " ");
        let from = time::now();
        let to = time::at(from.to_timespec() + Duration::weeks(1));

        let types = timeedit::multi_search(s[], base);

        let mut res = String::new();
        if types.is_empty() {
            return "So sorry, no match found.".to_string();
        } else {
            res.push_str("Schedule for: ");

            let codes = util::join(&types.iter().map(|x| x.code[]).collect(), ", ");
            res.push_str(codes[]);

            let events = timeedit::schedule(types, from, to, base);

            // If there are things today, list them all
            let today = timeedit::filter_upcoming(timeedit::filter_today(events.clone()));
            if !today.is_empty() {
                for event in today.iter() {
                    res.push_str(format!("\n{}", event.fmt_time_only())[]);
                }
            // Otherwise just print when the next is
            } else {
                let events = timeedit::filter_upcoming(events);

                if events.is_empty() {
                    res.push_str("\nYou're free!");
                } else {
                    res.push_str(format!("\nNext: {}", events[0].fmt_pretty())[]);
                }
            }
        }
        res
    }
}

impl Plugin for Schema {
    fn cmd(&mut self, cmd: &IrcCommand, writer: &IrcWriter, _info: &BotInfo) {
        match cmd.name {
            "schema" => writer.msg(cmd.channel[], self.find_schema(&cmd.args)[]),
            _ => {},
        }
    }
}

