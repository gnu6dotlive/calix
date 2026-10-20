use anyhow::{Context, Result};
use std::process::Stdio;
use std::sync::OnceLock;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use tokio::sync::mpsc;

static TOKIO_RT: OnceLock<tokio::runtime::Runtime> = OnceLock::new();

pub fn runtime() -> &'static tokio::runtime::Runtime {
    TOKIO_RT.get_or_init(|| {
        tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .thread_name("calix-tokio-worker")
            .build()
            .expect("Échec de l'initialisation du runtime Tokio")
    })
}

#[derive(Debug, Clone, PartialEq)]
pub enum LogStreamEvent {
    Line(String),
    ErrorLine(String),
    Finished(bool, Option<i32>),
}

pub struct ProcessRunner;

impl ProcessRunner {
    /// Exécute une commande de lecture et retourne la chaîne JSON complète via le runtime Tokio
    pub async fn run_json_command(binary_path: &str, args: &[&str]) -> Result<String> {
        let bin = binary_path.to_string();
        let owned_args: Vec<String> = args.iter().map(|s| s.to_string()).collect();
        let (tx, rx) = tokio::sync::oneshot::channel();

        runtime().spawn(async move {
            let mut cmd = Command::new(&bin);
            cmd.args(&owned_args);
            cmd.stdout(Stdio::piped());
            cmd.stderr(Stdio::piped());

            let output_res = cmd.output().await.with_context(|| {
                format!(
                    "Échec de l'exécution de {} avec les arguments {:?}",
                    bin, owned_args
                )
            });

            let res = match output_res {
                Ok(output) => {
                    if !output.status.success() {
                        let err_str = String::from_utf8_lossy(&output.stderr);
                        log::warn!(
                            "La commande a retourné une erreur (code {:?}): {}",
                            output.status.code(),
                            err_str
                        );
                    }
                    String::from_utf8(output.stdout)
                        .context("Sortie stdout non-UTF8 lors de l'exécution de la commande")
                }
                Err(e) => Err(e),
            };

            let _ = tx.send(res);
        });

        rx.await
            .map_err(|_| anyhow::anyhow!("Tâche annulée avant de renvoyer un résultat"))?
    }

    /// Exécute une opération de mutation (install, remove, upgrade) en streamant chaque ligne
    pub fn spawn_streaming_operation(
        binary_path: String,
        args: Vec<String>,
        tx: mpsc::UnboundedSender<LogStreamEvent>,
    ) {
        runtime().spawn(async move {
            let mut cmd = Command::new(&binary_path);
            cmd.args(&args);
            // Le backend CLI élève désormais par pkexec par défaut. On fixe
            // CALIX_ELEVATOR explicitement quand l'environnement ne le fait
            // pas, pour garantir une invite d'authentification graphique
            // Polkit sous Wayland/X11 quel que soit l'ascenseur retenu.
            if std::env::var("CALIX_ELEVATOR").is_err() {
                cmd.env("CALIX_ELEVATOR", "pkexec");
            }
            cmd.stdout(Stdio::piped());
            cmd.stderr(Stdio::piped());

            let mut child = match cmd.spawn() {
                Ok(child) => child,
                Err(e) => {
                    let _ = tx.send(LogStreamEvent::ErrorLine(format!(
                        "Erreur au lancement du processus : {}",
                        e
                    )));
                    let _ = tx.send(LogStreamEvent::Finished(false, None));
                    return;
                }
            };

            let stdout = child.stdout.take();
            let stderr = child.stderr.take();

            let tx_out = tx.clone();
            let stdout_task = tokio::spawn(async move {
                if let Some(stdout) = stdout {
                    let mut reader = BufReader::new(stdout).lines();
                    while let Ok(Some(line)) = reader.next_line().await {
                        if tx_out.send(LogStreamEvent::Line(line)).is_err() {
                            break;
                        }
                    }
                }
            });

            let tx_err = tx.clone();
            let stderr_task = tokio::spawn(async move {
                if let Some(stderr) = stderr {
                    let mut reader = BufReader::new(stderr).lines();
                    while let Ok(Some(line)) = reader.next_line().await {
                        if tx_err.send(LogStreamEvent::ErrorLine(line)).is_err() {
                            break;
                        }
                    }
                }
            });

            let _ = tokio::join!(stdout_task, stderr_task);

            match child.wait().await {
                Ok(status) => {
                    let success = status.success();
                    let code = status.code();
                    let _ = tx.send(LogStreamEvent::Finished(success, code));
                }
                Err(e) => {
                    let _ = tx.send(LogStreamEvent::ErrorLine(format!(
                        "Erreur lors de l'attente du processus : {}",
                        e
                    )));
                    let _ = tx.send(LogStreamEvent::Finished(false, None));
                }
            }
        });
    }
}
