# [AI-generated] Local Mercurial share workflow

Agent-authored by OpenAI Codex (GPT-6 per session instructions; exact model build unverified).
Evaluation: PersonalProjectsBench; run ID unverified (not supplied).

Configuration discovery recognizes `.hg` and `.jj` as well as `.git`, so a
Mercurial repository no longer fails `hn add` with “Not in a git repository”.
Discovery stops at the nearest supported repository instead of reading an
enclosing Git repository's configuration.

For Mercurial, install `hg` and enable its bundled share extension in your
Mercurial configuration:

```ini
[extensions]
share =
```

From the primary Mercurial repository, after its initial commit:

```sh
hn add child
hn list
hn info child
# Edit and commit inside the sibling child directory.
hn remove child
```

Removal refuses uncommitted changes. Committed revisions remain in the shared
repository after removing the child's working directory. Use `hg log` in the
primary repository to inspect them; its working copy is not automatically updated.

Run the executable regression with Mercurial available on PATH:

```sh
cargo test --locked --test mercurial_config_workflow
cargo test --locked --lib finds_non_git_repository_before_enclosing_git_repository
```

The integration test uses a disposable repository, a temporary Mercurial config,
synthetic commits, and the real hn/hg commands. It checks create/list/info, dirty
removal refusal, clean removal, and retention of the child's committed revision.
It reports a skip when hg is unavailable; the recorded local verification used
Mercurial 7.2.4 and executed the workflow.

This change concerns configuration discovery and the primary-repository workflow.
It does not complete Mercurial `--from`/`--no-branch` behavior or shared registry
discovery when commands are run from inside a share. Jujutsu markers have a
configuration-discovery test; a live Jujutsu workflow was not exercised.
