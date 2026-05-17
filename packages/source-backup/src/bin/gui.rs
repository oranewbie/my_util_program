#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]

use eframe::egui;
use source_backup::{
    default_output_for, run_backup, BackupOptions, BackupSummary, Progress as BkProgress,
};
use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, Receiver};
use std::thread;
use std::time::Duration;

fn main() -> eframe::Result {
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([580.0, 420.0])
            .with_min_inner_size([460.0, 360.0])
            .with_title("source-backup"),
        ..Default::default()
    };
    eframe::run_native(
        "source-backup",
        native_options,
        Box::new(|cc| {
            install_korean_fonts(&cc.egui_ctx);
            Ok(Box::new(App::default()))
        }),
    )
}

/// 시스템에 한글 폰트가 있으면 로드. 못 찾으면 기본 폰트로 진행 (한글 글리프 빠질 수 있음).
fn install_korean_fonts(ctx: &egui::Context) {
    let candidates: &[&str] = &[
        // Windows
        "C:/Windows/Fonts/malgun.ttf",
        "C:/Windows/Fonts/malgunbd.ttf",
        // Linux (Noto CJK 일반 경로)
        "/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc",
        "/usr/share/fonts/truetype/noto/NotoSansCJK-Regular.ttc",
        "/usr/share/fonts/noto-cjk/NotoSansCJK-Regular.ttc",
        // macOS
        "/System/Library/Fonts/AppleSDGothicNeo.ttc",
    ];
    for path in candidates {
        if let Ok(bytes) = std::fs::read(path) {
            let mut fonts = egui::FontDefinitions::default();
            fonts
                .font_data
                .insert("kr".to_owned(), egui::FontData::from_owned(bytes));
            fonts
                .families
                .entry(egui::FontFamily::Proportional)
                .or_default()
                .insert(0, "kr".to_owned());
            fonts
                .families
                .entry(egui::FontFamily::Monospace)
                .or_default()
                .push("kr".to_owned());
            ctx.set_fonts(fonts);
            return;
        }
    }
}

enum WorkerMsg {
    Progress(BkProgress),
    Finished(Result<BackupSummary, String>),
}

#[derive(Default)]
enum State {
    #[default]
    Idle,
    Running {
        rx: Receiver<WorkerMsg>,
        done: u64,
        total: u64,
        current: String,
        output: PathBuf,
    },
    Finished {
        output: PathBuf,
        files: u64,
        bytes: u64,
    },
    Error(String),
}

#[derive(Default)]
struct App {
    source: Option<PathBuf>,
    output: String,
    include_git_dir: bool,
    no_git: bool,
    state: State,
}

impl App {
    fn start_backup(&mut self, ctx: egui::Context) {
        let Some(source) = self.source.clone() else {
            return;
        };
        let output = if self.output.trim().is_empty() {
            None
        } else {
            Some(PathBuf::from(self.output.trim()))
        };
        let include_git_dir = self.include_git_dir;
        let no_git = self.no_git;

        // progress bar 비율 표시 위해 dry-run 으로 총 파일 수 먼저 카운트.
        // 실패해도 무시 (total=0 → spinner 모드).
        let total = count_files(&source, include_git_dir, no_git).unwrap_or(0);

        let final_output = output
            .clone()
            .unwrap_or_else(|| default_output_for(&source));

        let (tx, rx) = mpsc::channel::<WorkerMsg>();
        let ctx_clone = ctx.clone();

        thread::spawn(move || {
            let opts = BackupOptions {
                source,
                output,
                include_git_dir,
                no_git,
                dry_run: false,
            };
            let tx_progress = tx.clone();
            let result = run_backup(opts, move |p| {
                let _ = tx_progress.send(WorkerMsg::Progress(p));
                ctx_clone.request_repaint();
            });
            let _ = tx.send(WorkerMsg::Finished(result.map_err(|e| format!("{e:#}"))));
            ctx.request_repaint();
        });

        self.state = State::Running {
            rx,
            done: 0,
            total,
            current: String::new(),
            output: final_output,
        };
    }
}

fn count_files(source: &Path, include_git_dir: bool, no_git: bool) -> Option<u64> {
    let opts = BackupOptions {
        source: source.to_path_buf(),
        output: None,
        include_git_dir,
        no_git,
        dry_run: true,
    };
    run_backup(opts, |_| {}).ok().map(|s| s.files)
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // 워커 메시지 폴링
        let mut new_state: Option<State> = None;
        if let State::Running {
            rx, done, current, ..
        } = &mut self.state
        {
            loop {
                match rx.try_recv() {
                    Ok(WorkerMsg::Progress(BkProgress::File { count, rel_path, .. })) => {
                        *done = count;
                        *current = rel_path;
                    }
                    Ok(WorkerMsg::Progress(_)) => {}
                    Ok(WorkerMsg::Finished(Ok(summary))) => {
                        new_state = Some(State::Finished {
                            output: summary.output,
                            files: summary.files,
                            bytes: summary.bytes,
                        });
                        break;
                    }
                    Ok(WorkerMsg::Finished(Err(e))) => {
                        new_state = Some(State::Error(e));
                        break;
                    }
                    Err(mpsc::TryRecvError::Empty) => break,
                    Err(mpsc::TryRecvError::Disconnected) => break,
                }
            }
        }
        if let Some(s) = new_state {
            self.state = s;
        }

        if matches!(self.state, State::Running { .. }) {
            ctx.request_repaint_after(Duration::from_millis(80));
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("source-backup");
            ui.add_space(4.0);

            // Source 폴더 선택
            ui.horizontal(|ui| {
                if ui.button("폴더 선택...").clicked() {
                    if let Some(p) = rfd::FileDialog::new().pick_folder() {
                        // 출력이 비어있거나 이전 source 의 자동 생성 값이면 새로 채움
                        if self.output.is_empty() {
                            self.output = default_output_for(&p).display().to_string();
                        }
                        self.source = Some(p);
                    }
                }
                ui.add(
                    egui::Label::new(
                        self.source
                            .as_ref()
                            .map(|p| p.display().to_string())
                            .unwrap_or_else(|| "(선택 안 됨)".into()),
                    )
                    .truncate(),
                );
            });

            ui.horizontal(|ui| {
                ui.label("출력  ");
                ui.add_sized(
                    egui::vec2(ui.available_width(), 0.0),
                    egui::TextEdit::singleline(&mut self.output),
                );
            });

            ui.add_space(4.0);
            ui.checkbox(&mut self.include_git_dir, ".git 폴더도 포함");
            ui.checkbox(&mut self.no_git, ".gitignore 무시 (블랙리스트만 사용)");

            ui.separator();

            // Start 버튼
            let is_running = matches!(self.state, State::Running { .. });
            let can_start = self.source.is_some() && !is_running;
            if ui
                .add_enabled(
                    can_start,
                    egui::Button::new("백업 시작").min_size(egui::vec2(120.0, 32.0)),
                )
                .clicked()
            {
                self.start_backup(ctx.clone());
            }

            ui.add_space(8.0);

            // 진행/결과 표시
            match &self.state {
                State::Idle => {}
                State::Running {
                    done,
                    total,
                    current,
                    output,
                    ..
                } => {
                    let frac = if *total > 0 {
                        (*done as f32 / *total as f32).min(1.0)
                    } else {
                        0.0
                    };
                    ui.add(
                        egui::ProgressBar::new(frac)
                            .show_percentage()
                            .animate(*total == 0),
                    );
                    if *total > 0 {
                        ui.label(format!("{done} / {total} files"));
                    } else {
                        ui.label(format!("{done} files (총 개수 계산 실패, 진행만 표시)"));
                    }
                    ui.add(egui::Label::new(format!("→ {current}")).truncate());
                    ui.add(
                        egui::Label::new(format!("저장 위치: {}", output.display())).truncate(),
                    );
                }
                State::Finished {
                    output,
                    files,
                    bytes,
                } => {
                    ui.colored_label(egui::Color32::from_rgb(70, 170, 90), "완료");
                    ui.label(format!(
                        "{files} files, {} ({} bytes)",
                        human_bytes(*bytes),
                        bytes
                    ));
                    ui.add(egui::Label::new(format!("→ {}", output.display())).wrap());
                }
                State::Error(msg) => {
                    ui.colored_label(
                        egui::Color32::from_rgb(200, 80, 80),
                        format!("에러: {msg}"),
                    );
                }
            }
        });
    }
}

fn human_bytes(n: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    let mut v = n as f64;
    let mut i = 0;
    while v >= 1024.0 && i + 1 < UNITS.len() {
        v /= 1024.0;
        i += 1;
    }
    format!("{:.1} {}", v, UNITS[i])
}
