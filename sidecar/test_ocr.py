"""
Rust 없이 사이드카 단독 테스트
사용법: python test_ocr.py <이미지_경로>
"""

import sys
import json
import os

if len(sys.argv) < 2:
    print("사용법: python test_ocr.py <이미지_경로>")
    sys.exit(1)

image_path = os.path.abspath(sys.argv[1])
if not os.path.isfile(image_path):
    print(f"파일 없음: {image_path}")
    sys.exit(1)

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from ocr_sidecar import _load_reader, _run_ocr

print("모델 로딩 중 (최초 실행 시 다운로드)...")
reader, model_type = _load_reader()
print(f"모델 로딩 완료 (사용 모델: {model_type})\n")

results = _run_ocr(image_path, reader)
print(json.dumps({"success": True, "results": results}, ensure_ascii=False, indent=2))
