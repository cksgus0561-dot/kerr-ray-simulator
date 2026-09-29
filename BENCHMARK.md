# 실측 성능 — Kerr 3D v0.3

2026-09-15. 이전 실행에서 저장한 원자료를 정리했다. 이번 보고 작성 중 simulation 또는 benchmark를 다시 실행하지 않았다.
과거 CPU-only 측정은 [v0.2 이력](validation/history/V02_BENCHMARK.md)에 보존한다.

## 환경과 측정 범위

- CPU: Intel Core i5-12400F, CPU f64 reference는 worker 한 개에서 ray를 순차 계산.
- 실제 GPU: NVIDIA GeForce RTX 3060 Ti, Vulkan, DiscreteGpu, vendor 0x10de.
- Windows / Rust 1.98.1 / x86_64-pc-windows-msvc release, wgpu 30.0.1, egui/eframe 0.36.2.
- window client 1440×960, 실제 3D viewport 876×830 pixels, vsync 약 240 Hz.
- 각 조건 1.5초 warmup + 4초 frame 간격 수집. egui·presentation·camera 이동을 포함.
- GPU timestamp query가 아니며 CPU simulation 시간과 렌더링 FPS는 서로 다른 측정이다.
- 전용 격리 환경·고정 GPU clock을 사용하지 않았으며 반복 통계/신뢰구간은 없다.

## 계산과 저장 데이터 로드

| physical rays | CPU 계산 1회 s | scene load s | 저장 sample 수 |
|---:|---:|---:|---:|
| 256 | 0.2439413 | 0.2632321 | 200,538 |
| 1,024 | 0.9241227 | 1.0972395 | 792,467 |
| 4,096 | 3.9641895 | 4.3327711 | 3,175,919 |

CPU 원자료: [256](validation/cpu_256.txt), [1024](validation/cpu_1024.txt), [4096](validation/cpu_4096.txt).
이전 4096 계산 3.9641895초를 재사용한다. export 비용은 위 CPU 열에 포함하지 않는다.
scene load는 어댑터 생성 후 worker 로드 요청부터 결과가 준비되기까지다.
프로세스 시작/어댑터 초기화/첫 present까지 모두 합한 GUI startup은 별도 **미측정**이다.
독립 Python 검증의 약 19.61초는 전체 gzip 검사 비용이며 Rust load time이 아니다.

## 4096 physical rays에서 렌더링

| 조건 | FPS | 평균 frame ms | p95 frame ms | 누적 ray buffer upload |
|---|---:|---:|---:|---:|
| trajectory 256개 | 239.636 | 4.1730 | 4.6205 | 1 |
| trajectory 512개 | 239.804 | 4.1701 | 4.6143 | 2 |
| frame dragging OFF | 239.470 | 4.1759 | 4.5742 | 3 |
| frame dragging ON | 239.794 | 4.1702 | 4.5167 | 3 |
| detector overlay OFF | 239.691 | 4.1720 | 4.6108 | 3 |
| detector overlay ON | 239.808 | 4.1700 | 4.6369 | 3 |
| propagation playback | 239.804 | 4.1701 | 4.5973 | 3 |

카메라는 측정 중 계속 움직였다. subset 변경 이후 ray upload가 고정된 것은 camera 이동에 따른
경로 buffer 재생성/재업로드가 없었음을 보여준다. 이 count는 모든 종류의 GPU upload가 없다는 뜻은 아니다.

## 세 physical ray 수의 원자료 요약

| 조건 | 256 rays FPS | 1024 rays FPS | 4096 rays FPS |
|---|---:|---:|---:|
| trajectory 256개 | 239.760 | 239.698 | 239.636 |
| trajectory 512개 | 239.784 | 239.815 | 239.804 |
| frame dragging OFF | 214.960 | 239.770 | 239.470 |
| frame dragging ON | 239.798 | 239.791 | 239.794 |
| detector overlay OFF | 239.774 | 239.770 | 239.691 |
| detector overlay ON | 239.803 | 239.589 | 239.808 |
| propagation playback | 239.829 | 239.737 | 239.804 |

256 physical rays에서는 표시 상한을 512로 요청해도 실제 경로는 최대 256개다.
각 파일의 rendered_trajectories를 함께 확인해야 한다.

frame dragging ON/OFF, overlay ON/OFF 차이는 대부분 약 240 Hz cap 안의 변동이다.
256-ray field OFF의 214.960 FPS 구간은 그대로 보고하며 원인을 분리 측정하지 않았다.
field ON이 성능을 높였다고 해석하거나 이 구간을 숨기지 않는다.
이번 작업은 완료된 측정을 재사용하라는 요청에 따라 재측정하지 않았다.

## 원자료

- [256 JSON](validation/viz_256_final/render_measurements.json), [scene PNG](validation/viz_256_final/scene.png)
- [1024 JSON](validation/viz_1024/render_measurements.json), [scene PNG](validation/viz_1024/scene.png)
- [4096 JSON](validation/viz_4096/render_measurements.json), [scene PNG](validation/viz_4096/scene.png)
- [과거 run scene](validation/viz_legacy/scene.png): trajectory 없음, 실제 경로 0개.

legacy 측정 JSON의 표시 요청 수를 실제 경로 수로 해석하지 않는다. 과거 run의 로드 시간은
약 0.048143초이며 trajectory parsing을 포함하지 않으므로 새 run과 동등한 작업량이 아니다.
65,536 rays, uncapped FPS, 고해상도 출력, GPU 자체 pass 비용은 미측정이다.

## 2026-09-25: seed 기반 표준 생성/저장

환경: Windows 11 (10.0.26200), x86_64, Intel Core i5-12400F, Rust release
(optimized + debuginfo). GPU 물리 계산이나 CPU 병렬화를 추가하지 않았다.
동일 executable/source fingerprint에서 각 조건을 한 번씩 측정했다.
수치는 CLI의 계산/입력 대응 처리 시간이며 GPU FPS가 아니다.

| 조건 | 영역 | 셀 크기 | 광선 수 | 계산/대응 시간 | 저장 시간 | 결과 파일 크기 |
|---|---|---|---:|---:|---:|---:|
| 기존 centered preset | 32×32 M | 0.5×0.5 M | 4096 | 3.889758900 s | 0.008170100 s | Parquet 126689 B |
| 새 stratified default | 64×64 M | 0.5×0.5 M | 16384 | 12.422895600 s | 0.004908500 s | binary 395320 B |

새 common.json은 2854 B, 전체 raw 폴더는 398174 B이다. binary는
72 + 16384 + 24×15786 = 395320 B로 독립 Python 검사와 일치했다.
행 수/검출 비율이 다르므로 이 표는 동일 데이터의 압축 형식 비교가 아니다.

- 기존 4096: D3502 / C330 / E264 / NUM0.
- 새 seed 한 회: D15786 / C330 / E268 / NUM0. 새 seed에서는 수가 달라질 수 있다.
- 새 16384 재현: load/검증 0.031770300 s, fresh integration 11.990861800 s.
  status 및 hit u/v/t 비트 불일치 0; 초기조건 SHA-256 일치.
- 실제 소규모 viewer 실행: RTX 3060 Ti / Vulkan / DiscreteGpu, 4-ray seed 실행,
  scene ready 0.059333 s, 자동 binary 저장 및 응답 중인 실제 창 확인.
  이 값은 16384-ray GUI 성능 측정이 아니다. 새 GUI FPS는 측정하지 않았다.

원자료: `validation/seeded_perf_4096.txt`, `seeded_perf_16384.txt`,
`seeded_reproduce_16384.txt`, `seeded_independent_audit.json`,
`seeded_gui_launch.txt`, `seeded_gui_window.json`.

## 2026-09-26: ray-level CPU parallelism

Same machine (Core i5-12400F, 12 logical CPUs, Windows x86_64), same optimized
release executable, same saved 256-bit seed/chi/common settings. Serial and
parallel child processes ran sequentially, once each. Parallel used 11 workers.
Input preparation is excluded from the physics interval; worker creation,
dispatch and completion collection are included. Total includes generation,
preparation, hash/association/validation and standard save. Comparison hashes
are computed afterward and excluded. These are single-run wall-clock samples,
not averaged throughput or a universal speedup guarantee.

| Mode | Rays | Threads | Physics | Calculate + save |
|---|---:|---:|---:|---:|
| Serial reference | 16384 | 1 | 12.6394836 s | 12.7290165 s |
| Parallel | 16384 | 11 | 2.0608745 s | 2.1524231 s |
| Temporary wider region | 36864 | 11 | 4.0706499 s | 4.2792869 s |

Physics speedup: **6.1331x**. Total calculate/save speedup: **5.9138x**.
The larger run used 192x192 cells of 0.5M, covering 96x96M. The default generator
is still 128x128 / 64x64M. No change to the parallel code was needed.

Both 16384 runs: D15786/C330/E268/NUM0. Every canonical ray's initial state,
trajectory sample/affine bits, diagnostics, status, stop reason and detector
intersection had the same SHA-256 fingerprint (wall-clock timings excluded).
Both sim_1.bin files matched the pre-parallel archived file byte-for-byte.
Larger run: D36263/C330/E271/NUM0.

Fresh CLI reproduction of the pre-parallel 16384 archive also passed: initial
hash matched, zero status mismatches, zero u/v/t bit differences, maximum hit
errors all zero. That separate integration took 2.3200651 s. Its source
fingerprint comparison is false as expected after changing scheduler source;
physical reproduction comparisons pass without changing their tolerances.

Evidence: `validation/parallel_measurements_20260926/summary.json`, per-mode
`measurement.json` files, `validation/parallel_benchmark.txt` and
`validation/parallel_reproduce_16384.txt`. Harness: examples/ray_parallel_benchmark.rs.
No GPU physics, UI FPS, peak-memory, cross-platform or repeated statistical
timing measurements were performed in this change.
