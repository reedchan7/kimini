const INFO_PLIST: &str = include_str!("../packaging/macos/Info.plist");
const PACKAGE_LINUX: &str = include_str!("../scripts/package-linux.sh");
const PACKAGE_WINDOWS: &str = include_str!("../scripts/package-windows.ps1");
const PUBLISH_RELEASE: &str = include_str!("../scripts/publish-release.sh");
const SHIP_SH: &str = include_str!("../scripts/ship.sh");
const SHIP_SKILL: &str = include_str!("../.agents/skills/ship/SKILL.md");

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

#[test]
fn ship_pipeline_keeps_high_severity_safety_rails() {
    // Skill: no tags, true dry-run semantics, no skip-build / auto-stash.
    assert!(SHIP_SKILL.contains("**Do not** create or push tags here"));
    assert!(SHIP_SKILL.contains("Read-only plan only"));
    assert!(SHIP_SKILL.contains("--dry-run"));
    assert!(SHIP_SKILL.contains("`--skip-build` from ship (not exposed)"));
    assert!(!SHIP_SKILL.contains("git stash"));
    assert!(SHIP_SKILL.contains("require-canonical-remote"));

    // Deterministic helpers
    assert!(SHIP_SH.contains("Read-only: never fetch"));
    assert!(SHIP_SH.contains("never invoke cargo"));
    assert!(SHIP_SH.contains("require_canonical_remote"));
    assert!(SHIP_SH.contains("require_no_in_progress_git_op"));
    assert!(SHIP_SH.contains("publish requires --expected-sha"));
    assert!(SHIP_SH.contains("semver_core_cmp"));
    assert!(SHIP_SH.contains("latest_github_release_version"));
    assert!(SHIP_SH.contains("semver_gt"));
    assert!(SHIP_SH.contains(r"git@github\.com:|https://github\.com/"));
    assert!(!SHIP_SH.contains("SHIP_CANONICAL_REPO:-"));
    assert!(SHIP_SH.contains(r#"CANONICAL_REPO="reedchan7/kimini""#));

    // publish-release hardening used by ship
    assert!(PUBLISH_RELEASE.contains("SHIP_REQUIRE_CLEAN"));
    assert!(PUBLISH_RELEASE.contains("SHIP_EXPECTED_SHA"));
    assert!(PUBLISH_RELEASE.contains("SHIP_REMOTE"));
    assert!(PUBLISH_RELEASE.contains("refs/tags/${TAG}^{commit}"));
    assert!(PUBLISH_RELEASE.contains("working tree became dirty after packaging"));
    assert!(PUBLISH_RELEASE.contains("peeled commit could not be proven"));
}
