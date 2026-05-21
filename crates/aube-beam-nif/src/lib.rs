use aube::embed::{self, DepSelection, FrozenMode, InstallControl, InstallOptions, NetworkMode};
use rustler::{Encoder, Env, NifMap, NifResult, Term};
use std::path::PathBuf;
use std::sync::OnceLock;
use std::time::Instant;

mod atoms {
    rustler::atoms! {
        ok,
        error
    }
}

#[derive(Debug, NifMap)]
struct InstallOpts {
    cwd: String,
    frozen_lockfile: bool,
    no_frozen_lockfile: bool,
    prefer_frozen_lockfile: bool,
    prod: bool,
    dev: bool,
    no_optional: bool,
    offline: bool,
    prefer_offline: bool,
    ignore_scripts: bool,
    lockfile_only: bool,
    force: bool,
}

#[derive(Debug, NifMap)]
struct InstallResult {
    project_dir: String,
    duration_ms: u64,
}

#[derive(Debug, NifMap)]
struct InstallFailure {
    message: String,
    code: Option<String>,
}

#[rustler::nif(schedule = "DirtyIo")]
fn install<'a>(env: Env<'a>, opts: InstallOpts) -> NifResult<Term<'a>> {
    let (project_dir, options) = match options_from_nif(opts) {
        Ok(request) => request,
        Err(failure) => return Ok((atoms::error(), failure).encode(env)),
    };
    let runtime = match runtime() {
        Ok(runtime) => runtime,
        Err(failure) => return Ok((atoms::error(), failure).encode(env)),
    };

    initialize_embedder();
    let started = Instant::now();
    let result = runtime.block_on(embed::install(options));
    Ok(match result {
        Ok(()) => (
            atoms::ok(),
            InstallResult {
                project_dir: project_dir.to_string_lossy().into_owned(),
                duration_ms: started.elapsed().as_millis().min(u64::MAX as u128) as u64,
            },
        )
            .encode(env),
        Err(error) => (
            atoms::error(),
            InstallFailure {
                code: embed::error_code(&error),
                message: error.to_string(),
            },
        )
            .encode(env),
    })
}

fn options_from_nif(opts: InstallOpts) -> Result<(PathBuf, InstallOptions), InstallFailure> {
    let frozen_count = [
        opts.frozen_lockfile,
        opts.no_frozen_lockfile,
        opts.prefer_frozen_lockfile,
    ]
    .into_iter()
    .filter(|flag| *flag)
    .count();
    if frozen_count > 1 {
        return Err(invalid_argument(
            "frozen lockfile options are mutually exclusive",
        ));
    }
    if opts.prod && opts.dev {
        return Err(invalid_argument(
            "prod and dev install modes are mutually exclusive",
        ));
    }
    if opts.offline && opts.prefer_offline {
        return Err(invalid_argument(
            "offline and prefer_offline are mutually exclusive",
        ));
    }

    let project_dir = PathBuf::from(opts.cwd);
    let project_dir = std::fs::canonicalize(&project_dir).map_err(|error| InstallFailure {
        message: format!(
            "failed to resolve project directory {}: {error}",
            project_dir.display()
        ),
        code: Some(aube_codes::errors::ERR_AUBE_EMBED_INVALID_PROJECT.to_string()),
    })?;
    if !project_dir.is_dir() {
        return Err(InstallFailure {
            message: format!(
                "project directory is not a directory: {}",
                project_dir.display()
            ),
            code: Some(aube_codes::errors::ERR_AUBE_EMBED_INVALID_PROJECT.to_string()),
        });
    }

    let mut options = InstallOptions::new(project_dir.clone());
    options.frozen_mode = if opts.frozen_lockfile {
        FrozenMode::Frozen
    } else if opts.no_frozen_lockfile {
        FrozenMode::No
    } else {
        FrozenMode::Prefer
    };
    options.strict_no_lockfile = opts.frozen_lockfile;
    options.dep_selection = DepSelection::from_flags(opts.prod, opts.dev, opts.no_optional);
    options.network_mode = if opts.offline {
        NetworkMode::Offline
    } else if opts.prefer_offline {
        NetworkMode::PreferOffline
    } else {
        NetworkMode::Online
    };
    options.ignore_scripts = opts.ignore_scripts;
    options.lockfile_only = opts.lockfile_only;
    options.force = opts.force;
    options.control = InstallControl::silent();
    Ok((project_dir, options))
}

fn invalid_argument(message: impl Into<String>) -> InstallFailure {
    InstallFailure {
        message: message.into(),
        code: Some(aube_codes::errors::ERR_AUBE_FFI_INVALID_ARGUMENT.to_string()),
    }
}

fn initialize_embedder() {
    static INITIALIZED: OnceLock<()> = OnceLock::new();
    INITIALIZED.get_or_init(|| embed::initialize(&embed::AUBE, Vec::new()));
}

fn runtime() -> Result<&'static tokio::runtime::Runtime, InstallFailure> {
    static RUNTIME: OnceLock<Result<tokio::runtime::Runtime, String>> = OnceLock::new();
    RUNTIME
        .get_or_init(|| {
            tokio::runtime::Builder::new_multi_thread()
                .worker_threads(runtime_workers())
                .max_blocking_threads(128)
                .enable_all()
                .build()
                .map_err(|error| error.to_string())
        })
        .as_ref()
        .map_err(|message| InstallFailure {
            message: format!("failed to build aube NIF runtime: {message}"),
            code: Some(aube_codes::errors::ERR_AUBE_FFI_RUNTIME.to_string()),
        })
}

fn runtime_workers() -> usize {
    std::thread::available_parallelism()
        .map(|count| count.get().min(8))
        .unwrap_or(4)
}

rustler::init!("Elixir.Aube.Native");
