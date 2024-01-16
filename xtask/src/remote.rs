use crate::{flags, remote};
use anyhow::{anyhow, Context};
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

fn create_remote(sh: &Shell, remotes: &[String], remote: &OsString) -> anyhow::Result<()> {
    let remote = remote.to_string_lossy().to_string();

    if !remote.contains("github.com") {
        return Err(anyhow::anyhow!("URI has to refer to the github"));
    }

    let remote_name = remote
        .split('/')
        .rev()
        .skip(1)
        .next()
        .map(|x| Ok(x))
        .unwrap_or(Err(anyhow::anyhow!("Not a valid url")))?
        .to_string()
        .to_lowercase()
        + "-xtask";

    if remotes.iter().find(|x| *x == &remote).is_none() {
        cmd!(sh, "git remote add")
            .arg(&remote_name)
            .arg(&remote)
            .run()?;
    }

    cmd!(sh, "git fetch").arg(&remote_name).run()?;

    let branches = String::from_utf8(cmd!(sh, "git branch -r").output()?.stdout)?;

    println!("Remote branches for {}", &remote);

    for branch in branches
        .split('\n')
        .filter(|x| x.split('\n').next().is_some_and(|x| x == remote_name))
    {
        println!("\t{}", branch);
    }

    Ok(())
}

fn current_remote(sh: &Shell) -> anyhow::Result<String> {
    let symbolic_ref = String::from_utf8(cmd!(sh, "git symbolic-ref -q HEAD").output()?.stdout)?;

    return Ok(String::from_utf8(
        cmd!(sh, "git for-each-ref --format='%(upstream:short)'")
            .arg(symbolic_ref)
            .output()?
            .stdout,
    )?);
}

fn delete_remote(sh: &Shell, remotes: &[String], remote: &OsString) -> anyhow::Result<()> {
    let remote = remote.to_string_lossy().to_string();

    if remotes.iter().find(|x| *x == &remote).is_some() {
        if current_remote(sh)? == remote {
            cmd!(sh, "git checkout main").run()?;
        }

        cmd!(sh, "git remote remove").arg(remote).run()?;
    }

    Ok(())
}

/// Add a new remote and checkout to it
pub fn remote(sh: &Shell, flags: flags::Remote) -> anyhow::Result<()> {
    let _pd = sh.push_dir(crate::project_root());

    let remotes_cmd = cmd!(sh, "git remote");
    let remotes: Vec<String> = String::from_utf8(remotes_cmd.output()?.stdout)?
        .split('\n')
        .map(|x| x.to_string())
        .filter(|x| !x.trim().is_empty())
        .collect();

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
                return Err(anyhow::anyhow!("You need to specify github_uri"));
            }
        },
    }

    if flags.list {
        show_remotes(sh, &remotes)?;
    }

    Ok(())
}
