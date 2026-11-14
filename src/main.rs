//! vin: initialise research paper workspaces from YAML directory templates.

mod config;
mod init;
mod template;
mod tree;

use anyhow::{bail, Context, Result};
use clap::{Parser, Subcommand};
use config::Config;
use std::fs;
use std::io::IsTerminal;
use std::path::{Path, PathBuf};
use std::process::Command as Proc;

#[derive(Parser)]
#[command(name = "vin", version, about, arg_required_else_help = true)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Create a new paper workspace from a template
    #[command(visible_alias = "new")]
    Init {
        /// Workspace name; use `.` for the current directory.
        /// Defaults to the template's root folder name.
        name: Option<String>,
        /// Template to use (name or unique prefix); defaults to your default template
        #[arg(short, long)]
        template: Option<String>,
        /// Directory to create the workspace in
        #[arg(short = 'C', long, value_name = "DIR")]
        path: Option<PathBuf>,
        /// Overwrite files that already exist
        #[arg(short, long)]
        force: bool,
        /// Do not add .gitkeep files to empty directories
        #[arg(long)]
        no_gitkeep: bool,
        /// Run `git init` in the new workspace
        #[arg(long)]
        git: bool,
        /// Show what would be created without writing anything
        #[arg(short = 'n', long)]
        dry_run: bool,
    },
    /// List available templates
    #[command(visible_alias = "ls")]
    List,
    /// Preview a template's directory tree
    Show {
        /// Template to show; defaults to your default template
        template: Option<String>,
        /// Limit tree depth (1 = top level only)
        #[arg(short, long)]
        depth: Option<usize>,
    },
    /// Show or change the default template
    Default {
        /// Template to make the default; omit to show the current one
        template: Option<String>,
        /// Go back to the built-in default
        #[arg(long, conflicts_with = "template")]
        reset: bool,
    },
    /// Install a custom template from a YAML file
    Add {
        /// Path to the template YAML file
        file: PathBuf,
        /// Name for the template (defaults to the file name)
        #[arg(long)]
        name: Option<String>,
        /// Replace an existing custom template of the same name
        #[arg(short, long)]
        force: bool,
    },
}

fn main() {
    if let Err(err) = run() {
        eprintln!("{} {err:#}", style("error:", "1;31"));
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    match Cli::parse().command {
        Command::Init {
            name,
            template,
            path,
            force,
            no_gitkeep,
            git,
            dry_run,
        } => cmd_init(name, template, path, force, no_gitkeep, git, dry_run),
        Command::List => cmd_list(),
        Command::Show { template, depth } => cmd_show(template, depth),
        Command::Default { template, reset } => cmd_default(template, reset),
        Command::Add { file, name, force } => cmd_add(&file, name, force),
    }
}

/// Wrap text in an ANSI style when stdout is a terminal.
fn style(text: &str, code: &str) -> String {
    if std::io::stdout().is_terminal() && std::env::var_os("NO_COLOR").is_none() {
        format!("\x1b[{code}m{text}\x1b[0m")
    } else {
        text.to_string()
    }
}

fn cmd_list() -> Result<()> {
    let cfg = Config::load()?;
    let templates = template::all()?;
    let default = cfg.default_name().to_string();

    println!(
        "{} (default: {})\n",
        style("Available templates", "1"),
        style(&default, "1;32")
    );
    println!(
        "  {:<14} {:<9} {:>5} {:>6}   DESCRIPTION",
        "NAME", "SOURCE", "DIRS", "FILES"
    );
    for t in &templates {
        let stats = t.tree().map(|tr| tr.stats()).unwrap_or_default();
        let marker = if t.name == default { "*" } else { " " };
        let line = format!(
            "{marker} {:<14} {:<9} {:>5} {:>6}   {}",
            t.name,
            t.origin.label(),
            stats.dirs,
            stats.files,
            t.description
        );
        if t.name == default {
            println!("{}", style(&line, "32"));
        } else {
            println!("{line}");
        }
    }
    if !templates.iter().any(|t| t.name == default) {
        println!(
            "\nwarning: default template `{default}` does not exist; run `vin default <name>`"
        );
    }
    println!("\n  * = default    names can be shortened to a unique prefix (e.g. `phys`)");
    println!("  vin show <name>            preview a template");
    println!("  vin init <project> -t <name>   create a workspace");
    println!("  vin default <name>         change the default");
    println!("  custom templates live in {}", config::templates_dir()?.display());
    Ok(())
}

fn cmd_show(query: Option<String>, depth: Option<usize>) -> Result<()> {
    let cfg = Config::load()?;
    let templates = template::all()?;
    let t = template::resolve(query.as_deref().unwrap_or(cfg.default_name()), &templates)?;
    let tree = t.tree()?;
    let stats = tree.stats();
    println!(
        "{} [{}]: {}\n",
        style(&t.name, "1"),
        t.origin.label(),
        t.description
    );
    if let template::Origin::User(path) = &t.origin {
        println!("file: {}\n", path.display());
    }
    print!("{}", tree.render(&tree.root_name, depth));
    println!("\n{} directories, {} files", stats.dirs, stats.files);
    Ok(())
}

fn cmd_default(query: Option<String>, reset: bool) -> Result<()> {
    let mut cfg = Config::load()?;
    if reset {
        cfg.default_template = None;
        cfg.save()?;
        println!("default template reset to `{}`", config::FALLBACK_DEFAULT);
        return Ok(());
    }
    match query {
        None => println!("{}", cfg.default_name()),
        Some(q) => {
            let templates = template::all()?;
            let t = template::resolve(&q, &templates)?;
            cfg.default_template = Some(t.name.clone());
            cfg.save()?;
            println!("default template set to `{}`", t.name);
        }
    }
    Ok(())
}

fn cmd_add(file: &Path, name: Option<String>, force: bool) -> Result<()> {
    let yaml =
        fs::read_to_string(file).with_context(|| format!("reading {}", file.display()))?;
    let tree = tree::parse(&yaml).context("template is invalid")?;

    let name = match name {
        Some(n) => n,
        None => file
            .file_stem()
            .and_then(|s| s.to_str())
            .context("cannot derive a template name from the file; use --name")?
            .to_string(),
    };
    if name.is_empty()
        || !name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
    {
        bail!("template name `{name}` may only contain letters, digits, `_` and `-`");
    }

    let dir = config::templates_dir()?;
    fs::create_dir_all(&dir)?;
    let dest = dir.join(format!("{name}.yaml"));
    if dest.exists() && !force {
        bail!("custom template `{name}` already exists (use --force to replace it)");
    }
    fs::write(&dest, yaml)?;
    let stats = tree.stats();
    println!(
        "installed `{name}` ({} directories, {} files) -> {}",
        stats.dirs,
        stats.files,
        dest.display()
    );
    if template::builtin().iter().any(|b| b.name == name) {
        println!("note: this overrides the built-in template of the same name");
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn cmd_init(
    name: Option<String>,
    query: Option<String>,
    path: Option<PathBuf>,
    force: bool,
    no_gitkeep: bool,
    git: bool,
    dry_run: bool,
) -> Result<()> {
    let cfg = Config::load()?;
    let templates = template::all()?;
    let t = template::resolve(query.as_deref().unwrap_or(cfg.default_name()), &templates)?;
    let tree = t.tree()?;

    let parent = match path {
        Some(p) => p,
        None => std::env::current_dir()?,
    };
    let (target, project_name) = match name.as_deref() {
        Some(".") => {
            let abs = fs::canonicalize(&parent)
                .with_context(|| format!("cannot access {}", parent.display()))?;
            let n = abs
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("paper")
                .to_string();
            (abs, n)
        }
        Some(n) => {
            tree::validate_name(n)?;
            (parent.join(n), n.to_string())
        }
        None => (parent.join(&tree.root_name), tree.root_name.clone()),
    };

    let opts = init::Options {
        force,
        gitkeep: !no_gitkeep,
        dry_run,
    };
    let mut report = init::Report::default();
    init::materialize(&target, &tree.children, &project_name, opts, &mut report)?;

    let nothing_new = report.dirs_created == 0
        && report.files_created == 0
        && report.files_overwritten == 0;
    let verb = match (dry_run, nothing_new) {
        (true, _) => "Would create",
        (false, true) => "Nothing new to create for",
        (false, false) => "Created",
    };
    println!(
        "{verb} `{}` workspace at {}",
        t.name,
        target.display()
    );
    println!(
        "  {} directories, {} files{}",
        report.dirs_created,
        report.files_created,
        if report.files_overwritten > 0 {
            format!(", {} overwritten", report.files_overwritten)
        } else {
            String::new()
        }
    );
    if !report.files_skipped.is_empty() {
        println!(
            "  skipped {} existing files (use --force to overwrite)",
            report.files_skipped.len()
        );
    }

    if git && !dry_run {
        match Proc::new("git").arg("init").current_dir(&target).status() {
            Ok(s) if s.success() => {}
            Ok(_) => eprintln!("warning: `git init` failed"),
            Err(e) => eprintln!("warning: could not run git: {e}"),
        }
    }
    if !dry_run {
        println!("\nNext: cd {}", target.display());
    }
    Ok(())
}
