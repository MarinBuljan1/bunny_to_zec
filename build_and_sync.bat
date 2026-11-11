@echo off
setlocal
set "ROOT_DIR=%~dp0"
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

call "%ROOT_DIR%sync_docs.bat"
exit /b %ERRORLEVEL%
