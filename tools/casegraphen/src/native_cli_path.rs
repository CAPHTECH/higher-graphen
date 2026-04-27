use higher_graphen_core::Id;
use std::path::Path;

pub(super) fn path_segment(id: &Id) -> String {
    let mut segment = String::new();
    for byte in id.as_str().bytes() {
        match byte {
            b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'-' | b'_' => {
                segment.push(byte as char);
            }
            _ => segment.push_str(&format!("~{byte:02x}")),
        }
    }
    segment
}

pub(super) fn relative_store_path(store: &Path, path: &Path) -> String {
    path.strip_prefix(store)
        .unwrap_or(path)
        .display()
        .to_string()
}

pub(super) fn id_lossy(value: &str) -> Id {
    Id::new(value.to_owned()).expect("static id is valid")
}
