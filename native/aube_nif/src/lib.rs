use rustler::{Encoder, Env, NifMap, NifResult, Term};
use std::path::PathBuf;
use std::sync::OnceLock;

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
    node_linker: Option<String>,
    registry: Option<String>,
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
    let request = match request_from_opts(opts) {
        Ok(request) => request,
        Err(failure) => return Ok((atoms::error(), failure).encode(env)),
    };

    let result = runtime().block_on(aube::embedded::install(request));
    Ok(match result {
        Ok(outcome) => (
            atoms::ok(),
            InstallResult {
                project_dir: outcome.project_dir.to_string_lossy().into_owned(),
                duration_ms: outcome.duration_ms,
            },
        )
            .encode(env),
        Err(err) => (
            atoms::error(),
            InstallFailure {
                message: err.message,
                code: err.code,
            },
        )
            .encode(env),
    })
}

fn request_from_opts(opts: InstallOpts) -> Result<aube::embedded::InstallRequest, InstallFailure> {
    let frozen_count = [
        opts.frozen_lockfile,
        opts.no_frozen_lockfile,
        opts.prefer_frozen_lockfile,
    ]
    .into_iter()
    .filter(|flag| *flag)
    .count();
    if frozen_count > 1 {
        return Err(failure(
            "frozen lockfile options are mutually exclusive",
            Some("invalid_options"),
        ));
    }
    if opts.prod && opts.dev {
        return Err(failure(
            "prod and dev install modes are mutually exclusive",
            Some("invalid_options"),
        ));
    }
    if opts.offline && opts.prefer_offline {
        return Err(failure(
            "offline and prefer_offline are mutually exclusive",
            Some("invalid_options"),
        ));
    }

    let frozen_lockfile = if opts.frozen_lockfile {
        aube::embedded::FrozenLockfile::Frozen
    } else if opts.no_frozen_lockfile {
        aube::embedded::FrozenLockfile::No
    } else if opts.prefer_frozen_lockfile {
        aube::embedded::FrozenLockfile::Prefer
    } else {
        aube::embedded::FrozenLockfile::Auto
    };

    Ok(aube::embedded::InstallRequest {
        project_dir: PathBuf::from(opts.cwd),
        frozen_lockfile,
        prod: opts.prod,
        dev: opts.dev,
        no_optional: opts.no_optional,
        offline: opts.offline,
        prefer_offline: opts.prefer_offline,
        ignore_scripts: opts.ignore_scripts,
        lockfile_only: opts.lockfile_only,
        force: opts.force,
        node_linker: opts.node_linker,
        registry: opts.registry,
    })
}

fn failure(message: impl Into<String>, code: Option<&str>) -> InstallFailure {
    InstallFailure {
        message: message.into(),
        code: code.map(ToOwned::to_owned),
    }
}

fn runtime() -> &'static tokio::runtime::Runtime {
    static RUNTIME: OnceLock<tokio::runtime::Runtime> = OnceLock::new();
    RUNTIME.get_or_init(|| {
        tokio::runtime::Builder::new_multi_thread()
            .worker_threads(runtime_workers())
            .max_blocking_threads(128)
            .enable_all()
            .build()
            .expect("failed to build aube NIF runtime")
    })
}

fn runtime_workers() -> usize {
    std::thread::available_parallelism()
        .map(|count| count.get().min(8))
        .unwrap_or(4)
}

rustler::init!("Elixir.Aube.Native");
