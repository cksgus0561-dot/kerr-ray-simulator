# 후속 요청 완료 조건 대조표

2026-09-15. 붙여 넣은 후속 요청 §29의 20개 조건을 저장된 구현·시험·실행 자료와 대조했다.
현재 재개 작업은 구현을 다시 만들지 않고 미완료 문서·최종 확인과 두 보고 항목만 보완했다.
**20/20 완료.** GPU/GUI/머신러닝은 이번 완료 조건에 포함하지 않는다.

| 번호 | 완료 조건 | 근거 | 상태 |
|---|---|---|---|
| 1 | 기존 18개 시험 계속 통과 | 원본 파일 SHA-256 및 baseline ZIP 일치; debug_final/release_final 로그 | 완료 |
| 2 | 일반 3차원 Kerr 영측지선 | physics/geodesic.rs의 8변수 RHS; three_dimensional.rs | 완료 |
| 3 | theta 운동 | 일반 18광선에서 theta 변화>.01 assertion | 완료 |
| 4 | 일반 Carter Q 검증 | 비적도 탈출18·포획9, Q 절대/상대 오차; PHYSICS_VALIDATION | 완료 |
| 5 | 일반 3D ZAMO 방향 | tetrad::ray_3d, 정규직교·null·비단위 입력 거부 시험 | 완료 |
| 6 | SourcePlane | 네 배치 모드와 Emission별 방향·t_emit 시험 | 완료 |
| 7 | DetectorPlane | 기저·root·범위·가장자리·최초흡수 시험 | 완료 |
| 8 | 연속 t_hit | 스텝 내부 재적분 root; 정확 Schwarzschild 도착시간·수렴 시험 | 완료 |
| 9 | delta_t | 원본216행에서 t_hit-t_emit 일치; 발사시각 이동 시험 | 완료 |
| 10 | detector_events.csv | results/source_to_detector/detector_events.csv, 216개 원본 이벤트 | 완료 |
| 11 | 누적 수치 자료 | detector_accumulated.csv, u64 count 합216 | 완료 |
| 12 | 누적 PNG | 128×128 실제 PNG; 원본 count와 모든 픽셀 일치 | 완료 |
| 13 | 시간분해 수치 자료 | detector_time_bins.csv + frame_index.csv, Δt1/.1 검증 | 완료 |
| 14 | instantaneous frame | 기본81장, rebin805장; 전수 픽셀 검사 | 완료 |
| 15 | cumulative frame | 기본81장, rebin805장; 전수 픽셀 검사 | 완료 |
| 16 | Δt 변경 후 재적분 없이 재생성 | rebin CLI 시험; 저장된 source_to_detector_dt01 원본 checksum 일치 | 완료 |
| 17 | source_to_detector 실제 실행 | 256광선, Detected216/Captured20/Escaped20/실패0 | 완료 |
| 18 | README 갱신 | 3+1·면 좌표·시간·실행·수정 지점·한계 | 완료 |
| 19 | PHYSICS_VALIDATION 갱신 | 37개 시험·오차/허용량·회귀·검출·저장 감사·미지원 범위 | 완료 |
| 20 | 기존 실험 유지 | 기존5개 필수 실험+기존3개 scan 유지; 19개 회귀 fixture | 완료 |

## 나머지 요청의 확인

- 원본 이벤트와 hit 상태/진단을 분리했다. 모든 그림은 원본 count의 파생 출력이며 spin 텍스트를 그리지 않는다.
- 모든 요청 설정은 SourceDetectorExperiment 또는 JSON에서 직접 수정한다. CLI는 주요 옵션을 노출한다.
- run_metadata에 단위·좌표·시간·면·방향·t_emit·적분·버전·시각·상태·오차를 저장한다.
- 누락된 신규 실행 experiment_id를 보완하고 run_started와 run_metadata의 일치를 기존 신규 CLI 시험에서 검사했다.
- 오래된 벤치마크 JSON의 max_null 숫자 누락만 고쳤다. 원본 벤치마크는 보존하고 별도 폴더에 재측정했다.
- 두 시간 영상은 PNG sequence와 APNG로 실제 출력된다. FPS는 물리 Δt와 분리되어 있다.
- CPU 병렬화는 선택 항목으로 이번에는 추가하지 않았다. f64 reference와 보존량 검사는 유지했다.
- fmt, clippy --all-targets -- -D warnings, test, test --release 모두 exit 0. 새 code 변경 후 최종 로그를 다시 저장했다.

상세 수치는 [PHYSICS_VALIDATION.md](PHYSICS_VALIDATION.md),
성능은 [BENCHMARK.md](BENCHMARK.md), 19개 최종 보고 항목은 [FINAL_REPORT.md](FINAL_REPORT.md)를 참조한다.
