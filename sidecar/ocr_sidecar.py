"""
EasyOCR 사이드카 — Tauri 백엔드가 stdin/stdout JSON으로 호출

프로토콜:
  준비  stdout : {"status": "ready", "backend": "easyocr", "model": "custom|default"}
  요청  stdin  : {"image_path": "C:/..."}
  응답  stdout : {
    "success": true,
    "results": [{
      "text": "...",
      "confidence": 0.97,
      "bbox": [x1, y1, x2, y2],
      "text_type": "handwritten|printed|uncertain",
      "needs_correction": false
    }]
  }
  오류  stdout : {"success": false, "error": "..."}

text_type 분류 기준:
  confidence >= 0.80 → printed
  confidence <= 0.45 → handwritten
  그 사이              → 이미지 엣지 밀도로 재판별 (높으면 printed, 낮으면 handwritten)

needs_correction:
  confidence < 0.55 이면 True — 프론트엔드가 빨간 박스로 표시
"""

import sys
import json
import os

# ── 네트워크 차단 ────────────────────────────────────────────────
# 추론 중 외부 서버 접근을 OS-레벨 없이 Python 레벨에서 차단
import socket as _socket

_orig_getaddrinfo = _socket.getaddrinfo
_orig_create_connection = _socket.create_connection

def _block_network(*args, **kwargs):
    raise OSError("[OCR 사이드카] 네트워크 접근이 차단되어 있습니다.")

# 모델 경로 설정 후 소켓을 차단 (모델 로드 전에는 열어둬야 HF Hub 캐시 확인 가능)
# 실제 차단은 모델 로드 완료 후 _activate_network_block()에서 수행
def _activate_network_block():
    _socket.getaddrinfo = _block_network
    _socket.create_connection = _block_network

# ── 경로 설정 ────────────────────────────────────────────────────
_SCRIPT_DIR   = os.path.dirname(os.path.abspath(__file__))
_CUSTOM_MODEL = os.path.join(_SCRIPT_DIR, "custom_model")
_BUNDLE_MODEL = os.path.join(_SCRIPT_DIR, "models")   # 번들링된 모델 위치

# EasyOCR 기본 캐시를 번들 경로로 유도
if os.path.isdir(_BUNDLE_MODEL):
    os.environ["EASYOCR_MODULE_PATH"] = _BUNDLE_MODEL


# ── 텍스트 유형 분류 ─────────────────────────────────────────────
_PRINT_THRESHOLD     = 0.80   # 이 이상이면 인쇄체
_HW_THRESHOLD        = 0.45   # 이 이하면 손글씨
_CORRECTION_THRESHOLD = 0.55  # 이 미만이면 교정 필요 표시


def _classify(confidence: float, image_path: str, bbox: list) -> str:
    """신뢰도 + (중간값의 경우) 이미지 엣지 밀도로 텍스트 유형 판별."""
    if confidence >= _PRINT_THRESHOLD:
        return "printed"
    if confidence <= _HW_THRESHOLD:
        return "handwritten"

    # 중간 영역 → 텍스트 영역의 엣지 밀도로 판별
    try:
        import cv2
        import numpy as np
        from PIL import Image

        img = Image.open(image_path).convert("RGB")
        x1, y1, x2, y2 = bbox
        w, h = img.size
        x1, y1 = max(0, x1), max(0, y1)
        x2, y2 = min(w, x2), min(h, y2)
        if x2 <= x1 or y2 <= y1:
            return "uncertain"

        region = np.array(img.crop((x1, y1, x2, y2)))
        gray   = cv2.cvtColor(region, cv2.COLOR_RGB2GRAY)
        edges  = cv2.Canny(gray, 50, 150)
        density = float(np.count_nonzero(edges)) / edges.size

        return "printed" if density > 0.12 else "handwritten"
    except Exception:
        return "uncertain"


# ── 모델 로드 ────────────────────────────────────────────────────
def _load_reader():
    import easyocr

    custom_pth = None
    if os.path.isdir(_CUSTOM_MODEL):
        for fname in os.listdir(_CUSTOM_MODEL):
            if fname.endswith(".pth"):
                custom_pth = os.path.join(_CUSTOM_MODEL, fname)
                break

    if custom_pth:
        reader = easyocr.Reader(
            ["ko"],
            gpu=False,
            model_storage_directory=_CUSTOM_MODEL,
            user_network_directory=_CUSTOM_MODEL,
            recog_network="custom",
        )
        return reader, "custom"

    kwargs = {"gpu": False}
    if os.path.isdir(_BUNDLE_MODEL):
        kwargs["model_storage_directory"] = _BUNDLE_MODEL

    reader = easyocr.Reader(["ko"], **kwargs)
    return reader, "default"


# ── OCR 실행 ─────────────────────────────────────────────────────
def _run_ocr(image_path: str, reader) -> list:
    # cv2.imread()는 Windows에서 한글/특수문자 경로를 읽지 못함
    # PIL로 읽어 BGR numpy 배열로 변환 후 EasyOCR에 전달
    import numpy as np
    from PIL import Image as _PIL_Image
    img_pil = _PIL_Image.open(image_path).convert("RGB")
    img_np  = np.array(img_pil)[:, :, ::-1]  # RGB → BGR (EasyOCR/cv2 포맷)

    raw = reader.readtext(img_np, detail=1, paragraph=False)
    results = []
    for (bbox_pts, text, conf) in raw:
        xs = [p[0] for p in bbox_pts]
        ys = [p[1] for p in bbox_pts]
        bbox = [int(min(xs)), int(min(ys)), int(max(xs)), int(max(ys))]
        conf_f = round(float(conf), 4)

        results.append({
            "text":             text,
            "confidence":       conf_f,
            "bbox":             bbox,
            "text_type":        _classify(conf_f, image_path, bbox),
            "needs_correction": conf_f < _CORRECTION_THRESHOLD,
        })
    return results


# ── 메인 루프 ────────────────────────────────────────────────────
def main():
    try:
        reader, model_type = _load_reader()
        # 모델 로드 완료 → 이제 네트워크 차단
        _activate_network_block()
        sys.stdout.write(
            json.dumps({"status": "ready", "backend": "easyocr", "model": model_type}) + "\n"
        )
        sys.stdout.flush()
    except Exception as exc:
        sys.stdout.write(json.dumps({"status": "error", "error": str(exc)}) + "\n")
        sys.stdout.flush()
        sys.exit(1)

    for raw in sys.stdin:
        raw = raw.strip()
        if not raw:
            continue

        try:
            req        = json.loads(raw)
            image_path = req.get("image_path", "")

            if not os.path.isfile(image_path):
                resp = {"success": False, "error": f"파일을 찾을 수 없음: {image_path}"}
            else:
                resp = {"success": True, "results": _run_ocr(image_path, reader)}

        except json.JSONDecodeError as exc:
            resp = {"success": False, "error": f"JSON 파싱 오류: {exc}"}
        except Exception as exc:
            resp = {"success": False, "error": str(exc)}

        sys.stdout.write(json.dumps(resp, ensure_ascii=False) + "\n")
        sys.stdout.flush()


if __name__ == "__main__":
    main()
