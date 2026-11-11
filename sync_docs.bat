@echo off
setlocal
set "ROOT_DIR=%~dp0"
pushd "%ROOT_DIR%" || exit /b 1
set "DOCS_DIR=%CD%\docs"

call :SyncSite
set "RC=%ERRORLEVEL%"
popd
endlocal & exit /b %RC%

:SyncSite
if not exist "%DOCS_DIR%" mkdir "%DOCS_DIR%"
if errorlevel 1 exit /b %ERRORLEVEL%

call :CopyFile index.html
if errorlevel 1 exit /b %ERRORLEVEL%
call :CopyFile manifest.json
if errorlevel 1 exit /b %ERRORLEVEL%
call :SyncDir static
if errorlevel 1 exit /b %ERRORLEVEL%
call :SyncDir icons
if errorlevel 1 exit /b %ERRORLEVEL%
call :SyncDir assets
if errorlevel 1 exit /b %ERRORLEVEL%
call :SyncDir pkg
if errorlevel 1 exit /b %ERRORLEVEL%

rem These are generated deployment copies, not source files.
for %%F in (static\style.scss static\style.css.map pkg\.gitignore pkg\README.md) do (
    if exist "%DOCS_DIR%\%%F" (
        del /Q "%DOCS_DIR%\%%F"
        if errorlevel 1 exit /b 1
    )
)
type nul > "%DOCS_DIR%\.nojekyll"
if errorlevel 1 exit /b %ERRORLEVEL%
powershell -NoProfile -ExecutionPolicy Bypass -File "%ROOT_DIR%tools\write_service_worker.ps1" -SiteDirectory "%DOCS_DIR%" -WorkerTemplate "%ROOT_DIR%service-worker.js"
exit /b %ERRORLEVEL%

:CopyFile
if not exist "%~1" (
    echo ERROR: Required source file "%~1" is missing.
    exit /b 1
)
copy /Y "%~1" "%DOCS_DIR%\%~1" >nul
exit /b %ERRORLEVEL%

:SyncDir
if not exist "%~1\" (
    echo ERROR: Required source directory "%~1" is missing. Run build_and_sync.bat.
    exit /b 1
)
robocopy "%~1" "%DOCS_DIR%\%~1" /MIR /R:1 /W:1 >nul
set "COPY_RC=%ERRORLEVEL%"
if %COPY_RC% GEQ 8 (
    echo ERROR: Could not synchronize "%~1". Robocopy exit code: %COPY_RC%.
    exit /b %COPY_RC%
)
exit /b 0
