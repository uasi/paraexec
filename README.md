# paraexec

paraexec (Parallel exec) is a command-line tool that allows you to run multiple commands in parallel, with organized output display.

## Usage

```
paraexec ( <separator> [<label>/] [<ENV>=<value>...] <command> [<argument>...] )+
```

- `<separator>`: A string used to separate different commands
- `<label>/`: (Optional) A custom label for the command output (defaults to `<command>`)
- `<ENV>=<value>`: (Optional) Environment variables for the command
- `<command>`: The command to execute
- `<argument>`: (Optional) Arguments for the command

## Examples

1. Run two simple commands in parallel:

```
$ paraexec :: echo Hello :: echo World
echo | Hello
echo | World
echo = exit status: 0
echo = exit status: 0
```

2. Run commands with custom labels and environment variables:

```
$ paraexec ,, frontend/ NODE_ENV=production npm run build ,, backend/ cargo build --release
frontend | asset main.js 142 bytes [compared for emit] [minimized] (name: main)
backend  |    Compiling backend v0.1.0 (/app/backend)
frontend | webpack 5.1.0 compiled successfully in 198 ms
frontend = exit status: 0
backend  |     Finished `release` profile [optimized] target(s) in 10.55s
backend  = exit status: 0
```

3. stdout/stderr and exit statuses:

```
$ paraexec :: sh -c 'echo stdout ; echo stderr >&2' :: false
false = exit status: 1
sh    | stdout
sh    * stderr
sh    = exit status: 0
$ echo $?
1
```

## Installation

```
$ cargo build --release
$ cp target/release/paraexec /path/to/bin/
```
