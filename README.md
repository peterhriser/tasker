# Tasker
Tasker is a task-runner/cli-as-code tool.

# Planned Features
- ~support inserting variables into commands~
- ~support creating help statement~
- ~support letting people have required arguments~
- ~support globals~
- ~support self referential commands~
- support source variables from env variables
- support calling tasks from other taskfiles
- support OS context switching
- support dependency management wrappers (integrate into poetry, venv)
- command types to support: shell, task, script,
- support automatic build commands ex: detect Dockerfile exists, tasker build docker automatically works
- ~runtime/environment contexts~
- support tasker as a CLI build tool
    - override tasker name
    - rust library
    - compile yaml and code into dist?
- add error handling configuration (on-fail: do x)
- ~dry run support~
- convert everything to try get matches
- add CWD commands
# assumptions
- assume args are ordered to start

# things to check out
did not plan this but there is an app that has "Taskfile"s and uses a fairly similar yaml format. Not trying to replicate but could be a good reference for features to add: https://taskfile.dev/api/
example: namespacing