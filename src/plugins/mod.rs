use irc::{ Irc };

pub use plugins::basic::*;
pub use plugins::schema::*;

mod basic;
mod schema;

pub fn register(irc: &mut Irc) {
    irc.register_plugin(box Basic::new());
    irc.register_plugin(box Schema);
}

