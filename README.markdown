A soon to be irc bot written in rust. I hope.

Command ideas
-------------

* .addinsult %s is dumb

TODO
----

* Config file with credentials.
* Prettier callback handling.
  I was thinking of regestering commands, like
    `register("pew", |irc, m| privmsg(irc, m.channel, "FIRING MY LAZER!!"))`

  Maybe something similar but extendable with plugins?
* `IrcMsg` should parse sender info.
* Admin handling and priviliged commands.
* Task handling.
  Every command should have their own task, so it can fail okay.
  We need a stdin task so we can control our bot directly.

  Problem: sockets are NOT sendable over tasks atm. 
    Synced queues with a specific socket handler could work?
* Output cleaning.
  Don't output PING/PONG and NOTICE and maybe other stuff, unless specified with cmd line flag.

