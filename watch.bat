@echo off
setlocal
pushd "%~dp0" || exit /b 1

call build_and_sync.bat
if errorlevel 1 goto :Failed

rem Sass-only edits do not need to rebuild or optimize WASM.
start "" /B sass --watch --no-source-map static/style.scss:static/style.css
start "" /B cargo watch -w src -w Cargo.toml -w Cargo.lock -s "build_and_sync.bat"
start "" /B cargo watch -w static/style.css -w icons -w assets -w index.html -w manifest.json -w service-worker.js -s "sync_docs.bat"
start "" /B browser-sync start --server --files "static/*.css, pkg/*, assets/*.json" --startPath index.html

echo Watchers started. Close this terminal to stop the development processes.
popd
endlocal & exit /b 0

:Failed
set "RC=%ERRORLEVEL%"
echo Initial build failed. Watchers were not started.
popd
endlocal & exit /b %RC%
