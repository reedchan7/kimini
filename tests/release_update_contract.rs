const INFO_PLIST: &str = include_str!("../packaging/macos/Info.plist");
const PACKAGE_LINUX: &str = include_str!("../scripts/package-linux.sh");
const PACKAGE_WINDOWS: &str = include_str!("../scripts/package-windows.ps1");
const PUBLISH_RELEASE: &str = include_str!("../scripts/publish-release.sh");

#[test]
fn release_pipeline_keeps_updates_signed_for_every_app_and_architecture() {
    for key in [
        "SUFeedURL",
        "SUPublicEDKey",
        "SUEnableAutomaticChecks",
        "SUAutomaticallyUpdate",
        "SUVerifyUpdateBeforeExtraction",
    ] {
        assert!(INFO_PLIST.contains(key), "missing Sparkle setting {key}");
    }

    assert!(PUBLISH_RELEASE.contains("sign-sparkle-update.sh"));
    for feed in [
        "Kimini-macos-aarch64.xml",
        "Kimini-macos-x86_64.xml",
        "Kimini-Web-macos-aarch64.xml",
        "Kimini-Web-macos-x86_64.xml",
    ] {
        assert!(PUBLISH_RELEASE.contains(feed), "missing update feed {feed}");
    }
}

#[test]
fn portable_release_matrix_is_complete_and_architecture_specific() {
    assert!(PUBLISH_RELEASE.contains("--include-portable"));
    for platform in ["linux", "windows"] {
        for app in ["Kimini", "Kimini-Web"] {
            for arch in ["aarch64", "x86_64"] {
                let extension = if platform == "linux" { "tar.gz" } else { "zip" };
                let asset = format!("{app}-${{VERSION}}-{platform}-{arch}.{extension}");
                assert!(PUBLISH_RELEASE.contains(&asset), "missing asset {asset}");
            }
        }
    }
    assert!(PACKAGE_LINUX.contains("cargo build --locked --release"));
    assert!(PACKAGE_LINUX.contains("$(basename \"$archive\")"));
    assert!(PACKAGE_WINDOWS.contains("cargo build --locked --release"));
    assert!(PUBLISH_RELEASE.contains("shasum -a 256 -c"));
}
