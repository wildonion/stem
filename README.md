

Onion Virtual Machine for STEM

## version bumping 

### Test

> create `dev` and `release` branches. 

```bash
git checkout -b dev # git checkout -b release
git branch --set-upstream-to=origin/dev
git add .
git commit -m "feat: adding great feature"
git push origin main
```

### CI/CD Notes:

at the start of each workflow, by default github creates unique `GITHUB_TOKEN` secrets per each workflow, make sure it has read and write permissions for the current workflow: `Settings > Actions > General`.

### Just in Case:

> the [skip ci] inside the commit message of semantic release would ignore running ci/cd pipeline 
for version bumping, [skip ci] is an special words used in git committing to ignore running pipeline
other than doing that the version bumping process would stuck in an infinite loop of running ci/cd.

**MAJOR.MINOR.PATCH**
**BREAKING CHANGE.feat.fix**

- **fix** bumps the patch version, **feat** bumps the minor version, **BREAKING CHANGE** bumps the major version.
- **feat:** on main -> bumps the minor version (e.g., 1.0.0 to 1.1.0).
- **fix:** on main -> bumps the patch version (e.g., 1.0.0 to 1.0.1).
- **feat:** on dev -> bumps a prerelease minor version (e.g., 1.0.0-dev.1 to 1.1.0-dev.1).
- **fix:** on dev -> bumps a prerelease patch version (e.g., 1.0.0-dev.1 to 1.0.1-dev.1).
- **Any commit** on a release branch -> bumps a release candidate version (e.g., 1.0.0-rc.1).
- **docs** or **chore** won't bump the version.
