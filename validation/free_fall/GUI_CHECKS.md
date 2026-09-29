# Free-fall grid: 실제 Windows GUI 확인

2026-09-17. Computer Use `@oai/sky`로 release kerr-viewer 창을 직접 클릭,
입력, 드래그, 스크롤했다. 코드만 읽은 결과를 GUI PASS로 처리하지 않았다.
GPU: NVIDIA GeForce RTX 3060 Ti / Vulkan / DiscreteGpu.

시작 데이터는 기존 `results/visualization_4096`이다. 기본 4096 physical rays,
256 rendered trajectories, chi=0.6, D3502/C330/E264/NUM0을 확인했다.

| 실제 조작 | 결과 | 관찰 |
|---|---|---|
| Cartesian Reference Grid toggle | PASS | 고정 직선 격자 표시/숨김 |
| Kerr Free-Fall Grid toggle | PASS | 녹색 lattice 표시/숨김, 숨겼다 다시 켜도 캐시 재사용 |
| Frame Dragging toggle | PASS | 기존 화살표만 표시/숨김; 자유낙하 격자와 독립 |
| 표시 조합 | PASS | 각 단독, Cartesian+Free-Fall, Free-Fall+Frame Dragging, 모두 ON/OFF 확인 |
| Play/Pause | PASS | lattice와 격자 시간이 진행; Pause 후 시간·형태 유지 |
| Reset | PASS | 전체 lattice와 격자 시간 0 복원, 캐시 생성 횟수 유지 |
| Speed | PASS | 3.0→0.5 M/wall s 입력 후 느린 재생, 캐시 생성 횟수 유지 |
| 재생 중 orbit | PASS | 마우스 drag로 시점 변화, 격자 시간 계속 증가 |
| 재생 중 pan | PASS | Pan drag 모드와 마우스 drag로 화면 이동, 재생 유지 |
| 재생 중 zoom | PASS | 휠로 크기 변화, 재생 유지 |
| extent 변경 | PASS | 10→12, 격자 시간 reset, cache builds 1→2 |
| density 변경 | PASS | 9→7, 총 node 729→343, cache builds 2→3 |
| chi 변경 | PASS | 장면 chi=0 반영 후 캐시 생성 횟수 3→4 |
| chi=0 | PASS | probe 164: t=0, r=4.00000 → t=4.308, r=2.70906; theta=1.57080, phi=-1.57080 유지; dphi/dt=0 |
| chi=0.6 | PASS | 격자 낙하·방위각 변형 확인; 정확한 각속도/방향은 별도 수치시험으로 검증 |
| 광선 계산과 분리 | PASS | toggle/재생/속도/범위/밀도/카메라에서 photon worker 진행 표시 없음, photon 시간 0 및 기존 detector count 유지 |

chi=0 GUI 확인을 위해 **물리 chi를 변경할 때만** 기존 정상 경로의 photon
계산이 한 번 실행됐다. 그 결과는 검증 전용
`validation/free_fall/gui_runs/simulation_1`에 자동 저장됐다.
이 실험의 D3488/C332/E276/NUM0은 chi=0 결과이며, chi=0.6 회귀 기준과 혼동하지 않는다.
기존 저장 결과는 덮어쓰지 않았다. 격자 조작은 이 계산/자동저장을 실행하지 않았다.

## 증거 이미지

- `chi06_initial.png`: chi=0.6 초기 격자.
- `chi06_paused_t30.png`: t=30.042에서 Pause.
- `chi06_playback_camera.png`: 재생 중 orbit/pan/zoom 후, t=16.159.
- `all_layers_on.png`: 3개 표시 동시 ON.
- `chi0_initial.png`: chi=0 초기 probe 및 metric 진단.
- `chi0_radial_t4.png`: 동일 probe가 반지름 방향으로 이동, 각속도 0.

## 성능 관찰과 검증 범위

실제 UI에 표시된 cache precompute 시간:
chi=0.6/extent10/density9: 0.202 s;
chi=0.6/extent12/density9: 0.259 s;
chi=0.6/extent12/density7: 0.114 s;
chi=0/extent12/density7: 0.104 s.
검증 장면에서는 UI FPS 약 239.6~239.9를 관찰했다. 이는 정식 benchmark나
모든 밀도/하드웨어의 성능 보장이 아니다.

직접 right/middle-button drag는 수행하지 않았다. pan은 GUI의 Pan drag 모드로
검증했다. 음수 chi는 기존 물리 API에서 지원하지 않으므로 검사 대상이 아니다.
이번 조작 중 새 GUI 버그는 발견하지 않았고 추가 소스 수정은 없었다.

전체 수치시험과 회귀 로그는 같은 폴더의 `initial_tests.txt`, `debug_tests.txt`,
`release_tests.txt`, `clippy.txt`, `reproduce_256.txt`, `reproduce_4096.txt`에 있다.
