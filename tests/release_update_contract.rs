const INFO_PLIST: &str = include_str!("../packaging/macos/Info.plist");
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
