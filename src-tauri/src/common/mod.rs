//! What the modules share without depending on each other or on the app (ADR-029): Mexican identifiers, dates and
//! the shape of contact data. It depends on nothing.

pub mod contact;
pub mod dates;
pub mod ids;

#[cfg(test)]
mod tests {
    /// `common` depends on nothing of the app: the modules can share it and still leave on their own.
    #[test]
    fn common_depends_on_nothing_of_the_app() {
        for e in std::fs::read_dir(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src/common")).unwrap() {
            let text = std::fs::read_to_string(e.unwrap().path()).unwrap();
            for line in text.lines().filter(|l| l.contains(concat!("crate", "::"))) {
                assert!(line.contains(concat!("crate", "::common")), "common reaches into the app: {line}");
            }
        }
    }
}
