# Project below

> Quickly find or run commands in many projects below the current directory

As I have many projects I often do the same tasks in multiple projects (like `git fetch`).

Doing something like this is tedious:

```bash
ls
cd first
git fetch
cd ..
cd second
git fetch
cd ..
…
```

And can be done with this tool in a much simpler way:

```bash
gitBelow fetch
```

Also finding projects of a certain programming language below the current directory gets fairly easy this way:

```bash
project-below --file=Cargo.toml
```

## Basic Idea

This tool is always used in the following way:

```bash
project-below [OPTIONS] [COMMAND]...
```

With the `OPTIONS` the sub-folders are filtered.
Then in every matching folder the `COMMAND` is executed.
Check `--help` for the available filters.

For example lets run `git status` in every sub-folder which contains a `package.json` (probably some Node.js project):

```bash
project-below --file=package.json git status
```

This can be simplified with aliases like it is done in the [examples](#examples).
The first part (executable and options) always stays the same for this kind of query, only the command (or its arguments) changes.
You can put the first part in an alias and use the alias then:

```bash
alias npmBelow='project-below --file=package.json'
npmBelow git status
```

## Examples

### [git](https://git-scm.com/)

Run `git status` or `git fetch` in all git projects below the current directory:

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
cargoBelow clean
```

### [Docker](https://www.docker.com/) / [Podman](https://podman.io/)

```bash
alias dockerBelow='project-below --file=Dockerfile docker'
dockerBelow build .

alias podmanBelow='project-below --file=Dockerfile podman'
podmanBelow build .
```

### [dotnet](https://docs.microsoft.com/en-us/dotnet/core/tools/)

```bash
alias dotnetBelow='project-below --file="*.sln" dotnet'
dotnetBelow test
dotnetBelow build
dotnetBelow clean
```

### [NPM](https://www.npmjs.com/)

```bash
alias npmBelow='project-below --file=package.json npm'
npmBelow test

alias npmBelow-clean='project-below --file=package.json --directory=node_modules rm -rf node_modules'
npmBelow-clean
```

### [PKGBUILD](https://wiki.archlinux.org/title/PKGBUILD)

```bash
alias makepkgBelow='project-below --file=PKGBUILD makepkg'
makepkgBelow -f
```

### [PlatformIO](https://platformio.org/)

```bash
alias pioBelow='project-below --file=platformio.ini pio'
pioBelow test

alias pioBelow-clean='project-below --file=platformio.ini --directory=.pio rm -rf .pio'
pioBelow-clean
```

### [website-stalker](https://github.com/EdJoPaTo/website-stalker)

```bash
alias website-stalker-below='project-below --file=website-stalker.yaml website-stalker'
website-stalker-below run --all
```

### Add your own example

Feel free to add your own example via Pull Request!

Please keep `git` and `cargo` as the first examples.
After that all examples are alphabetically sorted.

## Tips

### Test everything

Test your setup first without running a command and use without command or add `echo` instead.
When building a command including for example `rm` it's wise to test it before running it.

```diff
-alias cargoBelow='project-below --file=Cargo.toml rm -rf target'
+alias cargoBelow='project-below --file=Cargo.toml'
```

```diff
-alias cargoBelow='project-below --file=Cargo.toml      rm -rf target'
+alias cargoBelow='project-below --file=Cargo.toml echo rm -rf target'
```

### Smart `cd` change directory

You can create a smart `cd` command relatively easy with `project-below` and [`fzf`](https://github.com/junegunn/fzf).
For example switching into one of the git repositories can be done like this:

```bash
alias cdg='cd "$(project-below --directory=.git | fzf)"'
cdg
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

Some tools use a pager.
For example `git` uses `less` for some commands.
`project-below` sets the environment variable `PAGER` to `cat` in order to work around this.
If you have set `GIT_PAGER` or another tool specific pager this will not help here.
For example include it into your alias:

```diff
-alias gitBelow='              project-below --directory=.git git'
+alias gitBelow='GIT_PAGER=cat project-below --directory=.git git'
```

## How did I end up creating this project?

For some use cases this problem can be solved with `find` or [`fd`](https://github.com/sharkdp/fd), but gets annoying for some package managers.
`gitBelow` for example can be realized like this:

```bash
gitBelow() {
  find . -name ".git" -type d -print -execdir git --no-pager $@ \;
}

gitBelow fetch
```

Sadly this approach lacks auto-completion of `git` commands: `gitBelow stat<Tab>` does not auto-complete to `gitBelow status`.
But this is only an annoyance.

For commands like `npm` it's not even simple to come up with a useful solution.
`find` does not use ignore files and ends up with all the `package.json` files in the `node_modules` folder (which is often a lot).
`fd` on the other hand uses ignore files but `--exec` is always executed from the working directory from where the command was started.
`git` for example has `-C` to use another path, but tools like `npm` do not.
This requires some workarounds like spawning a `bash` with a `cd` command at first.
It works but there won't be auto-completion of commands either.
Also, it creates a lot of bash / alias dark magic my future me wants to understand.
It is also way harder to adapt to new use cases or other dependency managers.

In turn, I created this tool which helps me to do exactly what I need in a simple way.

As this tool uses the same directory walker like [`fd`](https://github.com/sharkdp/fd) or [`rg`](https://github.com/BurntSushi/ripgrep) it's way faster than `find` can ever be and uses smart features like ignore files, it skips hidden folders and so on.
