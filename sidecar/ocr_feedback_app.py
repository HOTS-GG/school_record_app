"""
OCR 피드백 도구 (독립 실행형)
실행: python ocr_feedback_app.py

- 이미지 파일 선택 → 전체 텍스트 자동 인식
- 인식 결과를 수정하고 저장
- 수정된 데이터는 corrections.jsonl에 누적 (파인튜닝용)
"""

import sys
import os
import json
import datetime
import tkinter as tk
from tkinter import ttk, filedialog, messagebox
from pathlib import Path

_SCRIPT_DIR = Path(__file__).parent
_CORRECTIONS_FILE = _SCRIPT_DIR / "corrections.jsonl"

# ── 모델 로드 ────────────────────────────────────────────────────
_reader = None

def _get_reader():
    global _reader
    if _reader is not None:
        return _reader

    bundle_dir = _SCRIPT_DIR / "models"
    kwargs = {"gpu": False}
    if bundle_dir.is_dir():
        os.environ["EASYOCR_MODULE_PATH"] = str(bundle_dir)
        kwargs["model_storage_directory"] = str(bundle_dir)

    import easyocr
    _reader = easyocr.Reader(["ko", "en"], **kwargs)
    return _reader


def run_ocr(image_path: str) -> list:
    """이미지에서 텍스트 인식 (손글씨/인쇄체 구분 없이 전부)"""
    import numpy as np
    from PIL import Image

    reader = _get_reader()

    img_np = np.array(Image.open(image_path).convert("RGB"))[:, :, ::-1]
    raw = reader.readtext(img_np, detail=1, paragraph=False)

    results = []
    for (bbox_pts, text, conf) in raw:
        xs = [p[0] for p in bbox_pts]
        ys = [p[1] for p in bbox_pts]
        results.append({
            "text":       text,
            "confidence": round(float(conf), 4),
            "bbox":       [int(min(xs)), int(min(ys)), int(max(xs)), int(max(ys))],
        })
    return results


def save_correction(image_path: str, raw_text: str, corrected_text: str):
    """corrections.jsonl에 교정 데이터 추가"""
    record = {
        "image_path":     image_path,
        "raw_text":       raw_text,
        "corrected_text": corrected_text,
        "saved_at":       datetime.datetime.now().isoformat(),
    }
    with open(_CORRECTIONS_FILE, "a", encoding="utf-8") as f:
        f.write(json.dumps(record, ensure_ascii=False) + "\n")


# ── GUI ──────────────────────────────────────────────────────────
class OcrFeedbackApp(tk.Tk):
    def __init__(self):
        super().__init__()
        self.title("OCR 피드백 도구")
        self.geometry("960x700")
        self.minsize(700, 500)
        self.configure(bg="#f8f8f8")
        self._image_path = None
        self._ocr_results = []
        self._correction_vars = []
        self._build_ui()

    # ── UI 구성 ──────────────────────────────────────────────────
    def _build_ui(self):
        # 상단 툴바
        toolbar = tk.Frame(self, bg="#f0f0f0", pady=6, padx=10)
        toolbar.pack(fill=tk.X, side=tk.TOP)

        tk.Button(
            toolbar, text="📂  이미지 열기", command=self._pick_image,
            bg="#3b82f6", fg="white", padx=14, pady=6,
            font=("Segoe UI", 10, "bold"), relief=tk.FLAT, cursor="hand2",
        ).pack(side=tk.LEFT, padx=(0, 8))

        self._run_btn = tk.Button(
            toolbar, text="▶  OCR 실행", command=self._run_ocr,
            bg="#10b981", fg="white", padx=14, pady=6,
            font=("Segoe UI", 10, "bold"), relief=tk.FLAT, cursor="hand2",
            state=tk.DISABLED,
        )
        self._run_btn.pack(side=tk.LEFT, padx=(0, 8))

        self._save_btn = tk.Button(
            toolbar, text="💾  교정 저장", command=self._save_all,
            bg="#f59e0b", fg="white", padx=14, pady=6,
            font=("Segoe UI", 10, "bold"), relief=tk.FLAT, cursor="hand2",
            state=tk.DISABLED,
        )
        self._save_btn.pack(side=tk.LEFT)

        self._status_var = tk.StringVar(value="이미지를 선택하세요.")
        tk.Label(
            toolbar, textvariable=self._status_var,
            bg="#f0f0f0", fg="#6b7280", font=("Segoe UI", 9),
        ).pack(side=tk.LEFT, padx=16)

        # 본문 영역 (이미지 | 결과 패널)
        paned = tk.PanedWindow(self, orient=tk.HORIZONTAL, sashwidth=5, bg="#d1d5db")
        paned.pack(fill=tk.BOTH, expand=True, padx=6, pady=(0, 6))

        # 좌: 이미지 미리보기
        left = tk.Frame(paned, bg="#f8f8f8")
        paned.add(left, minsize=300, width=460)

        self._canvas = tk.Canvas(left, bg="#e5e7eb", highlightthickness=0)
        self._canvas.pack(fill=tk.BOTH, expand=True)
        self._canvas.bind("<Configure>", self._redraw_image)

        # 우: 결과 목록 (스크롤)
        right = tk.Frame(paned, bg="#f8f8f8")
        paned.add(right, minsize=240)

        header = tk.Frame(right, bg="#f8f8f8")
        header.pack(fill=tk.X, pady=(6, 2), padx=6)
        tk.Label(header, text="인식 결과 (수정 가능)", bg="#f8f8f8",
                 fg="#374151", font=("Segoe UI", 10, "bold")).pack(side=tk.LEFT)
        self._count_var = tk.StringVar(value="")
        tk.Label(header, textvariable=self._count_var, bg="#f8f8f8",
                 fg="#9ca3af", font=("Segoe UI", 9)).pack(side=tk.RIGHT)

        scroll_frame = tk.Frame(right, bg="#f8f8f8")
        scroll_frame.pack(fill=tk.BOTH, expand=True, padx=6)
        vsb = ttk.Scrollbar(scroll_frame, orient=tk.VERTICAL)
        vsb.pack(side=tk.RIGHT, fill=tk.Y)
        self._result_canvas = tk.Canvas(
            scroll_frame, bg="#f8f8f8", highlightthickness=0, yscrollcommand=vsb.set
        )
        self._result_canvas.pack(side=tk.LEFT, fill=tk.BOTH, expand=True)
        vsb.config(command=self._result_canvas.yview)
        self._result_inner = tk.Frame(self._result_canvas, bg="#f8f8f8")
        self._result_canvas.create_window((0, 0), window=self._result_inner, anchor="nw")
        self._result_inner.bind(
            "<Configure>",
            lambda e: self._result_canvas.configure(
                scrollregion=self._result_canvas.bbox("all")
            ),
        )
        self._result_canvas.bind("<MouseWheel>",
            lambda e: self._result_canvas.yview_scroll(-1 * (e.delta // 120), "units"))

        self._pil_image = None
        self._tk_image  = None

    # ── 이미지 선택 ──────────────────────────────────────────────
    def _pick_image(self):
        path = filedialog.askopenfilename(
            filetypes=[("이미지", "*.png *.jpg *.jpeg *.bmp *.webp"), ("모두", "*")]
        )
        if not path:
            return
        self._image_path = path
        self._status_var.set(Path(path).name)
        self._load_image_preview(path)
        self._run_btn.config(state=tk.NORMAL)
        self._clear_results()

    def _load_image_preview(self, path: str):
        from PIL import Image
        img = Image.open(path)
        self._pil_image = img
        self._redraw_image()

    def _redraw_image(self, event=None):
        if self._pil_image is None:
            return
        from PIL import ImageTk
        cw = self._canvas.winfo_width()  or 460
        ch = self._canvas.winfo_height() or 500
        img = self._pil_image.copy()
        img.thumbnail((cw, ch), resample=1)
        self._tk_image = ImageTk.PhotoImage(img)
        self._canvas.delete("all")
        self._canvas.create_image(cw // 2, ch // 2, image=self._tk_image, anchor=tk.CENTER)
        self._draw_bboxes(img.width, img.height, cw, ch)

    def _draw_bboxes(self, tw, th, cw, ch):
        if not self._ocr_results:
            return
        ow, oh = self._pil_image.size
        sx = tw / ow
        sy = th / oh
        ox = (cw - tw) // 2
        oy = (ch - th) // 2
        for r in self._ocr_results:
            x1, y1, x2, y2 = r["bbox"]
            color = "#ef4444" if r["confidence"] < 0.55 else "#3b82f6"
            self._canvas.create_rectangle(
                ox + x1 * sx, oy + y1 * sy,
                ox + x2 * sx, oy + y2 * sy,
                outline=color, width=2,
            )

    # ── OCR 실행 ─────────────────────────────────────────────────
    def _run_ocr(self):
        if not self._image_path:
            return
        self._status_var.set("모델 로딩 중… (최초 1회, 잠시 기다리세요)")
        self._run_btn.config(state=tk.DISABLED)
        self.update()

        try:
            results = run_ocr(self._image_path)
            self._ocr_results = results
            self._show_results(results)
            self._redraw_image()
            self._status_var.set(f"인식 완료 — {len(results)}건")
            self._save_btn.config(state=tk.NORMAL if results else tk.DISABLED)
        except Exception as e:
            messagebox.showerror("OCR 오류", str(e))
            self._status_var.set("오류 발생")
        finally:
            self._run_btn.config(state=tk.NORMAL)

    # ── 결과 표시 ────────────────────────────────────────────────
    def _clear_results(self):
        for w in self._result_inner.winfo_children():
            w.destroy()
        self._correction_vars.clear()
        self._ocr_results = []
        self._count_var.set("")

    def _show_results(self, results: list):
        self._clear_results()
        self._count_var.set(f"{len(results)}건")
        for i, r in enumerate(results):
            card = tk.Frame(
                self._result_inner, bg="white",
                highlightbackground="#e5e7eb", highlightthickness=1,
                padx=8, pady=6,
            )
            card.pack(fill=tk.X, pady=2)

            conf_pct = int(r["confidence"] * 100)
            conf_color = "#ef4444" if conf_pct < 55 else "#6b7280"
            tk.Label(card, text=f"신뢰도 {conf_pct}%",
                     bg="white", fg=conf_color, font=("Segoe UI", 8)).pack(anchor="w")

            var = tk.StringVar(value=r["text"])
            self._correction_vars.append(var)
            entry = tk.Entry(card, textvariable=var, font=("Malgun Gothic", 10),
                             bg="#f9fafb", relief=tk.FLAT,
                             highlightbackground="#d1d5db", highlightthickness=1)
            entry.pack(fill=tk.X, pady=(2, 0))

    # ── 교정 저장 ─────────────────────────────────────────────────
    def _save_all(self):
        if not self._image_path or not self._ocr_results:
            return
        count = 0
        for i, r in enumerate(self._ocr_results):
            corrected = self._correction_vars[i].get().strip()
            if corrected != r["text"]:
                save_correction(self._image_path, r["text"], corrected)
                count += 1
        self._status_var.set(f"저장 완료 — {count}건 교정 데이터 기록")
        messagebox.showinfo("저장 완료",
            f"수정된 항목 {count}건이 corrections.jsonl에 저장됐습니다.")


# ── 엔트리포인트 ────────────────────────────────────────────────
if __name__ == "__main__":
    # tkinter DPI 조정 (Windows 고해상도 모니터)
    try:
        from ctypes import windll
        windll.shcore.SetProcessDpiAwareness(1)
    except Exception:
        pass

    app = OcrFeedbackApp()
    app.mainloop()
