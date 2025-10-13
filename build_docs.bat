@echo off
setlocal enabledelayedexpansion

echo === Building site into docs/ ===

REM Ensure docs folder structure exists up front
echo Creating docs folder structure...
mkdir docs 2>nul
mkdir docs\pkg 2>nul
mkdir docs\static 2>nul
mkdir docs\assets 2>nul
mkdir docs\icons 2>nul

REM Build CSS (try sass, then npx sass; otherwise keep existing CSS)
if exist static\style.scss (
  where sass >nul 2>nul
  if %ERRORLEVEL%==0 (
    echo Compiling SCSS to CSS ^(sass^)...
    sass static/style.scss:static/style.css || echo SCSS compile failed, continuing with existing CSS...
  ) else (
    where npx >nul 2>nul
    if %ERRORLEVEL%==0 (
      echo Compiling SCSS to CSS ^(npx sass^)...
      npx sass static/style.scss static/style.css || echo SCSS compile failed, continuing with existing CSS...
    ) else (
      echo Sass not found; using existing static/style.css
    )
  )
) else (
  echo No SCSS found, skipping sass step.
)

REM Build WASM to docs/pkg (or fallback to existing pkg)
echo Building WASM to docs/pkg...
where wasm-pack >nul 2>nul
if %ERRORLEVEL%==0 (
  wasm-pack build --target web --out-dir docs/pkg || goto :error
) else (
  echo wasm-pack not found. Attempting to copy existing pkg/ ...
  if exist pkg (
    xcopy /E /I /Y pkg docs\pkg >nul || goto :error
  ) else (
    echo No existing pkg folder found. Please install wasm-pack or run a build to generate pkg.
    goto :error
  )
)

REM Copy static files
echo Copying site files to docs/ ...
copy /Y index.html docs\index.html >nul
copy /Y manifest.json docs\manifest.json >nul
REM Generate an enabled service worker for docs by uncommenting lines and fixing URLs
echo Preparing service worker...
> _sw_build.ps1 echo $content = Get-Content -Raw 'service-worker.js'
>> _sw_build.ps1 echo $content = [regex]::Replace($content, '^[\s]*//[\s]?', '', 'Multiline')
>> _sw_build.ps1 echo $content = $content -replace '"/', '"'
>> _sw_build.ps1 echo Set-Content -Path 'docs/service-worker.js' -Encoding UTF8 -Value $content
powershell -NoProfile -ExecutionPolicy Bypass -File _sw_build.ps1 || goto :error
del _sw_build.ps1 >nul 2>nul

REM Copy folders
xcopy /E /I /Y static docs\static >nul
xcopy /E /I /Y assets docs\assets >nul
xcopy /E /I /Y icons docs\icons >nul

echo Done. Output is in %CD%\docs. Push to GitHub and set Pages source to /docs.
exit /b 0

:error
echo Build failed. Ensure 'sass' and 'wasm-pack' are installed and on PATH.
exit /b 1
