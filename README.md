# Project below

> Quickly run commands in many projects below the current directory

As I have many projects I often do the same tasks in multiple projects (like `git fetch` or `git status --short --branch`).

For some use cases this works fine with `find` or [`fd`](https://github.com/sharkdp/fd), but gets annoying for some package managers.
`gitBelow` for example can be realized like this:

```bash
gitBelow() {
  find . -name ".git" -type d -print -execdir git --no-pager $@ \;
}

gitBelow fetch
```

Sadly this approach lacks autocompletion of `git` commands: `gitBelow stat<Tab>` does not autocomplete to `gitBelow status`.
But this is only an annoyance.

For commands like `npm` it's not even simple to come up with a useful solution.
`find` does not use ignore files and ends up with all the `package.json` files in the `node_modules` folder (which is often a lot).
`fd` on the other hand uses ignore files but `--exec` is always executed from the working directory from where the command was started.
`git` for example has `-C` to use another path, but tools like `npm` do not.
This requires some workarounds like spawning a `bash` with a `cd` command at first.
It works but there won't be auto-completion of commands either.
Also, it creates a lot of bash / alias dark magic my future me wants to understand or simply adapt to other dependency managers.

In turn, I created this tool which helps me to do exactly what I need in a simple way.

As this tool uses the same directory walker like [`fd`](https://github.com/sharkdp/fd) or [`rg`](https://github.com/BurntSushi/ripgrep) it's way faster than `find` can ever be and uses smart features like ignore files, it skips hidden folders, …

## Examples

### [git](https://git-scm.com/)

Show all `git status` in git projects blow the current directory:

```bash
alias gitBelow='project-below --directory=.git git'
gitBelow status
gitBelow status --short --branch
gitBelow fetch
```

### [cargo](https://doc.rust-lang.org/cargo/)

```bash
alias cargoBelow='project-below --file=Cargo.toml cargo'
cargoBelow check
```

### [NPM](https://www.npmjs.com/)

```bash
alias npmBelow='project-below --file=package.json npm'
npmBelow test

alias npmBelow-clean='project-below --file=package.json --directory=node_modules rm -rf node_modules'
npmBelow-clean
```

### [PlatformIO](https://platformio.org/)

```bash
alias pioBelow='project-below --file=platformio.ini pio'
pioBelow test

alias pioBelow-clean='project-below --file=platformio.ini --directory=.pio rm -rf .pio'
pioBelow-clean
```

### Add your own example

Feel free to add your own example via Pull Request!

Please keep `git` and `cargo` as the first examples.
After that all examples are alphabetically sorted.

## Tips

### Test everything

Test your setup first with a simple command like `pwd`.
When building a command including rm for example it's wise to test it before running it.

```diff
-alias cargoBelow='project-below --file=Cargo.toml rm -rf target'
+alias cargoBelow='project-below --file=Cargo.toml pwd'
```

### Use `nice`

Builds on a smaller machine with not as much computing power are annoying to run in the background.
`nice` helps.
You can include it in your alias and all the commands will run via `nice`:

```diff
-alias cargoBelow='project-below --file=Cargo.toml      cargo'
+alias cargoBelow='project-below --file=Cargo.toml nice cargo'
```

### PAGER

Some tools use a pager like `git` uses `less` for some commands.
This tool sets the environment variable `PAGER` to `cat` in order to work around this.
If you have set `GIT_PAGER` or another tool specific pager this will not help here.
For example include it into your alias:

```diff
-alias gitBelow='              project-below --directory=.git git'
+alias gitBelow='GIT_PAGER=cat project-below --directory=.git git'
```
