@echo off
setlocal
set "ROOT_DIR=%~dp0"
if "%~1"=="--lock-held" goto :Run
powershell -NoProfile -ExecutionPolicy Bypass -File "%ROOT_DIR%tools\run_site_command.ps1" -Operation Build
exit /b %ERRORLEVEL%

:Run
pushd "%ROOT_DIR%" || exit /b 1

call :Build
set "RC=%ERRORLEVEL%"
popd
endlocal & exit /b %RC%

:Build
call sass --no-source-map "static/style.scss" "static/style.css"
if errorlevel 1 exit /b %ERRORLEVEL%

call wasm-pack build --target web -- --locked
if errorlevel 1 exit /b %ERRORLEVEL%

rem Sync shares the lock already owned by this build.
call "%ROOT_DIR%sync_docs.bat" --lock-held
exit /b %ERRORLEVEL%
