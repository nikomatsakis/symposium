fn main() {
    let manifest_dir = std::env::current_dir().expect("failed to get current directory");
    let out_dir = manifest_dir.join("target").join("artifacts");

    let result = symposium_artifacts::assemble(&manifest_dir, &out_dir);

    for entry in std::fs::read_dir(&result.artifacts_dir).unwrap() {
        let entry = entry.unwrap();
        println!("{}", entry.path().display());
    }
}
