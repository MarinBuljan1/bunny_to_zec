param(
  [string]$In = "static/style.scss",
  [string]$Out = "static/style.css",
  [int]$TimeoutSeconds = 60
)

function Has-Cmd($name) {
  try { Get-Command $name -ErrorAction Stop | Out-Null; return $true } catch { return $false }
}

function Find-ToolPath([string]$tool) {
  $candidates = @()
  if ($tool -eq 'sass') {
    $candidates += @(
      Join-Path $env:APPDATA 'npm\sass.cmd'),
      (Join-Path $env:ProgramFiles 'nodejs\sass.cmd'),
      'sass.cmd',
      'sass'
  } elseif ($tool -eq 'npx') {
    $candidates += @(
      Join-Path $env:ProgramFiles 'nodejs\npx.cmd'),
      'npx.cmd',
      'npx'
  }
  foreach ($c in $candidates) { if ($null -ne $c -and (Get-Command $c -ErrorAction SilentlyContinue)) { return (Get-Command $c).Source } }
  return $null
}

if (-not (Test-Path $In)) {
  Write-Host "No SCSS found; skipping sass step."; exit 0
}

$exe = $null
$args = $null

# Prefer a locally installed Dart Sass if available, then fall back to npx
if ($null -ne (Find-ToolPath 'sass')) {
  $exe = (Find-ToolPath 'sass')
  $args = @("$In`:$Out", '--no-source-map')
} elseif ($null -ne (Find-ToolPath 'npx')) {
  $exe = (Find-ToolPath 'npx')
  $args = @('--yes','sass', $In, $Out, '--no-source-map')
  if ($TimeoutSeconds -lt 120) { $TimeoutSeconds = 120 }
} else {
  Write-Host "Sass not found; using existing $Out"; exit 0
}

try {
  Write-Host "Running: $exe $($args -join ' ') (timeout ${TimeoutSeconds}s)"
  $p = Start-Process -FilePath $exe -ArgumentList $args -PassThru -WindowStyle Hidden
  if (-not ($p.WaitForExit($TimeoutSeconds * 1000))) {
    try { $p.Kill() } catch {}
    Write-Host "SCSS compile timed out; continuing with existing CSS..."
  } else {
    if ($p.ExitCode -ne 0) {
      Write-Host "SCSS compile exited with code $($p.ExitCode); continuing with existing CSS..."
    } else {
      Write-Host "SCSS compiled successfully to $Out"
    }
  }
} catch {
  Write-Host "SCSS compile failed; continuing with existing CSS..."
}
exit 0
