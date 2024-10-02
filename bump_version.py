import sys
import re

# Get the new version from the command line arguments
new_version = sys.argv[1]

# Update version in cargo_toml.toml
with open("Cargo.toml", "r") as file:
    cargo_toml = file.read()

cargo_toml = re.sub(r'(?<=\[package\][\s\S]*?version\s*=\s*")(\d+\.\d+\.\d+[^"]*)', new_version, cargo_toml)

with open("Cargo.toml", "w") as file:
    file.write(cargo_toml)

# Log the update process
print(f"Updated cargo_toml.toml to version {new_version}")

# Update version in Dockerfile
with open("Dockerfile", "r") as file:
    dockerfile = file.read()

dockerfile = re.sub(r'LABEL version=".*"', f'LABEL version="{new_version}"', dockerfile)

with open("Dockerfile", "w") as file:
    file.write(dockerfile)

# Log the update process
print(f"Updated Dockerfile to version {new_version}")