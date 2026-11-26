param(
    [Parameter(Mandatory)] [int] $ProcessId,
    [Parameter(Mandatory)] [string] $Source,
    [Parameter(Mandatory)] [string] $Target
)

$ErrorActionPreference = 'Stop'
$ATTEMPTS = 40
$RETRY_MS = 250

function Wait-ForExit {
    $running = Get-Process -Id $ProcessId -ErrorAction SilentlyContinue
    if ($running) { $running.WaitForExit() }
}

function Copy-Update {
    for ($attempt = 1; $attempt -le $ATTEMPTS; $attempt++) {
        try {
            Copy-Item -LiteralPath $Source -Destination $Target -Force
            return
        } catch {
            Start-Sleep -Milliseconds $RETRY_MS
        }
    }
}

Wait-ForExit
Copy-Update
Remove-Item -LiteralPath $Source -ErrorAction SilentlyContinue
Start-Process -FilePath $Target
