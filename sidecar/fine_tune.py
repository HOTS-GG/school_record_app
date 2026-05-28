"""
EasyOCR 한국어 인식 모델 파인튜닝 스크립트

사용법:
  python fine_tune.py --db <project.db> [--epochs 10] [--batch 32] [--lr 1e-4]
  python fine_tune.py --db <project.db> --extra-data <aihub_dir>

출력: sidecar/custom_model/korean_g2.pth  (사이드카가 자동으로 로드)

아키텍처: EasyOCR generation2 (VGG + BiLSTM + CTC)
  - 입력: 32×128 grayscale
  - CTC loss 로 end-to-end 학습
"""

import argparse
import json
import os
import sqlite3
import sys
from pathlib import Path

import numpy as np
import torch
import torch.nn as nn
from PIL import Image
from torch.utils.data import DataLoader, Dataset

# ── 경로 ─────────────────────────────────────────────────────────
_SCRIPT_DIR  = Path(__file__).parent
_CUSTOM_DIR  = _SCRIPT_DIR / "custom_model"
_BUNDLE_DIR  = _SCRIPT_DIR / "models"
_OUTPUT_PATH = _CUSTOM_DIR / "korean_g2.pth"

# EasyOCR 모델 캐시 경로 오버라이드
if _BUNDLE_DIR.is_dir():
    os.environ["EASYOCR_MODULE_PATH"] = str(_BUNDLE_DIR)


# ── DB 에서 교정 데이터 로드 ──────────────────────────────────────
def load_corrections(db_path: str) -> list[dict]:
    """OcrResult 중 corrected_text 가 있는 행 + 원본 이미지 경로를 반환."""
    conn = sqlite3.connect(db_path)
    rows = conn.execute("""
        SELECT r.bbox, r.raw_text, r.corrected_text,
               s.image_path
        FROM   OcrResult r
        JOIN   OcrSession s ON s.id = r.session_id
        WHERE  r.corrected_text IS NOT NULL
          AND  r.corrected_text != ''
    """).fetchall()
    conn.close()

    samples = []
    for bbox_json, raw, corrected, img_path in rows:
        try:
            bbox = json.loads(bbox_json)
            x1, y1, x2, y2 = int(bbox[0]), int(bbox[1]), int(bbox[2]), int(bbox[3])
        except Exception:
            continue
        label = corrected.strip()
        if not label:
            continue
        samples.append({"image_path": img_path, "bbox": (x1, y1, x2, y2), "label": label})
    return samples


# ── AI-Hub 데이터 로드 ───────────────────────────────────────────
def load_aihub_data(data_dir: str) -> list[dict]:
    """
    AI-Hub 손글씨 OCR 데이터셋 (데이터셋 #131 등) 로드.

    지원하는 두 가지 구조:

    [구조 A] AI-Hub 표준 JSON 어노테이션 형식
      <data_dir>/
        Training/Images/kor_1_1_00001.jpg
        Training/Annotations/kor_1_1_00001.json
          {"annotations": [{"text": "안녕", "bbox": [x,y,w,h]}, ...]}
      (Validation/ 도 같은 구조)

    [구조 B] 단순 이미지+텍스트 쌍 (직접 준비한 경우)
      <data_dir>/
        images/0001.png
        labels/0001.txt   (첫 줄 = 레이블)
    """
    samples = []
    data_dir = Path(data_dir)

    # ── 구조 A: AI-Hub JSON 어노테이션 ──
    json_found = False
    for split in ("Training", "Validation", ""):
        base   = data_dir / split if split else data_dir
        ann_dir = base / "Annotations"
        img_dir = base / "Images"
        if not ann_dir.is_dir() or not img_dir.is_dir():
            continue
        json_found = True
        for ann_file in sorted(ann_dir.rglob("*.json")):
            try:
                ann = json.loads(ann_file.read_text(encoding="utf-8"))
            except Exception:
                continue

            # 이미지 파일 찾기 (stem 동일)
            img_file = img_dir / (ann_file.stem + ".jpg")
            if not img_file.exists():
                img_file = img_dir / (ann_file.stem + ".png")
            if not img_file.exists():
                # rglob 으로 하위 폴더 탐색
                hits = list(img_dir.rglob(ann_file.stem + ".*"))
                img_file = hits[0] if hits else None
            if not img_file:
                continue

            annotations = ann.get("annotations") or ann.get("label") or []
            if isinstance(annotations, dict):
                annotations = [annotations]

            for item in annotations:
                text = (item.get("text") or item.get("label") or "").strip()
                if not text:
                    continue
                bbox_raw = item.get("bbox") or item.get("boundingBox")
                if bbox_raw and len(bbox_raw) == 4:
                    x, y, w, h = bbox_raw
                    bbox = (int(x), int(y), int(x + w), int(y + h))
                else:
                    bbox = None   # 이미지 전체가 한 단어 크롭인 경우
                samples.append({"image_path": str(img_file), "bbox": bbox, "label": text})

    if json_found:
        return samples

    # ── 구조 B: 단순 images/ + labels/ ──
    img_dir = data_dir / "images"
    lbl_dir = data_dir / "labels"
    if not img_dir.is_dir() or not lbl_dir.is_dir():
        print(f"[경고] AI-Hub 데이터 디렉토리 구조를 인식하지 못했습니다: {data_dir}")
        print("       Training/Images + Training/Annotations  또는")
        print("       images/ + labels/  구조를 지원합니다.")
        return samples

    for img_file in sorted(img_dir.glob("*")):
        if img_file.suffix.lower() not in (".png", ".jpg", ".jpeg", ".bmp"):
            continue
        lbl_file = lbl_dir / (img_file.stem + ".txt")
        if not lbl_file.exists():
            continue
        label = lbl_file.read_text(encoding="utf-8").strip().splitlines()[0].strip()
        if label:
            samples.append({"image_path": str(img_file), "bbox": None, "label": label})

    return samples


# ── 이미지 전처리 ─────────────────────────────────────────────────
_IMG_H = 32
_IMG_W = 128

def crop_and_resize(image_path: str, bbox: tuple | None) -> np.ndarray | None:
    """
    이미지를 로드, bbox 영역을 잘라 32×128 grayscale 로 리사이즈.
    bbox=None 이면 이미지 전체를 사용.
    반환: shape (1, 32, 128) float32 in [0, 1]
    """
    try:
        img = Image.open(image_path).convert("L")   # grayscale
        if bbox is not None:
            x1, y1, x2, y2 = bbox
            w, h = img.size
            x1 = max(0, min(x1, w)); x2 = max(0, min(x2, w))
            y1 = max(0, min(y1, h)); y2 = max(0, min(y2, h))
            if x2 <= x1 or y2 <= y1:
                return None
            img = img.crop((x1, y1, x2, y2))

        # 종횡비 유지 리사이즈
        ow, oh = img.size
        ratio = _IMG_H / oh
        nw    = min(int(ow * ratio), _IMG_W)
        img   = img.resize((nw, _IMG_H), Image.BICUBIC)

        # 패딩 (오른쪽에 흰색)
        canvas = Image.new("L", (_IMG_W, _IMG_H), 255)
        canvas.paste(img, (0, 0))
        arr = np.array(canvas, dtype=np.float32) / 255.0   # [0,1]
        arr = (arr - 0.5) / 0.5                            # [-1,1]
        return arr[np.newaxis, ...]   # (1, H, W)
    except Exception as e:
        print(f"[경고] 이미지 처리 실패: {image_path} — {e}")
        return None


# ── Dataset ──────────────────────────────────────────────────────
class CorrectionDataset(Dataset):
    def __init__(self, samples: list[dict], character: str):
        self.char_to_idx = {c: i + 1 for i, c in enumerate(character)}   # 0=blank
        self.items = []
        skipped = 0
        for s in samples:
            img = crop_and_resize(s["image_path"], s["bbox"])
            if img is None:
                skipped += 1
                continue
            label_idx = [self.char_to_idx[c] for c in s["label"] if c in self.char_to_idx]
            if not label_idx:
                skipped += 1
                continue
            self.items.append((img, label_idx, s["label"]))
        if skipped:
            print(f"  → {skipped}개 샘플 건너뜀 (이미지 오류 또는 미지원 문자)")

    def __len__(self):
        return len(self.items)

    def __getitem__(self, idx):
        img, label_idx, label_str = self.items[idx]
        return torch.tensor(img, dtype=torch.float32), label_idx, label_str


def collate_fn(batch):
    imgs, labels, label_strs = zip(*batch)
    imgs = torch.stack(imgs)
    lengths = torch.tensor([len(l) for l in labels], dtype=torch.long)
    flat_labels = torch.tensor([c for l in labels for c in l], dtype=torch.long)
    return imgs, flat_labels, lengths, label_strs


# ── 학습 루프 ─────────────────────────────────────────────────────
def train(args):
    device = torch.device("cuda" if torch.cuda.is_available() else "cpu")
    print(f"[정보] 디바이스: {device}")
    if device.type == "cuda":
        print(f"       GPU: {torch.cuda.get_device_name(0)}")

    # ── EasyOCR 로더로 모델 + 문자셋 초기화 ──
    print("[정보] EasyOCR 모델 로드 중…")
    import easyocr
    from easyocr.recognition import get_recognizer, CTCLabelConverter

    model_dir = str(_BUNDLE_DIR) if _BUNDLE_DIR.is_dir() else None
    reader_tmp = easyocr.Reader(
        ["ko", "en"],
        gpu=(device.type == "cuda"),
        model_storage_directory=model_dir,
        verbose=False,
    )
    character   = reader_tmp.character
    converter   = reader_tmp.converter
    recognizer  = reader_tmp.recognizer   # DataParallel wrapper

    # DataParallel 내부 모델 꺼내기
    if hasattr(recognizer, "module"):
        model = recognizer.module
    else:
        model = recognizer
    model = model.to(device)
    model.train()

    # ── 데이터 로드 ──
    print("[정보] 교정 데이터 로드 중…")
    samples = load_corrections(args.db)
    print(f"  → DB 교정 데이터: {len(samples)}건")

    if args.extra_data:
        extra = load_aihub_data(args.extra_data)
        print(f"  → AI-Hub 추가 데이터: {len(extra)}건")
        samples += extra

    if not samples:
        print("[오류] 학습 데이터가 없습니다. DB에 교정 데이터를 먼저 입력하세요.")
        sys.exit(1)

    dataset = CorrectionDataset(samples, character)
    print(f"  → 유효 샘플: {len(dataset)}건")

    if len(dataset) == 0:
        print("[오류] 유효한 샘플이 없습니다.")
        sys.exit(1)

    loader = DataLoader(
        dataset,
        batch_size=min(args.batch, len(dataset)),
        shuffle=True,
        collate_fn=collate_fn,
        num_workers=0,
    )

    # ── 옵티마이저 ──
    # SequenceModeling + Prediction만 빠른 학습률; FeatureExtraction은 낮게
    feat_params = list(model.FeatureExtraction.parameters())
    other_params = (
        list(model.SequenceModeling.parameters()) +
        list(model.Prediction.parameters())
    )
    optimizer = torch.optim.AdamW([
        {"params": feat_params,  "lr": args.lr * 0.1},
        {"params": other_params, "lr": args.lr},
    ], weight_decay=1e-4)
    scheduler = torch.optim.lr_scheduler.CosineAnnealingLR(
        optimizer, T_max=args.epochs, eta_min=args.lr * 0.01
    )
    criterion = nn.CTCLoss(blank=0, reduction="mean", zero_infinity=True)

    # ── 학습 ──
    print(f"\n[학습 시작] epochs={args.epochs}, batch={args.batch}, lr={args.lr}")
    best_loss = float("inf")

    for epoch in range(1, args.epochs + 1):
        total_loss = 0.0
        n_batches  = 0

        for imgs, flat_labels, lengths, _ in loader:
            imgs        = imgs.to(device)
            flat_labels = flat_labels.to(device)

            # forward
            preds = model(imgs, flat_labels)   # (T, B, num_class)
            preds = preds.log_softmax(2).permute(1, 0, 2)   # (T, B, C) for CTC

            T          = preds.size(0)
            input_lens = torch.full((imgs.size(0),), T, dtype=torch.long)

            loss = criterion(preds, flat_labels, input_lens, lengths)

            optimizer.zero_grad()
            loss.backward()
            nn.utils.clip_grad_norm_(model.parameters(), 5.0)
            optimizer.step()

            total_loss += loss.item()
            n_batches  += 1

        scheduler.step()
        avg_loss = total_loss / max(n_batches, 1)
        lr_now   = optimizer.param_groups[1]["lr"]
        print(f"  Epoch {epoch:3d}/{args.epochs} | loss={avg_loss:.4f} | lr={lr_now:.2e}")

        if avg_loss < best_loss:
            best_loss = avg_loss
            _save_model(model, device)
            print(f"           → 모델 저장 (loss 개선)")

    print(f"\n[완료] 최적 모델 저장 위치: {_OUTPUT_PATH}")
    print("       다음 번 OCR 실행 시 자동으로 파인튜닝된 모델을 사용합니다.")


def _save_model(model: nn.Module, device: torch.device):
    _CUSTOM_DIR.mkdir(parents=True, exist_ok=True)
    # DataParallel 형태로 저장 (EasyOCR 로드 형식과 일치)
    wrapped = nn.DataParallel(model).to(device)
    torch.save(wrapped.state_dict(), str(_OUTPUT_PATH))


# ── CLI ──────────────────────────────────────────────────────────
def main():
    parser = argparse.ArgumentParser(description="EasyOCR 한국어 파인튜닝")
    parser.add_argument("--db",         required=True,  help="프로젝트 DB 경로 (.db)")
    parser.add_argument("--extra-data", default=None,   help="AI-Hub 추가 데이터 디렉토리")
    parser.add_argument("--epochs",     type=int,   default=15,   help="에폭 수 (기본: 15)")
    parser.add_argument("--batch",      type=int,   default=32,   help="배치 크기 (기본: 32)")
    parser.add_argument("--lr",         type=float, default=1e-4, help="학습률 (기본: 1e-4)")
    args = parser.parse_args()

    if not os.path.isfile(args.db):
        print(f"[오류] DB 파일이 없습니다: {args.db}")
        sys.exit(1)

    train(args)


if __name__ == "__main__":
    main()
