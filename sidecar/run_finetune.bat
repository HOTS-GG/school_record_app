@echo off
echo ============================================
echo  EasyOCR Fine-Tuning Tool
echo ============================================
echo.

rem -- If folder was dragged onto this bat file, use it directly
if not "%~1"=="" (
    set AIHUB_PATH=%~1
    goto got_path
)

:ask_path
echo AI-Hub data folder path:
echo e.g. D:\School-record-app\writeDB\OCR folder
echo.
set AIHUB_PATH=
set /p AIHUB_PATH="AI-Hub path: "

rem -- Strip surrounding quotes that Windows adds on drag-drop
set AIHUB_PATH=%AIHUB_PATH:"=%

if "%AIHUB_PATH%"=="" (
    echo  Path cannot be empty. Please try again.
    echo.
    goto ask_path
)

:got_path
echo.
echo Max samples (default 50000):
set /p MAX_EXTRA="Max samples [50000]: "
if "%MAX_EXTRA%"=="" set MAX_EXTRA=50000

echo.
echo Epochs (default 15):
set /p EPOCHS="Epochs [15]: "
if "%EPOCHS%"=="" set EPOCHS=15

echo.
echo Starting fine-tuning...
echo  Path: %AIHUB_PATH%
echo.

python -X utf8 "%~dp0fine_tune.py" --extra-data "%AIHUB_PATH%" --max-extra %MAX_EXTRA% --epochs %EPOCHS%

echo.
if %ERRORLEVEL%==0 (
    echo [OK] Fine-tuning complete!
) else (
    echo [ERROR] Fine-tuning failed. Check the messages above.
)
echo.
pause
