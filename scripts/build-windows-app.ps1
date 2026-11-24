$ErrorActionPreference = "Stop"

$Root = (Resolve-Path (Join-Path $PSScriptRoot "..")).Path
$Target = "x86_64-pc-windows-msvc"
$ReleaseBinary = Join-Path $Root "target\$Target\release\a22-app.exe"
$DistributionRoot = Join-Path $Root "target\windows"
$DistributionDirectory = Join-Path $DistributionRoot "A22R"
$PackagedBinary = Join-Path $DistributionDirectory "A22R.exe"
$Archive = Join-Path $DistributionRoot "A22R-windows-x86_64.zip"

Push-Location $Root
try {
    & cargo +1.99.1 build -p a22-app --release --target $Target --locked
    if ($LASTEXITCODE -ne 0) {
        throw "Cargo failed with exit code $LASTEXITCODE."
    }

    if (-not (Test-Path -LiteralPath $ReleaseBinary -PathType Leaf)) {
        throw "Release binary was not created: $ReleaseBinary"
    }

    if (Test-Path -LiteralPath $DistributionDirectory) {
        Remove-Item -LiteralPath $DistributionDirectory -Recurse -Force
    }
    if (Test-Path -LiteralPath $Archive) {
        Remove-Item -LiteralPath $Archive -Force
    }

    New-Item -ItemType Directory -Path $DistributionDirectory -Force | Out-Null
    Copy-Item -LiteralPath $ReleaseBinary -Destination $PackagedBinary
    Compress-Archive -Path (Join-Path $DistributionDirectory "*") `
        -DestinationPath $Archive -CompressionLevel Optimal

    Write-Host "Built Windows application:"
    Write-Host "  $PackagedBinary"
    Write-Host "Created archive:"
    Write-Host "  $Archive"
}
finally {
    Pop-Location
}
