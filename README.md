

## Project STEM 🤖

**STEM** is an injectable brain model with every neuron leveraging the power of actor worker model as a multithreaded and parallel based smart entity, the communication between each neuron however can be taken place either remotely or locally by calling each other's method directly or sending async message known as electric nerve impulses through synapses to get each other's state eventually make an strong synaptic connection through RPC and p2p gossipsub.

## Setup Themis
> first clone the repo then install the followings (refer to https://docs.cossacklabs.com/themis/installation/installation-from-packages/ if you don't want to build themis from source):

```bash
wget http://archive.ubuntu.com/ubuntu/pool/main/o/openssl/libssl1.1_1.1.1f-1ubuntu2_amd64.deb
sudo dpkg -i libssl1.1_1.1.1f-1ubuntu2_amd64.deb
sudo apt update -y && sudo apt upgrade && sudo apt install -y libpq-dev pkg-config build-essential libudev-dev libssl-dev librust-openssl-dev
git clone https://github.com/cossacklabs/themis.git
cd themis
make
sudo make install
# install themis on MacOS M1
brew install libthemis
```
## 🧱 WIPs

- update crates, new rust edition, [wasm32 updation](https://blog.rust-lang.org/2024/04/09/updates-to-rusts-wasi-targets.html)
- schemas.rs, mathista, informationr.rs
- compile to .so, .wasm

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

at the start of each workflow, by default github creates unique `GITHUB_TOKEN` secrets per each workflow, make sure it has read and write permissions for the current workflow: `Settings > Actions > General > Workflow permissions`.

### Just in Case:

> the [skip ci] inside the commit message of semantic release would ignore running ci/cd pipeline 
for version bumping, [skip ci] is an special words used in git committing to ignore running pipeline
other than doing that the version bumping process would stuck in an infinite loop of running ci/cd.

**MAJOR (BREAKING CHANGE).MINOR (feat).PATCH (fix)**

- **fix** bumps the patch version, **feat** bumps the minor version, **BREAKING CHANGE** bumps the major version.
- **feat:** on main -> bumps the minor version (e.g., 1.0.0 to 1.1.0).
- **fix:** on main -> bumps the patch version (e.g., 1.0.0 to 1.0.1).
- **feat:** on dev -> bumps a prerelease minor version (e.g., 1.0.0-dev.1 to 1.1.0-dev.1).
- **fix:** on dev -> bumps a prerelease patch version (e.g., 1.0.0-dev.1 to 1.0.1-dev.1).
- **Any commit** on a release branch -> bumps a release candidate version (e.g., 1.0.0-rc.1).
- **docs** or **chore** won't bump the version.
