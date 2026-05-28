@echo off
chcp 65001 > nul
echo PyInstaller로 ocr_sidecar.exe 빌드 중...

pip install pyinstaller

pyinstaller ^
    --onefile ^
    --name ocr_sidecar ^
    --hidden-import surya ^
    --hidden-import surya.model.detection.segformer ^
    --hidden-import surya.model.recognition.model ^
    --hidden-import surya.model.recognition.processor ^
    --hidden-import surya.ocr ^
    --hidden-import torch ^
    --hidden-import PIL ^
    --hidden-import transformers ^
    --collect-all surya ^
    ocr_sidecar.py

if %errorlevel% neq 0 (
    echo 빌드 실패
    pause
    exit /b 1
)

echo.
echo 빌드 완료: dist\ocr_sidecar.exe
pause
