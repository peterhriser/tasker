# Tasker
Tasker is a task-runner/cli-as-code tool.

# Usage
Tasker uses a Taskfile, a yaml formatted file that can be used to define CLIs.

You run commands like so:
```
tasker subcommand arg1 arg2
```

## Taskfile Structure
### Tasks
A taskfile has a list of of commands under the task stanza. Each task posseses a list of commands, a name, and arguments.

When running a task, Tasker compiles commands into a shell script.

Example:
```
tasks:
    - name: my-task
      commands:
        - shell: echo hello
        - shell: echo world
      description: most basic task
```
### Commands
A command is a instruction that can have one of two types:
#### Shell
A Shell command uses the Operating Systems base shell to run a command. On Linux like systems, it is `sh` and on Windows it is `cmd`.

```
tasks:
    - name: my-task
      commands:
        - shell: echo hello
        - shell: echo world
      description: most basic task
```
#### Task
A Task command calls another task. This takes arguments supplied as well as other variables defined to parse a subscript. That subscript is then inserted into the command structure.

```
tasks:
    - name: my-task
      commands:
        - shell: echo hello
        - task: foo-bar
        - shell: echo goodbye
      description: most basic task
    - name: foo-bar
      commands:
        - shell: echo foo
        - shell: echo bar
```

This would construct a set of commands like so:
```
echo hello
echo foo
echo bar
echo goodbye
```

### Argmuent
A task can take arguments. You can take arguments and insert them into commands.

Arguments are defined in the `args` stanza. An arg has a name, a type, optionally a description, and optionally a default value.

Arguments are parsed from either CLI arguments, contexts, defaults or environment variables (formatted as `TASKER_varname`).

```
tasks:
  - name: greet
    commands:
      - shell: echo Hello ${first_name} ${last_name}
      - shell: echo Goodbye ${first_name} ${last_name}
    description: "greets a user by name"
    args:
      - name: first_name
        type: string
      - name: last_name
        type: string
        default: ", whoever you are"
```

You could then run the task like so:
```
# these equate to the same command
tasker greet Peter
TASKER_FIRST_NAME=Peter tasker greet
```

You can ommit a variable from CLI Input with a `--`.

# Planned Features
- ~support inserting variables into commands~
- ~support creating help statement~
- ~support letting people have required arguments~
- ~support globals~
- ~support self referential commands~
- ~support dry run~
- ~convert everything to try get matches~
- ~runtime/environment contexts~
- ~improve capturing shell commands to support pipes, &&, etc~
- add CWD commands
- support source variables from env variables
- add error handling configuration (on-fail: do x)
- support calling tasks from other taskfiles
- support OS context switching
- support dependency management wrappers (integrate into poetry, venv)
- command types to support: shell, task, script,
- support automatic build commands ex: detect Dockerfile exists, tasker build docker automatically works
- support tasker as a CLI build tool
    - override tasker name
    - rust library
    - compile yaml and code into dist?
- add validate file command
- support Global taskfile reference (i.e calling tasker from anywhere defaults to one in home)
- tasker setting file in home
- have args better match clap settings
  - potentially use `From<Struct>` to parse directly

# assumptions
- assume args are ordered to start

# things to check out
did not plan this but there is an app that has "Taskfile"s and uses a fairly similar yaml format. Not trying to replicate but could be a good reference for features to add: https://taskfile.dev/api/
example: namespacing