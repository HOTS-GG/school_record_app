"""
EasyOCR 모델 파일을 sidecar/models/ 에 미리 다운로드
배포 전 1회 실행. 이후 앱은 인터넷 없이 models/ 디렉토리만 사용.

사용법: python download_models.py
"""

import os
import sys

_SCRIPT_DIR  = os.path.dirname(os.path.abspath(__file__))
_MODEL_DIR   = os.path.join(_SCRIPT_DIR, "models")

os.makedirs(_MODEL_DIR, exist_ok=True)

print(f"모델 저장 경로: {_MODEL_DIR}")
print("EasyOCR 한국어 모델 다운로드 중 (인터넷 필요, 최초 1회)...")

import easyocr
reader = easyocr.Reader(
    ["ko"],
    gpu=False,
    model_storage_directory=_MODEL_DIR,
    verbose=True,
)

print("\n다운로드 완료!")
print(f"models/ 디렉토리에 저장된 파일:")
for f in sorted(os.listdir(_MODEL_DIR)):
    size = os.path.getsize(os.path.join(_MODEL_DIR, f))
    print(f"  {f}  ({size // 1024 // 1024} MB)")
print("\n이제 인터넷 없이 OCR을 사용할 수 있습니다.")
