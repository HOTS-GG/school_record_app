@echo off
echo ============================================
echo  EasyOCR Fine-Tuning Tool
echo ============================================
echo.

set DB_PATH=%~1
set EPOCHS=%~2
set BATCH=%~3

if "%DB_PATH%"=="" (
    echo Usage: run_finetune.bat ^<project.db^> [epochs] [batch_size]
    echo.
    echo Example:
    echo   run_finetune.bat "C:\Users\user\docs\school.db"
    echo   run_finetune.bat "C:\Users\user\docs\school.db" 20 16
    echo.
    echo Drag and drop the .db file here, or type the path:
    set /p DB_PATH="DB path: "
)

if "%EPOCHS%"=="" set EPOCHS=15
if "%BATCH%"=="" set BATCH=32

echo.
echo DB     : %DB_PATH%
echo Epochs : %EPOCHS%
echo Batch  : %BATCH%
echo.

python -X utf8 "%~dp0fine_tune.py" --db "%DB_PATH%" --epochs %EPOCHS% --batch %BATCH%

echo.
if %ERRORLEVEL%==0 (
    echo [OK] Fine-tuning complete! The custom model will be used on the next OCR run.
) else (
    echo [ERROR] Fine-tuning failed. Check the messages above.
)
pause
