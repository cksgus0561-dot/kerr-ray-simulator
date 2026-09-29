# 시각화 단계 완료 조건 대조 — v0.3

2026-09-15. 현재 코드·이전 실행·저장 로그만으로 확인했다. 이번에는 새 기능·시험/계산 재실행 없음.
상태는 완료 / 제한 있음 / 미완료다. 구현 존재와 실제 GUI 조작 검증을 구분한다.
과거 source-to-detector 20개 조건은 [v0.2 이력](validation/history/V02_COMPLETION_CHECKLIST.md)에 보존한다.

| 요구사항 | 상태 | 확인 근거 또는 남은 점 |
|---|---|---|
| 기존 37개 + 신규 22개 시험 | 완료 | debug/release 실제 로그 각각 59/59 |
| fmt / clippy | 완료 | 기존 최종 성공 로그 |
| wgpu + egui 실행 / NVIDIA 선택 | 완료 | RTX 3060 Ti Vulkan 실제 창/측정 JSON |
| 직선 grid / auto-fit / manual 설정 / axes | 완료 | 구현·grid/bounds 시험·scene 표시. 모든 수동 drag 조합을 검증한 것은 아님 |
| Kerr 중심 / +Z spin / horizon / ergosphere | 완료 | scene, spin별 정확 반지름 시험 |
| SourcePlane / DetectorPlane / launch grid | 완료 | 기존 면 기저 재사용, orientation 시험, 실제 표시 |
| 4096 기본 physical / 256 기본 rendered | 완료 | 공유 설정, 저장 결과, GUI, 분리 시험 |
| 16×16 회귀 preset 유지 | 완료 | 216/20/20; 과거 176개 파일 동일 |
| 256/512 subset, 전체 detector count 유지 | 완료 | 실제 측정·GUI subset 변경·자동시험 |
| 실제 3D 경로 / ray 상태 / 선택 강조 | 완료 | 저장 경로·상태·ID 검사 및 실제 창 |
| 재사용 trajectory 저장 / load | 완료 | f64 gzip, metadata, roundtrip, 4096 load |
| trajectory 없는 legacy 처리 | 완료 | rendered 0, unavailable, 216 events 이미지 |
| frame dragging / 밀도 설정 | 완료 | 기존 metric, chi=0/finite 자동시험, 실제 화살표와 ON/OFF 측정 |
| 단면 및 3D field 모든 GUI 조합 | 제한 있음 | 설정/geometry 있음. 3D 고밀도 조합 전수 수동 시험 아님 |
| Play / propagation / cumulative 시간 진행 | 완료 | 실제 t=210.950, shown3276/3502 캡처 |
| accumulated / instantaneous / cumulative 수치 | 완료 | 원본 continuous events와 bin 자동시험 |
| Pause / Stop / Reset time / time slider / speed의 최종 수동 조작 | 제한 있음 | 구현·logic 시험 있음, 버튼별 마지막 수동 확인 미완료 |
| camera orbit/pan/zoom/reset/focus/fit 전수 수동 조작 | 제한 있음 | Kerr focus·zoom·fit·측정 중 이동 확인; 일부 입력과 Source/Detector focus 미확인 |
| 공통 code/JSON/CLI/GUI 설정 | 완료 | SessionConfig, physics key·JSON 시험 |
| 표시 조작 시 재적분 없음 | 완료 | key 제외·자동시험·실측 cache 유지 |
| worker / generation ID | 완료 | 실제 경계 연결·최신 요청/cancel 시험. 단일 ray/load 즉시 중단은 아님 |
| trajectory GPU cache / batch | 완료 | moving camera 측정 시 ray upload 고정 |
| screenshot | 완료 | 실제 window PNG 생성 |
| detector/config export 모든 GUI 버튼 | 제한 있음 | 파일/roundtrip 검증, 모든 버튼 수동 검증 아님 |
| detector PNG/APNG/frame sequence | 완료 | 기존 출력 구조와 저장 결과 유지 |
| 3D video/frame sequence | 미완료 | 현재 미구현, 전체 창 screenshot은 있음 |
| 실제 FPS / field/overlay ON/OFF | 완료 | 세 ray 수, 일곱 조건 원자료. vsync cap 제한 |
| GUI 전체 startup / uncapped GPU 시간 | 제한 있음 | scene load만 별도 계측, process+adapter+first-present 합산은 없음 |
| 16,384 / 65,536 ray 대규모 실행 | 제한 있음 | 설정 가능, 실측하지 않음 |
| README / 좌표 convention / 최종 성능 문서 | 완료 | README, VISUALIZATION, PHYSICS_VALIDATION, BENCHMARK, FINAL_REPORT |

전체 GUI 버튼 검증 완료를 주장하지 않는다. 남은 수동 검증은 [최종 보고 B/L](FINAL_REPORT.md)에 명시했다.
핵심 물리 재설계, CUDA, geodesic GPU compute, ML 등은 이번 완료 조건에 추가하지 않았다.
