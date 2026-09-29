# 최종 보고 — Kerr 3D 시각화 v0.3

2026-09-15. 이번 최종 점검에서는 **시뮬레이터 소스·물리 엔진·저장 결과를 변경하지 않았다.**
새 기능, 시험 재실행, 무거운 simulation 재계산 없이 기존 실행 로그·코드·데이터 검사·실제 창의 캡처를 대조했다.
수정한 파일은 이 보고서, BENCHMARK.md, COMPLETION_CHECKLIST.md뿐이다. 이전 v0.2 문서는 validation/history/V02_*에 보존되어 있다.

**구현과 전체 GUI 조작 검증은 구분한다.** 실행·저장 재사용·성능 측정은 완료했지만,
일부 카메라/재생 버튼의 최종 수동 조작 검증은 제한이 있어 시각화 단계 전체를 무조건 완료라고 선언하지 않는다.

## A. 완료된 기능 — 완료 / 일부 검증 제한 있음

- CPU f64 Kerr 영측지선·ZAMO·Dormand–Prince·detector 엔진을 재사용하는 wgpu + egui 프로그램.
- 직선 3D 좌표격자, 자동 범위와 수동 extent/spacing, 좌표축, +Z 회전축, Kerr 중심.
- 현재 spin의 정확한 지평선 반지름과 에르고면 함수에서 만드는 geometry.
- SourcePlane/DetectorPlane의 실제 위치·경계·기저·normal, 발사점과 detector hit.
- 실제 저장된 3D 경로, 상태별 색, 선택 ray 강조 및 있는 관측값만 표시.
- static/propagation/detector playback, 누적·시간 bin·cumulative 검출 분포.
- 계량 기반 frame dragging, 밀도와 단면/3D sampling 설정.
- 표시량과 물리량 분리, 공유 SessionConfig, worker와 generation ID, GPU batch/cache.
- 저장 run 로드, 신규 계산 결과 저장, screenshot·detector PNG/CSV·설정 export 경로.

핵심 logic과 저장은 자동시험을 통과했다. 모든 GUI 버튼·설정 조합의 수동 시험이 완료된 것은 아니다.

## B. 실제 GUI 검증 결과 — 제한 있음

| 항목 | 상태 | 실제 확인 범위 |
|---|---|---|
| wgpu + egui 창 / 3D scene | 완료 | RTX 실제 창과 scene PNG. grid, 축, 면, 발사점, 경로, 지평선·에르고면 표시 |
| 저장 4096-ray run | 완료 | 저장 f64 trajectory 로드, physical 4096 / rendered 256, 상태 count 표시. 재적분 필요 없음 |
| trajectory 없는 과거 run | 완료 | physical 256 / rendered 0, 216 detector hits와 누적 이미지, 노란 TRAJECTORY UNAVAILABLE 표시 |
| 경로 subset 변경 | 완료 | GUI에서 표시 수를 바꿔도 physical count와 detector count 유지; 256/512 benchmark 완료 |
| ray 선택 | 완료 | 선택 경로 강조, 선택 ray 상태와 hit 없는 경우 없음 표시 확인 |
| frame dragging | 완료 | 실제 계량 화살표 표시와 ON/OFF 측정. chi=0 및 유한성은 자동시험으로 확인 |
| Play / 시간에 따른 전파 | 완료 | 저장 t 기준 시간이 증가하고 경로와 현재 위치가 진행하는 실제 창 확인 |
| cumulative detector | 완료 | t_BL=210.950에서 shown 3276 / total 3502를 실제 창에서 확인 |
| accumulated detector | 완료 | 과거 run shown 216 / total 216 및 전체 분포 확인 |
| instantaneous bin logic | 완료 | 기존 f64 bin 결과와 자동시험 일치; GUI 버튼 조작의 최종 확인과 구분 |
| Pause / Stop / Reset time / time slider / 속도 변경 | 제한 있음 | 구현·playback logic 시험은 있음. 각 버튼과 slider를 연속 조작한 최종 수동 확인은 미완료 |
| orbit / pan / wheel zoom / reset / focus / fit | 제한 있음 | 움직이는 camera의 실측, Kerr focus·zoom·초기 fit 확인. mouse orbit/pan과 reset·Source/Detector focus 버튼 전수 확인은 미완료 |
| screenshot / detector/config export | 제한 있음 | 실제 전체 창 PNG 및 detector 출력·저장 roundtrip은 확인. export panel의 모든 버튼 수동 조작은 미완료 |
| async 재계산과 stale 결과 차단 | 완료 | worker/generation/cancel 자동시험, 실제 계산/표시 연결과 physics key 분리 확인; 모든 drag 조합의 수동 stress test는 아님 |

마지막 추가 조작 시에는 열린 viewer를 전면 활성화하지 못했고, 캡처도 해당 창의 유효한 화면이 아니었다.
따라서 그 상태에 클릭을 보내지 않았으며 미확인 조작을 성공으로 기록하지 않았다.

실제 증거:
[4096 scene](validation/viz_4096/scene.png),
[frame dragging](validation/interactive_frame_dragging.png),
[cumulative playback](validation/interactive_cumulative.png),
[legacy trajectory unavailable](validation/viz_legacy/scene.png).

## C. GPU / backend — 완료

**NVIDIA GeForce RTX 3060 Ti / Vulkan / DiscreteGpu**, vendor **0x10de**.
실제 surface-compatible 어댑터 중 discrete NVIDIA 우선 선택을 확인했다.
features/limits 원문은 [실측 JSON](validation/viz_4096/render_measurements.json)에 있다.
CPU는 f64 물리 계산, GPU는 렌더링을 담당한다. CUDA 또는 geodesic compute shader는 추가하지 않았다.

## D. physical rays / rendered trajectories — 완료

연구 기본값 **64×64 = 4096 physical rays**, 표시 기본값 **256 trajectories**.
512 등으로 변경 가능하며 전체 detector 이벤트·누적/시간 bin·상태 통계에는 항상 전체 ray를 사용한다.
Morton 순서와 균등 간격을 이용해 SourcePlane 전체에서 subset을 선택하고 선택 ray도 포함한다.
표시 설정 변경은 physics key에 포함되지 않는다. 회귀 preset 16×16은 유지된다.

## E–F. 저장된 회귀 및 연구 결과 — 완료

| physical rays | Detected | Captured | Escaped | NumericalFailure | 최대 절대 null |
|---:|---:|---:|---:|---:|---:|
| 256 | 216 | 20 | 20 | 0 | 1.1393695e-8 |
| 1024 | 872 | 84 | 68 | 0 | 1.8809260e-8 |
| 4096 | 3502 | 330 | 264 | 0 | 1.8531864e-8 |

모든 행에서 Active=0, E/Lz 상대오차=0. 4096의 최대 Carter Q 절대오차는 4.1697490e-10이다.
이는 해당 저장 run에서의 오차이며 모든 가능한 궤도의 오차 상한이 아니다.
원본 과거 256-ray run의 **176개 파일은 이전 배포본과 SHA-256이 일치**했다.
새 run의 저장→로드 후 상태, detector event count, 누적 count, ray identity와 시간 일관성이 보존되었다.
원자료: [독립 데이터 검사](validation/visualization_data_audit.json).

## G. tests 결과 — 완료

기존 실행 로그를 합산한 결과 **기존 37 + 시각화 22 = 59개**, debug/release 각각 **59 passed / 0 failed**.
0-test binary/doc-test를 개수에 더하지 않았다. fmt check 및 clippy --all-targets -- -D warnings도 기존 실행에서 성공했다.
이번 최종 점검에서는 다시 실행하지 않았다. 로그의 환경 경로 canonicalize 경고는 Rust clippy 진단 실패가 아니다.

- [debug 로그](validation/viz_debug_tests.txt), [release 로그](validation/viz_release_tests.txt)
- [fmt check](validation/viz_fmt_check.txt), [clippy](validation/viz_clippy.txt)
- [물리 검증과 22개 신규 시험](PHYSICS_VALIDATION.md)

22개는 BL 변환, 기본값 분리, grid/bounds, camera, physics key, playback, frame dragging,
면 convention, ID/선택/subset, detector bin, worker/cancel, JSON 및 손실 없는 경로 roundtrip 등을 검사한다.

## H. performance — 측정 완료 / 측정 범위 제한 있음

### CPU 계산과 저장 scene 로드 (초)

| physical rays | CPU 계산 1회 | scene load | 저장 sample 수 |
|---:|---:|---:|---:|
| 256 | 0.2439413 | 0.2632321 | 200,538 |
| 1,024 | 0.9241227 | 1.0972395 | 792,467 |
| 4,096 | 3.9641895 | 4.3327711 | 3,175,919 |

CPU 계산은 이전 실제 실행 값이며 이번에 재계산하지 않았다.
scene load는 어댑터 생성 이후 worker에 저장 run을 요청하여 결과가 준비되기까지의 구간이다.
프로세스 생성+GPU 드라이버 초기화+첫 present 전체 startup 시간은 별도로 계측하지 않았다.

### 저장 4096-ray run의 렌더링

| 조건 | FPS | 평균 frame ms | p95 frame ms | 누적 ray buffer upload |
|---|---:|---:|---:|---:|
| trajectory 256개 | 239.636 | 4.1730 | 4.6205 | 1 |
| trajectory 512개 | 239.804 | 4.1701 | 4.6143 | 2 |
| frame dragging OFF | 239.470 | 4.1759 | 4.5742 | 3 |
| frame dragging ON | 239.794 | 4.1702 | 4.5167 | 3 |
| detector overlay OFF | 239.691 | 4.1720 | 4.6108 | 3 |
| detector overlay ON | 239.808 | 4.1700 | 4.6369 | 3 |
| propagation playback | 239.804 | 4.1701 | 4.5973 | 3 |

실제 창, 876×830 viewport, 각 조건 warmup 1.5초 + 측정 4초, camera 이동 포함.
vsync 약 240 Hz에 제한된 frame 간격이며 egui와 presentation을 포함한다.
ON/OFF 차이는 측정 변동 범위로 해석한다. GPU 자체 처리시간이나 최대 uncapped FPS가 아니다.
256-ray field OFF 구간의 214.960 FPS도 원자료에 남아 있으며, 원인 분리 측정을 하지 않아
OFF/ON의 인과적 성능 차이로 주장하지 않는다. [전체 측정표](BENCHMARK.md)를 참고한다.

## I. trajectory 저장·재사용 — 완료

`trajectory_samples.csv.gz`에 `ray_id, affine, t, r, theta, phi, p_t, p_r, p_theta, p_phi`를 저장한다.
원본 f64를 보존하며 gzip은 무손실이다. JSON은 float_roundtrip으로 읽는다.
`scene_metadata.json`, `detector_events.csv`, `ray_diagnostics.csv`, `detector_hit_states.csv`와
공유 설정·checksum을 함께 저장한다. 기본 stride=1이고 옵션 sampling도 첫/끝 상태를 유지한다.
저장된 4096 run은 **3,175,919 samples / 209,324,514 bytes gzip**이다.
전체 시간 순서와 모든 ray 첫/끝 f64 bit, 연속 hit time을 검사했고 작은 roundtrip 시험은 내부 sample까지 정확 일치한다.
BufReader를 사용하는 worker에서 로드하므로 GUI 이벤트 루프가 대용량 파일 parsing을 수행하지 않는다.
경로 없는 과거 event 데이터로 경로를 추정하거나 보간해 생성하지 않는다.

화면 변환은 기존 BL oblate convention:
`X=sqrt(r²+a²) sin(theta) cos(phi)`, `Y=sqrt(r²+a²) sin(theta) sin(phi)`, `Z=r cos(theta)`.
이는 좌표 시각화이며 유클리드 공간 임베딩이나 Kerr–Schild 시간 좌표가 아니다.

## J. worker / generation ID — 완료

장기 실행 worker thread가 Calculate/Load 요청을 받는다. 대기 요청을 최신 것으로 모으고,
AtomicU64 generation이 일치하는 완성 결과만 Arc로 전달한다. UI도 generation을 확인한다.
계산 중 기존 장면을 유지한다. 새 요청의 취소 확인은 ray 사이에서 이루어지며 단일 적분 도중 또는 진행 중 load를 즉시 끊지는 않는다.
카메라·표시·후처리 값은 physics key에서 제외하고 실제 물리조건 변경은 debounce 후 worker에 보낸다.

## K. GPU buffer caching — 완료

경로·hit·grid·field는 batch GPU buffer로 저장한다. 카메라와 재생은 uniform만 변경한다.
4096 실측에서 ray upload는 subset 256→512→256에 따라 1→2→3으로 증가했고,
이후 field/overlay/playback 및 camera 이동 구간에서 3으로 유지되었다.
subset 또는 선택 ray 변경은 표시 geometry 갱신 사유지만 physics 재계산 사유는 아니다.
모든 physical ray 결과는 CPU 측에 유지한다.

## L. 남아 있는 제한사항 — 제한 있음 / 일부 미완료

1. 위 B표의 미확인 GUI 입력은 최종 수동 검증 **미완료**다. 전체 조작 완료를 주장하지 않는다.
2. `bin/kerr-viewer.exe`는 실제 창을 검증한 이전 빌드이고, `target/release/kerr-viewer.exe`는 마지막 59-test 빌드다.
   후자의 추가 키보드 camera shortcut을 실제 창에서 최종 확인하지 않았다. 이번 보고 작업에서 binary를 교체하지 않았다.
3. GPU 표현은 f32, sample 사이 화면 위치는 Cartesian 선형 보간이다. 원본 f64 적분 결과를 수정하지 않으며 연속 해의 정확한 dense output을 뜻하지 않는다.
4. BL 외부 chart와 극축 부근 제한, 수치 포획 cutoff를 유지한다. 지평선 내부 적분은 하지 않는다.
5. field는 `omega=-g_tphi/g_phiphi`의 좌표 각속도이며 화살표는 `omega*(-Y,X,0)`에 명시된 표시 시간 척도를 곱한다.
   국소 힘·속도나 곡률 자체가 아니다. horizon 내부/invalid coordinate sample은 제외한다.
6. detector I는 ray count histogram이다. proper-area 보정 flux, 스펙트럼, 편광, 회절·간섭을 계산하지 않는다.
   3D 공통 clock은 Arrival(t_BL); Travel(delta_t) 분포는 기존 headless rebin을 사용한다.
7. 16,384/65,536 rays의 실행 시간·메모리·전체 표시 성능은 **미측정**. 결과 교체 시 일회성 geometry 생성/upload의 frame 지연도 별도 계측하지 않았다.
8. 3D 장면 video/frame sequence export는 **미구현**. detector PNG/APNG/sequence 및 전체 창 screenshot은 있다.
9. arbitrary Explicit 발사의 ray별 방향 override는 물리 데이터에 보존되지만 선택 UI의 방향 표시는 공통 SourcePlane 방향을 사용할 수 있다.
   기본 uniform-grid preset에는 해당하지 않으며 이런 실험은 원본 초기조건을 함께 확인해야 한다.
10. CPU 병렬 ray solver·GPU geodesic compute·CUDA·ML·Maxwell/Einstein 동역학은 이번 단계 범위 밖이다.

## 실행 방법

현재 Cargo 프로젝트 루트에서, 저장 결과만 여는 명령:

```powershell
.\target\release\kerr-viewer.exe --input results/visualization_4096
.\target\release\kerr-viewer.exe --input results/source_to_detector
```

소스에서 실행하는 기존 명령:

```powershell
. .\env.ps1
cargo run --release --offline -- visualize --input results/visualization_4096
cargo run --release --offline -- visualize --config examples/visualization.json
```

`--input`은 저장 결과를 로드한다. `--config` 새 실험은 물리 계산을 실행한다.
이번 최종 검증에서는 위 새 실험 명령을 실행하지 않았다.

## Where to modify the code

| 파일 | 사용자가 수정할 내용 |
|---|---|
| src/session.rs | 공유 기본값, 표시 subset, grid/field, playback/camera, stride |
| examples/visualization.json | 동일 SessionConfig의 GUI/CLI 실험 입력 |
| src/experiments/source_detector.rs | SourcePlane/DetectorPlane, 발사 조건, 회귀 preset |
| src/experiments/observables.rs | 관측량 |
| src/simulation/worker.rs | 비동기 실행 경계와 generation |
| src/simulation/storage.rs | f64 저장/로드와 진단 유지 |
| src/visualization/geometry.rs, renderer.rs, scene.wgsl | geometry와 GPU batch/cache |
| src/visualization/ui.rs, app.rs | 조작·진단·카메라·재생 |
| src/physics/field.rs | 기존 metric에서 계산하는 omega |

현재 재개 작업에서는 이 파일들을 수정하지 않았다. 향후 GPU 계산 backend는 simulation 경계 뒤에 추가하고,
같은 초기조건에 대해 CPU f64 기준의 궤도·hit time·상태·null/E/Lz/Q와 수렴을 비교해야 한다.
