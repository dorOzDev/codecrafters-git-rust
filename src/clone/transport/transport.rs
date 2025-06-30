use std::io::{self, Result};

use git_url_parse::GitUrl;


enum Protocol {
    HTTP,
    SSH,
}

fn detect_protoctol(git_url : GitUrl) -> Protocol {
    match git_url.scheme {
        git_url_parse::Scheme::Http => {
            return Protocol::HTTP
        }
        git_url_parse::Scheme::Https => {
            return Protocol::HTTP
        }
        git_url_parse::Scheme::Ssh => {
            return Protocol::SSH
        }
        _ => {
            panic!("Unsupported protocol: {}", git_url.scheme)
        },
    }
}

pub async fn clone_git(repo_url: &str) -> Result<()> {
    let git_url = GitUrl::parse(repo_url).map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?;
    println!("{:#?}", git_url);
    
    Ok(())
}
