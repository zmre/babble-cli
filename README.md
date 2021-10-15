# Babble

## Inspiration

I am a fan of [rainbowstream](https://github.com/orakaro/rainbowstream) as a way to view tweets from the command line, but I've had the following issues:

* It has stopped working for long periods due to twitter API issues
* It has stopped working for other periods because of python dependency hells that are probably just issues on my system.

It's actually working again for the moment, but I started building this while it was broken on my machine.

I like the Tweetbot interface a fair bit. I can put different lists in different columns and keep an eye on local news, security news, and my main feed at one time. With Babble, I wanted to do the same thing inside tmux. Actually, I initially envisioned this as more of a terminal app that could make its own columns, but since I use tmux extensively anyway, I realized I can just use that to manage the windows and then spin this up to show various things as I like in the different panes or windows.

## Current Status

For now, it's mostly meant for my own use. I don't have the right API credentials to publish it. But it wouldn't be hard to do that if folks ask. Just submit an issue.

It also supports markdown output, which is useful for me in conjunction with my notes system so I can capture my own social media activity into a note if I want to.

It also still needs some work to make the code cleaner, but given that it works great for me, that may never happen.

## Usage

Here are some of the arguments I use when I run it:

```
babble-cli home
babble-cli list -n Boulder\ News
babble-cli --stream list -n Security
babble-cli --markdown me
```

## TODO

* Allow per-day filter on me
* Get my likes
* Refactor to reuse code better
* Allow for quick preview or launch of URLs by listening for input and maybe labeling URLs with a launch letter or some such
* Allow for easier quitting in stream mode
