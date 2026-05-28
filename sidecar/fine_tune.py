"""
EasyOCR 한국어 인식 모델 파인튜닝 스크립트

사용법:
  python fine_tune.py --db <project.db> [--epochs 15] [--batch 32] [--lr 1e-4]
  python fine_tune.py --db <project.db> --extra-data <aihub_dir> [--max-extra 50000]

출력: sidecar/custom_model/korean_g2.pth  (사이드카가 자동으로 로드)

AI-Hub 데이터 경로 예시:
  D:\School-record-app\writeDB\다양한 형태의 한글 문자 OCR

  → Training/[원천]Training_필기체.zip  (이미지)
  → Training/[라벨]Training_필기체.zip  (JSON 레이블)
  압축 해제 없이 zip 내부에서 직접 읽습니다.
"""

import argparse
import io
import json
import os
import random
import sqlite3
import sys
import zipfile
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

if _BUNDLE_DIR.is_dir():
    os.environ["EASYOCR_MODULE_PATH"] = str(_BUNDLE_DIR)


# ── DB 교정 데이터 로드 ───────────────────────────────────────────
def load_corrections(db_path: str) -> list[dict]:
    conn = sqlite3.connect(db_path)
    rows = conn.execute("""
        SELECT r.bbox, r.corrected_text, s.image_path
        FROM   OcrResult r
        JOIN   OcrSession s ON s.id = r.session_id
        WHERE  r.corrected_text IS NOT NULL AND r.corrected_text != ''
    """).fetchall()
    conn.close()

    samples = []
    for bbox_json, corrected, img_path in rows:
        try:
            bbox = json.loads(bbox_json)
            x1, y1, x2, y2 = int(bbox[0]), int(bbox[1]), int(bbox[2]), int(bbox[3])
        except Exception:
            continue
        label = corrected.strip()
        if label:
            samples.append({"kind": "file", "path": img_path,
                            "bbox": (x1, y1, x2, y2), "label": label})
    return samples


# ── AI-Hub zip 데이터 로드 ────────────────────────────────────────
def load_aihub_data(data_dir: str, max_samples: int | None = None) -> list[dict]:
    """
    AI-Hub '다양한 형태의 한글 문자 OCR' 데이터셋 로드.

    구조:
      <data_dir>/Training/[원천]Training_필기체.zip  → 이미지
      <data_dir>/Training/[라벨]Training_필기체.zip  → JSON 레이블
      <data_dir>/Training/[원천]Training_인쇄체.zip  → 이미지 (인쇄체)
      <data_dir>/Training/[라벨]Training_인쇄체.zip  → JSON 레이블 (인쇄체)

    JSON 형식:
      { "info": {"text": "가"}, "image": {"file_name": "00130001001.jpg"}, ... }
      레이블은 info.text 또는 text.letter.value

    zip 압축 해제 없이 직접 읽습니다.
    """
    data_dir = Path(data_dir)
    samples  = []

    for split in ("Training", "Validation"):
        split_dir = data_dir / split
        if not split_dir.is_dir():
            continue

        # zip 파일 쌍 찾기: 필기체 + 인쇄체 모두
        zip_pairs: list[tuple] = []   # (src_zip, lbl_zip, type_name)
        src_map: dict[str, Path] = {}
        lbl_map: dict[str, Path] = {}
        for f in split_dir.iterdir():
            name = f.name
            if not name.endswith(".zip"):
                continue
            for type_name in ("필기체", "인쇄체"):
                if type_name not in name:
                    continue
                if "원천" in name:
                    src_map[type_name] = f
                elif "라벨" in name:
                    lbl_map[type_name] = f

        for type_name in ("필기체", "인쇄체"):
            src_zip = src_map.get(type_name)
            lbl_zip = lbl_map.get(type_name)
            if src_zip and lbl_zip:
                zip_pairs.append((src_zip, lbl_zip, type_name))

        if not zip_pairs:
            print(f"[경고] {split}/ 에서 zip 파일을 찾지 못했습니다.")
            continue

        for src_zip, lbl_zip, type_name in zip_pairs:
            print(f"  [{split}/{type_name}] 레이블 zip 인덱싱 중… ({lbl_zip.name})")

            # stem → 샘플 목록 (인쇄체는 1개 이미지에서 여러 bbox 추출)
            img_samples: dict[str, list[dict]] = {}

            with zipfile.ZipFile(lbl_zip, "r") as zl:
                json_names = [n for n in zl.namelist()
                              if n.endswith(".json") and not n.endswith("/")]
                for jname in json_names:
                    try:
                        ann  = json.loads(zl.read(jname).decode("utf-8"))
                    except Exception:
                        continue

                    # ── 필기체: 이미지 전체 = 글자 1개 ──
                    text = (ann.get("info", {}).get("text")
                            or ann.get("text", {}).get("letter", {}).get("value")
                            or "").strip()
                    if text:
                        fname = ann.get("image", {}).get("file_name", "")
                        stem  = Path(fname).stem if fname else Path(jname).stem
                        img_samples[stem] = [{"bbox": None, "label": text}]
                        continue

                    # ── 인쇄체: A4 페이지 이미지 + 단어 bbox 목록 ──
                    words = ann.get("text", {}).get("word", [])
                    if not words:
                        continue
                    fname = ann.get("image", {}).get("file_name", "")
                    stem  = Path(fname).stem if fname else Path(jname).stem
                    entries = []
                    for w in words:
                        wb    = w.get("wordbox", [])
                        label = w.get("value", "").strip()
                        if label and len(wb) == 4:
                            x1, y1, x2, y2 = wb
                            entries.append({"bbox": (x1, y1, x2, y2), "label": label})
                    if entries:
                        img_samples[stem] = entries

            print(f"  [{split}/{type_name}] 이미지 zip 스캔 중… ({src_zip.name})")
            type_samples: list[dict] = []
            with zipfile.ZipFile(src_zip, "r") as zi:
                img_names = [n for n in zi.namelist()
                             if n.lower().endswith((".jpg", ".png"))
                             and not n.endswith("/")]
                for iname in img_names:
                    entries = img_samples.get(Path(iname).stem)
                    if not entries:
                        continue
                    for e in entries:
                        type_samples.append({
                            "kind":     "zip",
                            "zip_path": str(src_zip),
                            "zip_name": iname,
                            "bbox":     e["bbox"],
                            "label":    e["label"],
                        })

            print(f"  [{split}/{type_name}] {len(type_samples):,}건 발견")
            samples.extend(type_samples)

    if not samples:
        print("[경고] AI-Hub 데이터를 찾지 못했습니다.")
        return samples

    # 무작위로 max_samples 만큼 추출
    if max_samples and len(samples) > max_samples:
        random.shuffle(samples)
        samples = samples[:max_samples]
        print(f"  → {max_samples:,}건으로 제한")

    return samples


# ── 이미지 전처리 ─────────────────────────────────────────────────
_IMG_H = 32
_IMG_W = 128

def _open_image(sample: dict) -> Image.Image | None:
    """sample dict 에서 PIL Image 반환."""
    try:
        if sample["kind"] == "zip":
            with zipfile.ZipFile(sample["zip_path"], "r") as z:
                data = z.read(sample["zip_name"])
            return Image.open(io.BytesIO(data)).convert("L")
        else:
            return Image.open(sample["path"]).convert("L")
    except Exception as e:
        return None


def crop_and_resize(sample: dict) -> np.ndarray | None:
    img = _open_image(sample)
    if img is None:
        return None

    bbox = sample.get("bbox")
    if bbox is not None:
        x1, y1, x2, y2 = bbox
        w, h = img.size
        x1 = max(0, min(x1, w)); x2 = max(0, min(x2, w))
        y1 = max(0, min(y1, h)); y2 = max(0, min(y2, h))
        if x2 <= x1 or y2 <= y1:
            return None
        img = img.crop((x1, y1, x2, y2))

    ow, oh = img.size
    nw = min(int(ow * (_IMG_H / oh)), _IMG_W)
    img = img.resize((nw, _IMG_H), Image.BICUBIC)

    canvas = Image.new("L", (_IMG_W, _IMG_H), 255)
    canvas.paste(img, (0, 0))
    arr = np.array(canvas, dtype=np.float32) / 255.0
    arr = (arr - 0.5) / 0.5
    return arr[np.newaxis, ...]   # (1, H, W)


# ── Dataset ──────────────────────────────────────────────────────
class CorrectionDataset(Dataset):
    def __init__(self, samples: list[dict], character: str):
        self.char_to_idx = {c: i + 1 for i, c in enumerate(character)}
        self.items: list[tuple] = []
        skipped = 0

        # DB 교정 데이터 (건수가 적으므로 미리 로드)
        for s in samples:
            if s["kind"] != "file":
                continue
            arr = crop_and_resize(s)
            if arr is None:
                skipped += 1
                continue
            idx = [self.char_to_idx[c] for c in s["label"] if c in self.char_to_idx]
            if not idx:
                skipped += 1
                continue
            self.items.append((arr, idx, s["label"]))

        # AI-Hub 데이터는 (sample_dict, label) 만 저장 후 __getitem__ 에서 lazy load
        self.lazy: list[dict] = [
            s for s in samples if s["kind"] == "zip"
        ]
        # lazy 검증: 레이블에 미지원 문자 있는 것 필터
        self.lazy = [s for s in self.lazy
                     if any(c in self.char_to_idx for c in s["label"])]

        if skipped:
            print(f"  → {skipped}건 건너뜀 (이미지 오류 또는 미지원 문자)")

    def __len__(self):
        return len(self.items) + len(self.lazy)

    def __getitem__(self, idx):
        if idx < len(self.items):
            arr, label_idx, label_str = self.items[idx]
            return torch.tensor(arr, dtype=torch.float32), label_idx, label_str

        s   = self.lazy[idx - len(self.items)]
        arr = crop_and_resize(s)
        if arr is None:
            arr = np.zeros((1, _IMG_H, _IMG_W), dtype=np.float32)
        label_idx = [self.char_to_idx[c] for c in s["label"] if c in self.char_to_idx]
        return torch.tensor(arr, dtype=torch.float32), label_idx, s["label"]


def collate_fn(batch):
    imgs, labels, label_strs = zip(*batch)
    imgs    = torch.stack(imgs)
    lengths = torch.tensor([len(l) for l in labels], dtype=torch.long)
    flat    = torch.tensor([c for l in labels for c in l], dtype=torch.long)
    return imgs, flat, lengths, label_strs


# ── 학습 루프 ─────────────────────────────────────────────────────
def train(args):
    device = torch.device("cuda" if torch.cuda.is_available() else "cpu")
    print(f"[정보] 디바이스: {device}")
    if device.type == "cuda":
        print(f"       GPU: {torch.cuda.get_device_name(0)}")

    print("[정보] EasyOCR 모델 로드 중…")
    import easyocr

    model_dir = str(_BUNDLE_DIR) if _BUNDLE_DIR.is_dir() else None
    reader = easyocr.Reader(["ko", "en"], gpu=(device.type == "cuda"),
                            model_storage_directory=model_dir, verbose=False)
    character  = reader.character
    recognizer = reader.recognizer
    model = recognizer.module if hasattr(recognizer, "module") else recognizer
    model = model.to(device)
    model.train()

    print("[정보] 데이터 로드 중…")
    samples = []
    if args.db:
        db_samples = load_corrections(args.db)
        print(f"  → DB 교정 데이터: {len(db_samples)}건")
        samples += db_samples

    if args.extra_data:
        extra = load_aihub_data(args.extra_data, args.max_extra)
        print(f"  → AI-Hub 데이터: {len(extra)}건")
        samples += extra

    if not samples:
        print("[오류] 학습 데이터가 없습니다. --db 또는 --extra-data 를 지정하세요.")
        sys.exit(1)

    dataset = CorrectionDataset(samples, character)
    print(f"  → 전체 유효 샘플: {len(dataset):,}건")

    if len(dataset) == 0:
        print("[오류] 유효한 샘플이 없습니다.")
        sys.exit(1)

    loader = DataLoader(dataset, batch_size=min(args.batch, len(dataset)),
                        shuffle=True, collate_fn=collate_fn,
                        num_workers=0, pin_memory=(device.type == "cuda"))

    feat_params  = list(model.FeatureExtraction.parameters())
    other_params = (list(model.SequenceModeling.parameters()) +
                    list(model.Prediction.parameters()))
    optimizer = torch.optim.AdamW([
        {"params": feat_params,  "lr": args.lr * 0.1},
        {"params": other_params, "lr": args.lr},
    ], weight_decay=1e-4)
    scheduler = torch.optim.lr_scheduler.CosineAnnealingLR(
        optimizer, T_max=args.epochs, eta_min=args.lr * 0.01)
    criterion = nn.CTCLoss(blank=0, reduction="mean", zero_infinity=True)

    print(f"\n[학습 시작] epochs={args.epochs}  batch={args.batch}  lr={args.lr}")
    best_loss = float("inf")

    for epoch in range(1, args.epochs + 1):
        total_loss = n_batches = 0
        for imgs, flat_labels, lengths, _ in loader:
            imgs        = imgs.to(device)
            flat_labels = flat_labels.to(device)

            preds = model(imgs, flat_labels)
            preds = preds.log_softmax(2).permute(1, 0, 2)
            T     = preds.size(0)
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
        print(f"  Epoch {epoch:3d}/{args.epochs} | loss={avg_loss:.4f}"
              f" | lr={optimizer.param_groups[1]['lr']:.2e}", end="")

        if avg_loss < best_loss:
            best_loss = avg_loss
            _save_model(model, device)
            print("  [저장]")
        else:
            print()

    print(f"\n[완료] 모델 저장: {_OUTPUT_PATH}")
    print("       다음 OCR 실행 시 자동으로 적용됩니다.")


def _save_model(model: nn.Module, device: torch.device):
    _CUSTOM_DIR.mkdir(parents=True, exist_ok=True)
    wrapped = nn.DataParallel(model).to(device)
    torch.save(wrapped.state_dict(), str(_OUTPUT_PATH))


# ── CLI ──────────────────────────────────────────────────────────
def main():
    p = argparse.ArgumentParser(description="EasyOCR 한국어 파인튜닝")
    p.add_argument("--db",         default=None,
                   help="프로젝트 DB 경로 (.db) — OCR 교정 데이터 출처 (선택)")
    p.add_argument("--extra-data", default=None,
                   help="AI-Hub 데이터 루트 디렉토리 (선택)")
    p.add_argument("--max-extra",  type=int, default=50000,
                   help="AI-Hub 에서 최대 사용할 샘플 수 (기본: 50000)")
    p.add_argument("--epochs",     type=int,   default=15)
    p.add_argument("--batch",      type=int,   default=32)
    p.add_argument("--lr",         type=float, default=1e-4)
    args = p.parse_args()

    if not args.db and not args.extra_data:
        print("[오류] --db 또는 --extra-data 중 하나는 반드시 지정해야 합니다.")
        sys.exit(1)

    if args.db and not os.path.isfile(args.db):
        print(f"[오류] DB 파일 없음: {args.db}")
        sys.exit(1)

    train(args)


if __name__ == "__main__":
    main()
