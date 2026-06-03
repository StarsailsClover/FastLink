@echo off
REM FastLink Publish Script for Windows
REM Run this after creating GitHub repository

echo ======================================
echo    FastLink Publish Script
echo ======================================
echo.

REM Check if in correct directory
if not exist "Cargo.toml" (
    echo [ERROR] Not in FastLink directory
    exit /b 1
)

echo [INFO] Directory: %cd%
echo.

REM Remove old origin if exists
echo [STEP 1] Configuring remote...
git remote remove origin 2>nul
git remote add origin https://github.com/StarsailsClover/FastLink.git

REM Verify remote
echo [INFO] Remote URL:
git remote -v
echo.

REM Push main branch
echo [STEP 2] Pushing main branch...
git push -u origin main
if errorlevel 1 (
    echo [ERROR] Failed to push main branch
    exit /b 1
)
echo [OK] Main branch pushed
echo.

REM Push tags
echo [STEP 3] Pushing tags...
git push origin v0.2.0-alpha
git push origin v26.5-20260531
echo [OK] Tags pushed
echo.

echo ======================================
echo    [SUCCESS] Publish Complete!
echo ======================================
echo.
echo Next steps:
echo   1. Visit https://github.com/StarsailsClover/FastLink
echo   2. Create a new Release with tag v0.2.0-alpha
echo   3. See PUBLISH_GUIDE.md for detailed instructions
echo.
pause
