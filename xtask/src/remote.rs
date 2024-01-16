use crate::flags;

use std::ffi::OsString;
use xshell::{cmd, Shell};

fn show_remotes(sh: &Shell, remotes: &[String]) -> anyhow::Result<()> {
    println!("\nRemotes:");
    for remote in remotes {
        let remote_path_cmd = cmd!(sh, "git remote get-url").arg(remote);
        let remote_path = String::from_utf8(remote_path_cmd.output()?.stdout)?
            .trim()
            .to_string();
        println!("\t{}: {}", remote, remote_path);
    }

    Ok(())
}

/// Returns parsed remote_name from github uri OR
fn parse_github_uri(remote: &str) -> anyhow::Result<String> {
    if !remote.contains("github.com") {
        return Err(anyhow::anyhow!("URI has to refer to the github"));
    }

    Ok(remote
        .replace(':', "/")
        .split('/')
        .rev()
        .nth(1)
        .map(Ok)
        .unwrap_or(Err(anyhow::anyhow!("Not a valid url")))?
        .to_string()
        .to_lowercase()
        + "-xtask")
}

fn create_remote(sh: &Shell, remotes: &[String], remote: &OsString) -> anyhow::Result<()> {
    let remote = remote.to_string_lossy().to_string();
    let remote_name = parse_github_uri(&remote)?;

    if !remotes.iter().any(|x| x.trim() == remote_name.trim()) {
        cmd!(sh, "git remote add")
            .arg(&remote_name)
            .arg(&remote)
            .run()?;
    }

    cmd!(sh, "git fetch").arg(&remote_name).run()?;

    let branches = String::from_utf8(cmd!(sh, "git branch -r").output()?.stdout)?;

    println!("\nRemote branches for {}", &remote);

    for branch in branches
        .split('\n')
        .filter(|x| x.split('/').next().is_some_and(|x| x.trim() == remote_name))
    {
        println!("\t{}", branch);
    }

    Ok(())
}

fn current_remote(sh: &Shell) -> anyhow::Result<String> {
    let symbolic_ref = String::from_utf8(cmd!(sh, "git symbolic-ref -q HEAD").output()?.stdout)?;

    Ok(String::from_utf8(
        cmd!(sh, "git for-each-ref --format='%(upstream:short)'")
            .arg(symbolic_ref)
            .output()?
            .stdout,
    )?)
}

fn delete_remote(sh: &Shell, remotes: &[String], remote: &OsString) -> anyhow::Result<()> {
    let remote = remote.to_string_lossy().to_string();
    let remote = parse_github_uri(&remote).unwrap_or(remote);

    if remotes.iter().any(|x| x == &remote) {
        if current_remote(sh)? == remote {
            cmd!(sh, "git checkout main").run()?;
        }

        cmd!(sh, "git remote remove").arg(remote).run()?;
    }

    Ok(())
}

fn get_remotes(sh: &Shell) -> anyhow::Result<Vec<String>> {
    Ok(String::from_utf8(cmd!(sh, "git remote").output()?.stdout)?
        .split('\n')
        .map(|x| x.to_string())
        .filter(|x| !x.trim().is_empty())
        .collect())
}

/// Add a new remote and checkout to it
pub fn remote(sh: &Shell, flags: flags::Remote) -> anyhow::Result<()> {
    let _pd = sh.push_dir(crate::project_root());

    let remotes = get_remotes(sh)?;

    match flags.remote {
        Some(remote) => {
            if flags.delete {
                delete_remote(sh, &remotes, &remote)?;
            } else {
                create_remote(sh, &remotes, &remote)?;
            };
        },
        None => {
            if !flags.list {
                return Err(anyhow::anyhow!("You need to specify github_uri or -l flag"));
            }
        },
    }

    if flags.list {
        show_remotes(sh, &get_remotes(sh)?)?;
    }

    Ok(())
}
