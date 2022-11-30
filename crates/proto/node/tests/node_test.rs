use proto_core::{Downloadable, Executable, Installable, Proto, Resolvable, Tool, Verifiable};
use proto_node::NodeLanguage;
use std::fs;
use std::path::Path;

fn create_proto(dir: &Path) -> Proto {
    Proto {
        temp_dir: dir.join("temp"),
        tools_dir: dir.join("tools"),
    }
}

#[tokio::test]
async fn downloads_verifies_installs_tool() {
    let fixture = assert_fs::TempDir::new().unwrap();
    let proto = create_proto(fixture.path());
    let mut tool = NodeLanguage::new(&proto, Some("18.0.0"));

    tool.setup("18.0.0").await.unwrap();

    assert!(!tool.get_download_path().unwrap().exists());
    assert!(!tool.get_checksum_path().unwrap().exists());
    assert!(tool.get_install_dir().unwrap().exists());

    if cfg!(windows) {
        assert_eq!(
            tool.get_bin_path().unwrap(),
            &proto.tools_dir.join("node/18.0.0/node.exe")
        );
    } else {
        assert_eq!(
            tool.get_bin_path().unwrap(),
            &proto.tools_dir.join("node/18.0.0/bin/node")
        );
    }
}

mod downloader {
    use super::*;
    use proto_node::download::get_archive_file;

    #[tokio::test]
    async fn sets_path_to_temp() {
        let fixture = assert_fs::TempDir::new().unwrap();
        let proto = create_proto(fixture.path());
        let tool = NodeLanguage::new(&proto, Some("18.0.0"));

        assert_eq!(
            tool.get_download_path().unwrap(),
            proto
                .temp_dir
                .join("node")
                .join(get_archive_file("18.0.0").unwrap())
        );
    }

    #[tokio::test]
    async fn downloads_to_temp() {
        let fixture = assert_fs::TempDir::new().unwrap();
        let tool = NodeLanguage::new(&create_proto(fixture.path()), Some("18.0.0"));

        let to_file = tool.get_download_path().unwrap();

        assert!(!to_file.exists());

        tool.download(&to_file, None).await.unwrap();

        assert!(to_file.exists());
    }

    #[tokio::test]
    async fn doesnt_download_if_file_exists() {
        let fixture = assert_fs::TempDir::new().unwrap();
        let tool = NodeLanguage::new(&create_proto(fixture.path()), Some("18.0.0"));

        let to_file = tool.get_download_path().unwrap();

        assert!(tool.download(&to_file, None).await.unwrap());
        assert!(!tool.download(&to_file, None).await.unwrap());
    }
}

mod installer {
    use super::*;

    #[tokio::test]
    async fn sets_dir_to_tools() {
        let fixture = assert_fs::TempDir::new().unwrap();
        let proto = create_proto(fixture.path());
        let tool = NodeLanguage::new(&proto, Some("18.0.0"));

        assert_eq!(
            tool.get_install_dir().unwrap(),
            proto.tools_dir.join("node").join("18.0.0")
        );
    }

    #[tokio::test]
    #[should_panic(expected = "InstallMissingDownload(\"Node.js\")")]
    async fn errors_for_missing_download() {
        let fixture = assert_fs::TempDir::new().unwrap();
        let tool = NodeLanguage::new(&create_proto(fixture.path()), Some("18.0.0"));

        let dir = tool.get_install_dir().unwrap();

        tool.install(&dir, &tool.get_download_path().unwrap())
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn doesnt_install_if_dir_exists() {
        let fixture = assert_fs::TempDir::new().unwrap();
        let tool = NodeLanguage::new(&create_proto(fixture.path()), Some("18.0.0"));

        let dir = tool.get_install_dir().unwrap();

        fs::create_dir_all(&dir).unwrap();

        assert!(!tool
            .install(&dir, &tool.get_download_path().unwrap())
            .await
            .unwrap());
    }
}

mod resolver {
    use super::*;

    #[tokio::test]
    async fn updates_struct_version() {
        let fixture = assert_fs::TempDir::new().unwrap();
        let mut tool = NodeLanguage::new(&create_proto(fixture.path()), None);

        assert_ne!(tool.resolve_version("node", None).await.unwrap(), "node");
        assert_ne!(tool.get_resolved_version(), "node");
    }

    #[tokio::test]
    async fn resolve_latest() {
        let fixture = assert_fs::TempDir::new().unwrap();
        let mut tool = NodeLanguage::new(&create_proto(fixture.path()), None);

        assert_ne!(
            tool.resolve_version("latest", None).await.unwrap(),
            "latest"
        );
    }

    #[tokio::test]
    async fn resolve_stable() {
        let fixture = assert_fs::TempDir::new().unwrap();
        let mut tool = NodeLanguage::new(&create_proto(fixture.path()), None);

        assert_ne!(
            tool.resolve_version("stable", None).await.unwrap(),
            "stable"
        );
    }

    #[tokio::test]
    async fn resolve_lts_wild() {
        let fixture = assert_fs::TempDir::new().unwrap();
        let mut tool = NodeLanguage::new(&create_proto(fixture.path()), None);

        assert_ne!(tool.resolve_version("lts-*", None).await.unwrap(), "lts-*");
    }

    #[tokio::test]
    async fn resolve_lts_dash() {
        let fixture = assert_fs::TempDir::new().unwrap();
        let mut tool = NodeLanguage::new(&create_proto(fixture.path()), None);

        assert_ne!(
            tool.resolve_version("lts-gallium", None).await.unwrap(),
            "lts-gallium"
        );
    }

    #[tokio::test]
    async fn resolve_lts_slash() {
        let fixture = assert_fs::TempDir::new().unwrap();
        let mut tool = NodeLanguage::new(&create_proto(fixture.path()), None);

        assert_ne!(
            tool.resolve_version("lts/gallium", None).await.unwrap(),
            "lts/gallium"
        );
    }

    #[tokio::test]
    async fn resolve_alias() {
        let fixture = assert_fs::TempDir::new().unwrap();
        let mut tool = NodeLanguage::new(&create_proto(fixture.path()), None);

        assert_ne!(
            tool.resolve_version("Gallium", None).await.unwrap(),
            "Gallium"
        );
    }

    #[tokio::test]
    async fn resolve_version() {
        let fixture = assert_fs::TempDir::new().unwrap();
        let mut tool = NodeLanguage::new(&create_proto(fixture.path()), None);

        assert_eq!(
            tool.resolve_version("18.0.0", None).await.unwrap(),
            "18.0.0"
        );
    }

    #[tokio::test]
    async fn resolve_version_with_prefix() {
        let fixture = assert_fs::TempDir::new().unwrap();
        let mut tool = NodeLanguage::new(&create_proto(fixture.path()), None);

        assert_eq!(
            tool.resolve_version("v18.0.0", None).await.unwrap(),
            "18.0.0"
        );
    }

    #[tokio::test]
    #[should_panic(expected = "VersionUnknownAlias(\"lts-unknown\")")]
    async fn errors_invalid_lts() {
        let fixture = assert_fs::TempDir::new().unwrap();
        let mut tool = NodeLanguage::new(&create_proto(fixture.path()), None);

        tool.resolve_version("lts-unknown", None).await.unwrap();
    }

    #[tokio::test]
    #[should_panic(expected = "VersionUnknownAlias(\"unknown\")")]
    async fn errors_invalid_alias() {
        let fixture = assert_fs::TempDir::new().unwrap();
        let mut tool = NodeLanguage::new(&create_proto(fixture.path()), None);

        tool.resolve_version("unknown", None).await.unwrap();
    }

    #[tokio::test]
    #[should_panic(expected = "VersionResolveFailed(\"99.99.99\")")]
    async fn errors_invalid_version() {
        let fixture = assert_fs::TempDir::new().unwrap();
        let mut tool = NodeLanguage::new(&create_proto(fixture.path()), None);

        tool.resolve_version("99.99.99", None).await.unwrap();
    }
}

mod verifier {
    use super::*;

    #[tokio::test]
    async fn sets_path_to_temp() {
        let fixture = assert_fs::TempDir::new().unwrap();
        let proto = create_proto(fixture.path());
        let tool = NodeLanguage::new(&proto, Some("18.0.0"));

        assert_eq!(
            tool.get_checksum_path().unwrap(),
            proto.temp_dir.join("node").join("18.0.0-SHASUMS256.txt")
        );
    }

    #[tokio::test]
    async fn downloads_to_temp() {
        let fixture = assert_fs::TempDir::new().unwrap();
        let tool = NodeLanguage::new(&create_proto(fixture.path()), Some("18.0.0"));
        let to_file = tool.get_checksum_path().unwrap();

        assert!(!to_file.exists());

        tool.download_checksum(&to_file, None).await.unwrap();

        assert!(to_file.exists());
    }

    #[tokio::test]
    async fn doesnt_download_if_file_exists() {
        let fixture = assert_fs::TempDir::new().unwrap();
        let tool = NodeLanguage::new(&create_proto(fixture.path()), Some("18.0.0"));

        let to_file = tool.get_checksum_path().unwrap();

        assert!(tool.download_checksum(&to_file, None).await.unwrap());
        assert!(!tool.download_checksum(&to_file, None).await.unwrap());
    }

    #[tokio::test]
    #[should_panic(expected = "VerifyInvalidChecksum")]
    async fn errors_for_checksum_mismatch() {
        let fixture = assert_fs::TempDir::new().unwrap();
        let tool = NodeLanguage::new(&create_proto(fixture.path()), Some("18.0.0"));
        let dl_path = tool.get_download_path().unwrap();
        let cs_path = tool.get_checksum_path().unwrap();

        tool.download(&dl_path, None).await.unwrap();
        tool.download_checksum(&cs_path, None).await.unwrap();

        // Empty the checksum file
        fs::write(&cs_path, "").unwrap();

        tool.verify_checksum(&cs_path, &dl_path).await.unwrap();
    }
}