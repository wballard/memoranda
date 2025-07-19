# Package Manager Integration

## Homebrew (macOS/Linux)

### For users:
Once published to Homebrew core or via our tap:
```bash
# Via Homebrew core (after approval)
brew install memoranda

# Or via our tap
brew tap wballard/memoranda
brew install memoranda
```

### For maintainers:
To publish to a Homebrew tap:

1. Create a repository named `homebrew-memoranda`
2. Copy the formula file:
```bash
cp homebrew/memoranda.rb /path/to/homebrew-memoranda/Formula/memoranda.rb
```
3. Update the SHA256 hash for each release:
```bash
# Get the SHA256 for the release tarball
wget https://github.com/wballard/memoranda/archive/v0.1.0.tar.gz
sha256sum v0.1.0.tar.gz
```

## Cargo (Rust Package Manager)

### For users:
```bash
cargo install memoranda
```

### For maintainers:
Publishing to crates.io is automated via GitHub Actions on tag push.
Manual publishing:
```bash
cargo publish --token $CARGO_REGISTRY_TOKEN
```

## Scoop (Windows)

### For users:
```powershell
# Via our bucket (once created)
scoop bucket add memoranda https://github.com/wballard/scoop-memoranda
scoop install memoranda
```

### For maintainers:
Create a Scoop manifest in a separate `scoop-memoranda` repository:

```json
{
    "version": "0.1.0",
    "description": "Memory-augmented note-taking system with MCP server capabilities",
    "homepage": "https://github.com/wballard/memoranda",
    "license": "MIT",
    "architecture": {
        "64bit": {
            "url": "https://github.com/wballard/memoranda/releases/download/v0.1.0/memoranda-x86_64-pc-windows-msvc.zip",
            "hash": "sha256:REPLACE_WITH_ACTUAL_HASH",
            "extract_dir": "memoranda-x86_64-pc-windows-msvc"
        }
    },
    "bin": "memoranda.exe",
    "checkver": {
        "github": "https://github.com/wballard/memoranda"
    },
    "autoupdate": {
        "architecture": {
            "64bit": {
                "url": "https://github.com/wballard/memoranda/releases/download/v$version/memoranda-x86_64-pc-windows-msvc.zip"
            }
        }
    }
}
```

## Chocolatey (Windows)

### For users:
```powershell
# Once published to Chocolatey
choco install memoranda
```

### For maintainers:
Create a Chocolatey package in a separate repository with a `.nuspec` file:

```xml
<?xml version="1.0" encoding="utf-8"?>
<package xmlns="http://schemas.microsoft.com/packaging/2015/06/nuspec.xsd">
  <metadata>
    <id>memoranda</id>
    <version>0.1.0</version>
    <title>Memoranda</title>
    <authors>Will Ballard</authors>
    <projectUrl>https://github.com/wballard/memoranda</projectUrl>
    <licenseUrl>https://github.com/wballard/memoranda/blob/main/LICENSE</licenseUrl>
    <requireLicenseAcceptance>false</requireLicenseAcceptance>
    <description>Memory-augmented note-taking system with MCP server capabilities for coding agents</description>
    <tags>mcp notes ai agent claude cli</tags>
  </metadata>
  <files>
    <file src="tools\**" target="tools" />
  </files>
</package>
```

## APT (Debian/Ubuntu)

### For maintainers:
Create a `.deb` package and host it in a PPA or custom repository:

1. Create the package structure
2. Build with `dpkg-deb --build memoranda`
3. Host in a repository with `Packages` index

## AUR (Arch Linux)

### For maintainers:
Create a `PKGBUILD` file in the AUR:

```bash
# Maintainer: Your Name <your.email@example.com>
pkgname=memoranda
pkgver=0.1.0
pkgrel=1
pkgdesc="Memory-augmented note-taking system with MCP server capabilities"
arch=('x86_64')
url="https://github.com/wballard/memoranda"
license=('MIT')
depends=()
makedepends=('rust' 'cargo')
source=("$pkgname-$pkgver.tar.gz::https://github.com/wballard/$pkgname/archive/v$pkgver.tar.gz")
sha256sums=('REPLACE_WITH_ACTUAL_HASH')

build() {
    cd "$pkgname-$pkgver"
    cargo build --release --locked
}

package() {
    cd "$pkgname-$pkgver"
    install -Dm755 "target/release/$pkgname" "$pkgdir/usr/bin/$pkgname"
    install -Dm644 "completions/memoranda.bash" "$pkgdir/usr/share/bash-completion/completions/memoranda"
    install -Dm644 "completions/_memoranda" "$pkgdir/usr/share/zsh/site-functions/_memoranda"
    install -Dm644 "completions/memoranda.fish" "$pkgdir/usr/share/fish/vendor_completions.d/memoranda.fish"
}
```

## Release Checklist

When releasing a new version:

1. ✅ Update version in `Cargo.toml`
2. ✅ Create Git tag: `git tag v0.1.0`
3. ✅ Push tag: `git push origin v0.1.0`
4. ✅ GitHub Actions will build and publish to crates.io
5. ⏳ Update Homebrew formula SHA256
6. ⏳ Update Scoop manifest hash
7. ⏳ Update AUR PKGBUILD
8. ⏳ Test installations across platforms