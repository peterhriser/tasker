# todo
- ~support inserting variables into commands~
- ~support creating help statement~
- ~support letting people have required arguments~
- ~support globals~
- support source variables from env variables
- support self referential commands
- support calling sub-configs
- support OS context switching
- support dependency management wrappers (integrate into poetry, venv)
- command types to support: one line command, script,
- support automatic build commands ex: detect Dockerfile exists, tasker build docker automatically works
- runtime/environment contexts
# assumptions
- assume args are ordered to start

# things to check out
did not plan this but there is an app that has "Taskfile"s and uses a fairly similar yaml format. Not trying to replicate but could be a good reference for features to add: https://taskfile.dev/api/
example: namespacing