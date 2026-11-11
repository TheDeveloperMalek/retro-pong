@echo off
setlocal
cd /d "%~dp0"
where cargo >nul 2>nul
if errorlevel 1 (
  echo The required build tools are missing or are not on PATH.
  echo Install the project build prerequisites once, then run this file again.
  pause
  exit /b 1
)
echo Building and launching Retro Pong...
cargo run --release
if errorlevel 1 (
  echo.
  echo Retro Pong could not be built or launched. Review the error above.
  pause
  exit /b 1
)
endlocal
