# OCD - OfflineCodeTester
An offline alternative to [SimpleCodeTester](https://github.com/I-Al-Istannen/SimpleCodeTester) for everyone who is too afraid to write an E-Mail.

## Rust > Java
SimpleCodeTester is written in Java hence being very slow.
OfflineCodeTester is written in Rust hence being very fast.

## Running
When you run ocd, it looks for a file called `ocd.toml` in the current directory.

`ocd.toml` should contain the following information

```toml
class_path = "path/to/your/java/root" # For an intellij project this should be "out/production/<your-project-name>"
main_class = "path.to.your.Main"

[interaction]
path = "path/to/your/interactions" # Path to the folder containing your interactions
pattern = [ ".*\.txt" ] # An array of Regex patterns a filename needs to fully match to be considered an interaction file. Optional

[runner] # Optional
thread_count = 0 # The number of interactions to run in parallel. Optional
timeout = 1000 # How long (in milliseconds) ocd should wait for your java programm to respond. Optional
```
The default values of all optional fields are the ones defined here

To run ocd for your project, add a `ocd.toml` file to the source directory and run `ocd` through the command line.

```sh
/path/to/your/project/source$ /path/to/ocd
```

### Known issues
- Running on Windows? (didn't test it there, would be strange if everything worked out of the box)