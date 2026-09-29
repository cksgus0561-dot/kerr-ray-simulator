# 3D visualization — v0.3

기존 CPU f64 Kerr 영측지선·검출기 엔진을 그대로 사용한다. wgpu는 표시만 담당한다.
`src/physics/field.rs`는 기존 계량에서 ZAMO 각속도를 읽는 작은 공용 함수다.
검증된 Kerr 계량·적분기·검출 root 코드를 시각화 때문에 변경하지 않았다.

## 실행

프로젝트 루트에서:

```powershell
. .\env.ps1
# 이미 저장된 연구 결과: 영측지선 재계산 없음
cargo run --release --offline -- visualize --input results/visualization_4096
# 경로를 저장하지 않았던 기존 256-ray 결과
cargo run --release --offline -- visualize --input results/source_to_detector
# 새 연구 실험: 4096 physical rays / 256 rendered trajectories
cargo run --release --offline -- visualize --config examples/visualization.json
# 기존 회귀 preset: 256 physical rays
cargo run --release --offline -- visualize --preset regression
# GUI 없이 전체 결과와 재사용할 경로 저장; 새 폴더 필요
cargo run --release --offline -- visualize-prepare --grid 32 --output results/my_1024_run
```

`bin/kerr-viewer.exe`는 별도 Rust 설치 없이 실행할 수 있는 Windows 프로그램이다.
`kerr-viewer.args.json`이 exe 옆에 있으면 그 JSON 문자열 배열을 동일한 CLI 인자로 사용한다.
`kerr-viewer.json`은 동일한 SessionConfig 파일이다. args 파일이 우선한다.
각 경로를 현재 프로젝트에 맞추거나 아래 launch 스크립트를 사용한다.

## 조작

- 왼쪽 드래그: orbit. 오른쪽/가운데 드래그: pan. `Pan drag`를 켜면 왼쪽도 pan.
- 휠: zoom. `Reset camera`, `Fit scene`, `Kerr center`, `Source`, `Detector` 버튼.
- 경로 클릭 또는 우측 Ray ID 입력과 Select: 선택 강조. Clear: 해제.
- 전체 궤적 / Propagation / Detector 재생 모드는 독립적인 표시 선택이다.
- Play/Pause, Stop, Reset time, 시간 슬라이더, Physical M / wall s로 시간 재생.
- 장면 좌표·계량은 항상 결과에 기록된 chi를 사용한다. 새 계산 중에는 이전 결과가 유지된다.

상태 색: Detected 청록, Captured 주황, Escaped 보라, NumericalFailure 빨강, Active 회색.
선택한 ray는 노랑이다. Active는 affine/스텝 제한 도달 시 남을 수 있으며 Escaped로 바꾸지 않는다.
오른쪽에는 전체 count, CPU 시간, ray 초기조건, 존재하는 hit 값과 보존량 오차를 표시한다.

## 격자와 좌표

장면의 직선 Cartesian 형태 lattice와 SourcePlane의 발사 grid는 다른 것이다.
자동 lattice 범위는 Kerr 중심, 양 평면의 모서리, 표시할 실제 궤적의 범위에 6% margin을 더한다.
수동 extent/spacing은 물리에 영향을 주지 않는다. 렌더링 부하를 제한하려고 축마다 최대
40개 간격을 사용하며, 요청 spacing이 너무 작으면 extent/20이 실제 spacing이다.
실제 extent/spacing을 진단 창에 표시한다.

기존 `physics/coordinates.rs`의 BL oblate 변환을 그대로 사용한다.

```text
X = sqrt(r²+a²) sin(theta) cos(phi)
Y = sqrt(r²+a²) sin(theta) sin(phi)
Z = r cos(theta)
T = t_BL
```

이것은 곡률 공간의 유클리드 등거리 임베딩이나 Kerr-Schild 좌표가 아니다.
화면의 카메라는 좌표 그림을 관찰하는 도구이며 물리적 관측자의 tetrad 카메라가 아니다.
축 convention은 +Z 회전축이다. 양 평면의 경계, 중심에서 나온 local u/v/normal 화살표,
발사점과 hit를 같은 변환 convention에서 표시한다. u는 빨강, v는 초록, normal은 파랑이다.
검출 분포는 실제 DetectorPlane 위 texture와 별도 2D image panel로 표시한다.

## Kerr 구조와 frame dragging

사건의 지평선의 정확한 r_plus를 oblate 좌표에 넣은 surface mesh와,
각 theta의 정확한 r_ergo(theta)를 사용한 wire surface를 만든다.
적분 종료의 epsilon을 surface 반지름에 더하지 않는다.
mesh의 유한 삼각형 수와 f32 vertex는 표시 근사이며 원본 수치를 바꾸지 않는다.

`omega = -g_tphi / g_phiphi`를 기존 Kerr 공변 계량에서 계산한다.
표시 화살표는 좌표 속도 `d(X,Y,Z)/dt = omega*(-Y,X,0)`에 명시적인 표시 시간 간격을 곱한 것이다.
이 값은 각속도/좌표 속도이며 국소에서 측정한 선속도나 중력 힘이 아니다.
기본 z=0 단면 15×15개, 선택적으로 3D sampling. density, extent, 시간 배율을 조작할 수 있다.
chi=0의 회전 화살표는 0이어서 그려지지 않는다. 지평선 내부·극축의 유효하지 않은 BL 위치는 제외한다.
장식용 vortex, 왜곡 shader, 휘어진 기준 grid는 없다.

## 물리량과 표시량의 분리

`SessionConfig::default()`의 RESEARCH_RAY_GRID는 64×64=4096이다.
`SourceDetectorExperiment::default()`의 16×16=256 회귀 조건은 유지했다.
기본 표시 한도는 ViewConfig의 `max_rendered_trajectories=256` 한 곳에서 정한다.
모든 physical ray가 원본 상태, detector events, I(u,v), I(u,v,t)에 반영된다.
렌더링 subset은 source u/v의 Morton 순서에서 균등 간격으로 선택한다.
선택 ray가 subset 밖이면 그 ray를 포함한다. subset은 데이터를 지우거나 검출 count를 바꾸지 않는다.

표시 설정 변경은 물리 요청을 발생시키지 않는다. 물리 settings key에서 camera/view,
detector 해상도, 시간 bin/출력 설정을 제외했다. 실제 초기조건 변경은 450ms debounce 후 제출한다.
단일 long-lived worker가 요청 queue를 최신 것으로 합치고, 각 ray 사이에서 generation을 검사한다.
완료 시 worker와 UI 양쪽에서 generation을 확인하고 일치하는 immutable Arc 결과만 교체한다.
한 ray의 적분 중간을 강제로 끊지는 않는다. 큰 run 로드도 worker에서 수행한다.
큰 결과 준비가 끝난 뒤 GPU geometry 생성·upload는 render thread에서 한 번 수행된다.
따라서 결과 교체 순간의 단발 지연까지 완전히 없앤 streaming renderer는 아니다.

GPU buffer는 grid/planes/launch, trajectories, hit, field, horizon별 batch로 유지한다.
카메라·시간·visibility·두께는 uniform만 변경한다. 데이터/표시 subset 변경 시 경로 buffer를 갱신한다.
field density와 grid 범위는 해당 geometry만 갱신한다. detector bin 설정이 바뀌면 hit의 bin 정보와
count texture를 갱신한다. 매 frame 전체 trajectory Vec를 만들거나 업로드하지 않는다.

## 저장, 연속시간, export

새 run에는 다음을 저장한다.

- `detector_events.csv`: 기존 연속 f64 t_hit / delta_t, 전체 physical rays의 검출 이벤트.
- `ray_diagnostics.csv`, `detector_hit_states.csv`: 기존 진단 schema와 연속 교차 상태.
- `trajectory_samples.csv.gz`: `ray_id,affine,t,r,theta,phi,p_t,p_r,p_theta,p_phi`.
- `scene_metadata.json`: 초기/최종 상태, 모든 ray ID/status/오차/step count 및 파일 checksum.
- `run_metadata.json`, `visualization.json`: 물리 실험과 표시/카메라 설정.
- 기존 누적/시간분해 count CSV, PNG sequence, APNG.

기본 stride=1은 모든 승인 f64 상태를 저장한다. gzip은 무손실이다.
stride>1이면 매 N번째 승인 상태와 양 끝점을 저장하고 원본 sample count를 기록한다.
현재 메모리 결과는 여전히 모든 승인 상태를 유지한다. reload는 저장된 stride만 복원할 수 있다.
JSON은 float_roundtrip을 켜서 f64 왕복 값을 유지한다. 새 파일은 create_new를 사용한다.
새 폴더에 저장해야 하며 원본 run을 덮어쓰지 않는다.

큰 gzip CSV는 buffering/streaming parser로 읽고 실제 시각화용 최종 결과는 메모리에 유지한다.
4096-ray 저장 예제는 3,175,919 samples / gzip 209,324,514 bytes다.
과거 event-only run은 detector와 진단을 읽고 **TRAJECTORY UNAVAILABLE**를 표시한다.
event로 경로를 만들어내지 않는다. 필요할 때 사용자가 명시적으로 Recompute를 선택한다.

Propagation 위치는 저장된 물리 t에 따른 인접 좌표 샘플 사이 선형 표시 보간이다.
새로운 영측지선 해나 dense-output 정확도를 주장하지 않는다. GPU 표시 좌표·시간은 f32이며
큰 절대 t_emit나 매우 긴 시간 범위의 미세한 재생에는 정밀도 한계가 있다. 원본은 f64다.
검출 hit의 bin/rank는 CPU f64로 판정하고 GPU에는 정수 분류를 전달한다.
Instantaneous는 기존 half-open bin과 경계 반올림 규칙을 그대로 따른다.
Cumulative는 현재 t까지의 모든 hit, Accumulated는 전체 도착 count다.
3D 공통 재생 clock은 Arrival(t_BL)이다. Travel(delta_t) 분포는 기존 rebin CLI에서 지원한다.
Delta t나 detector resolution 변경은 events 후처리이며 적분을 재실행하지 않는다.

Export panel: 전체 창 screenshot, 현재 detector PNG + CSV, 현재 시각화 JSON, 전체 run 및
detector frame sequence/APNG. 3D 장면의 video encoder/3D frame sequence는 제공하지 않는다.
이미 존재하는 파일은 덮어쓰지 않는다. export 오류는 화면에 표시된다.

## Where to modify the code

| 파일 | 수정할 내용 |
|---|---|
| `src/session.rs` | 연구용 ray grid 기본값, 표시 개수, grid/field/두께/재생, 초기 카메라, export stride |
| `examples/visualization.json` | 코드와 같은 SessionConfig; GUI 재컴파일 없이 연구 조건 변경 |
| `src/experiments/source_detector.rs` | 물리 SourcePlane/DetectorPlane, 발사 방향·시각, 보존된 256-ray preset |
| `src/experiments/observables.rs` | 새 측정 함수; 기존 `experiments/output.rs`에 CSV 연결 |
| `src/simulation/mod.rs` | 검증된 적분기를 실행하고 전체 결과를 모으는 경계 |
| `src/simulation/worker.rs` | 비동기 실행, 취소/generation, 향후 CPU 병렬 작업 분배 |
| `src/simulation/storage.rs` | f64 경로와 원본 detector/진단 저장·로드 |
| `src/visualization/geometry.rs` | 좌표 geometry와 표시 subset; 물리 해를 수정하지 말 것 |
| `src/visualization/renderer.rs`, `scene.wgsl` | wgpu cache, batch와 display uniform |
| `src/visualization/ui.rs`, `app.rs` | 조작·진단·카메라·export·측정; physics key 분리 유지 |
| `src/physics/field.rs` | 기존 계량을 읽는 omega; 핵심 계량 수식을 중복 구현하지 말 것 |

카메라를 JSON에서 정확히 복원하려면 `view.fit_on_load=false`를 사용한다.
GUI의 config export는 이를 자동 적용한다. `true`는 자동 scene fitting을 선택한다.
64/128/256 등 grid 크기는 같은 SourcePattern 설정을 사용하며 16,384/65,536-ray run의
실제 메모리·전체 표시 성능은 아직 측정하지 않았다. 큰 grid일수록 표시 subset을 유지하는 것이 좋다.

## 검증과 다음 계산 backend

[PHYSICS_VALIDATION.md](PHYSICS_VALIDATION.md), [BENCHMARK.md](BENCHMARK.md),
[FINAL_REPORT.md](FINAL_REPORT.md)에 실제 시험과 측정, 확인한 범위를 기록한다.
physics core나 detector 변경 시 전체 물리 시험이 필수다. 화면 모양으로 물리를 검증하지 않는다.
향후 GPU geodesic backend는 현재 simulation/PhysicsResult 경계 뒤에 추가한다.
같은 초기조건·event 시점에서 CPU f64 기준과 궤적, 상태, t_hit, null/E/Lz/Q, step 수렴을 비교해야 한다.
현재 CUDA, wgpu geodesic compute, Maxwell evolution, dynamic gravity, ML은 구현하지 않는다.
