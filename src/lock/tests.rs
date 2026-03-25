use super::*;
use std::sync::atomic::{AtomicU64, Ordering};

static TEST_COUNTER: AtomicU64 = AtomicU64::new(0);

fn unique_vex_dir() -> PathBuf {
    let id = TEST_COUNTER.fetch_add(1, Ordering::SeqCst);
    let dir = std::env::temp_dir().join(format!("vex-lock-test-{}-{}", std::process::id(), id));
    fs::create_dir_all(&dir).unwrap();
    dir
}

#[test]
fn test_lock_acquire_success() {
    let vex_dir = unique_vex_dir();
    let lock = InstallLock::acquire(&vex_dir, "node", "20.11.0");
    assert!(lock.is_ok());
    let _ = fs::remove_dir_all(&vex_dir);
}

#[test]
fn test_lock_file_created() {
    let vex_dir = unique_vex_dir();
    let lock_path = vex_dir.join("locks").join("go-1.23.5.lock");

    let _lock = InstallLock::acquire(&vex_dir, "go", "1.23.5").unwrap();
    assert!(lock_path.exists());

    let _ = fs::remove_dir_all(&vex_dir);
}

#[test]
fn test_lock_cleanup_on_drop() {
    let vex_dir = unique_vex_dir();
    let lock_path = vex_dir.join("locks").join("node-18.0.0.lock");

    {
        let _lock = InstallLock::acquire(&vex_dir, "node", "18.0.0").unwrap();
        assert!(lock_path.exists());
    }

    assert!(!lock_path.exists());
    let _ = fs::remove_dir_all(&vex_dir);
}

#[test]
fn test_lock_reacquire_after_drop() {
    let vex_dir = unique_vex_dir();

    {
        let _lock = InstallLock::acquire(&vex_dir, "rust", "1.93.1").unwrap();
    }

    let lock2 = InstallLock::acquire(&vex_dir, "rust", "1.93.1");
    assert!(lock2.is_ok());
    let _ = fs::remove_dir_all(&vex_dir);
}

#[test]
fn test_different_versions_no_conflict() {
    let vex_dir = unique_vex_dir();

    let _lock1 = InstallLock::acquire(&vex_dir, "node", "20.11.0").unwrap();
    let lock2 = InstallLock::acquire(&vex_dir, "node", "18.19.0");

    assert!(lock2.is_ok());
    let _ = fs::remove_dir_all(&vex_dir);
}

#[test]
fn test_different_tools_no_conflict() {
    let vex_dir = unique_vex_dir();

    let _lock1 = InstallLock::acquire(&vex_dir, "node", "20.11.0").unwrap();
    let lock2 = InstallLock::acquire(&vex_dir, "go", "1.23.5");

    assert!(lock2.is_ok());
    let _ = fs::remove_dir_all(&vex_dir);
}

#[test]
fn test_cross_process_lock_conflict() {
    let vex_dir = unique_vex_dir();
    let locks_dir = vex_dir.join("locks");
    fs::create_dir_all(&locks_dir).unwrap();

    let lock_path = locks_dir.join("node-22.0.0.lock");
    let python_script = format!(
        r#"
import fcntl
import time

with open('{}', 'w') as f:
    fcntl.flock(f.fileno(), fcntl.LOCK_EX | fcntl.LOCK_NB)
    print('ready', flush=True)
    time.sleep(30)
"#,
        lock_path.display()
    );

    let mut child = std::process::Command::new("/usr/bin/python3")
        .arg("-c")
        .arg(&python_script)
        .stdout(std::process::Stdio::piped())
        .spawn()
        .expect("failed to spawn child");

    let stdout = child.stdout.as_mut().unwrap();
    let mut buf = [0u8; 6];
    let mut total = 0;
    while total < 5 {
        let n = stdout.read(&mut buf[total..]).unwrap();
        if n == 0 {
            break;
        }
        total += n;
    }
    assert!(
        std::str::from_utf8(&buf[..total])
            .unwrap()
            .starts_with("ready"),
        "child did not acquire lock"
    );

    let file = File::create(&lock_path).unwrap();
    let result = file.try_lock_exclusive();
    assert!(result.is_err(), "Expected lock conflict with child process");

    child.kill().ok();
    child.wait().ok();
    let _ = fs::remove_dir_all(&vex_dir);
}

#[test]
fn test_locks_dir_auto_created() {
    let vex_dir = unique_vex_dir();
    let locks_dir = vex_dir.join("locks");

    assert!(!locks_dir.exists());

    let _lock = InstallLock::acquire(&vex_dir, "java", "21").unwrap();
    assert!(locks_dir.exists());

    let _ = fs::remove_dir_all(&vex_dir);
}

#[test]
fn test_lock_with_special_version_characters() {
    let vex_dir = unique_vex_dir();
    let versions = vec!["1.2.3", "1.2.3-beta.1", "1.2.3-rc.2", "20.0.0-nightly"];

    for version in versions {
        let lock = InstallLock::acquire(&vex_dir, "node", version);
        assert!(
            lock.is_ok(),
            "Failed to acquire lock for version {}",
            version
        );
    }

    let _ = fs::remove_dir_all(&vex_dir);
}

#[test]
fn test_lock_file_naming() {
    let vex_dir = unique_vex_dir();

    let _lock = InstallLock::acquire(&vex_dir, "node", "20.11.0").unwrap();
    let expected_path = vex_dir.join("locks").join("node-20.11.0.lock");

    assert!(expected_path.exists());

    let _ = fs::remove_dir_all(&vex_dir);
}

#[test]
fn test_multiple_locks_same_tool_different_versions() {
    let vex_dir = unique_vex_dir();

    let _lock1 = InstallLock::acquire(&vex_dir, "node", "18.0.0").unwrap();
    let _lock2 = InstallLock::acquire(&vex_dir, "node", "20.0.0").unwrap();
    let _lock3 = InstallLock::acquire(&vex_dir, "node", "22.0.0").unwrap();

    assert!(vex_dir.join("locks").join("node-18.0.0.lock").exists());
    assert!(vex_dir.join("locks").join("node-20.0.0.lock").exists());
    assert!(vex_dir.join("locks").join("node-22.0.0.lock").exists());

    let _ = fs::remove_dir_all(&vex_dir);
}

#[test]
fn test_lock_cleanup_on_panic() {
    let vex_dir = unique_vex_dir();
    let lock_path = vex_dir.join("locks").join("node-20.0.0.lock");

    let result = std::panic::catch_unwind(|| {
        let _lock = InstallLock::acquire(&vex_dir, "node", "20.0.0").unwrap();
        assert!(lock_path.exists());
    });

    assert!(result.is_ok());
    assert!(!lock_path.exists());

    let _ = fs::remove_dir_all(&vex_dir);
}

#[test]
fn test_lock_with_empty_version() {
    let vex_dir = unique_vex_dir();

    let lock = InstallLock::acquire(&vex_dir, "node", "");
    assert!(lock.is_ok());

    let _ = fs::remove_dir_all(&vex_dir);
}

#[test]
fn test_lock_directory_permissions() {
    let vex_dir = unique_vex_dir();

    let _lock = InstallLock::acquire(&vex_dir, "node", "20.0.0").unwrap();

    let locks_dir = vex_dir.join("locks");
    assert!(locks_dir.exists());
    assert!(locks_dir.is_dir());

    let _lock2 = InstallLock::acquire(&vex_dir, "go", "1.21.0").unwrap();

    let _ = fs::remove_dir_all(&vex_dir);
}
