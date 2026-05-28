# 손글씨 OCR 기능 구현 계획

## 배경

학교생활기록부 앱에 손글씨 이미지를 텍스트로 변환하는 OCR 기능을 추가한다.
중국산 오픈소스 OCR 모델(RapidOCR/PaddleOCR 기반)을 완전 오프라인으로 구동하고,
모델 출력 결과와 사용자 교정값을 DB에 축적해 나중에 학습 데이터로 활용한다.

---

## 핵심 제약 조건

| 조건 | 내용 |
|------|------|
| 저사양 PC 기동 | 경량 런타임 필수 — PaddleOCR 풀 스택 불가 |
| 완전 오프라인 | 모델이 네트워크에 접근하면 안 됨 |
| 학습 데이터 수집 | 모델 출력 + 사용자 교정값을 DB에 적재 |
| Tauri 아키텍처 | Rust 백엔드가 모든 로직 처리, 컴포넌트는 Store만 사용 |

---

## 모델 선택: RapidOCR (ONNX 기반)

PaddleOCR 전체 스택 대신 **RapidOCR**를 사용한다.

```
PaddleOCR (PaddlePaddle 런타임) → ~2 GB RAM, GPU 권장
RapidOCR  (ONNX Runtime)        → ~300 MB RAM, CPU만으로 동작
```

RapidOCR는 PaddleOCR 모델 가중치를 ONNX로 변환한 버전이다.
동일한 인식 품질, 훨씬 낮은 자원 요구량.

- PyPI 패키지: `rapidocr-onnxruntime`
- 지원 언어: 한국어 포함 다국어
- 한국어 모델: `korean_PP-OCRv3_rec` (ONNX)

---

## 네트워크 격리 전략

모델을 "가두는" 방법은 설계 레벨에서 해결한다.

1. **사이드카 스크립트에 네트워크 임포트 없음** — `socket`, `urllib`, `requests` 등 일절 임포트하지 않음
2. **PyInstaller 번들 시 네트워크 모듈 제외** — `--exclude-module` 옵션 사용
3. **Tauri capabilities** — `default.json`에서 외부 HTTP 요청 권한 부여 안 함
4. **(선택) Windows 방화벽 규칙** — 설치 시 `ocr_sidecar.exe` 아웃바운드 차단 규칙 추가

---

## 전체 아키텍처

```
[OcrView.vue]
    ↓ (Store 경유, invoke() 직접 호출 금지)
[ocrStore.js] — Pinia
    ↓ invoke()
[commands/ocr.rs] — Rust
    ↓ tauri-plugin-shell sidecar spawn
[ocr_sidecar.exe] — PyInstaller 번들 바이너리
    ↓ stdin: JSON 요청 / stdout: JSON 응답
[RapidOCR + ONNX 모델 파일 (로컬)]
    ↓
[SQLite: ocr_session / ocr_result 테이블]
```

### 사이드카 통신 프로토콜 (stdin/stdout JSON)

```json
// 요청 (stdin)
{"image_path": "C:/Users/.../handwriting.jpg"}

// 성공 응답 (stdout)
{
  "success": true,
  "results": [
    {"text": "안녕하세요", "confidence": 0.97, "bbox": [x1,y1,x2,y2]},
    {"text": "홍길동",     "confidence": 0.94, "bbox": [x1,y1,x2,y2]}
  ]
}

// 실패 응답 (stdout)
{"success": false, "error": "모델 파일을 찾을 수 없습니다"}
```

---

## DB 스키마 추가 (schema v6)

```sql
-- OCR 세션 (이미지 1장 처리 단위)
CREATE TABLE IF NOT EXISTS ocr_session (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    image_path  TEXT NOT NULL,
    image_hash  TEXT NOT NULL,      -- SHA-256, 중복 탐지용
    created_at  TEXT NOT NULL
);

-- OCR 결과 + 사용자 교정값 (학습 데이터 원천)
CREATE TABLE IF NOT EXISTS ocr_result (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    session_id      INTEGER NOT NULL REFERENCES ocr_session(id) ON DELETE CASCADE,
    raw_text        TEXT NOT NULL,  -- 모델 원본 출력
    corrected_text  TEXT,           -- 사용자 교정값 (NULL = 교정 없음)
    confidence      REAL,           -- 모델 신뢰도 (0.0~1.0)
    bbox            TEXT,           -- JSON 문자열: 텍스트 영역 좌표
    corrected_at    TEXT
);
```

---

## 생성/수정 파일 목록

### Rust 백엔드

| 파일 | 변경 |
|------|------|
| `src-tauri/src/commands/ocr.rs` | **신규** — 4개 커맨드 구현 |
| `src-tauri/src/main.rs` | **수정** — OCR 커맨드 등록, shell 플러그인 추가 |
| `src-tauri/src/schema.sql` | **수정** — `ocr_session`, `ocr_result` 테이블 추가 |
| `src-tauri/src/db.rs` | **수정** — v6 마이그레이션 추가 |
| `src-tauri/Cargo.toml` | **수정** — `tauri-plugin-shell` 의존성 추가 |
| `src-tauri/tauri.conf.json` | **수정** — sidecar 경로 등록 |
| `src-tauri/capabilities/default.json` | **수정** — shell sidecar 권한 추가 |

### Rust 커맨드 목록 (`commands/ocr.rs`)

```
ocr_image(image_path: String) -> OcrResponse
    사이드카 호출, 결과를 ocr_session/ocr_result에 저장

save_ocr_correction(result_id: i64, corrected_text: String) -> ()
    사용자 교정값 업데이트

get_ocr_history(limit: i64) -> Vec<OcrSession>
    최근 OCR 세션 목록 조회

export_ocr_dataset(output_path: String) -> ()
    (image_path, raw_text, corrected_text) 전체를 JSON/CSV로 내보내기
```

### Python 사이드카

| 파일 | 내용 |
|------|------|
| `sidecar/ocr_sidecar.py` | RapidOCR 추론 메인 스크립트 |
| `sidecar/requirements.txt` | `rapidocr-onnxruntime`, `pillow` |
| `sidecar/build.bat` | PyInstaller 빌드 스크립트 (Windows) |

### ONNX 모델 파일 (별도 다운로드 필요)

```
src-tauri/resources/models/
    det.onnx   — 텍스트 영역 감지 (DB++모델)
    rec.onnx   — 문자 인식 (한국어 PP-OCRv3)
    cls.onnx   — 텍스트 방향 분류
```

> **주의**: 모델 파일은 git에 포함하지 않는다 (용량 문제).
> `.gitignore`에 `src-tauri/resources/models/*.onnx` 추가 필요.

### Vue 프론트엔드

| 파일 | 변경 |
|------|------|
| `src/views/OcrView.vue` | **신규** — OCR 전용 화면 |
| `src/stores/ocrStore.js` | **신규** — OCR Pinia 스토어 |
| `src/router/index.js` | **수정** — `/ocr` 라우트 추가 |

---

## OCR 화면 UI 흐름

```
[이미지 파일 선택 (Tauri dialog)]
    ↓
[이미지 미리보기 표시]
    ↓
[OCR 실행 버튼]
    ↓
[인식 결과 텍스트 — 편집 가능]
    ↓
    ├─ [저장]           → DB에 교정값 기록
    ├─ [레코드에 삽입]   → ActivityRecord 텍스트 필드에 삽입
    └─ [데이터셋 내보내기] → JSON/CSV 파일 저장
```

---

## 개발 순서 권장

1. `sidecar/` — Python 환경 구축 및 RapidOCR 동작 확인
2. PyInstaller로 `ocr_sidecar.exe` 빌드 및 단독 실행 테스트
3. Rust: `tauri-plugin-shell` 추가, `commands/ocr.rs` 스켈레톤 작성
4. Rust ↔ sidecar 통신 테스트 (이미지 경로 전달 → JSON 수신)
5. DB 스키마 마이그레이션 (v6)
6. 4개 Rust 커맨드 완성
7. `ocrStore.js` 작성
8. `OcrView.vue` 작성
9. 라우터 등록 및 네비게이션 연결
10. 저사양 PC에서 통합 테스트

---

## 사전 준비 사항 (개발 PC)

- Python 3.10+
- `pip install rapidocr-onnxruntime pillow pyinstaller`
- 한국어 ONNX 모델 파일 다운로드
- Rust + Tauri CLI 환경 (기존과 동일)
