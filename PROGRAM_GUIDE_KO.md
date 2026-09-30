# Kerr Geodesic Lab 프로그램 사용설명서

[English](PROGRAM_GUIDE_EN.md) · [README](README.md)

기준: v0.3.0, 코드 커밋 `3eae1ef7981580b1b23a453157bd399a91acf879`. 2026-09-30에 실제 소스, CLI 도움말, Windows GUI 및 소규모 저장·재현 실행을 대조했다. 아래 명령은 **Cargo.toml이 있는 프로젝트 루트의 PowerShell**에서 실행한다. 예시의 `simulation_1`은 새 출력 위치를 가정한다. 기존 폴더가 있다면 프로그램이 출력한 실제 번호로 바꾼다.

## 1. 프로그램 소개

고정된 Kerr 시공간에서 빛의 3차원 영측지선(null geodesic)을 계산하고, 흡수형 검출면의 교차 위치와 도착시간을 구하는 프로그램이다. GUI는 실험 설정과 궤적 탐색에, CLI batch는 여러 회전량에서 반복 데이터를 생성하는 데 사용한다. 생성 데이터를 머신러닝 연구에 사용할 수 있지만 프로그램 자체에 모델 학습 기능은 없다.

단위는 `G=c=M=1`, `chi=a/M`이다. 물리는 CPU f64 적응형 Dormand–Prince 5(4), 화면은 wgpu GPU 렌더링이다. 동적 중력장, Maxwell 장, 중력 되먹임, CUDA/GPU 광선 solver를 계산하지 않는다.

## 2. 주요 기능

- 독립적인 3차원 광선 초기조건, ZAMO 기준 초기 운동량, Kerr 영측지선 적분.
- seed 기반 층화 위치 표본, ray 단위 CPU 병렬 계산.
- DET/CAP/ESC/NUM 등 상태, 검출 좌표 `u_hit, v_hit`, 연속 BL 도착시간 `t_hit`.
- 3D 궤적·검출기·지평선·에르고면, 물리 시간 재생과 카메라 조작.
- 계량 기반 프레임 끌림, 별도 시간축을 갖는 자유낙하 격자.
- 자동 표준 저장, seed로 초기조건을 다시 생성하는 실제 재현 계산.
- 누적/시간분해 검출 이미지, CSV, PNG/APNG와 별도 궤적 export.

현재 표준 초기조건은 다음과 같다.

| 항목 | 기본값 |
|---|---|
| `generator.cell_count` | [192, 192] → 36,864 rays |
| `generator.cell_size` | [0.5, 0.5] M → 전체 96×96 M |
| `generator.center` | [80, 0, 0] M |
| `generator.axis_1 / axis_2` | [0, 1, 0] / [0, 0, 1] |
| `generator.fixed_direction` | [-1, 0, 0], CartesianSpatial, 방향 잡음 없음 |
| `generator.t_emit` | 0 |
| `experiment.spin` | 0.6 |
| 검출면 | 중심 [-80, 0, 0] M, 법선 [1, 0, 0], 300×300 M |
| 검출 이미지 | 128×128 pixels |
| 표시 궤적 | 최대 256개; 물리 계산 및 검출 통계에는 모든 광선 사용 |

셀마다 독립적인 균일 위치 하나를 뽑는다. 셀 개수를 바꾸면 셀 크기는 유지되므로 발사 영역도 달라진다. 예를 들어 16×16은 256 rays, 8×8 M이다. 이는 32×32 M 영역의 **과거 256-ray 회귀 프리셋과 다르다**.

## 3. 시스템 요구사항

검증 환경은 Windows MSVC, Rust/cargo 1.98.1, NVIDIA GeForce RTX 3060 Ti / Vulkan / DiscreteGpu다. Rust 2024 edition, eframe/egui 0.36.2와 wgpu 30.0.1을 사용한다. Linux/macOS 빌드·실행은 이번 설명서에서 확인하지 않았으므로 지원을 보장하지 않는다.

- Rust와 Cargo, Windows C++ 빌드 도구 및 Windows SDK가 필요하다. [Rust 공식 설치 안내](https://rust-lang.org/tools/install/)의 rustup 설치 절차를 따른다.
- 최초 의존성 다운로드에는 인터넷이 필요하다. Cargo.lock은 의존성 버전을 고정하며, 컴파일러 버전까지 고정하지는 않는다.
- GUI는 wgpu 호환 GPU/드라이버가 필요하다. 가능하면 NVIDIA discrete GPU를 우선 선택한다. CLI 계산에는 GUI 창이 필요 없다.
- 한글 UI는 시스템 한글 글꼴을 읽는다. Windows에서는 맑은 고딕을 찾으며 글꼴을 재배포하지 않는다.
- Python은 빌드·실행에 필수가 아니다. 광선 수와 보관 궤적에 따라 메모리·디스크 사용량이 커진다.

## 4. 설치 및 빌드

저장소가 private이면 먼저 GitHub 접근 권한을 준비한다. Rust 설치 후 새 PowerShell을 열고 실행한다.

```powershell
git clone https://github.com/cksgus0561-dot/kerr-ray-simulator.git
cd kerr-ray-simulator
rustc --version
cargo --version
cargo fetch --locked
cargo build --release --locked --bins
```

실행 파일은 `target/release/kerr-ray.exe`와 `target/release/kerr-viewer.exe`다. 소스 checkout에 기존 계산 결과, 빌드 실행 파일, 개인 toolchain/cache가 포함된다고 가정하지 않는다.

`env.ps1`은 기존 로컬 개발환경용 보조 스크립트다. 일반 사용자는 설치한 Rust를 PATH에서 사용하면 된다. `--offline`은 모든 의존성을 이미 받은 환경에서만 사용한다.

명령 도움말:

```powershell
cargo run --release --locked -- --help
.\target\release\kerr-viewer.exe --help
.\target\release\kerr-ray.exe batch-simulate --help
```


## 5. 프로그램 실행

표준 seed 기반 GUI:

```powershell
.\target\release\kerr-viewer.exe --config .\examples\seeded.json
```

Cargo를 통한 동일 진입점:

```powershell
cargo run --release --locked -- visualize --config .\examples\seeded.json
```

이 명령은 시작하자마자 새 simulation을 백그라운드에서 계산하고 자동 저장한다. 가볍게 GUI를 확인하려면:

```powershell
.\target\release\kerr-viewer.exe --config .\examples\seeded.json --grid 16 --rendered 256
```

`kerr-ray.exe visualize`는 GUI, `simulate-save`는 창 없는 단일 계산, `batch-simulate`는 창 없는 반복 계산이다. GUI에는 batch 실행 버튼이 없다.

인자 없이 `kerr-viewer.exe`를 실행하면 실행 파일 옆 `kerr-viewer.args.json`의 인자 배열을 우선 읽고, 인자가 없으면 `kerr-viewer.json`을 설정으로 읽는다. 둘 다 없으면 기본 설정으로 새 계산을 한다. 명시적 `--config` 또는 `--input`은 이 launcher 설정을 우회하므로 첫 실행에는 위 명령을 권장한다.

이미 내보낸 **시각화 결과 폴더**를 여는 예:

```powershell
.\target\release\kerr-viewer.exe --input .\results\my_visualization
```

이 폴더는 직접 생성한 export여야 한다. `--input`은 `run_metadata.json`, 이벤트와 선택적 궤적이 있는 시각화 결과를 읽는다. **common.json + sim_N.bin 폴더는 왼쪽 표준 저장 UI 또는 CLI reproduce로 연다.** 두 종류를 혼용하지 않는다.

## 6. GUI 사용법

### 화면 구성과 언어

왼쪽은 스크롤 가능한 설정, 가운데는 3D 화면, 오른쪽 **관측 정보**, 아래쪽은 광선 재생·상태 표시다. 좌우 패널 너비도 조절할 수 있다. 접힌 삼각형을 눌러 세부 설정을 펼친다. 숫자 입력은 드래그하거나 클릭 후 입력하며, Shift+드래그로 미세 조절한다.

**설정 → 언어 → 한국어 / English**로 즉시 전환한다. 언어는 실행 파일 옆 `kerr-viewer.ui.json`에 저장되어 다음 실행에 적용된다. 기본은 English다. 언어 변경은 물리 재계산, seed/hash, 파일 schema, CLI 도움말을 바꾸지 않는다. 이 파일에 쓸 수 없으면 설정 영역에 오류가 나온다. 한글 글꼴을 찾지 못하면 한국어 선택이 비활성화되고 안내가 표시된다.

### 물리 설정과 새 계산

**장면 → 시뮬레이션 (백그라운드 재계산)**에서 다음을 조절한다.

| 실제 UI 명칭 | 의미 / 기본 |
|---|---|
| `chi = a/M` | 기본 0.6; 입력 guard [-0.9999, +0.9999], 정확한 극단값 제외 |
| **층화 표본 광선 생성기** | 표준 seed 기반 광선 생성 설정 |
| **발사 영역 중심 (M)** | 기본 [80, 0, 0] |
| **단위축 1 (XYZ)** / **단위축 2 (XYZ)** | 발사 영역의 정규직교 축 |
| **셀 개수 1 / 2** | 기본 192 / 192; 직접 입력 가능. 빠른 버튼은 16, 32, 64, 128, 256 |
| **셀 크기 (M)** | 기본 0.5 / 0.5; 영역 크기는 셀 수와 곱해서 결정 |
| **고정 CartesianSpatial 방향 (잡음 없음)** | 기본 [-1, 0, 0]; 정규화 후 기존 ZAMO 변환 사용 |
| `t_emit` | 공통 BL 발사시간, 기본 0 |
| **검출면 DetectorPlane** | **중심 (좌표 단위 M)**, **법선 (XYZ)**, **국소 v / 위쪽 방향 (XYZ)**, **너비 / 높이** |
| **기준 계산 정확도** | `rtol`, `atol`, `r_escape` |

축은 단위 길이이며 서로 직교해야 한다. 방향은 0벡터일 수 없다. CartesianSpatial 방향은 BL의 `dr/dtheta/dphi` 입력이 아니다.

물리 입력을 바꾸고 450 ms 동안 입력이 없으면 새 계산이 자동 제출된다. **현재 물리 설정으로 다시 계산**도 새 계산을 시작한다. 표준 경로에서는 매번 새 seed를 사용하므로 같은 설정으로 이 버튼을 눌러도 동일 표본을 재현하는 것은 아니다. 편집 사이에 오래 쉬면 계산·자동 저장이 여러 번 발생할 수 있다.

현재 표준 수치 기본값:

| 설정 | 값 |
|---|---|
| `rtol / atol` | 1e-12 / 1e-14 |
| `initial_step / min_step / max_step` | 0.05 / 1e-12 / 0.5 |
| `max_steps / max_affine` | 200000 / 1500 |
| `horizon_epsilon / escape_radius` | 1e-3 / 400 |
| `null_tolerance / detector_root_tolerance` | 1e-5 / 1e-10 |

GUI에 없는 수치 항목은 완전한 설정 JSON에서 지정한다. 아주 작은 수는 GUI 숫자 표시에서 자릿수가 생략될 수 있으므로 정확한 값은 설정 JSON을 확인한다. 허용 spin 범위 자체가 모든 초기조건의 수치 안정성을 보증하지는 않는다.

기존 JSON/프리셋에서는 **광원면 SourcePlane (기존 프리셋)**이 나타난다. 여기에는 광선 격자 X/Y와 국소 ZAMO 방향 선택 등이 있다. 표준 층화 generator와 다른 초기조건 경로다.

### 재계산과 정지의 구분

광선 계산은 별도 worker에서 실행되어 기존 장면과 카메라를 유지한다. 새 요청은 generation ID로 이전 요청을 대체하며 늦은 결과가 최신 장면을 덮어쓰지 않는다. 카메라·언어·표시 개수·선 두께·검출 이미지 해상도·시간 bin·재생 설정 변경은 광선 계산을 다시 시작하지 않는다.

**정지**와 **시간 초기화**는 재생만 처음으로 되돌리고 일시정지한다. **물리 계산을 취소하는 전용 버튼은 없다.** 새 물리 요청이 이전 요청을 대체할 수 있지만 진행 중인 광선 작업은 완료 경계에서 취소된다. 창을 강제로 닫는 것은 안전한 저장/재개 기능이 아니다.

### 3D 표시와 카메라

**장면 → 표시 항목**:

| 항목 | 표시 내용 |
|---|---|
| **Cartesian 기준 좌표격자**, **XYZ 좌표축** | 직선 위치 기준; 중력장이나 휘어진 공간 자체가 아님 |
| **Kerr 회전축 (+Z)** | Kerr 중심과 축 방향 |
| **사건의 지평선**, **에르고면** | 현재 chi의 Kerr 식으로 얻은 표면 |
| **광원면 SourcePlane**, **광선 발사점** | 발사 영역과 실제 초기 위치 |
| **검출면 DetectorPlane**, **검출기 도달점** | 검출면과 실제 hit |
| **광선 궤적**, **현재 광선 위치** | 계산된 경로와 재생 시각의 위치 |
| **프레임 끌림** | 계량 기반 각속도 화살표 |
| **3D 검출기 오버레이**, **검출기 이미지 패널** | 평면 위 분포 / 오른쪽 이미지 |

**표시 설정 (물리 재계산 없음)**에서 **표시할 궤적 수 (물리 광선 수는 유지)**를 조절한다. 기본 256에서 다른 값으로 바꾸면 표시 subset만 달라진다. 모든 물리 광선과 검출 결과는 유지한다. **선 두께 (px)**는 기본 1.35다. **좌표격자 자동 맞춤**을 끄면 **범위**, **간격**을 지정한다. 축마다 최대 40개 격자 구간으로 제한되므로 실제 간격은 오른쪽 진단을 확인한다.

카메라는 왼쪽 드래그로 회전, 오른쪽/가운데 드래그 또는 **이동 드래그**를 켠 왼쪽 드래그로 이동, 휠로 확대·축소한다. 키보드는 방향키 회전, Shift+방향키 이동, +/- 확대·축소다. 텍스트 입력 중에는 키보드 카메라 조작이 적용되지 않는다.

- **장면에 맞추기**: 표시 장면에 맞춤.
- **카메라 초기화**: 기본 카메라로 돌아가 장면에 맞춤.
- **Kerr 중심**, **광원**, **검출기**: 해당 대상을 중심으로 시점 변경.
- 오른쪽 **광선 ID → 선택 / 선택 해제**, 또는 화면의 표시 광선 클릭: 해당 광선 강조. 상태·초기 위치/방향·hit·스텝·실패 이유를 확인한다. 없는 검출값은 만들지 않는다.

카메라 조작은 궤적 buffer를 다시 만들지 않는다. 표시 subset/선택 또는 계산 결과가 바뀔 때 필요한 GPU geometry만 갱신한다.

### 좌표와 중력장 표시의 의미

화면은 오른손 Cartesian-like BL 공간 좌표이며 원점은 Kerr 중심, +Z는 회전축이다.

`X=sqrt(r²+a²) sin(theta) cos(phi)`, `Y=sqrt(r²+a²) sin(theta) sin(phi)`, `Z=r cos(theta)`, `T=t_BL`.

유클리드 공간의 임베딩이나 Kerr-Schild 시간 좌표가 아니다. 지평선은 `r+=1+sqrt(1-a²)`, 에르고면은 `r=1+sqrt(1-a² cos²(theta))`를 이 좌표로 표시한다.

프레임 끌림은 실제 `omega=-g_tphi/g_phiphi`에 기반한다. 화살표는 `omega*(-Y,X,0)*표시 시간 간격`이다. **장 표본 수 / 축**(기본 15), **3D 장 표시 (해제 시 z=0 단면)**, **장 표시 범위**(기본 12), **화살표 시간 간격 M**(기본 12)로 조절한다. 이는 좌표 각속도이며 광선에 추가하는 힘이 아니다.

### 자유낙하 격자

**Kerr 자유낙하 격자 (시각화 전용) → Kerr 자유낙하 격자**를 켠다. **범위** 기본 10, **격자점 / 축** 기본 9, **격자 M / 실제 초** 기본 3이다. 캐시가 준비되면 **격자 재생 / 격자 일시정지**, **격자 초기화**를 사용한다. Kerr 중심으로 확대하면 보기 쉽다.

E=1, Lz=0 자유낙하 관측자의 각 격자점을 이동시키고 살아 있는 이웃 점을 직선으로 연결한다. 모서리 내부 전체를 별도로 적분하는 매끄러운 곡선 격자는 아니다. 광선 재생과 별도인 `t_BL/M=0..80` 시계를 사용하며 숨기면 시간이 진행하지 않는다. 회전축과 지평선의 수치 여유 구간 0.02 M는 제외한다. **자유낙하 계량 진단**에서 유효 격자점과 계량 검사값을 볼 수 있다. 이 설정은 광선 solver를 재실행하지 않는다.

### 검출기와 광선 재생

**검출기 / 재생**의 첫 목록:

| 메뉴 항목 | 동작 |
|---|---|
| **전체 궤적** | 전체 경로 정적 표시 |
| **광선 전파** | 저장된 BL 시간까지 궤적 표시, 표본 사이 선형 보간 |
| **검출기 재생** | 전체 경로를 유지하며 시간에 맞는 위치/검출 표시 사용 |

두 번째 목록은 검출기 필터다.

| 메뉴 항목 | 동작 |
|---|---|
| **모든 검출 이벤트 누적** (선택 후 **전체 누적**) | 시간 슬라이더와 무관하게 모든 hit |
| **현재 시간 bin** (선택 후 **현재 시간 구간**) | 현재 시간 구간에 도착한 hit |
| **현재 시각까지 누적** | 현재 시간까지 도착한 hit |

검출기 시간 변화를 보려면 전체 누적 대신 시간 bin 또는 현재 시각까지 누적을 선택한다. **재생 / 일시정지**, **정지**, **시간 초기화**, 아래 `t_BL / M` 슬라이더를 사용한다. **물리 M / 실제 초**(기본 25)는 화면 재생 속도다. FPS를 물리 시간으로 해석하지 않는다. 주 시계는 절대 `t_hit`; `delta_t=t_hit-t_emit` 분포는 별도 rebin CLI에서 지원한다.

`Delta t` 기본 1은 bin 너비다. **픽셀 수 X / Y**는 기본 128 / 128이며 GUI에서 각 축 1..1024로 바꾼다. **흰색 기준 개수** 기본 4는 밝기 척도다. 이들은 저장된 모든 연속 이벤트를 후처리하며 재적분하지 않는다. 이미지에서 u는 오른쪽, v는 위쪽이다. `I(u,v)`는 픽셀별 도달 개수이며 복사 전달로 계산한 절대 광도가 아니다.

### 자동 저장·불러오기·재현

**표준 시뮬레이션 저장 데이터**:

1. **자동 저장 위치 (새 계산에만 적용)** 기본은 `results/standard`다.
2. 새 계산 완료 후 별도의 Save 없이 새 `simulation_N/common.json + sim_1.bin`을 저장한다.
3. **시뮬레이션 폴더 경로**에 해당 폴더를 입력하고 **폴더 불러오기**를 누른다.
4. 목록에서 `sim_N.bin`을 고른다. 여기까지는 저장 요약을 읽으며 현재 장면은 유지한다.
5. **선택한 시뮬레이션 재현 계산**을 누르면 seed로 재계산하고 새 궤적과 **재현 검증 PASS/FAIL**을 표시한다. 이 재현은 새 archive를 자동 저장하지 않는다.
6. **전체 비교 결과**에서 hash, 상태, hit 오차, 소스/버전 일치 등을 확인한다.

이전 Parquet archive도 읽을 수 있다. SourcePlane 정보가 없는 과거 형식은 광원 표시/focus가 제한되며 초기조건 자체를 임의로 만들지 않는다.

### 내보내기와 진단

**내보내기 (새 파일명만 사용)**의 기본 경로는 `results/visualization_export`다. 표준 자동 저장 경로와 별개다.

| 버튼 | 생성 파일 / 용도 |
|---|---|
| **화면 캡처** | `screenshot.png`, GUI 포함 현재 화면 |
| **시각화 설정 저장** | `visualization.json`, 공유 SessionConfig |
| **현재 검출기 이미지 + CSV 내보내기** | `detector_view.png`, `detector_view.csv` |
| **전체 실행 결과 + 프레임 시퀀스 저장** | 별도 시각화 export, 궤적·이벤트·PNG/APNG |

같은 파일은 덮어쓰지 않는다. 특히 전체 export는 **새 빈 폴더**가 필요하다. 먼저 screenshot/config를 넣은 폴더에 전체 export를 이어서 저장하지 않는다. **궤적 내보내기 간격**은 매 N번째 적분 표본과 양 끝점을 저장한다. 기본 1이며 메모리의 원본 표본이나 검출 결과를 바꾸지 않는다. 프레임 시퀀스는 **검출기 이미지**이며 3D viewport 영상 녹화 기능은 아니다.

**관측 정보**에서 GPU/backend, FPS, 물리 광선/표시 궤적 수, 상태 개수와 계산시간을 확인한다. **보존량 / GPU 캐시**는 null/E/Lz/Q 오차, 업로드 횟수, 실제 격자 간격, omega, p95 프레임 시간을 표시한다. 계산/내보내기 오류는 하단 상태, 자동 저장 오류는 저장 패널, 재현 오류는 해당 패널에 나타난다.

## 7. 단일 시뮬레이션 실행

GUI는 5절의 명령으로 시작한다. 계산이 끝나면 상태 개수와 **자동 저장됨** 메시지를 확인한다. CLI에서는:

```powershell
.\target\release\kerr-ray.exe simulate-save --config .\examples\seeded.json --output .\results\standard
```

`--config`를 생략하면 현재 표준 설정을 사용한다. `--output` 기본은 `results/standard`다. 가벼운 저장 확인:

```powershell
.\target\release\kerr-ray.exe simulate-save --grid 16 --output .\results\single_smoke
```

`--grid N`은 두 축 셀 수를 N으로 지정한다. `--preset regression`은 기존 SourcePlane 회귀 경로 및 Parquet 저장을 선택하므로 표준 seed 기반 BIN 실험과 혼동하지 않는다.

`simulate-save`에는 `--seed`나 `--chi` 옵션이 없다. spin 변경은 GUI 또는 완전한 SessionConfig JSON의 `experiment.spin`을 사용한다. 새 설정 작성은 **시각화 설정 저장**으로 완전한 JSON을 만든 뒤 복사본을 편집하는 방법이 안전하다. `examples/seeded.json`은 generator를 명시하고 나머지는 기본값으로 채운다. 기존 JSON에서 generator를 빠뜨리면 legacy 경로가 될 수 있다.

설정 파일은 새 실험을 정의한다. 표준 저장 폴더의 `common.json`은 재현 기록이므로 일반 `--config` 파일로 사용하거나 덮어쓰지 않는다. 코드/GUI/CLI JSON은 [src/session.rs](src/session.rs)의 SessionConfig를 공유한다.

## 8. 자동 batch simulation 실행

### 필수 옵션과 chi grid

정확한 형태는 다음과 같다.

`batch-simulate --simulations-per-chi N [--config SESSION_JSON] [--output ROOT] [--chi-indices -76,0,76] [--dry-run]`

| 옵션 | 의미 |
|---|---|
| `--simulations-per-chi N` | 필수 양의 정수. 선택한 **각 chi마다** N회 실행. 기본값 없음 |
| `--config SESSION_JSON` | generator/detector/numerics를 공유하는 seed 기반 설정. 생략 시 표준값 |
| `--output ROOT` | 기본 `results/standard` |
| `--chi-indices` | -76..76의 오름차순·중복 없는 정수 index 목록. 쉼표 사이 공백 없이 입력 |
| `--dry-run` | 계획만 출력. seed 생성·광선 계산·결과 저장 없음 |

기본 grid는 index `i=-76..76`의 153개다.

`s_i = i * atanh(0.999) / 76`, `chi_i = tanh(s_i)`.

코드는 양 끝값을 정확히 -0.999, +0.999로 지정하고 0과 부호 대칭을 포함한다. chi 자체가 균일 간격인 것은 아니다. `--chi-indices 0`은 chi=0 하나, `-76,0,76`은 -0.999, 0, +0.999다. batch에서 설정 파일의 단일 `experiment.spin` 대신 이 grid가 회전량을 결정한다.

전체 grid에서 N=1이면 **153 simulation**, N=10이면 **1530 simulation**이다. subset이면 `선택 index 수 × N`이다. 먼저 계획을 확인한다.

```powershell
.\target\release\kerr-ray.exe batch-simulate --simulations-per-chi 1 --dry-run
.\target\release\kerr-ray.exe batch-simulate --simulations-per-chi 10 --dry-run
```

전체 grid를 실제 계산하려는 경우:

```powershell
.\target\release\kerr-ray.exe batch-simulate --config .\examples\seeded.json --simulations-per-chi 1 --output .\results\batch
```

N=10은 위 명령의 `--simulations-per-chi 1`을 `--simulations-per-chi 10`으로 바꾼다. 설명서 작성 검증에서는 이 전체 계산을 실행하지 않고 dry-run으로 실행 수를 확인했다.

### 작은 실행 예제

다음은 원본 예제를 바꾸지 않고 임시 설정만 16×16으로 만든다. **chi=0에서 256 rays를 2회**, 새 seed로 계산한다.

```powershell
New-Item -ItemType Directory -Force .\work\guide-smoke | Out-Null
$smokeConfig = Get-Content .\examples\seeded.json -Raw | ConvertFrom-Json
$smokeConfig.generator.cell_count = @(16, 16)
$smokeJson = $smokeConfig | ConvertTo-Json -Depth 20
$smokePath = Join-Path (Get-Location).Path 'work/guide-smoke/session.json'
[System.IO.File]::WriteAllText($smokePath, $smokeJson, [System.Text.UTF8Encoding]::new($false))
.\target\release\kerr-ray.exe batch-simulate --config .\work\guide-smoke\session.json --simulations-per-chi 2 --chi-indices 0 --output .\results\batch_smoke
```

batch에는 `--grid` 옵션이 없으므로 작은 셀 수는 JSON에 넣는다. 표준 cell size 0.5 M를 유지해 이 예제의 발사 영역은 8×8 M다.

한 batch 전체가 새 `simulation_N` 폴더 하나를 사용한다. `common.json`은 한 번만 저장하며, chi index 오름차순으로 각 chi의 반복을 끝낸 뒤 다음 chi로 이동한다. 파일은 `sim_1.bin, sim_2.bin, ...` 순서다. simulation별 계산은 순차적이고, 각 simulation의 ray들은 CPU 병렬 실행된다.

마지막 `batch saved: ...; completed 2 / 2`를 확인하고 출력된 폴더를 사용한다.

```powershell
.\target\release\kerr-ray.exe inspect-simulation --input .\results\batch_smoke\simulation_1 --sim sim_2.bin
.\target\release\kerr-ray.exe reproduce --input .\results\batch_smoke\simulation_1 --sim sim_2.bin
```

**실제 확인:** 위와 같은 작은 설정으로 파일 2개와 common.json 생성, 번호 순서, 독립 seed, 정상 종료, sim_2.bin 재현 PASS를 확인했다. 기록된 ACT/DET/CAP/ESC/NUM은 각각 [0,0,252,4,0], [0,1,251,4,0]이었다. 새 seed이므로 다음 실행의 개수는 달라도 된다.

## 9. 결과 파일 구조

### 표준 archive

기본 출력 위치는 현재 작업 디렉터리 기준이다. 단일 실행은 새 폴더에 파일 하나, batch는 새 폴더 하나에 여러 파일을 저장한다.

```text
results/standard/
  simulation_1/
    common.json
    sim_1.bin
  simulation_2/
    common.json
    sim_1.bin

results/batch/
  simulation_1/
    common.json
    sim_1.bin
    sim_2.bin
    ...
```

`simulation_N`은 사용하지 않은 첫 양의 정수 번호를 원자적으로 예약한다. 기존 폴더/파일은 덮어쓰지 않는다. 중간 번호가 비어 있으면 그 번호를 사용할 수 있다.

**common.json**에는 다음이 들어간다.

- `numerics`: 적분 오차·스텝·최대 affine/step·capture/escape·null·검출 교차 허용오차.
- `detector`: 중심, 법선, 국소 축, 물리적 너비/높이. 이미지 픽셀 해상도는 포함하지 않는다.
- `convention`: 단위, 좌표계, 회전축, 부호 규약, local ZAMO energy=1.
- `simulator_version`, `build_source_fnv1a64`, `data_format_version="2-seeded-bin"`.
- `ray_generator`: 생성기/RNG 버전과 규칙, 영역·셀·방향·발사시간.
- `binary_format`: 바이트 순서, layout, status code, hit 열과 hash 규칙.
- batch인 경우 `batch`: chi sampling 규칙·범위·전체 점 수, `simulations_per_chi`, 선택 `chi_indices`.

chi는 simulation별 BIN에 있다. 언어, 카메라, 표시 개수, 검출 pixel/bin 설정은 표준 재현 기록이 아니다.

**sim_N.bin**은 little-endian raw binary다. N은 전체 ray 수, K는 DET 수다.

| 시작 offset (byte) | 필드 | 자료형 / 크기 |
|---|---|---|
| 0 | chi | f64 / 8 |
| 8 | seed | uint8[32] / 32 |
| 40 | initial_conditions_hash | SHA-256 / 32 |
| 72 | status | uint8[N] / N |
| 72+N | DET hit triples | K×[u_hit,v_hit,t_hit], f64 / 24K |

총 크기는 `72 + N + 24K` bytes다. 별도 header/trailer/padding/압축이 없다. status는 정준 광선 순서이고 hit은 그 순서에서 DET인 광선에만 저장된다. non-DET에는 hit triple이 없다. **현재 BIN에는 ray_id, 원시 초기 좌표·방향, energy 열, 전체 trajectory, 이미지가 저장되지 않는다.** 초기조건과 ID는 seed+generator로 복원한다. 과거 Parquet schema와 구분한다.

### 시각화 export

전체 실행 결과 export는 표준 BIN과 별개로 다음 자료를 저장한다.

```text
my_visualization/
  visualization.json
  run_started.json
  run_metadata.json
  scene_metadata.json
  ray_diagnostics.csv
  detector_hit_states.csv
  detector_events.csv
  trajectory_samples.csv.gz
  detector_accumulated.csv
  detector_accumulated.png
  detector_before_window.csv
  detector_time_bins.csv
  frame_index.csv
  derived_metadata.json
  detector_instantaneous.apng
  detector_cumulative.apng
  detector_frames/
    instantaneous/frame_000000.png
    cumulative/frame_000000.png
    ...
```

`detector_events.csv`는 ray_id, source u/v, t_emit, u_hit, v_hit, t_hit, delta_t, status를 포함한다. `trajectory_samples.csv.gz`의 열은 `ray_id, affine, t, r, theta, phi, p_t, p_r, p_theta, p_phi`다. f64 BL 표본을 손실 없는 gzip으로 보관하며 화면에서 Cartesian-like 좌표로 변환한다.

시간 간격과 pixel 해상도만 바꾸려면 이 **이벤트 export 폴더**에 대해:

```powershell
.\target\release\kerr-ray.exe rebin --input .\results\my_visualization --output .\results\rebinned --dt 0.1 --resolution 256x256 --fps 30
```

이 명령은 측지선을 다시 계산하지 않는다. 출력 폴더는 새 위치를 사용한다. 표준 BIN 폴더는 이 rebin 명령의 입력이 아니다. 필요하면 먼저 GUI에서 재현 후 전체 export를 한다.

## 10. seed와 재현 방법

새 표준 simulation마다 OS에서 독립적인 256-bit seed를 생성한다. chi나 실행 번호를 seed로 사용하는 방식이 아니다. ChaCha20 (`rand_chacha 0.9.0`)이 같은 seed와 generator 설정에서 같은 cell 위치 표본을 재생성한다.

`prepare_rays`는 유한값 검사, 방향 정규화와 signed zero 정규화를 거쳐 위치 XYZ, 방향 XYZ, t_emit의 결정론적 정렬을 수행한다. 이후 내부 ray_id=0..N-1을 부여한다. BIN에서 과거 ray_id를 주입하지 않는다.

`initial_conditions_hash`는 정렬된 각 광선의 위치 XYZ, 정규화된 방향 XYZ, t_emit을 f64 little-endian으로 이어 붙인 SHA-256이다. 저장·읽기·재현에서 확인하며 불일치하면 **측지선 적분을 시작하지 않고 오류로 종료**한다.

재현 절차:

1. 원본 `common.json`과 원하는 `sim_N.bin`을 같은 폴더에 보존한다.
2. inspect로 읽기·초기조건 hash 검증·저장 상태 요약을 확인한다.
3. reproduce로 초기조건 생성부터 실제 Kerr 적분까지 다시 실행한다.
4. `result: PASS`와 상세 차이를 확인한다.

```powershell
.\target\release\kerr-ray.exe inspect-simulation --input .\results\standard\simulation_1 --sim sim_1.bin
.\target\release\kerr-ray.exe reproduce --input .\results\standard\simulation_1 --sim sim_1.bin
```

예제 폴더 번호는 실제 출력에 맞춘다. `--sim` 생략 시 목록의 첫 파일을 쓰지만 재현 대상을 명시하는 편이 좋다. 동일 seed 실행을 위한 현재 사용자 인터페이스는 이 **reproduce** 경로다. `simulate-save`를 다시 실행하면 새 seed를 사용한다.

PASS는 광선 수, canonical 순서/ID, 복원 초기 State와 solver 시작 상태, 상태·hit 유무·개수 및 hit 허용오차를 검사한다. hit 기준은
`abs(error) <= 1e-9 + 1e-12 * max(abs(original),abs(new))`다.
hit bit mismatch 및 동일 소스/버전 여부도 보고하지만 그것 자체를 PASS 조건으로 삼지는 않는다. 따라서 PASS를 모든 환경에서의 bitwise 동일성 보증으로 읽지 않는다. 과거의 초기 Kerr State 전체가 파일에 따로 저장되는 것도 아니다.

## 11. CPU 병렬화

기본 worker 수는 Rust의 `available_parallelism()` 결과에서 1을 뺀 값, 최소 1이다. 실제 작업 thread 수는 광선 수로도 제한한다. `KERR_RAY_THREADS`를 양의 정수로 지정하면 덮어쓴다. 1은 직렬 계산, 0이나 잘못된 값은 오류다.

8절의 작은 설정/결과를 만든 뒤 다음처럼 실행할 수 있다.

```powershell
$env:KERR_RAY_THREADS = "4"
.\target\release\kerr-ray.exe batch-simulate --config .\work\guide-smoke\session.json --simulations-per-chi 2 --chi-indices 0 --output .\results\batch_threads
$env:KERR_RAY_THREADS = "1"
.\target\release\kerr-ray.exe reproduce --input .\results\batch_smoke\simulation_1 --sim sim_2.bin
Remove-Item Env:KERR_RAY_THREADS -ErrorAction SilentlyContinue
```

환경변수는 해당 PowerShell에서 시작한 프로그램에 적용된다. 결과는 작업 완료 순서가 아니라 정준 ray 순서로 조립한다. GPU는 렌더링용이며 이 설정으로 GPU 계산이 활성화되지는 않는다. batch는 simulation들을 동시에 여러 개 실행하는 방식이 아니다.

## 12. status 설명

| BIN code | 약자 / 내부 enum | 의미 | 궤적 색 |
|---|---|---|---|
| 0 | ACT / Active | 아직 capture/escape/detection에 도달하지 않음. 최대 affine까지 계산하고 남을 수 있음 | 회색 |
| 1 | DET / Detected | 흡수형 검출면의 유효 경계 안에 최초 도달하여 종료 | 청록색 |
| 2 | CAP / Captured | r <= r+ + horizon_epsilon의 수치 종료 경계 도달 | 주황색 |
| 3 | ESC / Escaped | r >= escape_radius이며 바깥으로 진행 | 보라색 |
| 4 | NUM / NumericalFailure | NaN, null 조건 위반, 스텝 한계 또는 적분 실패 등 | 붉은색 |

최대 step 초과를 ESC로 취급하지 않는다. CAP는 지평선 내부를 적분했다는 뜻이 아니다. NUM은 물리적 광선 분류로 정상화하지 말고 진단을 확인한다. batch는 NUM을 결과로 보관할 수 있으므로 명령이 성공했다는 사실과 모든 ray의 수치 성공을 구분한다. 선택한 광선은 강조색으로 바뀐다.

## 13. 문제 해결 / 주의사항

| 증상 | 확인할 내용 |
|---|---|
| cargo를 찾지 못함 / linker 오류 | Rust PATH, 새 PowerShell, Windows C++ 도구와 SDK 설치 |
| GPU 초기화 실패 | wgpu 호환 드라이버 및 콘솔의 adapter 메시지. NVIDIA 선택은 사용 가능한 장치에 따라 달라짐 |
| 한글 선택 불가 / 언어 유지 실패 | 시스템 한글 글꼴, 실행 파일 옆 UI 설정 파일의 읽기/쓰기 권한 |
| 자동 저장 실패 | 현재 작업 디렉터리와 **자동 저장 위치 (새 계산에만 적용)**의 쓰기 권한. 계산 결과는 메모리에 남지만 저장 성공은 아님 |
| 파일/폴더가 이미 있음 | export에는 새 빈 경로 사용. 표준 archive는 자동으로 새 simulation_N을 예약 |
| BIN을 --input으로 열 수 없음 | **폴더 불러오기 → 선택한 시뮬레이션 재현 계산** 또는 CLI reproduce 사용 |
| 궤적 데이터 없음 | 과거 이벤트 전용 시각화 결과는 detector 데이터를 유지하지만 궤적을 만들지 않음 |
| 재생해도 검출 이미지가 같음 | 전체 누적 대신 현재 시간 bin/현재 시각까지 누적 선택 |
| 화면에 광선이 적음 | 표시 subset과 physical ray 수를 구분. 표시 항목과 현재 재생 시각 확인 |
| hash mismatch / 재현 FAIL | common.json과 BIN 조합, 생성기/소스 버전, 파일 손상과 상세 비교 결과 확인. 기준을 완화해서 통과시키지 않음 |
| batch 필수 옵션 오류 | `--simulations-per-chi N`은 필수이며 양수. index는 chi 실수가 아니라 -76..76 정수 |
| 기대하지 않은 시작 설정 | 명시적 --config 사용; launcher sidecar와 과거 legacy JSON 여부 확인 |

물리 좌표는 BL 지평선에서 좌표 특이성을 가지며 내부로 적분하지 않는다. 자유낙하 격자나 직선 좌표격자를 실제 공간의 모양으로 해석하지 않는다.

강제 종료/Ctrl+C 또는 I/O 오류 후에는 이미 완료한 파일과 불완전한 파일/폴더가 남을 수 있다. 폴더 존재만으로 batch 완료를 판단하지 않는다. 완료 로그와 inspect 검증을 확인한다. batch 자동 재개 기능은 없다. 다른 output root로 다시 실행하면 새 seed를 사용한다.

검증 명령:

```powershell
cargo fmt --check
cargo check --locked
cargo test --locked --no-fail-fast
cargo clippy --locked -- -D warnings
cargo build --release --locked --bins
```

이번 기준에서는 **148개 중 146개 통과, 기존 실패 2개**다. [tests/ray_initial_conditions.rs](tests/ray_initial_conditions.rs)의 다음 과거 bitwise 회귀 실패는 문서 작성에서 수정하지 않았다.

- `regression_256_all_initial_states_match_legacy_and_saved_baseline`
- `regression_4096_all_initial_states_match_legacy_and_saved_baseline`

fmt/check/clippy/release build는 통과했다. CLI 도움말, 작은 단일 계산·batch 및 재현, 실제 GUI 한/영 전환·자동 저장·폴더 선택·재현 PASS·자유낙하 격자 표시/시간 진행을 확인했다. 언어 sidecar의 저장과 읽기 구현도 대조했다. 이번 작업이 모든 GUI 입력, 전체 153-point production 데이터, 다른 OS 또는 모든 초기조건의 새 물리 검증을 의미하지는 않는다.

## 14. 빠른 시작 예제

**GUI**

1. 3절대로 Rust/Windows 빌드 도구를 설치한다.
2. 4절의 clone 및 release build를 실행한다.
3. 5절의 `--config .\examples\seeded.json` 명령으로 GUI를 연다.
4. **설정 → 언어**에서 언어를 선택한다.
5. 자동으로 시작한 기본 계산이 끝나면 물리 광선 36,864개, 표시 궤적 최대 256개와 자동 저장 메시지를 확인한다.
6. **광선 전파**, **현재 시각까지 누적**, **시간 초기화 → 재생**으로 탐색한다.
7. 같은 계산을 다시 확인하려면 10절의 reproduce를 사용한다.

**Batch**

1. 8절의 dry-run으로 N=1 → 153, N=10 → 1530인지 확인한다.
2. 같은 절의 작은 16×16 / chi index 0 / 반복 2 예제를 먼저 실행한다.
3. 생성된 `common.json, sim_1.bin, sim_2.bin`을 확인하고 sim_2.bin을 재현한다.
4. 전체 데이터 생성이 필요할 때만 표준 generator와 전체 grid 명령을 실행한다.

더 자세한 기준은 [SEEDED_RUN.md](SEEDED_RUN.md), [BATCH_SIMULATION.md](BATCH_SIMULATION.md), [CPU_PARALLEL.md](CPU_PARALLEL.md), [VISUALIZATION.md](VISUALIZATION.md)에 있다. 실제 설정/실행 근거는 [src/session.rs](src/session.rs), [src/experiments/stratified.rs](src/experiments/stratified.rs), [src/standard_run/cli.rs](src/standard_run/cli.rs), [src/standard_run/batch_cli.rs](src/standard_run/batch_cli.rs), [src/standard_run/seeded.rs](src/standard_run/seeded.rs), [src/visualization/ui.rs](src/visualization/ui.rs)에서 확인할 수 있다.
