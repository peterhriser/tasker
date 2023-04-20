# Tasker
Tasker is a task-runner/cli-as-code tool.

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
- ~support source variables from env variables~
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

# assumptions
- assume args are ordered to start

# things to check out
did not plan this but there is an app that has "Taskfile"s and uses a fairly similar yaml format. Not trying to replicate but could be a good reference for features to add: https://taskfile.dev/api/
example: namespacing