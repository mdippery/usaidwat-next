usaidwat
========

Are you a Redditor? Do you want to view the subreddits in which a particular
user commonly posts? If you answered "yes" to all three of these questions,
then **usaidwat** is the crate for you!

Installation
------------

**usaidwat** can be installed via Cargo:

    $ cargo install usaidwat

Usage
-----

A `usaidwat` binary is installed with the crate. `usaidwat` will analyze a
user's last 100 comments and provide statistics.

To list a Redditor's comments, use the `log` subcommand:

    $ usaidwat log mipadi

To list a count of subreddits in which a user has posted, use the `tally`
subcommand:

    $ usaidwat tally mipadi

You will see output like the following:

```
apple              6
AskReddit         61
battlefield3       2
books              2
django             1
Games              1
nyc                1
personalfinance    1
photography        1
programming       20
redditcasual       1
wikipedia          1
worldnews          2
```

Which indicates that mipadi has commented in `/r/AskReddit` 61 times (out of
their last 100 comments).

To see the comments for a specific subreddit, tack on that subreddit:

    $ usaidwat log mipadi AskReddit

All the comments for the given subreddit will be printed.

And with the power of artificial "intelligence", you can get a quick summary
of a user's last 100 comments, along with a tone and sentiment analysis, to
let you quickly ascertain if the Redditor is a jerk!

    $ usaidwat summary mipadi

There are many more commands available; run `usaidwat -h` to see a complete
listing.

Claude Setup
------------

The `summary` command sends a Redditor's comment history to Claude for
summarization, as well as tone and sentiment analysis:

    $ usaidwat summary mipadi

To use this feature, you must have access to the Claude API.

To enable access:

1. Set up a [Claude API account].
2. Generate an [API key].
3. Copy and paste the generated key.
4. Store the generated key in your shell's `$CLAUDE_API_KEY` environment 
   variable. Follow your shell's procedure for configuring environment
   variables, but generally this involves running

       $ export CLAUDE_API_KEY='copied api key'

   In your shell session or in your shell's configuration ("rc") file
   (e.g., `~/.bashrc` or `~/.zshrc`).

**You are responsible for the costs of your use of the Claude API!**

By default, `usaidwat summary` will use the cheapest Claude model available;
see `usaidwat summary -h` for other options.

**usaidwat** currently only works with Claude, although more providers may be
enabled in the future.

Testing
-------

Test suites can be run with Cargo:

    $ cargo test

[API key]: https://platform.claude.com/settings/keys
[Claude API account]: https://platform.claude.com/
