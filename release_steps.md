# Release Steps for BaseCamp 0.3.0

## Completed Steps

1. Updated version in Cargo.toml to 0.3.0 ✅
2. Created a CHANGELOG.md file documenting the changes ✅
3. Committed all changes with message "Release version 0.3.0" ✅
4. Created a tag for version 0.3.0 ✅

## Steps to Complete

### 1. Create a GitHub Repository

- Go to https://github.com/new
- Name: "basecamp"
- Description: "A streamlined tool for managing multiple codebases and repositories"
- Choose public or private
- Do not initialize with README, .gitignore, or license
- Click "Create repository"

### 2. Push the Local Repository to GitHub

```bash
# Add the GitHub repository as a remote
git remote add origin https://github.com/YOUR_USERNAME/basecamp.git

# Push the main branch
git push -u origin main

# Push the tag to trigger the release CI/CD workflow
git push origin v0.3.0
```

### 3. Verify Release Artifacts

- Check GitHub Actions to ensure the CI/CD workflow is running
- Verify that release artifacts are created for Linux, macOS, and Windows
- Download and test the artifacts to ensure they work correctly

### 4. Publish to Package Registries (Future)

- Publish to crates.io: `cargo publish`
- Create Homebrew formula
- Create APT package

## Notes

- The GitHub Actions workflow will automatically create a release when you push the v0.3.0 tag
- Make sure your GitHub repository has the necessary secrets configured for the workflow
- The CI/CD workflow will build the release binaries for Linux, macOS, and Windows
