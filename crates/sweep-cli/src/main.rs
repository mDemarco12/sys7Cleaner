use clap::{Parser, Subcommand};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::atomic::AtomicBool;
use std::time::SystemTime;
use sweep_core::model::{PlanItem, ReclaimPlan, ScanTarget};

#[derive(Parser)]
#[command(name = "sweep", about = "Headless engine CLI for the System 7 disk cleanup app")]
struct Cli {
    /// Reroot the entire catalog under this directory instead of $HOME.
    /// Used by the staging harness to run real scans/trashes against a
    /// disposable sandbox with zero risk to real data.
    #[arg(long, global = true)]
    root: Option<PathBuf>,

    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Scan all (or selected) catalog targets and print a ScanSummary as JSON.
    Scan {
        #[arg(long, value_delimiter = ',')]
        targets: Option<Vec<String>>,
    },
    /// Build a ReclaimPlan from selected targets (scans them fresh) and print it as JSON.
    Plan {
        #[arg(long, value_delimiter = ',', required = true)]
        targets: Vec<String>,
        #[arg(long)]
        permanent: bool,
    },
    /// Execute a ReclaimPlan (read from stdin or --plan-file) against the real filesystem.
    Apply {
        #[arg(long)]
        plan_file: Option<PathBuf>,
        /// Log what would happen without touching the filesystem.
        #[arg(long)]
        dry_run: bool,
    },
    /// List every catalog target with its tier and blurb.
    Targets,
}

fn home_or_root(root: &Option<PathBuf>) -> PathBuf {
    root.clone().unwrap_or_else(|| dirs_home())
}

fn dirs_home() -> PathBuf {
    std::env::var_os("HOME").map(PathBuf::from).expect("HOME must be set")
}

fn selected_targets(catalog: &[ScanTarget], ids: &Option<Vec<String>>) -> Vec<ScanTarget> {
    match ids {
        None => catalog.to_vec(),
        Some(ids) => catalog.iter().filter(|t| ids.contains(&t.id.to_string())).cloned().collect(),
    }
}

fn main() {
    let cli = Cli::parse();
    let home = home_or_root(&cli.root);
    let catalog = sweep_core::catalog::build_catalog(&home);

    match cli.command {
        Command::Targets => {
            for t in &catalog {
                println!("{:<28} [{:?}/{:?}] {}", t.id, t.safety, t.granularity, t.blurb);
            }
        }
        Command::Scan { targets } => {
            let chosen = selected_targets(&catalog, &targets);
            let cancel = AtomicBool::new(false);
            let summary = sweep_core::run_scan(&chosen, &cancel);
            println!("{}", serde_json::to_string_pretty(&summary).unwrap());
        }
        Command::Plan { targets, permanent } => {
            let chosen: Vec<ScanTarget> = catalog.iter().filter(|t| targets.contains(&t.id.to_string())).cloned().collect();

            if let Some(refused) = chosen.iter().find(|t| t.refuse_delete) {
                eprintln!("refusing to plan deletion for '{}': {}", refused.id, refused.blurb);
                std::process::exit(1);
            }

            let cancel = AtomicBool::new(false);
            let mut items = Vec::new();
            for target in &chosen {
                for folder in sweep_core::planning::folder_breakdown(target, &cancel) {
                    items.push(PlanItem {
                        target_id: target.id.to_string(),
                        path: folder.path,
                        expected_disk_bytes: folder.disk_bytes,
                        expected_dev: folder.dev,
                        expected_ino: folder.ino,
                    });
                }
            }

            let plan = ReclaimPlan { items, permanent, created_at: SystemTime::now() };
            println!("{}", serde_json::to_string_pretty(&plan).unwrap());
        }
        Command::Apply { plan_file, dry_run } => {
            let plan_json = match plan_file {
                Some(path) => std::fs::read_to_string(path).expect("failed to read plan file"),
                None => {
                    use std::io::Read;
                    let mut buf = String::new();
                    std::io::stdin().read_to_string(&mut buf).expect("failed to read plan from stdin");
                    buf
                }
            };
            let plan: ReclaimPlan = serde_json::from_str(&plan_json).expect("invalid plan JSON");
            let allowlist: HashMap<String, sweep_core::reclaim::TargetAllowlist> = sweep_core::allowlist_map(&catalog);

            if dry_run {
                let ops = sweep_core::fsops::DryRunFileOps::new();
                let outcome = sweep_core::reclaim::execute(&plan, &allowlist, &home, &ops);
                for line in ops.take_log() {
                    println!("{line}");
                }
                println!("{}", serde_json::to_string_pretty(&outcome).unwrap());
            } else {
                let ops = sweep_core::fsops::RealFileOps;
                let outcome = sweep_core::reclaim::execute(&plan, &allowlist, &home, &ops);
                println!("{}", serde_json::to_string_pretty(&outcome).unwrap());
            }
        }
    }
}
