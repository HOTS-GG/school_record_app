@echo off
echo Building ocr_sidecar.exe with PyInstaller...

pip install pyinstaller

pyinstaller ^
    --onefile ^
    --name ocr_sidecar ^
    --hidden-import surya ^
    --hidden-import easyocr ^
    --hidden-import torch ^
    --hidden-import PIL ^
    --collect-all easyocr ^
    ocr_sidecar.py

if %errorlevel% neq 0 (
    echo Build failed.
    pause
    exit /b 1
)

echo.
echo Build complete: dist\ocr_sidecar.exe
pause