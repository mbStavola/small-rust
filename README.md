# Small Rust

This repo contains a bunch of really small "scripts" written in Rust. 

Partially inspired by Sean Barrett's [Advice for Writing Small Programs in C][stb], but mostly stems from my incredible ability to completely lose all Bash knowledge after closing an `.sh` file.

Some of them do useful things, some of them were just written for fun. Please do not have the expectation that these are written in a super robust fashion; most were written for one-off tasks and experiments.

If you want to play around with these programs, you just need [Rust][rust]. Scripting support is currently available only on Nightly (current as of 1.93.0), but once stabilized I'll remove the +nightly flags. Please note that these scripts are built in debug mode, not release. This makes it easy to make quick changes but performance will suffer for more involved tasks, though these scripts aren't meant for that kind of work.

Be aware that dev work for this repo is done on UNIX-y systems, so they might not work on Windows.

[stb]: https://www.youtube.com/watch?v=eAhWIO1Ra6M
[rust]: https://www.rust-lang.org/tools/install
