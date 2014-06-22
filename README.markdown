About
=====

An irc bot written in [rust][], for fun and profit! See [rfc2812][] for irc implementation details.

Be aware that rust is fast changing and I may not keep the code up to date.

TODO
====

* Decide on writer interface (&str vs String vs &String...)
* Logging
* Auto-op in our channel

Implement some behaviour
-----------------------

* Easy
    * .help
    * .nextep (external)
* Medium
    * .cmds
    * auto op
    * .uptime
    * .duke - Duke Nukem quotes
    * .insult - Monkey Island insults!
* Harder
    * .help [cmd]
    * .lithcourse
    * .lithschema
    * latest manga (or something)
    * habitrpg hooks?

[rust]: http://www.rust-lang.org "rust"
[rfc2812]: http://tools.ietf.org/html/rfc2812 "irc reference"

