use std::map;
use std::map::*;

use core::io;
use core::io::*;

struct Conf {
    m: ~HashMap<~str, ~[~str]>,
}

fn load(file: ~str) -> @Conf {
    let conf = ~HashMap::<~str, ~[~str]>();

    let res = io::file_reader(&Path(file));

    if (res.is_err()) {
        fail res.get_err();
    }

    let reader = res.get();

    loop {
        let line = reader.read_line().trim();

        // We're at the end
        if reader.eof() {
            break;
        }

        // Skip empty lines and comments
        if line == ~"" || line.starts_with(~"#") {
            loop;
        }

        let parts = line.split_char('=');

        if parts.len() < 2 {
            println(fmt!("Malformed: %s", line));
            loop;
        }

        let key = parts[0].trim().to_lower();
        let val = parts[1].trim();

        conf.insert(key, vec::map(val.split_char(','), |v| v.trim()));
    }

    @Conf { m: move conf }
}

// Only one valid value for this setting
fn get(conf: &Conf, key: ~str) -> Result<~str, ()> {
    if conf.m.contains_key(key) {
        Ok(conf.m.get(key)[0])
    }
    else {
        Err(())
    }
}

fn get_uint(conf: &Conf, key: ~str) -> Result<uint, ()> {
    if conf.m.contains_key(key) {
        Ok(uint::from_str(conf.m.get(key)[0]).get())
    }
    else {
        Err(())
    }
}

