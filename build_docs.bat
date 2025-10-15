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
type nul > docs\.nojekyll 2>nul

REM Build CSS (skippable)
if defined SKIP_SASS (
  echo SKIP_SASS is set; skipping SCSS compilation.
) else (
  echo Compiling SCSS to CSS...
  powershell -NoProfile -ExecutionPolicy Bypass -File tools\sass_build.ps1 -TimeoutSeconds 120 || echo SCSS compile failed, continuing with existing CSS...
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
REM Generate an enabled service worker for docs (require prod sw)
echo Preparing service worker...
if not exist service-worker.prod.js (
  echo ERROR: service-worker.prod.js not found. Please ensure it exists in repo.
  goto :error
)
copy /Y service-worker.prod.js docs\service-worker.js >nul || goto :error

REM Copy folders
xcopy /E /I /Y static docs\static >nul
xcopy /E /I /Y assets docs\assets >nul
xcopy /E /I /Y icons docs\icons >nul

echo Done. Output is in %CD%\docs. Push to GitHub and set Pages source to /docs.
exit /b 0

:error
echo Build failed. Ensure 'sass' and 'wasm-pack' are installed and on PATH.
exit /b 1
