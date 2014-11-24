use irc::{ Irc };

pub use plugins::basic::*;
pub use plugins::schema::*;
pub use plugins::insults::*;

mod basic;
mod schema;
mod insults;

pub fn register(irc: &mut Irc) {
    irc.register_plugin(box Basic::new());
    irc.register_plugin(box Schema);
    irc.register_plugin(box Insults::new());
}

