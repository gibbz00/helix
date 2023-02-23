use std::path::Path;

use git::objs::tree::EntryMode;
use git::sec::trust::DefaultForLevel;
use git::{Commit, ObjectId, Repository, ThreadSafeRepository};
use git_repository as git;

use super::DiffProvider;

pub struct Git;

impl Git {
    fn open_repo(path: &Path, ceiling_dir: Option<&Path>) -> Option<ThreadSafeRepository> {
        // custom open options
        let mut git_open_opts_map = git::sec::trust::Mapping::<git::open::Options>::default();

        // On windows various configuration options are bundled as part of the installations
        // This path depends on the install location of git and therefore requires some overhead to lookup
        // This is basically only used on windows and has some overhead hence it's disabled on other platforms.
        // `gitoxide` doesn't use this as default
        let config = git::permissions::Config {
            system: true,
            git: true,
            user: true,
            env: true,
            includes: true,
            git_binary: cfg!(windows),
        };
        // change options for config permissions without touching anything else
        git_open_opts_map.reduced = git_open_opts_map.reduced.permissions(git::Permissions {
            config,
            ..git::Permissions::default_for_level(git::sec::Trust::Reduced)
        });
        git_open_opts_map.full = git_open_opts_map.full.permissions(git::Permissions {
            config,
            ..git::Permissions::default_for_level(git::sec::Trust::Full)
        });

        let mut open_options = git::discover::upwards::Options::default();
        if let Some(ceiling_dir) = ceiling_dir {
            open_options.ceiling_dirs = vec![ceiling_dir.to_owned()];
        }

        ThreadSafeRepository::discover_with_environment_overrides_opts(
            path,
            open_options,
            git_open_opts_map,
        )
        .ok()
    }
}

impl DiffProvider for Git {
    fn get_diff_base(&self, file: &Path) -> Option<Vec<u8>> {
        debug_assert!(!file.exists() || file.is_file());
        debug_assert!(file.is_absolute());

        // TODO cache repository lookup
        let repo = Git::open_repo(file.parent()?, None)?.to_thread_local();
        let head = repo.head_commit().ok()?;
        let file_oid = find_file_in_commit(&repo, &head, file)?;

        let file_object = repo.find_object(file_oid).ok()?;
        let mut data = file_object.detach().data;
        // convert LF to CRLF if configured to avoid showing every line as changed
        if repo
            .config_snapshot()
            .boolean("core.autocrlf")
            .unwrap_or(false)
        {
            let mut normalized_file = Vec::with_capacity(data.len());
            let mut at_cr = false;
            for &byte in &data {
                if byte == b'\n' {
                    // if this is a LF instead of a CRLF (last byte was not a CR)
                    // insert a new CR to generate a CRLF
                    if !at_cr {
                        normalized_file.push(b'\r');
                    }
                }
                at_cr = byte == b'\r';
                normalized_file.push(byte)
            }
            data = normalized_file
        }
        Some(data)
    }
}

/// Finds the object that contains the contents of a file at a specific commit.
fn find_file_in_commit(repo: &Repository, commit: &Commit, file: &Path) -> Option<ObjectId> {
    let repo_dir = repo.work_dir()?;
    let rel_path = file.strip_prefix(repo_dir).ok()?;
    let tree = commit.tree().ok()?;
    let tree_entry = tree.lookup_entry_by_path(rel_path).ok()??;
    match tree_entry.mode() {
        // not a file, everything is new, do not show diff
        EntryMode::Tree | EntryMode::Commit | EntryMode::Link => None,
        // found a file
        EntryMode::Blob | EntryMode::BlobExecutable => Some(tree_entry.object_id()),
    }
}

#[cfg(test)]
mod test {
    use std::{fs::File, io::Write, path::Path, process::Command};

    use tempfile::TempDir;

    use super::{DiffProvider, Git};

    fn exec_git_cmd(args: &str, git_dir: &Path) {
        let res = Command::new("git")
            .arg("-C")
            .arg(git_dir) // execute the git command in this directory
            .args(args.split_whitespace())
            .env_remove("GIT_DIR")
            .env_remove("GIT_ASKPASS")
            .env_remove("SSH_ASKPASS")
            .env("GIT_TERMINAL_PROMPT", "false")
            .env("GIT_AUTHOR_DATE", "2000-01-01 00:00:00 +0000")
            .env("GIT_AUTHOR_EMAIL", "author@example.com")
            .env("GIT_AUTHOR_NAME", "author")
            .env("GIT_COMMITTER_DATE", "2000-01-02 00:00:00 +0000")
            .env("GIT_COMMITTER_EMAIL", "committer@example.com")
            .env("GIT_COMMITTER_NAME", "committer")
            .env("GIT_CONFIG_COUNT", "2")
            .env("GIT_CONFIG_KEY_0", "commit.gpgsign")
            .env("GIT_CONFIG_VALUE_0", "false")
            .env("GIT_CONFIG_KEY_1", "init.defaultBranch")
            .env("GIT_CONFIG_VALUE_1", "main")
            .output()
            .unwrap_or_else(|_| panic!("`git {args}` failed"));
        if !res.status.success() {
            println!("{}", String::from_utf8_lossy(&res.stdout));
            eprintln!("{}", String::from_utf8_lossy(&res.stderr));
            panic!("`git {args}` failed (see output above)")
        }
    }

    fn create_commit(repo: &Path, add_modified: bool) {
        if add_modified {
            exec_git_cmd("add -A", repo);
        }
        exec_git_cmd("commit -m message", repo);
    }

    fn empty_git_repo() -> TempDir {
        let tmp = tempfile::tempdir().expect("create temp dir for git testing");
        exec_git_cmd("init", tmp.path());
        exec_git_cmd("config user.email test@helix.org", tmp.path());
        exec_git_cmd("config user.name helix-test", tmp.path());
        tmp
    }

    #[test]
    fn missing_file() {
        let temp_git = empty_git_repo();
        let file = temp_git.path().join("file.txt");
        File::create(&file).unwrap().write_all(b"foo").unwrap();

        assert_eq!(Git.get_diff_base(&file), None);
    }

    #[test]
    fn unmodified_file() {
        let temp_git = empty_git_repo();
        let file = temp_git.path().join("file.txt");
        let contents = b"foo".as_slice();
        File::create(&file).unwrap().write_all(contents).unwrap();
        create_commit(temp_git.path(), true);
        assert_eq!(Git.get_diff_base(&file), Some(Vec::from(contents)));
    }

    #[test]
    fn modified_file() {
        let temp_git = empty_git_repo();
        let file = temp_git.path().join("file.txt");
        let contents = b"foo".as_slice();
        File::create(&file).unwrap().write_all(contents).unwrap();
        create_commit(temp_git.path(), true);
        File::create(&file).unwrap().write_all(b"bar").unwrap();

        assert_eq!(Git.get_diff_base(&file), Some(Vec::from(contents)));
    }

    /// Test that `get_file_head` does not return content for a directory.
    /// This is important to correctly cover cases where a directory is removed and replaced by a file.
    /// If the contents of the directory object were returned a diff between a path and the directory children would be produced.
    #[test]
    fn directory() {
        let temp_git = empty_git_repo();
        let dir = temp_git.path().join("file.txt");
        std::fs::create_dir(&dir).expect("");
        let file = dir.join("file.txt");
        let contents = b"foo".as_slice();
        File::create(file).unwrap().write_all(contents).unwrap();

        create_commit(temp_git.path(), true);

        std::fs::remove_dir_all(&dir).unwrap();
        File::create(&dir).unwrap().write_all(b"bar").unwrap();
        assert_eq!(Git.get_diff_base(&dir), None);
    }

    /// Test that `get_file_head` does not return content for a symlink.
    /// This is important to correctly cover cases where a symlink is removed and replaced by a file.
    /// If the contents of the symlink object were returned a diff between a path and the actual file would be produced (bad ui).
    #[cfg(any(unix, windows))]
    #[test]
    fn symlink() {
        #[cfg(unix)]
        use std::os::unix::fs::symlink;
        #[cfg(not(unix))]
        use std::os::windows::fs::symlink_file as symlink;
        let temp_git = empty_git_repo();
        let file = temp_git.path().join("file.txt");
        let contents = b"foo".as_slice();
        File::create(&file).unwrap().write_all(contents).unwrap();
        let file_link = temp_git.path().join("file_link.txt");
        symlink("file.txt", &file_link).unwrap();

        create_commit(temp_git.path(), true);
        assert_eq!(Git.get_diff_base(&file_link), None);
        assert_eq!(Git.get_diff_base(&file), Some(Vec::from(contents)));
    }
}
