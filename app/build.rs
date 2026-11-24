fn main() {
    slint_build::compile("ui/main.slint").expect("gagal mengompilasi ui/main.slint");

    // Sematkan ikon + info versi ke gravi-island.exe (hanya di Windows, dan hanya bila ikon ada).
    #[cfg(windows)]
    {
        let icon = std::path::Path::new("../assets/gravi.ico");
        if icon.exists() {
            let mut r = winresource::WindowsResource::new();
            r.set_icon(icon.to_str().unwrap());
            r.set("ProductName", "Gravi Island");
            r.set("FileDescription", "Gravi Island");
            let _ = r.compile();
        }
    }
    println!("cargo:rerun-if-changed=ui");
    println!("cargo:rerun-if-changed=../assets/gravi.ico");
}
