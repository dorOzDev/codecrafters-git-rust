use crate::{hash::GitHash, objects::{write_object, Person}};


pub struct Commit {
    pub tree: GitHash,
    pub parent_tree: Option<GitHash>,
    pub message: String,
    pub committer: Person,
    pub author: Person, 
}

impl Commit {
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut lines = vec![format!("tree {}", self.tree.to_hex())];
        
        if let Some(ref parent) = self.parent_tree {
            lines.push(format!("parent {}", parent.to_hex()));
        }

        lines.push(format!("author {}", self.author));
        lines.push(format!("committer {}", self.committer));
        lines.push(String::new());
        lines.push(self.message.clone());

        lines.join("\n").into_bytes()
    }
}

pub fn process_commit(commit: &Commit) -> std::io::Result<GitHash> {
    let content = commit.to_bytes();
    return write_object(super::ObjectType::Commit, &content);
}