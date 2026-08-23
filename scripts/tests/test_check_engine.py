"""`check_engine` — the #265 pin/tree/binary agreement gate.

Every case here is a state the gate must REFUSE, plus its control. A gate tested
only on the passing path is a gate nobody has proved fires, which is the shape
this whole EPIC exists to correct.
"""

from __future__ import annotations

import json
import subprocess

import pytest

from check_engine import (
    Drift,
    assert_agrees,
    assert_capabilities,
    manifest_staleness,
    read_pin,
    reported_engine,
)

SHA = "84740d48f0e1a2b3c4d5e6f708192a3b4c5d6e7f"


# --- read_pin -------------------------------------------------------------


def write_manifest(tmp_path, body: str):
    path = tmp_path / "Cargo.toml"
    path.write_text(body, encoding="utf-8")
    return path


def test_a_rev_pin_and_a_tag_pin_are_both_read(tmp_path):
    rev = write_manifest(
        tmp_path,
        '[dependencies]\nquire-rs = { git = "https://x", rev = "84740d4" }\n',
    )
    assert read_pin(rev) == ("rev", "84740d4")

    tag = write_manifest(
        tmp_path / "t" if (tmp_path / "t").mkdir() or True else tmp_path,
        '[dependencies]\nquire-rs = { git = "https://x", tag = "v0.45.0" }\n',
    )
    assert read_pin(tag) == ("tag", "v0.45.0")


def test_an_unpinned_git_dependency_is_refused(tmp_path):
    """The state the gate exists for: a dep that resolves to whatever the
    default branch happens to be at build time is the opposite of provenance."""
    manifest = write_manifest(
        tmp_path, '[dependencies]\nquire-rs = { git = "https://x" }\n'
    )
    with pytest.raises(Drift, match="declares no `rev`/`tag`"):
        read_pin(manifest)


def test_another_crates_pin_is_not_read_as_ours(tmp_path):
    """`quire-rs` must be matched as the dependency NAME. A sibling entry
    pinning its own rev must not supply ours."""
    manifest = write_manifest(
        tmp_path,
        '[dependencies]\nother-crate = { git = "https://y", rev = "deadbee" }\n'
        'quire-rs = { git = "https://x", rev = "84740d4" }\n',
    )
    assert read_pin(manifest) == ("rev", "84740d4")


def test_a_missing_manifest_is_refused(tmp_path):
    with pytest.raises(Drift, match="no consumer manifest"):
        read_pin(tmp_path / "absent" / "Cargo.toml")


# --- reported_engine ------------------------------------------------------


def test_a_payload_without_an_engine_block_is_refused():
    """A binary predating quire-cli#68 cannot say what it links. Refusing is
    the point — accepting it silently is the state that produced four passes of
    unsourced numbers."""
    with pytest.raises(Drift, match="predates quire-cli#68"):
        reported_engine({"totals": {"backed": 1, "total": 2}})


def test_a_malformed_engine_block_is_refused():
    with pytest.raises(Drift, match="not a version"):
        reported_engine({"engine": {"cli": "0.30.2", "capabilities": []}})
    with pytest.raises(Drift, match="not a list"):
        reported_engine({"engine": {"cli": "0.1", "engine": "0.45.0", "capabilities": "all"}})


def test_a_well_formed_block_yields_its_version_and_tokens():
    version, tokens = reported_engine(
        {"engine": {"cli": "0.30.2", "engine": "0.45.0", "capabilities": ["binding_census"]}}
    )
    assert version == "0.45.0"
    assert tokens == ["binding_census"]


# --- assert_agrees --------------------------------------------------------


def test_a_tag_pin_agrees_with_the_version_the_binary_reports():
    assert_agrees("v0.45.0", "0.45.0", SHA)
    assert_agrees("0.45.0", "0.45.0", SHA)


def test_a_rev_pin_agrees_with_an_abbreviated_sha():
    assert_agrees("84740d4", "84740d4", SHA)
    assert_agrees("84740d4", "84740d48", SHA)


def test_a_manifest_and_a_binary_that_disagree_are_refused():
    """The defect the whole gate is for: the manifest is what somebody wrote,
    the payload is what actually ran."""
    with pytest.raises(Drift, match="pins `v0.45.0` but the binary"):
        assert_agrees("v0.45.0", "0.42.0", SHA)


def test_a_prefix_shorter_than_gits_abbreviation_floor_is_refused():
    """Found by this test: the first implementation accepted ANY prefix of the
    resolved sha, so a binary reporting `8` satisfied a pin on `84740d4…`. One
    hex digit prefixes one sha in sixteen; that is not evidence."""
    with pytest.raises(Drift):
        assert_agrees("84740d4", "8", SHA)
    with pytest.raises(Drift):
        assert_agrees("84740d4", "84740", SHA)
    # The control: at the floor, it is a real abbreviation and does agree.
    assert_agrees("84740d4", "84740d4", SHA)


def test_a_describe_suffix_does_not_satisfy_a_bare_tag_pin():
    """Also found here: the first implementation allowed `reported` to merely
    START WITH the declared tag, so `0.45.0-3-gabc1234` — three commits past
    v0.45.0 — read as agreement with a pin on v0.45.0."""
    with pytest.raises(Drift):
        assert_agrees("v0.45.0", "0.45.0-3-gabc1234", SHA)


# --- assert_capabilities --------------------------------------------------


def test_a_missing_capability_aborts_and_names_the_token():
    """#265 AC-4: abort, do not omit the metric and continue. A sweep missing
    `binding_census` still prints a coverage percentage."""
    with pytest.raises(Drift, match="binding_census"):
        assert_capabilities(["metrics_envelope"], ["binding_census"])


def test_every_missing_token_is_named_not_just_the_first():
    with pytest.raises(Drift) as caught:
        assert_capabilities([], ["binding_census", "metrics_envelope"])
    assert "binding_census" in str(caught.value)
    assert "metrics_envelope" in str(caught.value)


def test_the_control_a_binary_carrying_them_all_passes():
    assert_capabilities(["binding_census", "metrics_envelope"], ["binding_census"])
    assert_capabilities([], [])


# --- manifest_staleness (advisory, never a failure) ------------------------


def test_manifest_staleness_is_advisory_and_names_the_two_numbers(tmp_path):
    """Deliberately NOT a failure: this crate's manifest has read 0.33.0 since
    long before the tags it ships under, so promoting this would make `ci` red
    on every correct configuration and the gate would be switched off."""
    subprocess.run(["git", "init", "-q", str(tmp_path)], check=True)
    (tmp_path / "Cargo.toml").write_text('[package]\nversion = "0.33.0"\n', encoding="utf-8")
    subprocess.run(["git", "-C", str(tmp_path), "add", "-A"], check=True)
    subprocess.run(
        ["git", "-C", str(tmp_path), "-c", "user.email=t@t", "-c", "user.name=t",
         "commit", "-qm", "x"],
        check=True,
    )
    subprocess.run(["git", "-C", str(tmp_path), "tag", "v0.45.0"], check=True)

    advisory = manifest_staleness(tmp_path)
    assert advisory is not None
    assert "0.33.0" in advisory and "v0.45.0" in advisory

    # The control: a manifest that agrees with its tag says nothing.
    (tmp_path / "Cargo.toml").write_text('[package]\nversion = "0.45.0"\n', encoding="utf-8")
    assert manifest_staleness(tmp_path) is None


# --- end to end -----------------------------------------------------------


def test_the_gate_fails_on_a_deliberately_mismatched_pin(tmp_path):
    """#265 AC-1, driven through the script's own entry point rather than its
    functions: a pin naming a commit this tree has never contained must exit
    non-zero."""
    import pathlib

    repo = pathlib.Path(__file__).resolve().parent.parent.parent
    consumer = tmp_path / "quire-cli"
    consumer.mkdir()
    (consumer / "Cargo.toml").write_text(
        '[dependencies]\nquire-rs = { git = "https://x", rev = "0000000000000000000000000000000000000000" }\n',
        encoding="utf-8",
    )
    done = subprocess.run(
        ["python3", str(repo / "scripts" / "check_engine.py"),
         "--repo", str(repo), "--consumer", str(consumer)],
        capture_output=True,
        text=True,
        check=False,
    )
    assert done.returncode == 1, done.stdout
    assert "does not exist in this repository" in done.stderr


def test_the_gate_passes_on_the_real_pin(tmp_path):
    """The control. Skips rather than fails when the sibling workspace is
    absent — a missing checkout is an environment fact, not drift."""
    import pathlib

    repo = pathlib.Path(__file__).resolve().parent.parent.parent
    consumer = repo.parent / "quire-cli"
    if not (consumer / "Cargo.toml").is_file():
        pytest.skip("no quire-cli workspace beside this repository")

    done = subprocess.run(
        ["python3", str(repo / "scripts" / "check_engine.py"),
         "--repo", str(repo), "--consumer", str(consumer)],
        capture_output=True,
        text=True,
        check=False,
    )
    assert done.returncode == 0, done.stderr
    assert "quire-cli pins" in done.stdout
    # The distance is REPORTED, never a verdict: during a campaign the consumer
    # pins the last merged commit while this tree moves ahead, and that is the
    # normal state. What makes distance a defect is a missing capability, which
    # `--require` catches by name.
    assert "HEAD" in done.stdout
    assert "check_engine: OK" in done.stdout


def test_a_payload_missing_a_required_capability_exits_non_zero(tmp_path):
    """#265 AC-4 end to end."""
    import pathlib

    repo = pathlib.Path(__file__).resolve().parent.parent.parent
    consumer = repo.parent / "quire-cli"
    if not (consumer / "Cargo.toml").is_file():
        pytest.skip("no quire-cli workspace beside this repository")

    pin = read_pin(consumer / "Cargo.toml")[1]
    payload = tmp_path / "payload.json"
    payload.write_text(
        json.dumps({"engine": {"cli": "0.30.2", "engine": pin, "capabilities": ["metrics_envelope"]}}),
        encoding="utf-8",
    )
    done = subprocess.run(
        ["python3", str(repo / "scripts" / "check_engine.py"),
         "--repo", str(repo), "--consumer", str(consumer),
         "--payload", str(payload), "--require", "binding_census"],
        capture_output=True,
        text=True,
        check=False,
    )
    assert done.returncode == 1, done.stdout
    assert "binding_census" in done.stderr


# --- defects the #283 review found, each with its control -----------------


def test_a_commented_out_pin_is_not_a_pin(tmp_path):
    """Found by review: the first regex matched a commented-out line. An
    operator commenting the old pin above the new one is the ordinary way this
    file gets edited, and it made the gate validate the dead line."""
    manifest = write_manifest(
        tmp_path,
        '[dependencies]\n'
        '# quire-rs = { git = "https://x", rev = "deadbee" }\n'
        'quire-rs = { git = "https://x", rev = "84740d4" }\n',
    )
    assert read_pin(manifest) == ("rev", "84740d4")


def test_a_differently_named_crate_does_not_supply_our_pin(tmp_path):
    """Found by review: no left word boundary, so `my-quire-rs` was read as
    ours. The existing sibling test only covered a PRECEDING entry."""
    manifest = write_manifest(
        tmp_path, '[dependencies]\nmy-quire-rs = { git = "https://y", rev = "ddddddd" }\n'
    )
    with pytest.raises(Drift, match="declares no `rev`/`tag`"):
        read_pin(manifest)


def test_the_toml_table_form_is_read(tmp_path):
    """Found by review: Cargo writes this whenever the entry grows past one
    line, and the first version rejected it as unpinned — a false failure
    accusing a correctly pinned consumer."""
    manifest = write_manifest(
        tmp_path,
        '[dependencies.quire-rs]\ngit = "https://x"\nrev = "84740d4"\n\n'
        '[dev-dependencies]\ntempfile = "3"\n',
    )
    assert read_pin(manifest) == ("rev", "84740d4")


def test_a_branch_name_is_not_a_pin(tmp_path):
    """Found by review: `rev = "main"` resolved fine and passed. It is the
    'resolves to whatever the default branch happens to be' state this
    function's own error text calls the opposite of provenance."""
    manifest = write_manifest(
        tmp_path, '[dependencies]\nquire-rs = { git = "https://x", rev = "main" }\n'
    )
    with pytest.raises(Drift, match="neither a commit sha nor a version tag"):
        read_pin(manifest)

    # The controls: both legitimate forms still pass.
    sha = write_manifest(
        tmp_path / "a" if (tmp_path / "a").mkdir() is None else tmp_path,
        '[dependencies]\nquire-rs = { git = "https://x", rev = "84740d4" }\n',
    )
    assert read_pin(sha) == ("rev", "84740d4")
    tag = write_manifest(
        tmp_path / "b" if (tmp_path / "b").mkdir() is None else tmp_path,
        '[dependencies]\nquire-rs = { git = "https://x", tag = "v0.45.0" }\n',
    )
    assert read_pin(tag) == ("tag", "v0.45.0")


def test_a_relative_consumer_path_is_not_reduced_to_its_basename(tmp_path):
    """Found by review, and the worst of the three: `repo.parent /
    consumer.name` threw away everything but the last component, so
    `--consumer nested/quire-cli` silently read `../quire-cli` instead and
    reported OK on a manifest it had never opened."""
    import pathlib

    repo = pathlib.Path(__file__).resolve().parent.parent.parent
    nested = tmp_path / "nested" / "quire-cli"
    nested.mkdir(parents=True)
    (nested / "Cargo.toml").write_text(
        '[dependencies]\nquire-rs = { git = "https://x", rev = "0000000000000000000000000000000000000000" }\n',
        encoding="utf-8",
    )
    done = subprocess.run(
        ["python3", str(repo / "scripts" / "check_engine.py"),
         "--repo", str(repo), "--consumer", str(nested)],
        capture_output=True, text=True, check=False,
    )
    assert done.returncode == 1, done.stdout
    assert "does not exist in this repository" in done.stderr


def test_an_unreachable_pin_is_refused(tmp_path):
    """Found by review: the module's headline third fact — 'a pin to a commit
    on an abandoned branch resolves fine and is still drift' — had no test at
    all. Built here as a real git repo with a real orphan commit."""
    import pathlib

    from check_engine import assert_reachable

    repo = tmp_path / "engine"
    repo.mkdir()
    run = lambda *a: subprocess.run(
        ["git", "-C", str(repo), "-c", "user.email=t@t", "-c", "user.name=t", *a],
        check=True, capture_output=True,
    )
    subprocess.run(["git", "init", "-q", "-b", "main", str(repo)], check=True)
    (repo / "f").write_text("1")
    run("add", "-A")
    run("commit", "-qm", "one")

    # An orphan branch: a real commit this repo contains, unreachable from HEAD.
    run("checkout", "-q", "--orphan", "abandoned")
    (repo / "g").write_text("2")
    run("add", "-A")
    run("commit", "-qm", "abandoned")
    orphan = subprocess.run(
        ["git", "-C", str(repo), "rev-parse", "HEAD"],
        capture_output=True, text=True, check=True,
    ).stdout.strip()
    run("checkout", "-q", "main")

    with pytest.raises(Drift, match="not reachable from this tree's HEAD"):
        assert_reachable(repo, orphan, "abandoned")

    # The control: HEAD itself is reachable from HEAD.
    head = subprocess.run(
        ["git", "-C", str(repo), "rev-parse", "HEAD"],
        capture_output=True, text=True, check=True,
    ).stdout.strip()
    assert_reachable(repo, head, "main")


def test_a_reported_version_may_carry_a_leading_v(tmp_path):
    """Found by review: only `declared` was v-normalized, so a payload
    reporting `v0.45.0` against `tag = "v0.45.0"` was refused with a
    self-contradictory message naming the same string twice."""
    assert_agrees("v0.45.0", "v0.45.0", SHA)
    assert_agrees("0.45.0", "v0.45.0", SHA)


def test_a_malformed_payload_file_prints_the_refusal_line_not_a_traceback(tmp_path):
    """Found by review: `json.loads` on --payload raised through the Drift
    handler, so a bad file exited with a raw traceback instead of the
    `check_engine: FAIL` line every other refusal prints."""
    import pathlib

    repo = pathlib.Path(__file__).resolve().parent.parent.parent
    consumer = repo.parent / "quire-cli"
    if not (consumer / "Cargo.toml").is_file():
        pytest.skip("no quire-cli workspace beside this repository")

    bad = tmp_path / "bad.json"
    bad.write_text("{not json", encoding="utf-8")
    done = subprocess.run(
        ["python3", str(repo / "scripts" / "check_engine.py"),
         "--repo", str(repo), "--consumer", str(consumer), "--payload", str(bad)],
        capture_output=True, text=True, check=False,
    )
    assert done.returncode == 1
    assert "check_engine: FAIL" in done.stderr
    assert "Traceback" not in done.stderr


def test_the_ac4_capability_abort_is_independent_of_the_pin_form(tmp_path):
    """Found by review: the original AC-4 end-to-end test stuffed the pin
    string into `engine.engine`, so it only reached `assert_capabilities`
    because today's pin happens to be a rev. With a tag pin it died earlier in
    `assert_agrees` and never mentioned the missing token.

    Driven against a fixture consumer whose pin this repo really contains, so
    the form is controlled here rather than inherited from the working tree."""
    import pathlib

    repo = pathlib.Path(__file__).resolve().parent.parent.parent
    head = subprocess.run(
        ["git", "-C", str(repo), "rev-parse", "HEAD"],
        capture_output=True, text=True, check=True,
    ).stdout.strip()

    consumer = tmp_path / "consumer"
    consumer.mkdir()
    (consumer / "Cargo.toml").write_text(
        f'[dependencies]\nquire-rs = {{ git = "https://x", rev = "{head}" }}\n',
        encoding="utf-8",
    )
    payload = tmp_path / "payload.json"
    payload.write_text(
        json.dumps({"engine": {"cli": "0.30.2", "engine": head,
                               "capabilities": ["metrics_envelope"]}}),
        encoding="utf-8",
    )
    done = subprocess.run(
        ["python3", str(repo / "scripts" / "check_engine.py"),
         "--repo", str(repo), "--consumer", str(consumer),
         "--payload", str(payload), "--require", "binding_census"],
        capture_output=True, text=True, check=False,
    )
    assert done.returncode == 1, done.stdout
    assert "binding_census" in done.stderr


# --- sweep_coverage: the deliverable script had no tests at all ------------


def test_the_sweep_refuses_to_print_a_zero_over_an_unmeasured_corpus(tmp_path, monkeypatch):
    """Found by review: when NO repository produced a readable payload, the
    sweep printed `"engine": null` and exited 0 — a zero headline over a corpus
    nothing measured, which is the silent-zero defect arrived at by a different
    route than the capability abort guards.

    `assert_capabilities` runs only on the first CLEAN payload, so a run where
    every repo errors never reaches it at all."""
    import sweep_coverage

    root = tmp_path / "dev"
    (root / "repo-a" / "spec").mkdir(parents=True)
    (root / "repo-a" / "spec" / "tests.md").write_text("# x\n", encoding="utf-8")

    monkeypatch.setattr(sweep_coverage, "build_engine", lambda *a, **k: "/nonexistent/quire")
    monkeypatch.setattr(sweep_coverage, "repos", lambda _root: [root / "repo-a"])
    monkeypatch.setattr(
        sweep_coverage, "coverage", lambda *a, **k: {"error": "engine absent"}
    )
    monkeypatch.setattr("sys.argv", ["sweep_coverage.py", str(root)])

    assert sweep_coverage.main() == 1


def test_the_sweep_takes_provenance_from_the_payload_and_aborts_on_a_missing_token(
    tmp_path, monkeypatch
):
    """AC-3 and AC-4 through the sweep's OWN loop, not through
    `check_engine.assert_capabilities` in isolation — the review's point being
    that nothing proved the sweep aborts."""
    import sweep_coverage

    root = tmp_path / "dev"
    (root / "repo-a" / "spec").mkdir(parents=True)

    monkeypatch.setattr(sweep_coverage, "build_engine", lambda *a, **k: "/built/quire")
    monkeypatch.setattr(sweep_coverage, "repos", lambda _root: [root / "repo-a"])
    monkeypatch.setattr("sys.argv", ["sweep_coverage.py", str(root)])

    # A payload from an engine predating the census: the sweep must abort.
    monkeypatch.setattr(
        sweep_coverage,
        "coverage",
        lambda *a, **k: {
            "totals": {"backed": 0, "total": 0},
            "groups": [],
            "untracked_symbols": [],
            "engine": {"cli": "0.29.0", "engine": "0.42.0", "capabilities": ["suspicions"]},
        },
    )
    assert sweep_coverage.main() == 1

    # The control: the same shape carrying the token measures and exits 0.
    monkeypatch.setattr(
        sweep_coverage,
        "coverage",
        lambda *a, **k: {
            "totals": {"backed": 1, "total": 2},
            "groups": [{"target": "test-case", "backed": 1, "total": 2}],
            "untracked_symbols": [],
            "engine": {
                "cli": "0.30.2",
                "engine": "84740d4",
                "capabilities": ["binding_census"],
            },
        },
    )
    assert sweep_coverage.main() == 0


def test_the_sweep_refuses_a_binary_that_changes_mid_run(tmp_path, monkeypatch):
    """The mid-run swap guard, which the review flagged as unreachable-and-
    untested. Driven through the loop with two payloads reporting different
    engines, which is the state it exists for."""
    import sweep_coverage

    root = tmp_path / "dev"
    for name in ("repo-a", "repo-b"):
        (root / name / "spec").mkdir(parents=True)

    versions = iter(["84740d4", "0000000"])

    def payload(*_a, **_k):
        return {
            "totals": {"backed": 0, "total": 0},
            "groups": [],
            "untracked_symbols": [],
            "engine": {
                "cli": "0.30.2",
                "engine": next(versions),
                "capabilities": ["binding_census"],
            },
        }

    monkeypatch.setattr(sweep_coverage, "build_engine", lambda *a, **k: "/built/quire")
    monkeypatch.setattr(
        sweep_coverage, "repos", lambda _root: [root / "repo-a", root / "repo-b"]
    )
    monkeypatch.setattr(sweep_coverage, "coverage", payload)
    monkeypatch.setattr("sys.argv", ["sweep_coverage.py", str(root)])

    assert sweep_coverage.main() == 1
