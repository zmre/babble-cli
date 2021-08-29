# Babble

## Inspiration

I am a fan of [rainbowstream](https://github.com/orakaro/rainbowstream) as a way to view tweets from the command line. But I've had the following issues:

* It has stopped working for long periods due to twitter API issues
* It has stopped working for other periods because of python dependency hells that are probably just issues on my system.

It's actually working again for the moment, but I started building this while it was still broken on my machine.

I like the Tweetbot interface a fair bit. I can put different lists in different columns and keep an eye on local news, security news, and my main feed at one time. With Babble, I wanted to do the same thing inside tmux. Actually, I initially envisioned this as more of a terminal app that could make its own columns, but since I use tmux extensively anyway, I realized I can just use that to manage the windows and then spin this up to show various things as I like in the different panes or windows.

## Current Status

For now, it's mostly meant for my own use. I don't have the right API credentials to publish it. But it wouldn't be hard to do that if folks ask. Just submit an issue.

It also still needs some work to make the code cleaner and to allow each option to either stream indefinitely (until `control-c`) or to just get the most recent X (15 for now, but I'll make that a config or CLI option).

## Usage

Here are some of the arguments I use when I run it:

```
babble-cli home
babble-cli list -n Boulder\ News
babble-cli list -n Security
babble-cli me
```

## TODO

* Make streaming mode for lists and me
* Allow per-day filter on me
* Get my likes
* Optional markdown output instead of terminal colors
* Refactor to reuse code better
* Allow for quick preview or launch of URLs by listening for input and maybe labeling URLs with a launch letter or some such
* Allow for easier quitting
