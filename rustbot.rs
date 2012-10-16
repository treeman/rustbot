
fn main() {
    let mut args = os::args();

    for args.each |s| {
        io::println(*s);
    }
}

