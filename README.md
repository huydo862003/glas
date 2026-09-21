# glas

A git multiplexer - sync repos across multiple providers.

## Install

```sh
# With Nix
nix profile install github:huydo862003/git-las

# From source
cargo install --path crates/glas
```

## Usage

```sh
# Initialize a workspace
glas init
glas init --meta-repo my-config   # custom meta-repo name (default: glas)

# Add remotes
glas remote add github https://github.com/myorg
glas remote add gitlab https://gitlab.com/myorg
glas remote add github https://github.com/myorg --token ghp_xxx

# List and remove remotes
glas remote list
glas remote remove gitlab

# Track repos
cd my-repo && glas repo add
glas repo add /path/to/external/repo   # symlinks into workspace

# Set which remote a repo pulls from
glas repo set primary github

# List tracked repos
glas repo list
glas repo status

# Sync all repos
glas push
glas pull

# Sync a single repo
glas push my-repo
glas pull my-repo
```

## How it works

`glas init` creates a `.glas/` directory that acts as both the workspace marker and a meta-repo (itself a git repo that gets synced). Configuration lives in `.glas/config.toml`. Repos tracked by the workspace live as siblings of `.glas/` in the workspace root. External repos are symlinked in.

Remote definitions are stored in the workspace config. Tokens are always stored globally in `~/.config/glas/secrets.toml` so credentials are never committed.

## Token resolution order

1. Environment variable `GLAS_TOKEN_<REMOTE>` (e.g. `GLAS_TOKEN_GITHUB`)
2. Global secrets file `~/.config/glas/secrets.toml`
3. Git config `glas.remote.<name>.token`
4. Provider CLI tool (gh, glab, tea, bb)
