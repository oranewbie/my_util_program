use anyhow::Result;
use clap::Parser;
use source_backup::{run_backup, BackupOptions, Progress};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(
    name = "source-backup",
    version,
    about = "폴더를 zip으로 백업 (.gitignore 우선, 폴백은 블랙리스트)"
)]
struct Cli {
    /// 백업할 소스 폴더
    source: PathBuf,

    /// 출력 zip 경로 (기본: <폴더명>-YYYYMMDD-HHMMSS.zip, 현재 디렉토리)
    #[arg(short, long)]
    output: Option<PathBuf>,

    /// .git 폴더 자체도 zip에 포함 (기본은 제외)
    #[arg(long)]
    include_git_dir: bool,

    /// .gitignore 무시하고 항상 블랙리스트만 사용
    #[arg(long)]
    no_git: bool,

    /// zip을 만들지 않고 포함될 파일 목록만 출력
    #[arg(long)]
    dry_run: bool,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    eprintln!("source : {}", cli.source.display());

    let opts = BackupOptions {
        source: cli.source.clone(),
        output: cli.output.clone(),
        include_git_dir: cli.include_git_dir,
        no_git: cli.no_git,
        dry_run: cli.dry_run,
    };

    let dry_run = cli.dry_run;

    let summary = run_backup(opts, |p| match p {
        Progress::Started { output, use_git } => {
            if dry_run {
                eprintln!("output : (dry-run, zip 생성 안 함)");
            } else {
                eprintln!("output : {}", output.display());
            }
            eprintln!(
                "mode   : {}",
                if use_git { ".gitignore" } else { "blacklist" }
            );
        }
        Progress::Dir { rel_path } => {
            if dry_run {
                println!("{}/", rel_path);
            }
        }
        Progress::File {
            rel_path, size, ..
        } => {
            if dry_run {
                println!("{}  ({} bytes)", rel_path, size);
            }
        }
        Progress::Done { .. } => {}
    })?;

    eprintln!(
        "{}   : {} files, {} bytes",
        if dry_run { "would" } else { "done " },
        summary.files,
        summary.bytes
    );
    Ok(())
}
