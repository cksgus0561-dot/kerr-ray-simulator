# 최종 보고 — 3+1 Kerr source-to-detector

## 이번 재개 작업의 결과

현재 저장된 구현을 보존하고 남은 확인·문서화를 마무리했다.
기존 Milestone 1–3 위에 구현되어 있던 3+1차원 CPU 엔진, SourcePlane/DetectorPlane,
원본 이벤트, count·PNG·시간분해 데이터, rebin을 재구현하지 않았다.
후속 요청의 완료 조건 **20/20**을 [체크리스트](COMPLETION_CHECKLIST.md)로 대조했다.

재개 후 코드는 보고용 두 항목만 보완했다:
`examples/benchmark_detector.rs`에서 max_null의 JSON 숫자 누락 수정,
`experiments/source_detector.rs`에서 신규 experiment_id 기록 추가.
기존 신규 CLI 시험에 ID 일관성 확인을 덧붙였으며 기존 18개 시험은 수정하지 않았다.
핵심 계량·적분·검출 물리 알고리즘은 이 재개 작업에서 변경하지 않았다.

## 요청한 최종 보고 19개 항목

| 번호 | 항목 | 결과 |
|---|---|---|
| 1 | 기존 18개 유지 | 모두 통과. 기존 시험 파일과 integrator.rs byte 단위 보존 확인 |
| 2 | 새 시험 수 | 기준점 대비 19개:3D6+detector7+dataset6; 재개 작업에서 시험 함수 수 추가 없음 |
| 3 | 총 시험·실패 | **37개, debug/release 각각 실패0**, fmt/clippy도 통과 |
| 4 | 일반 3D 최대 null | 탈출18개 1.076167e-12; 포획9개 8.388270e-9; 저장256광선 **1.139370e-8** |
| 5 | E 최대 상대오차 | **0** |
| 6 | Lz 최대 상대오차 | **0** |
| 7 | Carter Q 최대오차 | 저장256광선 **2.933121e-10 절대**, 상대 척도4.821477e-12; 3D 탈출 시험 절대1.423218e-11 |
| 8 | detector u_hit 수렴 오차 | **9.956481e-12 M** |
| 9 | detector v_hit 수렴 오차 | **1.877707e-10 M** |
| 10 | detector t_hit 수렴 오차 | **9.958399e-10 M** |
| 11 | 예제 발사 광선 수 | **256**, spin=.6,16×16, t_emit=0 |
| 12 | 종료 상태 | **Detected216 / Captured20 / Escaped20 / NumericalFailure0**, Active0 |
| 13 | 원본 CSV | [detector_events.csv](results/source_to_detector/detector_events.csv) |
| 14 | 누적 이미지 | [detector_accumulated.png](results/source_to_detector/detector_accumulated.png), [원본 count 배열](results/source_to_detector/detector_accumulated.csv) |
| 15 | 시간 frame | [instantaneous 첫 장](results/source_to_detector/detector_frames/instantaneous/frame_000000.png), [cumulative 첫 장](results/source_to_detector/detector_frames/cumulative/frame_000000.png); 아래 경로 설명 |
| 16 | 수정 파일 | source_detector.rs·JSON·config.rs·scenarios.rs·observables.rs; 아래 구체적 방법 |
| 17 | 실측 성능 | 256광선 순차 CPU 평균 **216.868790 ms**, p95 **239.981200 ms**; 워밍업1 제외,10배치 |
| 18 | 물리·수치 한계 | BL 외부/극축, 좌표면·좌표시간, 횡단 교차 가정, 유한 오차·count 모델; 아래 설명 |
| 19 | wgpu/GPU 확장 | CPU f64 비교, 비동기 작업, 세대 ID, 동일 사건/보존량, 어댑터·성능 계측; 아래 설명 |

수렴 오차는 fine(rtol1e-10,max_step1)와 reference(1e-13,.25)의 차이이며
u/v/t 각각 허용2e-7을 만족하고 coarse(1e-7,4)보다 감소했다.
표의 최대값은 명시한 집합에서 측정한 값이다. 전체 시험에서 가장 큰 null은
기존 a=.99 cutoff 민감도 시험의 **2.801244e-7**(허용2e-6)이다.
새 3D 시험 최대, 실제 예제 최대, 전체 시험 최대를 혼동하지 않는다.

## 실행 방법

프로젝트 루트에서 PowerShell:

```powershell
. .\env.ps1
cargo run --release --offline -- experiment source_to_detector --output results/my_source_run
cargo run --release --offline -- rebin --input results/source_to_detector --output results/my_dt01 --dt 0.1 --resolution 64x64 --fps 60
cargo run --release --offline -- experiment source_to_detector --config examples/source_detector.json --output results/my_config_run
.\check.ps1
```

매번 새 출력 폴더를 지정한다. 기존 source/detector 원본·파생 파일은 덮어쓰지 않는다.
`rebin`은 원본 이벤트와 metadata만 읽어 새로운 histogram/영상을 만들며 영측지선을 계산하지 않는다.
다른 컴퓨터에는 Rust/MSVC가 필요하고, crate는 포함된 vendor와 Cargo.lock으로 고정했다.

## 저장 자료와 재처리 검증

프로젝트 루트 기준 실제 경로:

```text
results/source_to_detector/detector_events.csv
results/source_to_detector/detector_hit_states.csv
results/source_to_detector/ray_diagnostics.csv
results/source_to_detector/detector_accumulated.csv
results/source_to_detector/detector_accumulated.png
results/source_to_detector/detector_time_bins.csv
results/source_to_detector/frame_index.csv
results/source_to_detector/detector_frames/instantaneous/frame_000000.png ... frame_000080.png
results/source_to_detector/detector_frames/cumulative/frame_000000.png ... frame_000080.png
results/source_to_detector/detector_instantaneous.apng
results/source_to_detector/detector_cumulative.apng
results/source_to_detector/run_metadata.json
results/source_to_detector_dt01/detector_frames/instantaneous/frame_000000.png ... frame_000804.png
results/source_to_detector_dt01/detector_frames/cumulative/frame_000000.png ... frame_000804.png
```

원본 216개 이벤트의 공간 count 합과 모든 시간 bin 합은 각각216이다.
Δt1은 128×128·81bin·30fps, Δt.1은 64×64·805bin·60fps이며 재생 속도는 물리시간을 바꾸지 않는다.
저장된 두 자료의 PNG frame **1,772장** 및 APNG 내부 **1,772 frame** 전부를
이벤트에서 독립 계산한 수치와 픽셀 단위로 비교했다. 시작/종료 시간과 재생 지연도 일치한다.
검사 중 영측지선 재계산은 없고, 기존1,796개 자료 파일의 checksum도 변하지 않았다.
[감사 결과](validation/saved_results_audit.json)와 [파일별 SHA-256](validation/saved_results_sha256.json)을 보존했다.

원본 이벤트 FNV-1a64: `6d7ce5a78b31e7e9`.
이전 실행의 build fingerprint는 `d29068dbfd75eff1`이며 그 뒤 보고/후처리 경계 처리가 보강되었다.
저장 자료가 현재 스냅샷에서 생성된 것처럼 provenance를 고쳐 쓰지 않았다.
저장 실험 식별자는 원래 출력 경로와 UTC 시작 시각을 연결한 **`source_to_detector-1789446362.777505`**로 해석한다.
기존 run_metadata는 경로·시각을 이미 포함하고, 새 실행부터 같은 convention의 experiment_id를 명시한다.
후처리 경계 규칙 추가 전의 Δt.1 자료도 현재 규칙으로 전수 검산하여 일치함을 확인했다.

## 직접 수정하는 방법

- [src/experiments/source_detector.rs](src/experiments/source_detector.rs)의
  SourceDetectorExperiment::default()에서 spin, 면 중심·크기, 격자 nu/nv, 방향, 발사시각,
  적분 허용오차·escape/max_step, 후처리 Δt·해상도·출력을 바꾼다.
- [examples/source_detector.json](examples/source_detector.json)은 같은 설정의 실행 가능한 예다.
  source.pattern.RectangularGrid의 nu/nv, source.plane, detector.plane 등을 편집해 `--config`로 실행한다.
- 면 방향은 Plane::from_normal_up의 normal/up을 바꾼다. 국소 물리 방향은 LocalZamo를 사용한다.
  광선별 시각·방향은 SourcePattern::Explicit 안의 Emission 옵션으로 지정한다.
- [src/config.rs](src/config.rs)는 전역 수치 기본값이다. source/detector 실험에서 덮어쓰는
  escape400/max_affine1500은 SourceDetectorExperiment에서 수정한다.
- [scenarios.rs](src/experiments/scenarios.rs)에 새 실험을 추가하고,
  [parameter_scan.rs](src/experiments/parameter_scan.rs)에서 범위를 바꾼다.
- [observables.rs](src/experiments/observables.rs)에 PhysicsResult를 읽는 함수를 하나 추가하고
  experiments/output.rs의 CSV 헤더·행에 연결하면 새 관측량을 저장할 수 있다.
- physics/와 detector/의 수학을 변경할 때는 전체 시험과 관심 관측량의 수렴 검증을 실행한다.

## 실측 성능과 한계

i5-12400F, Rust release, CPU 순차 f64. 최신 10배치 평균216.868790ms, p95=239.981200ms.
별도의 실제 출력1회는 물리211.686ms, 물리+원본231.0397ms,
PNG/APNG 후처리229.7469ms, 총461.2749ms였다. 자세한 범위와 원자료는 [BENCHMARK.md](BENCHMARK.md)에 있다.
오래된 벤치마크 JSON의 숫자 누락을 수정해 별도 폴더에 재측정했으며 기존 기록은 보존했다.
이 성능은 FPS가 아니다. GPU·병렬화·UI frame time은 미측정이다.

현재는 고정 Kerr 시공간, G=c=M=1, signature(-,+,+,+), spin<=.99, 기하광학이다.
BL의 r_plus+epsilon 외부만 적분하고 극축을 건너는 정칙 chart가 없다.
광원/검출면은 T=t_BL의 oblate Cartesian-like 좌표 사각형이며
유클리드 물리 공간의 평면이나 Kerr–Schild 좌표가 아니다. 모든 면은 보수적으로 r>2 밖에 둔다.
t_hit/delta_t는 좌표시간이고, 한 hit=1 count이며 고유시간·고유면적·에너지 플럭스 가중을 하지 않는다.
접선 접촉·한 스텝 내부의 같은 부호 복수 교차는 보장하지 않는다. max_step 수렴 검증이 필요하다.
오차 최대값은 승인 상태에서 얻고 국소 tolerance가 전역 오차를 보장하지 않는다.
PNG는 기본4 count에서 흰색 포화되지만 원본 u64 count는 변하지 않는다.

## 다음 wgpu/GPU 단계

CPU reference와 보존량 검사를 유지하며 동일 초기조건의 CPU 병렬화부터 대조할 수 있다.
GPU는 동일 affine 또는 검출 사건에서 r/theta/phi/t·운동량·u/v/t_hit·상태·null/E/Lz/Q를 비교하고
h,h/2,h/4 수렴과 임계·고스핀·포획 사례를 통과해야 한다.
UI는 별도 작업 스레드·generation ID·입력 debounce로 적분과 분리하며 카메라/재생은 재계산하지 않는다.
어댑터 열거·가능한 discrete NVIDIA 우선 선택·backend/features/limits 로그를 추가한다.
실제 frame time·복사량·workgroup을 측정한 뒤 최적화한다. CUDA는 별도 이점이 측정될 때만 검토한다.

## 문서·배포

주요 갱신 파일은 README, PHYSICS_VALIDATION, BENCHMARK, DESIGN, COMPLETION_CHECKLIST,
본 보고서, validation 최종 로그·읽기 감사 스크립트, 편집 가능한 JSON이다.
소스·vendor·검증 자료·저장 결과를 포함한 새 ZIP은 프로젝트 상위의
`kerr-ray-3d-source-and-results.zip`이다. target/ 및 로컬 Rust 도구체인은 제외한다.
M1–3 baseline ZIP과 이전 문서는 그대로 보존한다.
