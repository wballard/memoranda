# Memoranda Installation Script for Windows
# This script downloads and installs the latest version of memoranda

param(
    [string]$InstallDir = "$env:USERPROFILE\.local\bin",
    [switch]$Force = $false
)

# Configuration
$RepoOwner = "wballard"
$RepoName = "memoranda"
$BinaryName = "memoranda.exe"

# Colors for output
$Colors = @{
    Red = "Red"
    Green = "Green"
    Yellow = "Yellow"
    Blue = "Cyan"
}

function Write-Error-Message {
    param([string]$Message)
    Write-Host "Error: $Message" -ForegroundColor $Colors.Red
    exit 1
}

function Write-Info {
    param([string]$Message)
    Write-Host "Info: $Message" -ForegroundColor $Colors.Blue
}

function Write-Success {
    param([string]$Message)
    Write-Host "Success: $Message" -ForegroundColor $Colors.Green
}

function Write-Warning {
    param([string]$Message)
    Write-Host "Warning: $Message" -ForegroundColor $Colors.Yellow
}

function Get-Platform {
    if ([Environment]::Is64BitOperatingSystem) {
        return "x86_64-pc-windows-msvc"
    } else {
        Write-Error-Message "32-bit Windows is not supported"
    }
}

function Get-LatestVersion {
    try {
        $apiUrl = "https://api.github.com/repos/$RepoOwner/$RepoName/releases/latest"
        $response = Invoke-RestMethod -Uri $apiUrl -Method Get
        return $response.tag_name
    } catch {
        Write-Error-Message "Failed to fetch latest version: $($_.Exception.Message)"
    }
}

function Install-Memoranda {
    param(
        [string]$Platform,
        [string]$Version
    )
    
    $archiveName = "$BinaryName.replace('.exe', '')-$Platform.zip"
    $downloadUrl = "https://github.com/$RepoOwner/$RepoName/releases/download/$Version/$archiveName"
    
    Write-Info "Downloading memoranda $Version for $Platform..."
    Write-Info "Download URL: $downloadUrl"
    
    # Create temporary directory
    $tempDir = [System.IO.Path]::GetTempPath() + [System.Guid]::NewGuid().ToString()
    New-Item -ItemType Directory -Path $tempDir -Force | Out-Null
    
    try {
        # Download the archive
        $archivePath = Join-Path $tempDir $archiveName
        Write-Info "Downloading to: $archivePath"
        
        try {
            Invoke-WebRequest -Uri $downloadUrl -OutFile $archivePath -ErrorAction Stop
        } catch {
            Write-Error-Message "Failed to download memoranda: $($_.Exception.Message)"
        }
        
        # Extract the archive
        Write-Info "Extracting archive..."
        try {
            Expand-Archive -Path $archivePath -DestinationPath $tempDir -Force
        } catch {
            Write-Error-Message "Failed to extract archive: $($_.Exception.Message)"
        }
        
        # Create installation directory
        if (!(Test-Path $InstallDir)) {
            New-Item -ItemType Directory -Path $InstallDir -Force | Out-Null
        }
        
        # Move binary to installation directory
        Write-Info "Installing memoranda to $InstallDir..."
        $binaryPath = Join-Path $tempDir $BinaryName
        $installPath = Join-Path $InstallDir $BinaryName
        
        if (Test-Path $binaryPath) {
            Copy-Item $binaryPath $installPath -Force
        } else {
            Write-Error-Message "Binary not found in extracted archive"
        }
        
        Write-Success "memoranda $Version installed successfully!"
        
    } finally {
        # Cleanup
        if (Test-Path $tempDir) {
            Remove-Item $tempDir -Recurse -Force
        }
    }
}

function Test-PathEntry {
    $currentPath = [Environment]::GetEnvironmentVariable("PATH", "User")
    return $currentPath -split ';' | Where-Object { $_ -eq $InstallDir }
}

function Add-ToPath {
    if (!(Test-PathEntry)) {
        Write-Warning "The installation directory $InstallDir is not in your PATH."
        $response = Read-Host "Would you like to add it to your PATH? (y/N)"
        
        if ($response -eq 'y' -or $response -eq 'Y') {
            try {
                $currentPath = [Environment]::GetEnvironmentVariable("PATH", "User")
                $newPath = if ($currentPath) { "$currentPath;$InstallDir" } else { $InstallDir }
                [Environment]::SetEnvironmentVariable("PATH", $newPath, "User")
                Write-Success "Added $InstallDir to your PATH"
                Write-Info "Please restart your terminal or PowerShell session for the changes to take effect"
            } catch {
                Write-Warning "Failed to add to PATH: $($_.Exception.Message)"
                Write-Info "You can manually add $InstallDir to your PATH environment variable"
            }
        } else {
            Write-Info "To manually add to PATH, add this directory to your PATH environment variable:"
            Write-Info "  $InstallDir"
        }
    }
}

function Test-Installation {
    $binaryPath = Join-Path $InstallDir $BinaryName
    
    if (Test-Path $binaryPath) {
        Write-Success "Installation verified!"
        Write-Info "Run '$binaryPath --version' to check the version"
        Write-Info "Run '$binaryPath doctor' to verify setup"
        Write-Host ""
        Write-Host "To get started:"
        Write-Host "  1. Run 'memoranda doctor' to check your setup"
        Write-Host "  2. Add memoranda to your MCP configuration"
        Write-Host "  3. Start using memo tools in Claude Code!"
    } else {
        Write-Error-Message "Installation verification failed"
    }
}

function Main {
    Write-Host "🚀 Memoranda Installation Script for Windows" -ForegroundColor $Colors.Green
    Write-Host "===========================================" -ForegroundColor $Colors.Green
    Write-Host ""
    
    # Detect platform
    $platform = Get-Platform
    Write-Info "Detected platform: $platform"
    
    # Get latest version
    Write-Info "Fetching latest release information..."
    $version = Get-LatestVersion
    Write-Info "Latest version: $version"
    
    # Check if already installed
    $binaryPath = Join-Path $InstallDir $BinaryName
    if ((Test-Path $binaryPath) -and !$Force) {
        try {
            $output = & $binaryPath --version 2>$null
            $currentVersion = ($output -split ' ')[1]
            if ($currentVersion -eq $version.TrimStart('v')) {
                Write-Success "memoranda $version is already installed and up to date!"
                return
            } else {
                Write-Info "Updating from version $currentVersion to $version"
            }
        } catch {
            Write-Info "Unable to determine current version, proceeding with installation"
        }
    }
    
    # Install memoranda
    Install-Memoranda -Platform $platform -Version $version
    
    # Check and update PATH
    Add-ToPath
    
    # Verify installation
    Test-Installation
    
    Write-Host ""
    Write-Success "Installation complete! 🎉"
}

# Run main function
Main