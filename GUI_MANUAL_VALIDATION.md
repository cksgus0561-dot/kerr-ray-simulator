# Computer Use GUI 실조작 검증 — 2026-09-16

## 범위와 판정

기존 실조작 기록을 이어서 완료했다. 소스 확인만으로 PASS를 부여하지 않았다.
PASS는 실제 Windows 창에 Computer Use 입력을 보내고 화면 변화를 관찰한 항목이다.
결과: **22 PASS / 0 FAIL / 1 NOT TESTED** (아래 요청 필수 23개 항목, 수정 후 최종 판정).

GPU: NVIDIA GeForce RTX 3060 Ti / Vulkan / DiscreteGpu.
사용 run: results/visualization_4096, results/source_to_detector. 모두 기존 저장 데이터이며 새 물리 계산은 하지 않았다.

| 기능 | 실제 Computer Use 조작 | 결과 | 비고 |
|---|---|---|---|
| Play | 클릭 | PASS | 시간 증가와 실제 저장 경로 진행; pause 후 446.483→453.985 M 이어 재생 [화면](validation/manual_gui_2026-09-16/01_propagation_paused.png) |
| Pause | 클릭 후 재관찰 | PASS | 446.483 M이 유지됨. 최종 수정 빌드에서도 245.499 M에 정지 [화면](validation/manual_gui_2026-09-16/02_pause_time_unchanged.png) |
| Stop | 클릭 | PASS | 재생을 멈추고 t=0, cumulative shown=0 복귀 [화면](validation/manual_gui_2026-09-16/04_reset_time_zero.png) |
| Reset time | 클릭 | PASS | 180 M에서 0 M으로 복귀, cumulative 2472→0 [화면](validation/manual_gui_2026-09-16/04_reset_time_zero.png) |
| time slider | 마우스 드래그 | PASS | 0→180 M과 경로/누적 hit 2472 표시가 함께 변경 [화면](validation/manual_gui_2026-09-16/03_time_slider_midpoint.png) |
| orbit | 왼쪽 마우스 드래그 | PASS | 앞선 버그 수정 후 동일 드래그로 실제 시점 변화; 재생 중에도 확인 [화면](validation/manual_gui_2026-09-16/07_orbit_after_fix.png) |
| pan mode | Pan drag 클릭 + 왼쪽 드래그 | PASS | 장면이 평행 이동, 재생 중에도 시간 진행 [화면](validation/manual_gui_2026-09-16/08_pan_drag.png) |
| right/middle drag | 아니오 | NOT TESTED | Computer Use sky.drag API에 mouse button 지정 인수가 없어 해당 입력 자체 자동 검증 불가 |
| zoom | viewport mouse wheel | PASS | 확대와 축소에 따른 실제 장면 크기 변화 [화면](validation/manual_gui_2026-09-16/09_wheel_zoom.png) |
| Reset camera | 클릭 | PASS | orbit/pan/zoom 이후 기본 방향과 전체 배치로 복귀 [화면](validation/manual_gui_2026-09-16/10_reset_camera.png) |
| Fit scene | 클릭 | PASS | Kerr 중심 근접 시점에서 전체 장면 fitting으로 변경 [화면](validation/manual_gui_2026-09-16/12_fit_scene.png) |
| Kerr center | 클릭 | PASS | Kerr 중심을 주시하는 근접 시점으로 변경 [화면](validation/manual_gui_2026-09-16/11_kerr_focus.png) |
| Source focus | 클릭 | PASS | SourcePlane 중심과 발사 grid를 확대하여 표시 [화면](validation/manual_gui_2026-09-16/13_source_focus.png) |
| Detector focus | 클릭 | PASS | DetectorPlane 중심과 hit 분포를 확대하여 표시 [화면](validation/manual_gui_2026-09-16/14_detector_focus.png) |
| rendered trajectory count | 숫자 입력 | PASS | 256→32→512 확인, 재개 후 512→256 복귀. physical 4096 / D3502 C330 E264 F0 유지 [화면](validation/manual_gui_2026-09-16/37_rendered_256_restored.png) |
| line thickness | 숫자 입력 | PASS | 1.35→4.00 px로 실제 경로 선이 굵어짐; 물리 count 유지 [화면](validation/manual_gui_2026-09-16/17_thick_trajectories.png) |
| grid auto-fit | 체크박스 OFF/ON | PASS | 수동 범위와 자동 범위 간 격자 크기 변경, 계산 진행 표시 없음 [화면](validation/manual_gui_2026-09-16/20_grid_auto_fit_restored.png) |
| grid extent/spacing | 숫자 입력 | PASS | extent 200→100, spacing 25→10에서 범위/밀도 변화 [화면](validation/manual_gui_2026-09-16/19_grid_spacing_10.png) |
| frame dragging toggle | 체크박스 OFF/ON | PASS | field 화살표만 소거/복귀, 동일 카메라에서 ray 경로 유지 [화면](validation/manual_gui_2026-09-16/22_frame_dragging_on.png) |
| 3D field toggle | 체크박스 ON/OFF | PASS | z=0 단면과 부피 sampling 표시의 분명한 차이 확인 [화면](validation/manual_gui_2026-09-16/24_frame_dragging_clean.png) |
| detector mode | 드롭다운 선택 | PASS | Accumulated 3502 / Cumulative(t=180) 2472 / Instantaneous(dt=1) 82. dt=2에서는152. Static/Propagation/Detector 선택 확인 [화면](validation/manual_gui_2026-09-16/28_detector_instantaneous_dt2.png) |
| playback 중 camera 조작 | orbit + Pan drag + wheel | PASS | 재생을 멈추지 않고 orbit t33.455, pan t131.330, zoom t173.398. 마지막 누적 hit1426, GUI 응답 유지 [화면](validation/manual_gui_2026-09-16/34_playing_zoom.png) |
| trajectory 없는 과거 run | 기존 저장 run 실제 창 열기 | PASS | physical256 / rendered0 / D216 C20 E20 F0 / TRAJECTORY UNAVAILABLE / detector shown216, crash 없음 [화면](validation/manual_gui_2026-09-16/38_legacy_trajectory_unavailable.png) |

## 시간·계산 분리에서 직접 확인한 범위

- 화면의 `t_BL / M`, `Physical M / wall s`, `loaded f64 trajectories; stored stride 1` 표시를 확인했다.
- Pause 중에도 창의 프레임 표시는 갱신되지만 물리시간은 고정됐다. slider로 물리시간과 장면을 함께 탐색했다.
- 재생 속도 25→5를 GUI에서 변경하고 느린 시간 진행 상태로 camera 동시 조작을 했다.
- Δt 1→1.2→2 변경 시 같은 t=180에서 instantaneous hit가 82→98→152로 변했다.
- 표시 설정을 바꾸는 동안 physical rays4096, 전체 hit3502 및 상태 count가 유지됐고,
  계산 spinner/진행 메시지가 나타나지 않았으며 상태줄은 저장 f64 데이터 로드 완료 상태였다.
- 이는 GUI 동작의 확인이며 실제 시간 적분 공식이나 보존량을 이번에 재검증한 것은 아니다.

## GUI 버그와 최소 수정 이력

### 앞선 실조작에서 발견·수정한 버그

1. 재현: viewport 빈 부분에서 왼쪽 버튼 드래그를 두 번 수행해도 시점이 변하지 않았다.
2. 원인: camera가 egui의 frame 단위 dragged/drag_stopped 플래그와 pointer.delta에 의존해,
   빠른 press/move/release가 한 프레임에 들어오는 드래그를 놓칠 수 있었다.
3. 최소 수정 파일: `src/visualization/app.rs` 한 파일.
   viewport에서 시작한 pointer button/position을 유지하고 원본 입력 이벤트 순서대로 이동을 처리한다.
   물리 엔진, metric 기반 field, detector, 저장 형식은 수정하지 않았다.
4. 해당 viewer만 `cargo build --release --offline --bin kerr-viewer`로 빌드했고 exit0, 32.96초였다.
5. 같은 Computer Use 왼쪽 드래그를 다시 보내 camera 회전을 확인했다. Pan drag 및 재생 중 camera도 확인했다.
   수정 전 [화면](validation/manual_gui_2026-09-16/05_orbit_drag.png),
   수정 후 [화면](validation/manual_gui_2026-09-16/07_orbit_after_fix.png).

### 이번 재개 구간

- 새 버그 없음. 앞선 app.rs 수정과 성공한 빌드를 보존했다. 추가 소스 수정이나 재빌드 없음.
- cargo test, clippy, fmt를 재실행하지 않았다. 59개 시험의 과거 결과를 이번 GUI 수정 이후 새 시험 결과로 주장하지 않는다.
- 전체 물리 계산이나 새 저장 데이터 생성 없음.
- 처음 Computer Use 창을 열 때 실행 파일 옆에 이전 `--measure` 시작 옵션이 남아 있어
  자동 측정 모드가 잠시 시작됐다. 즉시 창을 종료하고 그 인수를 제거했다. 해당 실행을 검증/성능 결과로 사용하지 않았다.
  이후 모든 창은 --input 저장 결과 로드만 사용했으며 벤치마크를 다시 돌리지 않았다.
- `target/release/kerr-viewer.args.json`은 마지막에 저장4096 run을 여는 인수로 복귀시켰다.
  소스/실험 설정과 기존 결과 파일을 바꾼 것이 아니다. 과거 자동 시작 인수는 work/viewer_args_before_manual.json에 보존했다.

## 증거와 남은 제한

총 PNG 파일에는 중간 상태·수정 전 재현도 포함되어 있다. `05_orbit_drag.png`는 실패 재현이고 성공 증거가 아니다.
주요 증거:

- [Propagation 재생 중](validation/manual_gui_2026-09-16/34_playing_zoom.png)
- [Kerr center](validation/manual_gui_2026-09-16/11_kerr_focus.png)
- [Source focus](validation/manual_gui_2026-09-16/13_source_focus.png)
- [Detector focus](validation/manual_gui_2026-09-16/14_detector_focus.png)
- [Frame dragging 3D](validation/manual_gui_2026-09-16/24_frame_dragging_clean.png)
- [Frame dragging 단면](validation/manual_gui_2026-09-16/25_field_slice.png)
- [512 trajectories](validation/manual_gui_2026-09-16/36_rendered_512_midtime.png)
- [Trajectory unavailable](validation/manual_gui_2026-09-16/38_legacy_trajectory_unavailable.png)

오른쪽/가운데 버튼을 누른 드래그는 API가 제공하지 않아 **NOT TESTED**다.
Pan drag + 왼쪽 버튼 대체 조작은 독립적으로 **PASS**이며, 이것으로 right/middle 입력 자체가 검증됐다고 보지 않는다.

## 시각화 단계 종료 판단

요청한 GUI 자동조작 검증 작업은 이 보고로 종료할 수 있다.
기능·대체 pan 조작을 포함한 확인 가능 항목은 모두 PASS이며 남은 입력 한 가지는 명시적으로 NOT TESTED다.
따라서 **right/middle drag 미확인 제한을 인정하는 조건으로 시각화 단계를 닫을 수 있다.**
모든 mouse button 입력이 전수 검증됐다는 의미의 무조건 완료는 아니다.
GPU physics, ML, 추가 시각효과나 대규모 리팩터링은 시작하지 않았다.
