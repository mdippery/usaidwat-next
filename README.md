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

    $ usaidwat log reddit_user

To list a count of subreddits in which a user has posted, use the `tally`
subcommand:

    $ usaidwat tally reddit_user

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

Which indicates that reddit_user has commented in `/r/AskReddit` 61 times (out of
their last 100 comments).

To see the comments for a specific subreddit, tack on that subreddit:

    $ usaidwat log reddit_user AskReddit

All the comments for the given subreddit will be printed.

And with the power of artificial "intelligence", you can get a quick summary
of a user's last 100 comments, along with a tone and sentiment analysis, to
let you quickly ascertain if the Redditor is a jerk!

    $ usaidwat summary reddit_user

There are many more commands available; run `usaidwat -h` to see a complete
listing.

OpenAI Setup
------------

The `summary` command sends a Redditor's comment history to OpenAI for
summarization, as well as tone and sentiment analysis:

    $ usaidwat summary reddit_user

To use this feature, you must have access to OpenAI's API.

To enable access:

1. Set up an [OpenAI API account].
2. Generate an [API key].
3. Copy and paste the generated key.
4. Store the generated key in your shell's `$OPENAI_API_KEY` environment 
   variable. Follow your shell's procedure for configuring environment
   variables, but generally this involves running

   ```bash
   $ export OPENAI_API_KEY='copied api key'
   ```

   In your shell session or in your shell's configuration ("rc") file
   (e.g., `~/.bashrc` or `~/.zshrc`).

**You are responsible for the costs of your use of OpenAI's API!**
See the [openai module documentation] for more information on the cost of
using the OpenAI API.

By default, `usaidwat summary` will use the [cheapest OpenAI model] available;
see `usaidwat summary -h` for other options.

**usaidwat** currently only works with OpenAI, although more providers may be
enabled in the future.

Testing
-------

Test suites can be run with Cargo:

    $ cargo test

License
-------

usaidwat is licensed under the terms of the [Apache License 2.0]. Please
see the LICENSE file accompanying this source code or visit the previous
link for more information on licensing.

A note on versioning...
-----------------------

usaidwat is both an application and a library. Using it as an application is
much more common than incorporating it into another application as a library,
so its version number reflects that use case; that is, the major version
number (the first component) reflects an _epoch_ of the application and is
only incremented when there is a major new feature. Thus the _library_
portion does not completely adhere to [SemVer] guidelines: a minor version
update may not be backwards compatible. If you are incorporating usaidwat
into your application as a library, you should consider that minor version
bumps _may_ be backwards incompatible and define your version constraints
accordingly.

[Apache License 2.0]: https://www.apache.org/licenses/LICENSE-2.0
[API key]: https://platform.openai.com/settings/organization/api-keys
[OpenAI API account]: https://platform.openai.com/docs/overview
[SemVer]: https://semver.org/
[openai module documentation]: https://docs.rs/cogito-openai
[cheapest OpenAI model]: https://docs.rs/cogito/latest/cogito/trait.AIModel.html#tymethod.cheapest
