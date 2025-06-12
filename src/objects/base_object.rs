
pub enum ObjectType {
    Blob,
    Tree,
}

impl ObjectType {
    pub fn as_str(&self) -> &'static str {
        match self {
            ObjectType::Blob => "blob",
            ObjectType::Tree => "tree",
        }
    }
}

pub fn encode_object(obj_type: ObjectType, data: &[u8]) -> Vec<u8> {
    let header = format!("{} {}\0", obj_type.as_str(), data.len());
    let mut result = Vec::with_capacity(header.len() + data.len());
    result.extend(header.as_bytes());
    result.extend(data);
    result
}