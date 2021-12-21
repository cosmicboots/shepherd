# Shepherd

Shepherd is a git repository manager. This arises out of the personal need to
work with a collection of ever-changing git repositories, on more than one
machine.

The goal is to have a _single_ "repositories" file that can be synced
across devices. Shepherd would take this file, allow for syncing repository
configurations and bulk operations (fetch, unified log, etc).

## Building

1. Grab the source from [git.nixnet.services/boots/shepherd](https://git.nixnet.services/boots/shepherd).
2. Build using `cargo build --release`

## Basic Usage

The help message can be displayed by running `shepherd --help`.

The default path to the configuration file is `~/config/shepherd/config.toml`; however, a different location can be specified with the `--config` flag.

To add a repository to shepherd:
```
shepherd add <name> <git-url>
```

To list out currently tracked repositories:
```
shepherd list
```

To clone and update all tracked repositories:
```
shepherd fetch
```
