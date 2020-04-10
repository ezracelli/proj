# proj

A CLI utility for navigating between multiple projects.

## Installation

```bash
git clone git@github.com:ezracelli/proj.git
cd proj
cargo install --path .
```

## Usage

```bash
# add a project
proj add <NAME> <DIR>
proj a <NAME> <DIR>

# list projects
proj list [<NAME>...]
proj ls [<NAME>...]

# cd to a project
proj go <NAME>
proj g <NAME>

# open a project in vscode
proj open <NAME>
proj o <NAME>

# remove a project
proj remove <NAME>...
proj rm <NAME>...
```
