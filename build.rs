#[cfg(windows)]
fn build_windows() {
    let file = "src/platform/windows.cc";
    let file2 = "src/platform/windows_delete_test_cert.cc";
    cc::Build::new().file(file).file(file2).compile("windows");
    println!("cargo:rustc-link-lib=WtsApi32");
    println!("cargo:rerun-if-changed={}", file);
    println!("cargo:rerun-if-changed={}", file2);
}

#[cfg(target_os = "macos")]
fn build_mac() {
    let file = "src/platform/macos.mm";
    let mut b = cc::Build::new();
    if let Ok(os_version::OsVersion::MacOS(v)) = os_version::detect() {
        let v = v.version;
        if v.contains("10.14") {
            b.flag("-DNO_InputMonitoringAuthStatus=1");
        }
    }
    b.flag("-std=c++17").file(file).compile("macos");
    println!("cargo:rerun-if-changed={}", file);
}

#[cfg(all(windows, feature = "inline"))]
fn build_manifest() {
    use std::io::Write;
    if std::env::var("PROFILE").unwrap() == "release" {
        let mut res = winres::WindowsResource::new();
        res.set_icon("res/icon.ico")
            .set_language(winapi::um::winnt::MAKELANGID(
                winapi::um::winnt::LANG_ENGLISH,
                winapi::um::winnt::SUBLANG_ENGLISH_US,
            ))
            .set_manifest_file("res/manifest.xml");
        match res.compile() {
            Err(e) => {
                write!(std::io::stderr(), "{}", e).unwrap();
                std::process::exit(1);
            }
            Ok(_) => {}
        }
    }
}

fn install_android_deps() {
    let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap();
    if target_os != "android" {
        return;
    }
    let mut target_arch = std::env::var("CARGO_CFG_TARGET_ARCH").unwrap();
    if target_arch == "x86_64" {
        target_arch = "x64".to_owned();
    } else if target_arch == "x86" {
        target_arch = "x86".to_owned();
    } else if target_arch == "aarch64" {
        target_arch = "arm64".to_owned();
    } else {
        target_arch = "arm".to_owned();
    }
    let target = format!("{}-android", target_arch);
    let vcpkg_root = std::env::var("VCPKG_ROOT").unwrap();
    let mut path: std::path::PathBuf = vcpkg_root.into();
    if let Ok(vcpkg_root) = std::env::var("VCPKG_INSTALLED_ROOT") {
        path = vcpkg_root.into();
    } else {
        path.push("installed");
    }
    path.push(target);
    println!(
        "cargo:rustc-link-search={}",
        path.join("lib").to_str().unwrap()
    );
    println!("cargo:rustc-link-lib=ndk_compat");
    println!("cargo:rustc-link-lib=oboe");
    println!("cargo:rustc-link-lib=c++");
    println!("cargo:rustc-link-lib=OpenSLES");
}

fn main() {
    hbb_common::gen_version();
    install_android_deps();
    #[cfg(all(windows, feature = "inline"))]
    build_manifest();
    #[cfg(windows)]
    build_windows();
    let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap();
    if target_os == "macos" {
        #[cfg(target_os = "macos")]
        build_mac();
        println!("cargo:rustc-link-lib=framework=ApplicationServices");
    }
    
    // Pass environment variables to rustc for option_env!() macro
    eprintln!("[BUILD.RS DEBUG] Checking environment variables...");
    if let Ok(server) = std::env::var("RENDEZVOUS_SERVER") {
        eprintln!("[BUILD.RS] RENDEZVOUS_SERVER found: {} chars", server.len());
        println!("cargo:rustc-env=RENDEZVOUS_SERVER={}", server);
    } else {
        eprintln!("[BUILD.RS] RENDEZVOUS_SERVER: NOT SET");
    }
    if let Ok(key) = std::env::var("RS_PUB_KEY") {
        eprintln!("[BUILD.RS] RS_PUB_KEY found: {} chars", key.len());
        println!("cargo:rustc-env=RS_PUB_KEY={}", key);
    } else {
        eprintln!("[BUILD.RS] RS_PUB_KEY: NOT SET");
    }
    if let Ok(api) = std::env::var("API_SERVER") {
        eprintln!("[BUILD.RS] API_SERVER found: {} chars", api.len());
        println!("cargo:rustc-env=API_SERVER={}", api);
    } else {
        eprintln!("[BUILD.RS] API_SERVER: NOT SET");
    }
    if let Ok(pwd) = std::env::var("RS_PASSWORD") {
        eprintln!("[BUILD.RS] RS_PASSWORD found: {} chars", pwd.len());
        println!("cargo:rustc-env=RS_PASSWORD={}", pwd);
    } else {
        eprintln!("[BUILD.RS] RS_PASSWORD: NOT SET");
    }
    if let Ok(relay) = std::env::var("RS_FORCE_RELAY") {
        eprintln!("[BUILD.RS] RS_FORCE_RELAY found: {}", relay);
        println!("cargo:rustc-env=RS_FORCE_RELAY={}", relay);
    } else {
        eprintln!("[BUILD.RS] RS_FORCE_RELAY: NOT SET");
    }
    
    println!("cargo:rerun-if-changed=build.rs");
}
