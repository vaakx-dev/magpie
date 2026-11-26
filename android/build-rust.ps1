$ErrorActionPreference = 'Stop'

$Repo = Split-Path -Parent $PSScriptRoot
$JniLibs = Join-Path $PSScriptRoot 'app/src/main/jniLibs'
$Extension = Join-Path $PSScriptRoot 'app/src/main/assets/extension'
$Abis = @('arm64-v8a', 'x86_64')
$Targets = @('aarch64-linux-android', 'x86_64-linux-android')
$WasmTarget = 'wasm32-unknown-unknown'
$WasmProfile = 'wasm'
$Blocker = 'magpie_blocker.wasm'
$SdkCandidates = @((Join-Path $env:LOCALAPPDATA 'Android/Sdk'), (Join-Path $HOME 'Android/Sdk'))

function Invoke-Checked {
    & $args[0] $args[1..($args.Length - 1)]
    if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
}

if (-not $env:ANDROID_HOME) {
    $env:ANDROID_HOME = $SdkCandidates | Where-Object { Test-Path $_ } | Select-Object -First 1
}

rustup target add @Targets $WasmTarget
if (-not (Get-Command cargo-ndk -ErrorAction SilentlyContinue)) {
    cargo install cargo-ndk
}

Push-Location $Repo
try {
    Invoke-Checked cargo ndk -t $Abis[0] -t $Abis[1] -o $JniLibs build -p magpie-android --release
    Invoke-Checked cargo build -p magpie-blocker-wasm --target $WasmTarget --profile $WasmProfile
    $TargetDir = (cargo metadata --format-version 1 --no-deps | ConvertFrom-Json).target_directory
}
finally {
    Pop-Location
}

$Built = Join-Path $TargetDir "$WasmTarget/$WasmProfile/$Blocker"
$Bundled = Join-Path $Extension $Blocker
if (Get-Command wasm-opt -ErrorAction SilentlyContinue) {
    Invoke-Checked wasm-opt -O3 $Built -o $Bundled
}
else {
    Copy-Item $Built $Bundled
}

@($Abis | ForEach-Object { Get-Item (Join-Path $JniLibs "$_/libmagpie_core.so") }) + (Get-Item $Bundled) |
    Format-Table Directory, Name, Length
