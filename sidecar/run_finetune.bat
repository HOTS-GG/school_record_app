@echo off
echo ============================================
echo  EasyOCR Fine-Tuning Tool
echo ============================================
echo.
echo AI-Hub data folder path:
echo e.g. D:\School-record-app\writeDB\OCR folder
echo (Drag and drop the folder here, or type the path)
echo.
set /p AIHUB_PATH="AI-Hub path: "

echo.
echo Max samples to use (default 50000, more = longer training):
set /p MAX_EXTRA="Max samples [50000]: "
if "%MAX_EXTRA%"=="" set MAX_EXTRA=50000

echo.
echo Epochs (default 15):
set /p EPOCHS="Epochs [15]: "
if "%EPOCHS%"=="" set EPOCHS=15

echo.
echo ============================================
echo  AI-Hub : %AIHUB_PATH%
echo  Samples: %MAX_EXTRA%
echo  Epochs : %EPOCHS%
echo ============================================
echo.

python -X utf8 "%~dp0fine_tune.py" --extra-data "%AIHUB_PATH%" --max-extra %MAX_EXTRA% --epochs %EPOCHS%

echo.
if %ERRORLEVEL%==0 (
    echo [OK] Fine-tuning complete! The custom model will be used on the next OCR run.
) else (
    echo [ERROR] Fine-tuning failed. Check the messages above.
)
pause
