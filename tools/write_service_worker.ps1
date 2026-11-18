param(
    [Parameter(Mandatory = $true)][string]$SiteDirectory,
    [Parameter(Mandatory = $true)][string]$WorkerTemplate
)

$ErrorActionPreference = 'Stop'

try {
    $sitePath = (Resolve-Path -LiteralPath $SiteDirectory).Path
    $template = Get-Content -LiteralPath $WorkerTemplate -Raw
    if ([regex]::Matches($template, '__BUILD_VERSION__').Count -ne 1) {
        throw 'The service worker must contain exactly one build-version marker.'
    }

    $fingerprints = @('worker-template:' + $template) + @(foreach ($file in Get-ChildItem -LiteralPath $sitePath -Recurse -File | Sort-Object FullName) {
        if ($file.Name -eq 'service-worker.js') { continue }
        $relativePath = $file.FullName.Substring($sitePath.Length).Replace('\', '/')
        '{0}:{1}' -f $relativePath, (Get-FileHash -LiteralPath $file.FullName -Algorithm SHA256).Hash
    })
    $algorithm = [System.Security.Cryptography.SHA256]::Create()
    try {
        $hash = $algorithm.ComputeHash([System.Text.Encoding]::UTF8.GetBytes(($fingerprints -join "`n")))
        $version = [BitConverter]::ToString($hash).Replace('-', '').ToLowerInvariant()
    } finally {
        $algorithm.Dispose()
    }
    [System.IO.File]::WriteAllText(
        (Join-Path $sitePath 'service-worker.js'),
        $template.Replace('__BUILD_VERSION__', $version)
    )
    Write-Host "Prepared offline cache version $version"
} catch {
    Write-Error "Could not generate the service worker: $_"
    exit 1
}
