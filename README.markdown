An irc bot written in [rust][], for fun and profit!

Implemented commands
--------------------

* .about
* .botsnack
* .help
* .src
* .status

Command ideas
-------------

* .addinsult %s is dumb
* .brainfuck - compiler
* .cmds
* .duke - Duke Nukem quotes!
* .lithcourse
* .lithschema
* .nextep
* .pokedex
* .retort - Monkey Island insults + retort with top list!
* .uptime
* .help [cmd]

TODO
----

* Tests for all stuffs
* Correct all different warnings
* Config file with credentials.
* Plugin extendable
* `IrcMsg` should parse sender info.
* Admin handling and priviliged commands.
* Task handling.
  Every command should have their own task, so it can fail okay.
  We need a stdin task so we can control our bot directly.

  Problem: sockets are NOT sendable over tasks atm. 
    Synced queues with a specific socket handler could work?
* Output cleaning.
  Don't output PING/PONG and NOTICE and maybe other stuff, unless specified with cmd line flag.
* System commands, so we can use my existing scripts (i.e. nextep)
* Help system
* Figure out how to do random numbers

[rust]: http://www.rust-lang.org

