//! source-backup 핵심 로직.
//! CLI 와 GUI 모두에서 사용. 진행 상황은 콜백으로 전달.

use anyhow::{anyhow, Context, Result};
use chrono::{DateTime, Datelike, Local, Timelike};
use ignore::WalkBuilder;
use std::fs::File;
use std::io::{copy, BufReader, BufWriter};
use std::path::{Path, PathBuf};
use std::time::SystemTime;
use zip::write::SimpleFileOptions;
use zip::CompressionMethod;
use zip::ZipWriter;

/// `.git` 이 없을 때(또는 `no_git=true`) 적용되는 폴더 블랙리스트.
/// 경로 어디에 있든 이름이 일치하면 서브트리 전체를 건너뜀.
pub const BLACKLIST: &[&str] = &[
    // Python
    "venv", ".venv", "env",
    "__pycache__", ".pytest_cache", ".mypy_cache", ".ruff_cache",
    // Node / 프런트엔드
    "node_modules", "dist", "build",
    ".next", ".nuxt", ".turbo", ".cache",
    // Rust / Java / .NET
    "target", "bin", "obj", ".gradle",
    // 에디터
    ".idea", ".vscode",
];

/// 백업 옵션.
#[derive(Debug, Clone)]
pub struct BackupOptions {
    pub source: PathBuf,
    /// 없으면 `<폴더명>-YYYYMMDD-HHMMSS.zip` (현재 디렉토리)
    pub output: Option<PathBuf>,
    pub include_git_dir: bool,
    pub no_git: bool,
    /// true 면 zip 생성하지 않고 walk 만 수행
    pub dry_run: bool,
}

/// 진행 콜백에 전달되는 이벤트.
#[derive(Debug, Clone)]
pub enum Progress {
    /// 백업 시작. (출력 경로, gitignore 모드 여부)
    Started { output: PathBuf, use_git: bool },
    /// 파일 하나 처리 완료. count 는 누계.
    File { rel_path: String, size: u64, count: u64 },
    /// 디렉토리 엔트리 추가.
    Dir { rel_path: String },
    /// 완료. (총 파일 수, 총 바이트)
    Done { files: u64, bytes: u64 },
}

/// 백업 결과 요약.
#[derive(Debug, Clone)]
pub struct BackupSummary {
    pub output: PathBuf,
    pub use_git: bool,
    pub files: u64,
    pub bytes: u64,
}

/// 기본 출력 파일명 생성: `<폴더명>-YYYYMMDD-HHMMSS.zip`.
pub fn default_output_for(source: &Path) -> PathBuf {
    let name = source
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| "backup".to_string());
    let ts = Local::now().format("%Y%m%d-%H%M%S");
    PathBuf::from(format!("{name}-{ts}.zip"))
}

/// 백업 수행. `on_progress` 가 각 단계마다 호출됨.
///
/// 콜백은 빠르게 반환해야 함 (긴 작업 금지 — UI 스레드 외 곳에서 직접 호출됨).
pub fn run_backup<F>(opts: BackupOptions, mut on_progress: F) -> Result<BackupSummary>
where
    F: FnMut(Progress),
{
    let source = opts
        .source
        .canonicalize()
        .with_context(|| format!("소스 폴더 접근 실패: {}", opts.source.display()))?;
    if !source.is_dir() {
        return Err(anyhow!("폴더가 아님: {}", source.display()));
    }

    let has_git = source.join(".git").exists();
    let use_git = has_git && !opts.no_git;
    let include_git_dir = opts.include_git_dir;
    let dry_run = opts.dry_run;

    let output = opts
        .output
        .clone()
        .unwrap_or_else(|| default_output_for(&source));

    on_progress(Progress::Started {
        output: output.clone(),
        use_git,
    });

    let mut zip: Option<ZipWriter<BufWriter<File>>> = if dry_run {
        None
    } else {
        let file = File::create(&output)
            .with_context(|| format!("출력 파일 생성 실패: {}", output.display()))?;
        Some(ZipWriter::new(BufWriter::new(file)))
    };

    let mut walker = WalkBuilder::new(&source);
    walker
        .standard_filters(false)
        .git_ignore(use_git)
        .git_exclude(use_git)
        .git_global(use_git)
        .require_git(false)
        .hidden(false)
        .parents(false)
        .filter_entry(move |entry| {
            let name = entry.file_name().to_string_lossy().into_owned();
            if !include_git_dir && name == ".git" {
                return false;
            }
            if !use_git && BLACKLIST.contains(&name.as_str()) {
                return false;
            }
            true
        });

    let mut files: u64 = 0;
    let mut bytes: u64 = 0;

    for entry in walker.build() {
        let entry = entry.context("디렉토리 순회 중 오류")?;
        let path = entry.path();

        if path == source {
            continue;
        }

        let rel = path
            .strip_prefix(&source)
            .with_context(|| format!("상대 경로 계산 실패: {}", path.display()))?;
        let zip_path = rel.to_string_lossy().replace('\\', "/");

        let file_type = entry
            .file_type()
            .ok_or_else(|| anyhow!("파일 종류 알 수 없음: {}", path.display()))?;

        if file_type.is_dir() {
            if let Some(z) = zip.as_mut() {
                let opts = SimpleFileOptions::default()
                    .compression_method(CompressionMethod::Deflated);
                z.add_directory(&zip_path, opts)?;
            }
            on_progress(Progress::Dir {
                rel_path: zip_path.clone(),
            });
        } else if file_type.is_file() {
            let meta = entry
                .metadata()
                .with_context(|| format!("metadata 읽기 실패: {}", path.display()))?;
            let size = meta.len();

            if let Some(z) = zip.as_mut() {
                let mtime = meta.modified().unwrap_or(SystemTime::now());
                let opts = SimpleFileOptions::default()
                    .compression_method(CompressionMethod::Deflated)
                    .last_modified_time(to_zip_datetime(mtime));
                z.start_file(&zip_path, opts)?;
                let mut reader = BufReader::new(File::open(path)?);
                copy(&mut reader, z)?;
            }

            files += 1;
            bytes += size;
            on_progress(Progress::File {
                rel_path: zip_path,
                size,
                count: files,
            });
        }
        // symlink/그 외는 건너뜀
    }

    if let Some(z) = zip {
        z.finish()?;
    }

    on_progress(Progress::Done { files, bytes });

    Ok(BackupSummary {
        output,
        use_git,
        files,
        bytes,
    })
}

fn to_zip_datetime(t: SystemTime) -> zip::DateTime {
    let dt: DateTime<Local> = t.into();
    zip::DateTime::from_date_and_time(
        dt.year() as u16,
        dt.month() as u8,
        dt.day() as u8,
        dt.hour() as u8,
        dt.minute() as u8,
        dt.second() as u8,
    )
    .unwrap_or_default()
}
