import sys
import re

# Get the new version from the command line arguments
new_version = sys.argv[1]

# Update version in pyproject.toml
with open("Cargo.toml", "r") as file:
    pyproject = file.read()

pyproject = re.sub(r'version = ".*"', f'version = "{new_version}"', pyproject)

with open("Cargo.toml", "w") as file:
    file.write(pyproject)

# Log the update process
print(f"Updated pyproject.toml to version {new_version}")

# Update version in Dockerfile
with open("Dockerfile", "r") as file:
    dockerfile = file.read()

dockerfile = re.sub(r'LABEL version=".*"', f'LABEL version="{new_version}"', dockerfile)

with open("Dockerfile", "w") as file:
    file.write(dockerfile)

# Log the update process
print(f"Updated Dockerfile to version {new_version}")