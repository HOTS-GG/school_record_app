"""
OCR 피드백 도구 (독립 실행형)
실행: python ocr_feedback_app.py  /  또는 run_feedback.bat

구성
  왼쪽  : 원본 비율 이미지 + bbox 번호 오버레이
  오른쪽 : 번호 대응 인식 결과 (수정 가능) + 저장
"""

import sys, os, json, datetime, tkinter as tk
from tkinter import ttk, filedialog, messagebox
from pathlib import Path

_SCRIPT_DIR       = Path(__file__).parent
_CORRECTIONS_FILE = _SCRIPT_DIR / "corrections.jsonl"

# ── EasyOCR 로드 ─────────────────────────────────────────────────
_reader = None

def _get_reader():
    global _reader
    if _reader is not None:
        return _reader
    bundle = _SCRIPT_DIR / "models"
    kwargs = {"gpu": False}
    if bundle.is_dir():
        os.environ["EASYOCR_MODULE_PATH"] = str(bundle)
        kwargs["model_storage_directory"]  = str(bundle)
    import easyocr
    _reader = easyocr.Reader(["ko", "en"], **kwargs)
    return _reader

def run_ocr(image_path: str) -> list:
    import numpy as np
    from PIL import Image
    img_np = np.array(Image.open(image_path).convert("RGB"))[:, :, ::-1]
    raw = _get_reader().readtext(img_np, detail=1, paragraph=False)
    out = []
    for bbox_pts, text, conf in raw:
        xs = [p[0] for p in bbox_pts]; ys = [p[1] for p in bbox_pts]
        out.append({
            "text":       text,
            "confidence": round(float(conf), 4),
            "bbox":       [int(min(xs)), int(min(ys)), int(max(xs)), int(max(ys))],
        })
    return out

def save_correction(image_path, raw_text, corrected_text):
    rec = {"image_path": image_path, "raw_text": raw_text,
           "corrected_text": corrected_text,
           "saved_at": datetime.datetime.now().isoformat()}
    with open(_CORRECTIONS_FILE, "a", encoding="utf-8") as f:
        f.write(json.dumps(rec, ensure_ascii=False) + "\n")


# ── 메인 앱 ──────────────────────────────────────────────────────
class App(tk.Tk):
    PAD = 8
    BBOX_COLORS = ("#3b82f6", "#10b981", "#f59e0b", "#8b5cf6",
                   "#ec4899", "#06b6d4", "#84cc16", "#f97316")
    WARN_COLOR  = "#ef4444"

    def __init__(self):
        super().__init__()
        self.title("OCR 피드백 도구")
        self.geometry("1100x720")
        self.minsize(800, 500)
        self.configure(bg="#f3f4f6")

        self._img_path   = None
        self._pil_img    = None   # 원본 PIL Image
        self._tk_img     = None   # PhotoImage (GC 방지용 참조 유지)
        self._results    = []
        self._cvars      = []     # 교정 StringVar 목록
        self._scale      = 1.0
        self._img_ox     = 0      # 캔버스 내 이미지 왼쪽 여백
        self._img_oy     = 0      # 캔버스 내 이미지 위쪽 여백

        self._build_ui()

    # ── UI ───────────────────────────────────────────────────────
    def _build_ui(self):
        # ── 툴바 ──
        bar = tk.Frame(self, bg="#1e293b", pady=7, padx=10)
        bar.pack(fill=tk.X)

        def btn(parent, text, cmd, bg, state=tk.NORMAL):
            return tk.Button(parent, text=text, command=cmd,
                             bg=bg, fg="white", relief=tk.FLAT,
                             font=("Segoe UI", 10, "bold"),
                             padx=14, pady=5, cursor="hand2", state=state)

        btn(bar, "📂 이미지 열기", self._pick, "#3b82f6").pack(side=tk.LEFT, padx=(0,6))
        self._btn_run  = btn(bar, "▶ OCR 실행",  self._ocr,  "#10b981", tk.DISABLED)
        self._btn_run.pack(side=tk.LEFT, padx=(0,6))
        self._btn_save = btn(bar, "💾 교정 저장", self._save, "#f59e0b", tk.DISABLED)
        self._btn_save.pack(side=tk.LEFT)

        self._status = tk.StringVar(value="이미지를 선택하세요.")
        tk.Label(bar, textvariable=self._status,
                 bg="#1e293b", fg="#94a3b8",
                 font=("Segoe UI", 9)).pack(side=tk.LEFT, padx=14)

        # ── 본문 (PanedWindow) ──
        pw = tk.PanedWindow(self, orient=tk.HORIZONTAL,
                             sashwidth=6, sashrelief=tk.FLAT,
                             bg="#cbd5e1")
        pw.pack(fill=tk.BOTH, expand=True, padx=0, pady=0)

        # 왼쪽: 이미지 캔버스
        self._canvas = tk.Canvas(pw, bg="#1e293b", highlightthickness=0, cursor="crosshair")
        pw.add(self._canvas, minsize=400, width=660)
        self._canvas.bind("<Configure>", self._on_canvas_resize)

        # 오른쪽: 결과 패널
        right = tk.Frame(pw, bg="#f3f4f6")
        pw.add(right, minsize=280)
        self._build_result_panel(right)

    def _build_result_panel(self, parent):
        header = tk.Frame(parent, bg="#f3f4f6")
        header.pack(fill=tk.X, padx=10, pady=(8, 4))
        tk.Label(header, text="인식 결과", bg="#f3f4f6",
                 fg="#1e293b", font=("Segoe UI", 11, "bold")).pack(side=tk.LEFT)
        self._count_lbl = tk.Label(header, text="", bg="#f3f4f6",
                                    fg="#64748b", font=("Segoe UI", 9))
        self._count_lbl.pack(side=tk.RIGHT)

        sep = tk.Frame(parent, bg="#e2e8f0", height=1)
        sep.pack(fill=tk.X, padx=0)

        # 스크롤 가능 영역
        wrap = tk.Frame(parent, bg="#f3f4f6")
        wrap.pack(fill=tk.BOTH, expand=True)
        vsb = ttk.Scrollbar(wrap, orient=tk.VERTICAL)
        vsb.pack(side=tk.RIGHT, fill=tk.Y)
        self._rcanvas = tk.Canvas(wrap, bg="#f3f4f6",
                                   highlightthickness=0, yscrollcommand=vsb.set)
        self._rcanvas.pack(side=tk.LEFT, fill=tk.BOTH, expand=True)
        vsb.config(command=self._rcanvas.yview)
        self._rinner = tk.Frame(self._rcanvas, bg="#f3f4f6")
        self._rcanvas.create_window((0,0), window=self._rinner, anchor="nw", tags="inner")
        self._rinner.bind("<Configure>", self._on_rinner_resize)
        self._rcanvas.bind("<Configure>",
            lambda e: self._rcanvas.itemconfig("inner", width=e.width))
        for w in (self._rcanvas, self._rinner):
            w.bind("<MouseWheel>",
                lambda e: self._rcanvas.yview_scroll(-1*(e.delta//120), "units"))

    def _on_rinner_resize(self, _=None):
        self._rcanvas.configure(scrollregion=self._rcanvas.bbox("all"))

    # ── 이미지 ───────────────────────────────────────────────────
    def _pick(self):
        path = filedialog.askopenfilename(
            filetypes=[("이미지", "*.png *.jpg *.jpeg *.bmp *.webp"), ("모두", "*")])
        if not path:
            return
        from PIL import Image
        self._img_path = path
        self._pil_img  = Image.open(path).convert("RGB")
        self._results  = []
        self._clear_results()
        self._render_image()
        self._status.set(Path(path).name)
        self._btn_run.config(state=tk.NORMAL)
        self._btn_save.config(state=tk.DISABLED)

    def _on_canvas_resize(self, _=None):
        self._render_image()

    def _render_image(self):
        """원본 비율 유지, 캔버스에 fit. 스케일·오프셋 저장."""
        if self._pil_img is None:
            return
        from PIL import Image, ImageTk

        cw = self._canvas.winfo_width()  or 660
        ch = self._canvas.winfo_height() or 680
        ow, oh = self._pil_img.size

        # fit 스케일 (업스케일 없음)
        scale = min(cw / ow, ch / oh, 1.0)
        nw = max(1, int(ow * scale))
        nh = max(1, int(oh * scale))
        self._scale  = scale
        self._img_ox = (cw - nw) // 2
        self._img_oy = (ch - nh) // 2

        disp = self._pil_img.resize((nw, nh), Image.LANCZOS)
        self._tk_img = ImageTk.PhotoImage(disp)

        self._canvas.delete("all")
        self._canvas.create_image(self._img_ox, self._img_oy,
                                   image=self._tk_img, anchor=tk.NW)
        self._draw_bboxes()

    def _draw_bboxes(self):
        if not self._results:
            return
        s = self._scale
        ox, oy = self._img_ox, self._img_oy
        for i, r in enumerate(self._results):
            x1, y1, x2, y2 = r["bbox"]
            cx1 = ox + x1*s; cy1 = oy + y1*s
            cx2 = ox + x2*s; cy2 = oy + y2*s
            color = self.WARN_COLOR if r["confidence"] < 0.55 \
                    else self.BBOX_COLORS[i % len(self.BBOX_COLORS)]
            self._canvas.create_rectangle(cx1, cy1, cx2, cy2,
                                           outline=color, width=2)
            # 번호 배지
            num = str(i + 1)
            self._canvas.create_rectangle(cx1, cy1,
                                           cx1 + len(num)*8 + 6, cy1 + 16,
                                           fill=color, outline="")
            self._canvas.create_text(cx1 + 3, cy1 + 2, text=num,
                                      anchor=tk.NW, fill="white",
                                      font=("Arial", 8, "bold"))

    # ── OCR ──────────────────────────────────────────────────────
    def _ocr(self):
        if not self._img_path:
            return
        self._status.set("OCR 실행 중… (최초 1회 모델 로딩 시 1~2분 소요)")
        self._btn_run.config(state=tk.DISABLED)
        self.update()
        try:
            self._results = run_ocr(self._img_path)
            self._render_image()
            self._show_results()
            self._status.set(f"완료 — {len(self._results)}건 인식")
            self._btn_save.config(
                state=tk.NORMAL if self._results else tk.DISABLED)
        except Exception as e:
            messagebox.showerror("오류", str(e))
            self._status.set("오류 발생")
        finally:
            self._btn_run.config(state=tk.NORMAL)

    # ── 결과 목록 ─────────────────────────────────────────────────
    def _clear_results(self):
        for w in self._rinner.winfo_children():
            w.destroy()
        self._cvars.clear()
        self._count_lbl.config(text="")

    def _show_results(self):
        self._clear_results()
        self._count_lbl.config(text=f"{len(self._results)}건")

        for i, r in enumerate(self._results):
            color = self.WARN_COLOR if r["confidence"] < 0.55 \
                    else self.BBOX_COLORS[i % len(self.BBOX_COLORS)]
            conf_pct = int(r["confidence"] * 100)

            # ── 카드 ──
            card = tk.Frame(self._rinner, bg="white",
                             highlightbackground="#e2e8f0",
                             highlightthickness=1)
            card.pack(fill=tk.X, padx=8, pady=3, ipady=4)
            card.bind("<MouseWheel>",
                lambda e: self._rcanvas.yview_scroll(-1*(e.delta//120), "units"))

            # 번호 + 신뢰도
            meta = tk.Frame(card, bg="white")
            meta.pack(fill=tk.X, padx=6, pady=(4,2))

            num_badge = tk.Label(meta,
                text=f" {i+1} ", bg=color, fg="white",
                font=("Arial", 8, "bold"), padx=2)
            num_badge.pack(side=tk.LEFT, padx=(0, 6))

            tk.Label(meta, text=f"신뢰도 {conf_pct}%",
                     bg="white",
                     fg=self.WARN_COLOR if conf_pct < 55 else "#64748b",
                     font=("Segoe UI", 8)).pack(side=tk.LEFT)

            # 텍스트 입력
            var = tk.StringVar(value=r["text"])
            self._cvars.append(var)
            e = tk.Entry(card, textvariable=var,
                         font=("Malgun Gothic", 10),
                         bg="#f8fafc", relief=tk.FLAT,
                         highlightbackground="#cbd5e1",
                         highlightthickness=1)
            e.pack(fill=tk.X, padx=6, pady=(0,4), ipady=3)
            e.bind("<MouseWheel>",
                lambda ev: self._rcanvas.yview_scroll(-1*(ev.delta//120), "units"))

    # ── 저장 ─────────────────────────────────────────────────────
    def _save(self):
        if not self._img_path or not self._results:
            return
        count = 0
        for i, r in enumerate(self._results):
            corrected = self._cvars[i].get().strip()
            if corrected and corrected != r["text"]:
                save_correction(self._img_path, r["text"], corrected)
                count += 1
        msg = (f"수정된 항목 {count}건 저장 완료\n"
               f"({_CORRECTIONS_FILE.name})" if count else "수정된 항목이 없습니다.")
        self._status.set(f"저장 완료 — {count}건")
        messagebox.showinfo("저장", msg)


# ── 엔트리포인트 ─────────────────────────────────────────────────
if __name__ == "__main__":
    try:                                    # Hi-DPI 인식
        from ctypes import windll
        windll.shcore.SetProcessDpiAwareness(1)
    except Exception:
        pass
    App().mainloop()
