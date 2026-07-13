fn main() {
    println!("cargo:rerun-if-changed=assets/markion-logo.svg");
    println!("cargo:rerun-if-changed=assets/markion.ico");

    #[cfg(windows)]
    {
        let mut resource = winresource::WindowsResource::new();
        resource.set_icon("assets/markion.ico");
        resource.set("FileDescription", "Markion Markdown Editor");
        resource.set("ProductName", "Markion");
        resource
            .compile()
            .expect("failed to embed Windows application icon");
    }
}
