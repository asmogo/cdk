# Migration Plan: Centralized FFI Binding Generation & Publishing

## Objective
To centralize the generation, testing, and distribution of language bindings (Swift, Dart, Kotlin) in the main `cdk` repository. This eliminates duplicated build logic in downstream repositories, ensures version consistency, and simplifies the CI/CD pipelines of the language-specific repos.

---

## Phase 1: Preparation & GitHub Admin
Before touching any code, the GitHub organization/repository settings need to be configured to allow the `cdk` repository to push to the downstream repositories.

1. **Create a Machine User or Personal Access Token (PAT):**
   * Generate a GitHub PAT (Fine-grained or Classic) that has `repo` (Read/Write) access to `cashubtc/cdk-swift`, `cashubtc/cdk-dart`, and `cashubtc/cdk-kotlin`.
2. **Add Secret to `cdk` Repository:**
   * Go to `cashubtc/cdk` -> Settings -> Secrets and variables -> Actions.
   * Add a new repository secret named `BINDINGS_PUBLISH_TOKEN` containing the PAT.

---

## Phase 2: Standardize Output Directories in `cdk`
Currently, scripts like `generate-bindings.sh` dump files into various places (e.g., `Package.swift` in the repo root). We need to isolate the generated files into a "staging" directory that perfectly mirrors the structure of the downstream repositories.

1. **Update Swift Build Script (`bindings/swift/generate-bindings.sh`):**
   * Change the output directory of `Package.swift` from the repo root to a new `bindings/swift/dist/` folder.
   * Move the generated `Sources/` and `build/xcframework/` into `bindings/swift/dist/`.
   * *Goal:* The `bindings/swift/dist/` folder should look exactly like the root of the `cdk-swift` repository.
2. **Update Dart/Kotlin build commands (`justfile` or Nix configs):**
   * Ensure `just binding-dart` and `just binding-kotlin` output their generated files into clean `bindings/dart/dist/` and `bindings/kotlin/dist/` folders.
3. **Update existing CI (`ci.yml`):**
   * Verify that the existing `test-swift`, `test-dart`, etc., still run correctly against the new `dist/` paths. **Testing remains exactly as it is today to prevent regressions.**

---

## Phase 3: Create Publishing Workflow in `cdk`
Create a new GitHub Action workflow in `cdk` that triggers on merges to `main` (or on published releases) to build and push the bindings.

**File:** `.github/workflows/publish-bindings.yml`

```yaml
name: Publish Bindings to Downstream Repos

on:
  push:
    branches:
      - main
    paths:
      - 'crates/cdk-ffi/**'
      - 'bindings/**'
  # Uncomment to run on releases:
  # release:
  #   types: [published]

jobs:
  publish-swift:
    name: Publish Swift Bindings
    runs-on: macos-latest
    steps:
      - uses: actions/checkout@v4
      - uses: DeterminateSystems/nix-installer-action@main
      - uses: cachix/cachix-action@v16
        with:
          name: cashudevkit
          authToken: ${{ secrets.CACHIX_AUTH_TOKEN }}
      
      - name: Build Swift Bindings & XCFramework
        run: nix develop -L .#bindings --command just binding-swift

      - name: Checkout cdk-swift repo
        uses: actions/checkout@v4
        with:
          repository: cashubtc/cdk-swift
          path: downstream-repo
          token: ${{ secrets.BINDINGS_PUBLISH_TOKEN }}

      - name: Sync generated files
        run: |
          # Copy everything from the dist folder into the downstream repo
          # Note: Adjust paths based on Phase 2 changes
          cp -R bindings/swift/dist/* downstream-repo/
          
      - name: Create Pull Request in cdk-swift
        uses: peter-evans/create-pull-request@v6
        with:
          path: downstream-repo
          token: ${{ secrets.BINDINGS_PUBLISH_TOKEN }}
          commit-message: "chore: auto-update bindings from cdk@${{ github.sha }}"
          title: "Automated FFI Binding Update"
          branch: "auto-update-bindings"
          body: "Automated update of Swift bindings and XCFramework from cashubtc/cdk commit ${{ github.sha }}."

  # (Repeat similar job structures for publish-dart and publish-kotlin)
```
*Note: Using `peter-evans/create-pull-request` opens a PR in the downstream repo rather than force-pushing to main, giving maintainers a chance to review the diff before merging.*

---

## Phase 4: Clean up Downstream Repositories
Once the publishing pipeline from `cdk` is confirmed working, strip down the `cdk-swift`, `cdk-dart`, and `cdk-kotlin` repositories.

1. **Delete Build Scripts:**
   * Remove `generate-bindings.sh`, `rust-toolchain.toml`, and any Nix flake/shell configurations used for building Rust.
2. **Simplify CI/CD:**
   * Update the `.github/workflows/` in the downstream repos.
   * **Swift:** CI should only run `swift test`.
   * **Dart:** CI should only run `dart test` / `dart analyze`.
   * **Kotlin:** CI should only run `./gradlew test`.
3. **Update README.md:**
   * Add a notice: *"🚨 NOTE: The FFI bindings and core logic in this repository are auto-generated. Do not edit them here. To contribute to the core library, submit PRs to the main [cashubtc/cdk](https://github.com/cashubtc/cdk) repository."*

---

## Phase 5: Handling Binary Size (Future Optimization for Swift)
If the `CashuDevKitFFI.xcframework` becomes too large for Git tracking in `cdk-swift` (> 100MB, triggering GitHub LFS requirements):
1. Modify the `publish-bindings.yml` workflow to upload `CashuDevKitFFI.xcframework.zip` as a **GitHub Release Asset** on the `cdk-swift` repository instead of committing the binary.
2. The workflow will dynamically calculate the checksum, update `Package.swift`'s `url:` and `checksum:` fields to point to the new Release Asset, and only commit the `Package.swift` source file change.
