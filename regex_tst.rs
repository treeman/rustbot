#![feature(phase)]
#[phase(plugin)]
extern crate regex_macros;
extern crate regex;

struct IrcMsg {
    prefix: String,
    code: String,
    param: String,
}

// FIXME write some tests and cleanup this thing.
fn tst(s: &str) {
    println!("matching: {}", s);
    let re = regex!(r"^(:\w+)?\s*(\w+)\s+(.*)\r?$");
    let caps = re.captures(s);
    match caps {
        Some(x) => {
            println!("  prefix: {}", x.at(1));
            println!("  code: {}", x.at(2));
            println!("  param: {}", x.at(3));
        },
        None => println!("  no match"),
    }
}

fn main() {
    let re = regex!(r"^(\d{4})-(\d{2})-(\d{2})$");
    assert_eq!(re.is_match("2014-01-01"), true);
    let caps = re.captures("2014-01-01").unwrap();
    println!("{} {} {}", caps.at(1), caps.at(2), caps.at(3));

    tst(":pref 020 rustbot lblblb");
    tst("020 rustbot lblblb");
    tst("a");
}

