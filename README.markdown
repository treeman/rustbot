About
=====

An irc bot written in [rust][], for fun and profit! See [rfc2812][] for irc implementation details.

Be aware that rust is fast changing and I may not keep the code up to date.

TODO
====

* Logging
* Auto-op in our channel
* Upload to pastebin when we have many lines output
* Move messages from String to &str
* Autostart in screen

Implement some behaviour
-----------------------

* Medium
    * .cmds
    * auto op
    * .duke - Duke Nukem quotes
    * .insult - Monkey Island insults!
    * Post notification when I post a new blogpost at jonashieta.se
    * Mention whenever I make a commit at github  
        `curl -i https://api.github.com/users/treeman/events/public`

    * Choose one item from a list of options
    * Track the last time a given nick was in the channel, and the last time they spoke
    * .tell Queue a message for an offline nick that's automatically sent in-channel when they join
    * Use Google Translate to translate a given phrase
    * Display information about any posted Youtube link (video title, length, submitter, votes, comments, etc.)
    * Post notifications to channel when teams get a point
    * Url mentions, basically if someone links a URL , the bot displays a title of the page, and how many times its been mentioned.

* Harder
    * .help [cmd]
    * .lithcourse
    * .lithschema
    * latest manga (or something)
    * habitrpg hooks?

[rust]: http://www.rust-lang.org "rust"
[rfc2812]: http://tools.ietf.org/html/rfc2812 "irc reference"

