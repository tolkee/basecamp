# Publishing BaseCamp to Crates.io

## Prerequisites

1. Create an account on [crates.io](https://crates.io)
2. Log in with your account: `cargo login`
3. Update the repository URL in Cargo.toml with your actual GitHub username

## Preparing for Publication

1. Update your GitHub username in Cargo.toml:

   ```toml
   repository = "https://github.com/YOUR_USERNAME/basecamp"
   ```

2. Make sure all required fields are in Cargo.toml:

   - name: "basecamp"
   - version: "0.1.0"
   - description: "A streamlined tool for managing multiple codebases and repositories"
   - authors: ["BaseCamp Developers"]
   - repository: Your GitHub repository URL
   - license: "MIT"

3. Create a license file if not already present:
   ```
   touch LICENSE
   ```

## Publishing

1. Verify that the package compiles:

   ```
   cargo check
   ```

2. Package the crate to ensure it contains all necessary files:

   ```
   cargo package
   ```

3. Publish the crate:
   ```
   cargo publish
   ```

## After Publication

1. Update the README with installation instructions:

   ```
   cargo install basecamp
   ```

2. Tag your GitHub repository with the published version if you haven't already:
   ```
   git tag -a v0.1.0 -m "Release 0.1.0"
   git push origin v0.1.0
   ```

## Verifying Publication

1. Check that your crate appears on crates.io: https://crates.io/crates/basecamp
2. Try installing your crate from crates.io:
   ```
   cargo install basecamp
   ```
3. Verify that the installed version is 0.1.0:
   ```
   basecamp --version
   ```
