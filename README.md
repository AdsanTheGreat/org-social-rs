# org-social-rs

A Rust-based CLI and TUI client for the [Org-social](https://github.com/tanrax/org-social) decentralized social network. 

Current version is targeting 1.2 release.

## Overview

org-social-rs provides both command-line and terminal user interface tools for interacting with Org-social networks. This allows the user to use the network without the heavy overhead of emacs, though with some limitations (at least for now).

## Features

- **CLI Interface**: View feeds, filter posts, get some stats
- **TUI Interface**: Interactive terminal-based interface for browsing and navigating social feeds, powered by [ratatui](https://github.com/ratatui-org/ratatui)
- **Feed & Threaded view**: See posts chronologically or in a threaded view
- **Notification view**: See where you were mentioned/replied to
- **Post & Reply System**: Create and save new posts and replies to conversations
- **Poll system**: Display, counting votes (limited to thread view), replies (powered by reply system)

## TODO
Somewhat paired to the lib's features - as it's todo gets fullfilled, the same features should land here where applicable. 

In no particular order:
- Search & filter in the tui
- Formatting improvements
- Automatic remote feeds sync - powered by pre- and post- hooks, most likely bash commands from config. Or built-in support for a particular tool users prefer.
- UX improvements - controls are very random and all over the place.
- CLI feature parity - CLI should also have access to posting and replying.
- Bug fixes, edge case handling - the tool is not very tested.

## Installation

```bash
# Install from crates.io
cargo install org-social-rs
```

### Development Installation

```bash
# Clone the repository
git clone https://github.com/AdsanTheGreat/org-social-rs.git
cd org-social-rs

# Run the program, without installing
cargo run --release

# Build the project
cargo build --release
# Manually copy the binary to your bin directory
```

*OR*

```bash
# Build the release version of the program, put it in the Cargo bin directory
cargo install --path .
```

## Usage

The configuration file is in the default config directory -> org-social-rs folder, in a straightforward .toml format
Currently, setting default file/feed count is supported.

The config options can be overriden by their respective cli flags, for example:
```bash
# This always uses the file you provide here, ignoring the one in the config
org-social-rs --file path/to/social.org tui
```

### CLI Mode

Meant mostly for integrating as, for example, part of a bash script. 
Can force colored (on not) output with the --color flag.

CLI does not yet have all TUI features for displaying posts and feeds, or creating new posts & replies.

```bash
# View feed with latest posts
org-social-rs feed

# Limit number of posts
org-social-rs --count 10

# Filter posts from recent days
org-social-rs feed --days 7
```

### TUI Mode

Meant to serve as an actual client, exposing most features.

```bash
# Launch interactive terminal interface
org-social-rs tui
```

## Library

This project uses the [org-social-lib-rs](https://github.com/AdsanTheGreat/org-social-lib-rs) library for core functionality. If you want to integrate org-social into something, build a specific client (maybe a real gui?), feel free to check it out.

## Contributing

Report issues (there are probably a lot of them), submit pull requests, help is welcome.

## License

This project is licensed under the GNU General Public License v3.0.

## Related Projects

- [org-social.el](https://github.com/tanrax/org-social.el) - Original Emacs client
- [org-social](https://github.com/tanrax/org-social) - Protocol specification and documentation
