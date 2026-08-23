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
    assert "reachable from HEAD" in done.stdout


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
