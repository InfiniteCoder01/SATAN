fn list_files_recursively(
    dir: impl AsRef<std::path::Path>,
    callback: &mut impl FnMut(&std::path::Path),
) -> std::io::Result<()> {
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            list_files_recursively(&path, callback)?;
        } else {
            callback(&path);
        }
    }
    Ok(())
}

fn main() {
    let Ok(compiler) = std::env::var("CROSS_CC") else {
        panic!("You must install a cross compiler toolchain and put path of the gcc binary into CROSS_CC environment variable!");
    };

    let mut cc = cc::Build::new();
    cc.no_default_flags(true)
        .compiler(compiler)
        .warnings(true)
        .warnings_into_errors(true)
        .flag("-ffreestanding")
        .flag("-fno-builtin")
        .flag("-nostdlib")
        .flag("-nostartfiles")
        .flag("-nodefaultlibs");

    list_files_recursively("src", &mut |file| {
        if file.extension().is_some_and(|ext| ext == "c") {
            cc.file(file);
        }
    })
    .unwrap();

    cc.compile("satan");
}
