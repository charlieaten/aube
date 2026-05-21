use crate::commands;
use crate::commands::install::{
    DepSelection, FrozenMode, FrozenOverride, GlobalVirtualStoreFlags, InstallOptions,
};
use miette::{Context, IntoDiagnostic, miette};
use std::ffi::OsString;
use std::fmt;
use std::path::PathBuf;
use std::sync::OnceLock;
use std::time::Instant;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum FrozenLockfile {
    #[default]
    Auto,
    Frozen,
    No,
    Prefer,
}

impl FrozenLockfile {
    fn override_flag(self) -> Option<FrozenOverride> {
        match self {
            Self::Auto => None,
            Self::Frozen => Some(FrozenOverride::Frozen),
            Self::No => Some(FrozenOverride::No),
            Self::Prefer => Some(FrozenOverride::Prefer),
        }
    }
}

#[derive(Debug, Clone)]
pub struct InstallRequest {
    pub project_dir: PathBuf,
    pub frozen_lockfile: FrozenLockfile,
    pub prod: bool,
    pub dev: bool,
    pub no_optional: bool,
    pub offline: bool,
    pub prefer_offline: bool,
    pub ignore_scripts: bool,
    pub lockfile_only: bool,
    pub force: bool,
    pub node_linker: Option<String>,
    pub registry: Option<String>,
}

impl InstallRequest {
    pub fn new(project_dir: impl Into<PathBuf>) -> Self {
        Self {
            project_dir: project_dir.into(),
            frozen_lockfile: FrozenLockfile::Auto,
            prod: false,
            dev: false,
            no_optional: false,
            offline: false,
            prefer_offline: false,
            ignore_scripts: false,
            lockfile_only: false,
            force: false,
            node_linker: None,
            registry: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstallOutcome {
    pub project_dir: PathBuf,
    pub duration_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstallError {
    pub message: String,
    pub code: Option<String>,
}

impl InstallError {
    fn from_report(report: miette::Report) -> Self {
        let code = report.code().map(|code| code.to_string());
        Self {
            message: report.to_string(),
            code,
        }
    }
}

impl fmt::Display for InstallError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for InstallError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RunOutcome {
    pub exit_code: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RunError {
    pub message: String,
    pub code: Option<String>,
    pub exit_code: i32,
}

pub fn run(args: Vec<String>) -> Result<RunOutcome, RunError> {
    let handle = std::thread::Builder::new()
        .name("aube-embedded-cli".to_string())
        .stack_size(16 * 1024 * 1024)
        .spawn(move || run_on_cli_thread(args))
        .map_err(|err| RunError {
            message: format!("failed to start embedded aube CLI thread: {err}"),
            code: Some("embedded_thread_start".to_string()),
            exit_code: 1,
        })?;

    match handle.join() {
        Ok(result) => result,
        Err(_) => Err(RunError {
            message: "embedded aube CLI thread panicked".to_string(),
            code: Some("embedded_thread_panic".to_string()),
            exit_code: 1,
        }),
    }
}

fn run_on_cli_thread(args: Vec<String>) -> Result<RunOutcome, RunError> {
    let previous_cwd = std::env::current_dir().ok();
    let argv = std::iter::once(OsString::from("aube"))
        .chain(args.into_iter().map(OsString::from))
        .collect();

    let result = crate::run_cli_embedded(argv);
    crate::flush_cli_diagnostics();
    if let Some(cwd) = previous_cwd {
        let _ = std::env::set_current_dir(cwd);
        crate::dirs::reset_cwd();
    }

    match result {
        Ok(Some(exit_code)) => Ok(RunOutcome { exit_code }),
        Ok(None) => Ok(RunOutcome { exit_code: 0 }),
        Err(report) => Err(RunError {
            message: report.to_string(),
            code: report.code().map(|code| code.to_string()),
            exit_code: crate::report_exit_code(&report),
        }),
    }
}

pub async fn install(request: InstallRequest) -> Result<InstallOutcome, InstallError> {
    let started = Instant::now();
    install_inner(request)
        .await
        .map(|project_dir| InstallOutcome {
            project_dir,
            duration_ms: started.elapsed().as_millis().min(u64::MAX as u128) as u64,
        })
        .map_err(InstallError::from_report)
}

async fn install_inner(request: InstallRequest) -> miette::Result<PathBuf> {
    validate_request(&request)?;
    let project_dir = normalize_project_dir(&request.project_dir)?;
    let _install_guard = embedded_install_lock().lock().await;
    let _state_guard = InvocationStateGuard::new(&request);

    let frozen_override = request.frozen_lockfile.override_flag();
    let env = aube_settings::values::capture_env();
    let cli_flags = cli_flag_bag(&request, frozen_override);
    let files = commands::FileSources::load(&project_dir);
    let raw_workspace = aube_manifest::workspace::load_raw(&project_dir)
        .into_diagnostic()
        .wrap_err("failed to load workspace config")?;
    let ctx = files.ctx(&raw_workspace, &env, &cli_flags);
    let yaml_prefer_frozen = aube_settings::resolved::prefer_frozen_lockfile(&ctx);
    let mode = if request.force && frozen_override.is_none() {
        FrozenMode::No
    } else {
        FrozenMode::from_override(frozen_override, yaml_prefer_frozen)
    };

    let mut opts = InstallOptions::with_mode(mode);
    opts.project_dir = Some(project_dir.clone());
    opts.dep_selection = DepSelection::from_flags(request.prod, request.dev, request.no_optional);
    opts.ignore_scripts = request.ignore_scripts;
    opts.lockfile_only = request.lockfile_only;
    opts.force = request.force;
    opts.network_mode = network_mode(&request);
    opts.strict_no_lockfile = matches!(frozen_override, Some(FrozenOverride::Frozen));
    opts.cli_flags = cli_flags;
    opts.env_snapshot = env;
    opts.skip_root_lifecycle = false;

    commands::install::run(opts).await?;
    Ok(project_dir)
}

fn embedded_install_lock() -> &'static tokio::sync::Mutex<()> {
    static LOCK: OnceLock<tokio::sync::Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| tokio::sync::Mutex::new(()))
}

struct InvocationStateGuard {
    previous_progress_output: clx::progress::ProgressOutput,
}

impl InvocationStateGuard {
    fn new(request: &InstallRequest) -> Self {
        let previous_progress_output = clx::progress::output();
        commands::reset_invocation_state();
        commands::set_registry_override(request.registry.clone());
        commands::set_fetch_cli_overrides(Vec::new());
        commands::set_global_frozen_override(request.frozen_lockfile.override_flag());
        commands::set_global_virtual_store_flags(GlobalVirtualStoreFlags::default());
        commands::set_global_output_flags(commands::GlobalOutputFlags {
            ndjson: false,
            silent: true,
        });
        clx::progress::set_output(clx::progress::ProgressOutput::Text);
        Self {
            previous_progress_output,
        }
    }
}

impl Drop for InvocationStateGuard {
    fn drop(&mut self) {
        commands::reset_invocation_state();
        clx::progress::set_output(self.previous_progress_output);
    }
}

fn validate_request(request: &InstallRequest) -> miette::Result<()> {
    if request.project_dir.as_os_str().is_empty() {
        return Err(miette!("project_dir is required"));
    }
    if request.prod && request.dev {
        return Err(miette!("prod and dev install modes are mutually exclusive"));
    }
    if request.offline && request.prefer_offline {
        return Err(miette!("offline and prefer_offline are mutually exclusive"));
    }
    Ok(())
}

fn normalize_project_dir(project_dir: &PathBuf) -> miette::Result<PathBuf> {
    let expanded = if project_dir.is_absolute() {
        project_dir.clone()
    } else {
        std::env::current_dir()
            .into_diagnostic()
            .wrap_err("failed to read current directory")?
            .join(project_dir)
    };
    let canonical = std::fs::canonicalize(&expanded)
        .into_diagnostic()
        .wrap_err_with(|| format!("failed to resolve project dir {}", expanded.display()))?;
    if !canonical.is_dir() {
        return Err(miette!(
            "project dir is not a directory: {}",
            canonical.display()
        ));
    }
    Ok(canonical)
}

fn cli_flag_bag(
    request: &InstallRequest,
    frozen_override: Option<FrozenOverride>,
) -> Vec<(String, String)> {
    let mut out = Vec::new();
    if let Some(linker) = request.node_linker.as_deref() {
        out.push(("node-linker".to_string(), linker.to_string()));
    }
    if let Some(override_flag) = frozen_override {
        let (key, value) = override_flag.cli_flag_bag_entry();
        out.push((key.to_string(), value.to_string()));
    }
    out
}

fn network_mode(request: &InstallRequest) -> aube_registry::NetworkMode {
    if request.offline {
        aube_registry::NetworkMode::Offline
    } else if request.prefer_offline {
        aube_registry::NetworkMode::PreferOffline
    } else {
        aube_registry::NetworkMode::Online
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn write_package_json(dir: &std::path::Path, body: &str) {
        std::fs::write(dir.join("package.json"), body).expect("package.json should be written");
    }

    #[tokio::test]
    async fn installs_empty_project() {
        let tmp = tempfile::tempdir().expect("temp dir should be created");
        write_package_json(tmp.path(), r#"{"name":"fixture","version":"1.0.0"}"#);

        let outcome = install(InstallRequest::new(tmp.path()))
            .await
            .expect("empty project should install");

        assert_eq!(outcome.project_dir, tmp.path().canonicalize().unwrap());
        assert!(outcome.duration_ms < u64::MAX);
    }

    #[tokio::test]
    async fn repeated_installs_in_one_process_work() {
        let tmp = tempfile::tempdir().expect("temp dir should be created");
        write_package_json(tmp.path(), r#"{"name":"fixture","version":"1.0.0"}"#);

        install(InstallRequest::new(tmp.path())).await.unwrap();
        install(InstallRequest::new(tmp.path())).await.unwrap();
    }

    #[tokio::test]
    async fn different_project_dirs_work_sequentially() {
        let first = tempfile::tempdir().expect("temp dir should be created");
        let second = tempfile::tempdir().expect("temp dir should be created");
        write_package_json(first.path(), r#"{"name":"first","version":"1.0.0"}"#);
        write_package_json(second.path(), r#"{"name":"second","version":"1.0.0"}"#);

        install(InstallRequest::new(first.path())).await.unwrap();
        install(InstallRequest::new(second.path())).await.unwrap();
    }

    #[tokio::test]
    async fn frozen_lockfile_drift_returns_error() {
        let tmp = tempfile::tempdir().expect("temp dir should be created");
        write_package_json(
            tmp.path(),
            r#"{"name":"fixture","version":"1.0.0","dependencies":{"left-pad":"1.3.0"}}"#,
        );
        std::fs::write(
            tmp.path().join("aube-lock.yaml"),
            r#"
lockfileVersion: '9.0'

settings:
  autoInstallPeers: true
  excludeLinksFromLockfile: false

importers:

  .:
    dependencies:
      left-pad:
        specifier: 1.1.3
        version: 1.1.3

packages:

  left-pad@1.1.3:
    resolution: {integrity: sha512-m3z9QHpSXmd2H8Z5jnSXbGONPty4dFQfH1QpGgivzrEzICgsi50j9S+aGc77EaLoHpbw0BzP5+k1pp2UajTRuw==}
    deprecated: use String.prototype.padStart()

snapshots:

  left-pad@1.1.3: {}
"#,
        )
        .expect("lockfile should be written");
        let mut request = InstallRequest::new(tmp.path());
        request.frozen_lockfile = FrozenLockfile::Frozen;

        let err = install(request).await.expect_err("drift should error");
        assert!(err.message.contains("lockfile is out of date"));
    }
}
