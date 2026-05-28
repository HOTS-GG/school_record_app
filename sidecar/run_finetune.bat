@echo off
echo ============================================
echo  EasyOCR Fine-Tuning Tool
echo ============================================
echo.
echo [1] AI-Hub data directory (optional, press Enter to skip):
echo     e.g. D:\School-record-app\writeDB\OCR data folder
set /p AIHUB_PATH="AI-Hub path: "

echo.
echo [2] Project DB file (optional, press Enter to skip):
echo     e.g. C:\Users\user\docs\school.db
set /p DB_PATH="DB path: "

echo.
echo [3] Max AI-Hub samples (default 50000):
set /p MAX_EXTRA="Max samples [50000]: "
if "%MAX_EXTRA%"=="" set MAX_EXTRA=50000

echo.
echo [4] Epochs (default 15):
set /p EPOCHS="Epochs [15]: "
if "%EPOCHS%"=="" set EPOCHS=15

echo.
echo [5] Batch size (default 32):
set /p BATCH="Batch [32]: "
if "%BATCH%"=="" set BATCH=32

echo.
echo ============================================
if not "%AIHUB_PATH%"=="" echo  AI-Hub : %AIHUB_PATH%
if not "%DB_PATH%"==""    echo  DB     : %DB_PATH%
echo  Samples: %MAX_EXTRA%
echo  Epochs : %EPOCHS%
echo  Batch  : %BATCH%
echo ============================================
echo.

set CMD=python -X utf8 "%~dp0fine_tune.py" --epochs %EPOCHS% --batch %BATCH% --max-extra %MAX_EXTRA%
if not "%DB_PATH%"==""    set CMD=%CMD% --db "%DB_PATH%"
if not "%AIHUB_PATH%"=="" set CMD=%CMD% --extra-data "%AIHUB_PATH%"

%CMD%

echo.
if %ERRORLEVEL%==0 (
    echo [OK] Fine-tuning complete! The custom model will be used on the next OCR run.
) else (
    echo [ERROR] Fine-tuning failed. Check the messages above.
)
pause
