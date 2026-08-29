# Rebuilding a venv while a job is running out of it leaves it half-deleted and hangs the job

**Category**: tooling
**Tags**: python, venv, windows, benchmarks, debugging

## Description

`uv venv --clear` (and `python -m venv --clear`) DELETES before it recreates. On Windows the delete fails partway through when any process has files open in it, so the venv is left MISSING SOME PACKAGES rather than intact or absent.

What that looks like: a multi-process job whose parent already imported everything keeps running, but every newly spawned worker dies on `ModuleNotFoundError` for something innocuous (`certifi`). With `multiprocessing` spawn the failure is swallowed — the job hangs with every service idle, no error printed, no result file, and CPU near zero on both sides. It reads like a slow benchmark, not a broken environment.

This cost a full 1.18M-vector benchmark run: the upload phase had completed, the rebuild stripped the venv mid-run, and the eight search workers spawned into an unimportable environment.

Diagnosis: when a job goes idle with no output, run one of its own operations by hand in the same interpreter. `./.venv/Scripts/python.exe -c "import qdrant_client; ..."` surfaced the ModuleNotFoundError immediately, where reading logs and CPU graphs had suggested a slow index.

Rules:
- Never rebuild a venv while anything is running out of it. Stop the processes first.
- A builder that wraps `--clear` must catch the failure and say the venv may now be partially deleted — a bare "Access denied" reads as "nothing happened", which is the opposite of the truth.
- Pipe long background runs to a FILE, not to `| tail`. `tail` buffers everything until the process exits, so a hung run shows an empty log and there is no way to see how far it got.
