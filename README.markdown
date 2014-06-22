About
=====

An irc bot written in [rust][], for fun and profit! See [rfc2812][] for irc implementation details.

Be aware that rust is fast changing and I may not keep the code up to date.

TODO
====

* Add a command struct
* Add command callback
* Merge IrcWriter to include irc info/config (want channels for better join hook)
* Readd tests of parsing

Implement some behaviour
-----------------------

* Easy
    * .about
    * .botsnack
    * .cmds
    * .help
    * .nextep (external)
    * .src
    * .status
* Medium
    * auto op
    * .uptime
    * .help [cmd]
    * .duke - Duke Nukem quotes
    * .insult - Monkey Island insults!
    * .lithcourse
    * .lithschema
    * latest manga (or something)
    * habitrpg hooks?

[rust]: http://www.rust-lang.org "rust"
[rfc2812]: http://tools.ietf.org/html/rfc2812 "irc reference"

