@echo off
chcp 65001 > nul
echo [1/2] 패키지 설치 중...
pip install -r requirements.txt
if %errorlevel% neq 0 (
    echo 패키지 설치 실패
    pause
    exit /b 1
)

echo.
echo [2/2] 모델을 sidecar\models\ 에 번들링 중 (인터넷 필요, 최초 1회)...
python -X utf8 download_models.py
if %errorlevel% neq 0 (
    echo 모델 다운로드 실패
    pause
    exit /b 1
)

echo.
echo 설정 완료! 이후에는 인터넷 없이 동작합니다.
pause
