use std::path::PathBuf;

use anyhow::{Result, bail};
use git2::{Cred, RemoteCallbacks, Repository};

enum CloneType {
    HTTPS(String),
    SSH {
        url: String,
        private_key_path: PathBuf,
        public_key_path: Option<PathBuf>,
        passphrase: Option<String>,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let clone = CloneType::SSH {
        url: "git@github.com:neovim/nvim-lspconfig.git".to_string(),
        private_key_path: "/home/lcrown/.ssh/keys/github/id_rsa".into(),
        public_key_path: None,
        passphrase: Some("my-ssh-passphrase".to_string()),
    };
    let clone2 = CloneType::HTTPS("https://github.com/neovim/nvim-lspconfig".to_string());
    let _ = shallow_clone_repository(clone).await?;
    let _ = shallow_clone_repository(clone2).await?;
    Ok(())
}

fn repository_slug(url: &str) -> Result<String> {
    let parts = url.split("/").collect::<Vec<&str>>();
    if parts.len() < 2 {
        bail!("Malformed repository URL")
    }
    let project = parts[parts.len() - 2].split(":").last().unwrap();
    let reponame = parts.last().unwrap().replace(".git", "");
    return Ok(format!("{}-{}", project, reponame));
}

async fn shallow_clone_repository(clone: CloneType) -> Result<Repository> {
    let url = match clone {
        CloneType::HTTPS(ref url) => url.clone(),
        CloneType::SSH { ref url, .. } => url.clone(),
    };
    let slug = repository_slug(&url)?;
    let repo_path = PathBuf::from(format!("/tmp/repos/{}", slug));
    let callbacks = match clone {
        CloneType::HTTPS(_) => RemoteCallbacks::new(),
        CloneType::SSH {
            ref private_key_path,
            ref public_key_path,
            ref passphrase,
            ..
        } => {
            let mut callbacks = RemoteCallbacks::new();
            callbacks.credentials(|_, username_from_url, _| {
                Cred::ssh_key(
                    username_from_url.unwrap(),
                    public_key_path.as_deref(),
                    private_key_path,
                    passphrase.as_deref(),
                )
            });
            callbacks
        }
    };
    let mut fo = git2::FetchOptions::new();
    fo.depth(1);
    fo.remote_callbacks(callbacks);
    let mut builder = git2::build::RepoBuilder::new();
    builder.fetch_options(fo);

    Ok(builder.clone(&url, &repo_path)?)
}
