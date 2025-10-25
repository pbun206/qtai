# Overview

**Qtai** (**Q**uick **t**o **a**ccess **i**tems) is a dmenu/dmenu-like application manager. The main use of case of Qtai is launch a dmenu/dmenu-like application with pre-configured "items". Opposed to manually writing a shell script, Qtai is designed to reduce boilerplate code and be flexible.

Only Linux is tested, though it might work with other Unix-like systems like MacOS.

# Installation

With cargo and git installed, do the following commands in your shell:

```bash
git clone https://github.com/pbun206/qtai.git
cd qtai
cargo install --path .
```

Make sure the cargo bin directory is in [PATH](https://www.geeksforgeeks.org/linux-unix/how-to-set-path-permanantly-in-linux/) or else you will stuck running Qtai with `~/.cargo/bin/qtai`.

If not already, you should use a dmenu/dmenu-like application. I currently use [fuzzel](https://codeberg.org/dnkl/fuzzel), but any should work or it's a bug.

# Usage

## Setup

Before using it, you should generate a config file.

```bash
qtai generate-config-file
# or qtai gcf 
```

You probably want to change dmenu to your preferred dmenu-like application.

```bash
qtai change-menu "fuzzel -d"
# or qtai cm "fuzzel -d"
```

Now, you need a "runner". A runner is a shell command that runs an "item" as an argument. What your runner should be depends on your use case. If you want each "item" to be a binary, set the runner to `'$1'`. The shell replaces `'$1'` with the value of the selected item. Make sure you use single quotes here to avoid unwanted side effects.

```bash
qtai change-runner '$1'
# or qtai cr `$1`
```

However, one of the main features of Qtai is that you can change the runner to whatever you want. Here are some examples:

```bash
# Opens the Librewolf browser to the value of the item.
qtai cr 'librewolf --new-window $1'
# Opens the foot terminal with the value of the item as the starting directory.
qtai cr 'foot -D $1'
# Opens helix to the value of the item
qtai cr 'hx $1'
```

Now, let's create some items! In Qtai, we store items in "collections". 

```bash
qtai add-collection "important files"
qtai add-collection "important urls"
# or qtai ac "important urls"
```


Adding an item is more complicated.

```bash
qtai add-item "qtai github" "https://github.com/pbun206/qtai" "important urls"
# or qtai a qtai_github https://github.com/pbun206/qtai "important files"
```

The first argument is the key of the item. When you run the menu, it will display this key. The second argument is the value of the item. This value is what actually used in the runner. The last argument is the collection where the item to be added.

However, writing "important urls" is annoying to type. Autocomplete isn't a feature (at least yet) unfournately. However, if you only type part of it, qtai will either guess it or go into interactive mode to select between candidates.  

```bash
$ qtai add-item "qtai github" "https://github.com/pbun206/qtai" in
Multiple collections had been found.
What do you choose? (arrow or vi keys):
  important files
  important urls
```

Remove commands are similar:

```bash
qtai remove-item github
# If there is one item with "github", it will delete it automatically. If there are multiple,
# Qtai will make you select one of them.
qtai remove-collection important
# Similar to remove item but with collections

# qtai ri and qtai rc also work.
```

To list collections, do `qtai list` or `qtai l`.

Note that you can make the runner specfic for each collection. Check `qtai change-runner -h`

## Qtai Run

Here is a general overall of using `qtai run`

```bash
# Run Qtai with all collections
qtai run
# or qtai r...
# Run Qtai with collections with the substring "fil"
qtai run "fil"
# Run Qtai with collections that is exactly "files"
qtai run -s "files" 
# Run Qtai with a specific runner
qtai run -s "files" -r 'hx $1'
# Run Qtai with a TUI menu rather than using GUI menu
qtai terminal-run -s "files" -r 'hx $1'
```

# Configuration File

Although there is a command line, you could also directly edit the config file. Qtai uses toml. Here is an example config file with some notes on syntax:

```toml
# Required. If an input does not match any of the keys, it will always default to this runner.
default_runner = 'librewolf --new-window https://duckduckgo.com/?q="$1"'
# Required.
default_menu = "fuzzel -d"

# This header is automatically generated, but unneeded. Personally, I think it adds to readability.
[collections]
[collections."website"]
default_runner = 'librewolf --new-window $1'
"google" = "https://google.com"
# Quotations are optional
wikipedia = "https://en.wikipedia.org"
# Spaces are allowed in quotations though
"google scholar" = "https://scholar.google.com/"

# Quotations here is optional too!
[collections.directories]
# Default runners within collections are optional 
"home" = "~/"
"config" = "~/.config/"
"yolk" = "~/.config/yolk/"
"nextcloud" = "~/Nextcloud"
"art" = "~/Pictures/art/"

[collections."quick shortcuts"]
default_runner = '$1'
"hx" = "hx"
"gnome control center" = "XDG_CURRENT_DESKTOP=gnome && gnome-control-center "
```

Doing a config file edit with command line will not destroy preexisting comments. However, if you are a tidy person like me, editing the config file directly probably makes more sense.


# License

MIT
