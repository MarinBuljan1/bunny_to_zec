param(
    [Parameter(Mandatory = $true)]
    [ValidateSet('Build', 'Sync')]
    [string]$Operation
)

$ErrorActionPreference = 'Stop'
$projectRoot = Split-Path -Parent $PSScriptRoot
$targetDirectory = Join-Path $projectRoot 'target'
$lockPath = Join-Path $targetDirectory 'site-build.lock'
$lock = $null
$exitCode = 1

try {
    [System.IO.Directory]::CreateDirectory($targetDirectory) | Out-Null
    $deadline = [DateTime]::UtcNow.AddMinutes(2)
    while ($null -eq $lock) {
        try {
            # The OS releases this exclusive handle even if the process exits.
            # The file may remain; its existence does not mean a command is busy.
            $lock = [System.IO.File]::Open($lockPath, 'OpenOrCreate', 'ReadWrite', 'None')
        } catch [System.IO.IOException] {
            $errorCode = $_.Exception.HResult -band 0xffff
            if ($errorCode -notin @(32, 33)) { throw }
            if ([DateTime]::UtcNow -ge $deadline) { throw 'Another build or sync is still busy. Try again when it finishes.' }
            Start-Sleep -Milliseconds 100
        }
    }

    $scriptName = if ($Operation -eq 'Build') { 'build_and_sync.bat' } else { 'sync_docs.bat' }
    & (Join-Path $projectRoot $scriptName) --lock-held
    $exitCode = $LASTEXITCODE
} catch {
    Write-Host "ERROR: $Operation failed. $_"
} finally {
    if ($null -ne $lock) { $lock.Dispose() }
}
exit $exitCode
