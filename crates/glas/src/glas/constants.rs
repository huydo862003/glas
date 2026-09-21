pub const GLAS_DIR: &str = ".glas";
pub const GLAS_LOCAL_PATH: &str = ".glas/.local";
pub const CONFIG_FILE: &str = "config.toml";
pub const SECRETS_PATH: &str = ".glas/.local/secrets.toml";
pub const LOCAL_GITIGNORE: &str = r#"*
!.gitignore
"#;

// Global config lives in $HOME/.config/glas/
pub const GLOBAL_CONFIG_SUBDIR: &str = ".config/glas";
pub const GLOBAL_SECRETS_FILE: &str = "secrets.toml";

// File headers written at the top of generated TOML files
pub const CONFIG_HEADER: &str = r#"# glas workspace config
#
# [meta]
# repo = "glas"           # name of the .glas/ meta-repo on remotes
#
# [remotes.<name>]          # declare a hosting provider (name = provider, e.g. github)
# url  = "https://github.com"
# user = "myorg"            # default user/org for all repos on this remote
#
# [repo.<name>]             # track a repo in this workspace
# [repo.<name>.primary]     # remote to pull from
# name = "github"
# [[repo.<name>.remotes]]   # additional push targets (defaults to all remotes)
# name = "gitlab"
#
# Run `glas remote add --help` and `glas repo --help` for details

"#;

pub const SECRETS_HEADER: &str = r#"# glas secrets - DO NOT COMMIT
# Tokens for each remote, keyed by remote name
#
# [remotes.<name>]
# token = "ghp_..."

"#;
