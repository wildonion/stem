

Onion Virtual Machine for STEM

## version bumping 

### Test

> create dev and release branches to test. 

bash
git checkout -b dev # git checkout -b release
git add .
git commit -m "feat: initial commit"
git push origin main


### CI/CD Notes:

- setup a PAT with ead_repository and write_repository access.
- create CI/CD environment variable and name it GL_ACCESS_TOKEN with the value of your PAT.
- make sure you've unselected the protected optoion if the branch is not protected.

### Just in Case:

- **feat:** on main -> bumps the minor version (e.g., 1.0.0 to 1.1.0).
- **fix:** on main -> bumps the patch version (e.g., 1.0.0 to 1.0.1).
- **feat:** on dev -> bumps a prerelease minor version (e.g., 1.0.0-dev.1 to 1.1.0-dev.1).
- **fix:** on dev -> bumps a prerelease patch version (e.g., 1.0.0-dev.1 to 1.0.1-dev.1).
- **Any commit** on a release branch -> bumps a release candidate version (e.g., 1.0.0-rc.1).
- **docs** or **chore** won't bump the version
- **fix** bumps the patch version, **feat** bumps the minor version, **BREAKING CHANGE** bumps the major version.