extern crate std;

// Split a string on whitespace, don't include empty strings
pub fn space_split<'a>(s: &'a str) -> Vec<&'a str> {
    s.split(|c: char| -> bool {
        c == ' '
    }).filter(|s: &&str| -> bool {
        *s != ""
    }).collect()
}

// Split a string on newlines, don't include empty lines.
pub fn newline_split<'a>(s: &'a str) -> Vec<&'a str> {
    s.split(|c: char| -> bool {
        c == '\n'
    }).map(|s: &'a str| -> &'a str {
        s.trim()
    }).filter(|s: &&str| -> bool {
        *s != ""
    }).collect()
}


// Run an external command and fetch it's output.
// TODO maybe should not live here?
pub fn run_external_cmd(cmd: &str, args: &[&str]) -> String {
    let mut process = match std::io::process::Command::new(cmd).args(args).spawn() {
        Ok(p) => p,
        Err(e) => panic!("Runtime error: {}", e),
    };

    // FIXME
    //match process.stdout.unwrap().read_to_end() {
        //Ok(x) => {
            //// Hilarious :)
            //std::str::from_utf8(x.as_slice()).unwrap().to_string()
        //},
        //Err(e) => panic!("Read error: {}", e),
    //}
    "".to_string()
}

