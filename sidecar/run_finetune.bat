@echo off
chcp 65001 > nul
echo ============================================
echo  EasyOCR 한국어 파인튜닝 도구
echo ============================================
echo.

if "%~1"=="" (
    echo 사용법: run_finetune.bat <project.db> [epochs] [batch]
    echo.
    echo 예시:
    echo   run_finetune.bat "C:\Users\user\docs\학교기록.db"
    echo   run_finetune.bat "C:\Users\user\docs\학교기록.db" 20 16
    echo.
    echo DB 파일을 이 창으로 드래그해서 놓으세요:
    set /p DB_PATH="DB 경로: "
) else (
    set DB_PATH=%~1
)

if "%~2"=="" (set EPOCHS=15) else (set EPOCHS=%~2)
if "%~3"=="" (set BATCH=32)  else (set BATCH=%~3)

echo.
echo DB     : %DB_PATH%
echo Epochs : %EPOCHS%
echo Batch  : %BATCH%
echo.

python -X utf8 "%~dp0fine_tune.py" --db "%DB_PATH%" --epochs %EPOCHS% --batch %BATCH%

echo.
if %ERRORLEVEL%==0 (
    echo [완료] 파인튜닝 성공! 다음 OCR 실행 시 학습된 모델이 적용됩니다.
) else (
    echo [오류] 파인튜닝 실패. 위 메시지를 확인하세요.
)
pause
