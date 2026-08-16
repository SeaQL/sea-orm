# Release Process

This is the release checklist for SeaORM release candidates.

## 1. Choose Version

Check the latest published SeaORM version:

```sh
cargo search sea-orm --limit 5
git tag --list '2.0.0-rc*' --sort=-v:refname | head
```

For a new release candidate, bump to the next `2.0.0-rc.N`.

## 2. Start Clean

Release from `master` after the release PR has been merged and CI has passed.

```sh
git switch master
git pull --ff-only
git status --short
```

The worktree should be clean before starting.

## 3. Bump Versions

Run the bump script:

```sh
bash build-tools/bump.sh 2.0.0-rc.N
```

This creates two commits:

```text
2.0.0-rc.N
update examples
```

The first commit bumps the publishable crate manifests:

- `sea-orm`
- `sea-orm-codegen`
- `sea-orm-cli`
- `sea-orm-macros`
- `sea-orm-migration`
- `sea-orm-sync`

The second commit bumps example `sea-orm` / `sea-orm-migration` dependency comments.

`bump.sh` detects GNU vs BSD `sed`, so it runs on both Linux and macOS.

## 4. Write Changelog

Create a release note file:

```text
changelog/2.0.0-rc.N.md
```

Then link it from `CHANGELOG.md` under `2.0.0 - pending > Release Candidates`.

The changelog should call out:

- user-facing features and fixes
- dependency upgrades
- MSRV changes
- compatibility notes
- expected behavior changes
- example or migration updates

Commit it separately:

```sh
git add CHANGELOG.md changelog/2.0.0-rc.N.md
git commit -m "Add changelog for 2.0.0-rc.N"
```

## 5. Validate Locally

Run the main workspace and the separate workspaces:

```sh
cargo check --workspace
cargo check --manifest-path sea-orm-cli/Cargo.toml
cargo check --manifest-path sea-orm-migration/Cargo.toml
cargo check --manifest-path sea-orm-sync/Cargo.toml
```

Known warnings are acceptable only if they already exist and are unrelated to the release.

## 6. Push and Wait for CI

Push `master`:

```sh
git push origin master
```

Wait for GitHub Actions to pass before publishing. Do not publish while CI is still running or red.

## 7. Publish Crates

After CI passes, run:

```sh
bash build-tools/publish.sh
```

The publish order is:

1. `sea-orm-codegen`
2. `sea-orm-cli`
3. `sea-orm-macros`
4. `sea-orm`
5. `sea-orm-migration`
6. `sea-orm-sync`

If crates.io indexing causes a dependent publish to fail, wait briefly and retry the failed crate.

## 8. Tag and GitHub Release

After publishing succeeds:

```sh
git tag 2.0.0-rc.N
git push origin 2.0.0-rc.N
```

Create a GitHub Release using `changelog/2.0.0-rc.N.md` as the release body.

## 8b. Release prebuilt `sea-orm-cli` binaries

The `.github/workflows/sea-orm-cli-release.yml` workflow builds prebuilt
`sea-orm-cli` binaries and powers `cargo binstall sea-orm-cli`. It triggers on a
**separate** tag namespace, `sea-orm-cli@<version>`, not the `2.0.0-rc.N` release
tag — so it does nothing unless that tag is pushed.

Create the tag as **annotated**, because the workflow's `gh release create
--notes-from-tag` reads the tag's message (a lightweight tag yields empty notes):

```sh
git tag -a "sea-orm-cli@2.0.0-rc.N" -m "sea-orm-cli 2.0.0-rc.N"
git push origin "sea-orm-cli@2.0.0-rc.N"
```

The workflow then builds the 5 targets, attaches the archives to a draft release,
and publishes it. Confirm the assets appear on the `sea-orm-cli@2.0.0-rc.N`
release and that `cargo binstall sea-orm-cli` resolves.

## 9. Verify

Confirm crates.io shows the new versions:

```sh
cargo search sea-orm --limit 5
cargo search sea-orm-cli --limit 5
cargo search sea-orm-migration --limit 5
cargo search sea-orm-sync --limit 5
```

Also check that docs.rs builds have started or completed.
