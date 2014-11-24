extern crate time;

use irc::{ IrcWriter, IrcCommand, BotInfo, Plugin };
use util;

pub struct Basic {
    start: time::Tm,
}

impl Basic {
    pub fn new() -> Basic {
        Basic {
            start: time::now(),
        }
    }

    fn uptime(&self) -> String {
        let at = time::now();
        let dt = at.to_timespec().sec - self.start.to_timespec().sec;
        format!("I've been alive {}", format_duration(dt))
    }
}

impl Plugin for Basic {
    fn cmd(&mut self, cmd: &IrcCommand, writer: &IrcWriter, _info: &BotInfo) {
        match cmd.name {
            "uptime" => writer.msg(cmd.channel[], self.uptime()[]),
            _ => {},
        }
    }
}

// 12 days 2 hours 3 minutes 48 seconds
pub fn format_duration(mut sec: i64) -> String {
    let mut min: i64 = sec / 60;
    let mut hours: i64 = min / 60;
    let days: i64 = hours / 24;

    if sec > 0 {
        sec = sec - min * 60;
    }
    if hours > 0 {
        min = min - hours * 60;
    }
    if days > 0 {
        hours = hours - days * 24;
    }

    fn fmt(x: i64, s: &str) -> String {
        format!("{} {}{}", x, s, if x == 1 { "" } else { "s" })
    }
    let day_fmt = fmt(days, "day");
    let hour_fmt = fmt(hours, "hour");
    let min_fmt = fmt(min, "minute");
    let sec_fmt = fmt(sec, "second");

    let parts = {
        if days > 0 {
            vec![day_fmt, hour_fmt, min_fmt, sec_fmt]
        } else if hours > 0 {
            vec![hour_fmt, min_fmt, sec_fmt]
        } else if min > 0 {
            vec![min_fmt, sec_fmt]
        } else {
            vec![sec_fmt]
        }
    };
    util::join_strings(&parts, " ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format() {
        assert_eq!(format_duration(0)[], "0 seconds");
        assert_eq!(format_duration(1)[], "1 second");
        assert_eq!(format_duration(2)[], "2 seconds");
        assert_eq!(format_duration(93)[], "1 minute 33 seconds");
        assert_eq!(format_duration(3145400)[], "36 days 9 hours 43 minutes 20 seconds");
    }
}

