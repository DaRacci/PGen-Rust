<h1 align="center">PGen-Rust</h1>

---

### Getting Started

PGen is available on crates.io under the name rpgen standing for Rememberable Password Generator.
Install it with cargo by running
```bash
cargo install rpgen
```

Alternatively if you don't have cargo installed or just want to manually, You can get the latest version of PGen from the [github actions artifacts](https://github.com/DaRacci/PGen-Rust/actions/workflows/build.yml/)
or from one of these links: [linux](https://nightly.link/DaRacci/PGen-Rust/workflows/build/master/PGen-linux.zip), [windows](https://nightly.link/DaRacci/PGen-Rust/workflows/build/master/PGen-windows.zip), [macos](https://nightly.link/DaRacci/PGen-Rust/workflows/build/master/PGen-macos.zip)


### Usage
From the command line you can use
``./rpgen generate`` this will run PGen with the default rules
and generate a new configuration file according to your operating system 
```yaml
Windows: {FOLDERID_RoamingAppData} # eg: C:\Users\{USERNAME}\AppData\Roaming
Linux: $HOME/.config # or $XDG_CONFIG_HOME
Mac: $HOME/Library/Preferences
```

---

### Arguments
```
USAGE:
    rpgen [OPTIONS] <SUBCOMMAND>

OPTIONS:
    -a, --amount <AMOUNT>
            The number of passwords to generate (default: 3)

    -d, --digits-before <DIGITS_BEFORE>
            The number of digits before the words (default: 0)

    -D, --digits-after <DIGITS_AFTER>
            The number of digits after the words (default: 3)

        --debug
            Enable debug logging

    -h, --help
            Print help information

    -m, --min-length <MIN_LENGTH>
            The minimum length of each word (default: 5, min: 3)

    -M, --max-length <MAX_LENGTH>
            The maximum length of each word (default: 7, max: 9)

    -r, --match-random-char
            Do not use the same random character for each separator rather than a new random each
            time (default: true)

    -s, --separator-char <SEPARATOR_CHAR>
            The character to use to separate the words (default: "RANDOM")

    -S, --separator-alphabet <SEPARATOR_ALPHABET>
            The array of characters as separators (default: "!@$%.&*-+=?:;")

    -t, --transform <TRANSFORM>
            What transformation mode to use, Options are [NONE, CAPITALISE, ALL_EXCEPT_FIRST,
            UPPERCASE, RANDOM, ALTERNATING] (default: CAPITALISE)

    -V, --version
            Print version information

    -w, --words <WORDS>
            The number of words to generate for each password (default: 2)

SUBCOMMANDS:
    generate    Generate some new passwords.
    help        Print this message or the help of the given subcommand(s)
```

---

### Configuration file

When using the configuration file not all values must be present, the default values will be used in their place.

Below you will find the default configuration file.
```toml
words = 2
min_length = 5
max_length = 7
transform = 'CAPITALISE'
separator_char = 'RANDOM'
separator_alphabet = '!@$%.&*-+=?:;'
match_random_char = true
digits_before = 0
digits_after = 3
amount = 3
```

---

### Using a configuration file in another location
When running the generate subcommand you can specify a configuration file to use.
This path will first be treated as an absolute path and if not found looked for in the current working directory.

Some examples of this would be: 
```shell
./rpgen generate /home/racci/Documents/config.toml
./rpgen generate config.toml
./rpgen generate ../config.toml
```

---

### Rule hierarchy
When running PGen rules will be assigned with the last checked value as the final value.
Meaning that rules are assigned in an order of default, config file, supplied config file and finally cli arguments.

---

### Running from a script
Instead of writing `./rpgen` and whatever options you need, you can instead use a batch, powershell or shell file like these:
- Shell script (Assuming you have `rpgen` in your path):
```shell
#!/bin/bash
rpgen -w 10 -m 5 -r -t ALTERNATING -s RANDOM -S =-;. -d 5 -D 0 -a 50 generate ~/Documents/config
```
- Powershell / Batch script
```shell
"C:\Users\Racci\Programs\PGen\pgen" generate "C:\Users\Racci\Programs\PGen\rules.toml"
```
