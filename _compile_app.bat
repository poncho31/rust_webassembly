REM filepath: h:\PROJECTS\webassembly_unified_frontbackend\_compile_app.bat
@echo off
setlocal EnableDelayedExpansion

REM ===================================
REM Configuration
REM ===================================
set "CLIENT_DIR=client"
set "WASM_OUT_DIR=static/pkg"

REM ===================================
REM Build WASM Package
REM ===================================
echo [Step 1/3] Building WASM package...
echo -------------------------------------

cd %CLIENT_DIR% || (
    echo Error: Failed to change directory to %CLIENT_DIR%
    goto :error
)

call wasm-pack build --target web --out-dir %WASM_OUT_DIR% || (
    echo Error: WASM build failed
    cd ..
    goto :error
)

REM ===================================
REM Return to Root
REM ===================================
echo [Step 2/3] Returning to root directory...
echo -------------------------------------

cd .. || (
    echo Error: Failed to return to root directory
    goto :error
)

REM ===================================
REM Start Server
REM ===================================
echo [Step 3/3] Starting server...
echo -------------------------------------

cargo run || (
    echo Error: Server failed to start
    goto :error
)

goto :end

:error
echo.
echo Build process failed!
pause
exit /b 1

:end
echo.
echo Build completed successfully!
exit /b 0