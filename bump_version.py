import sys
import re

# Check if a new version was provided
if len(sys.argv) < 2:
    print("Usage: python script.py <new_version>")
    sys.exit(1)

# Get the new version from the command line arguments
new_version = sys.argv[1]

# ---------- update the main Cargo
# Read the Cargo.toml file
with open("Cargo.toml", "r") as file:
    lines = file.readlines()

# Update the version in the Cargo.toml file
for i in range(len(lines)):
    if lines[i].startswith("version ="):
        lines[i] = f'version = "{new_version}"\n'  # Update the line with the new version
        break  # Exit the loop once the version is updated

# Write the updated content back to Cargo.toml
with open("Cargo.toml", "w") as file:
    file.writelines(lines)

# Log the update process
print(f"Updated Cargo.toml to version {new_version}")
# --------------------------------------


# ---------- update the stemlib Cargo

# Read the Cargo.toml file
with open("stemlib/Cargo.toml", "r") as file:
    lines = file.readlines()

# Update the version in the Cargo.toml file
for i in range(len(lines)):
    if lines[i].startswith("version ="):
        lines[i] = f'version = "{new_version}"\n'  # Update the line with the new version
        break  # Exit the loop once the version is updated

# Write the updated content back to Cargo.toml
with open("stemlib/Cargo.toml", "w") as file:
    file.writelines(lines)

# Log the update process
print(f"Updated stemlib/Cargo.toml to version {new_version}")
# --------------------------------------

# Update the version in Dockerfile
with open("Dockerfile", "r") as file:
    dockerfile = file.read()

# Update the version label in the Dockerfile
dockerfile = re.sub(r'LABEL version=".*"', f'LABEL version="{new_version}"', dockerfile)

# Write the updated content back to Dockerfile
with open("Dockerfile", "w") as file:
    file.write(dockerfile)

# Log the update process for Dockerfile
print(f"Updated Dockerfile to version {new_version}")
