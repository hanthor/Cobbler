# Application ID (reverse domain name)
app-id := "com.github.hanthor.Cobbler"
# The manifest file
manifest := "build-aux/" + app-id + ".json"
# The build directory
build-dir := "build-dir"
# The repository directory for the flatpak repo
repo-dir := "repo"
# The command to run flatpak-builder via flatpak
builder := "flatpak run org.flatpak.Builder"

# Default recipe: list available recipes
default:
    @just --list

# Ensure the flatpak-builder flatpak is installed
setup:
    @flatpak remote-add --user --if-not-exists flathub https://flathub.org/repo/flathub.flatpakrepo
    @flatpak info org.flatpak.Builder >/dev/null 2>&1 || flatpak install --user flathub org.flatpak.Builder -y

# Build the flatpak application
build: setup
    {{builder}} --force-clean --user --install-deps-from=flathub --repo={{repo-dir}} {{build-dir}} {{manifest}}

# Run the application directly from the build directory (useful for quick testing)
run: setup
    {{builder}} --run {{build-dir}} {{manifest}} {{app-id}}

# Install the application locally for the current user
install: setup
    {{builder}} --user --install --force-clean {{build-dir}} {{manifest}}

# Create a flatpak bundle file
bundle:
    flatpak build-bundle {{repo-dir}} {{app-id}}.flatpak {{app-id}}

# Clean up build artifacts
clean:
    rm -rf {{build-dir}} {{repo-dir}} .flatpak-builder

# Enter a shell inside the flatpak sandbox (useful for debugging)
shell: setup
    {{builder}} --run {{build-dir}} {{manifest}} sh
