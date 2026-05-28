@echo off
echo [1/2] Installing packages...
python -m pip install -r requirements.txt
if %errorlevel% neq 0 (
    echo Package install failed.
    pause
    exit /b 1
)

echo.
echo [2/2] Downloading EasyOCR models to sidecar\models\ ...
python -X utf8 download_models.py
if %errorlevel% neq 0 (
    echo Model download failed.
    pause
    exit /b 1
)

echo.
echo Done. OCR will work offline from now on.
pause