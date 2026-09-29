# 설계와 현재 범위

## 보존한 기준점과 확장

Milestone 1–3의 계량, CPU f64 DP5(4), ZAMO, CLI 실험을 보존했다.
사용자의 후속 요청에 따라 GUI/GPU보다 일반 3+1차원과 검출 데이터 출력을 먼저 확장했다.
3차원 계산이 있다는 이유로 원래 Milestone 4–8 전체를 완료했다고 간주하지 않는다.
CPU 병렬화·실시간 UI·GPU 구현은 아직 없다.

## 책임과 의존성

- `physics`: BL 계량, 해석적 미분, 8변수 해밀턴 RHS, ZAMO, DP5(4), 보존량, 좌표 변환.
- `detector`: `physics` 상태를 읽어 좌표면·교차·HitEvent·시간 bin을 정의한다.
- `compute`: CPU 적분을 실행하고 선택적 검출 이벤트를 적용한다. 상태·종료 원인·진단을 반환한다.
- `experiments`: 초기조건, 매개변수와 실행을 정의하며 compute/detector/output을 조립한다.
- `output`: HitEvent와 count를 읽어 원본 CSV, 수치 배열, PNG/APNG와 metadata를 출력한다.
- `main`: CLI 라우팅. source_to_detector와 rebin, 기존 실험 인터페이스를 유지한다.

핵심 물리는 output이나 렌더러를 호출하지 않는다. 후처리는 원본 HitEvent만으로 재실행된다.
검출기는 계산 중 사건을 판정하지만 그림은 물리 상태를 수정하지 않는다.

## 핵심 자료구조

`SourceDetectorExperiment` 하나에서 spin/source/detector/integration/postprocess/output을 수정한다.
광선마다 독립적인 State, Diagnostics, PhysicsResult를 사용한다. RayIntegrator는 작은 backend 경계이며
CPU의 detector 지원은 integrate_with_detector()로 제공한다. 향후 GPU에는 동일 사건·결과 계약도 필요하다.
레거시 State 슬롯 6개를 보존한 뒤 theta,p_theta를 추가했다. 외부 코드는 from_bl()·접근자를 사용한다.

## 이벤트와 원본 수명

1. 초기조건·면 domain을 검증하고 run_started.json을 저장한다.
2. 각 광선을 적분하며 최초 유효 검출의 연속 t와 운동량을 기록한다.
3. detector_events.csv를 매 hit마다 flush하고 마지막에 sync_all한다.
4. 모든 광선 진단 및 run_metadata.json을 저장한다.
5. 원본 이벤트에서 공간 histogram과 시간 bin을 계산한 뒤 PNG/APNG를 출력한다.

출력 파일은 create_new로 생성한다. 부분 출력 실패가 생겨도 원본을 덮어쓰지 않는다.
완료 원본이 있으면 새 디렉터리로 rebin한다. 소스 데이터 checksum이 다르면 rebin을 거부한다.
이벤트/스텝은 f64, count는 u64, PNG는 명시적으로 매핑한 u16이다.
시간×공간 전체의 dense 배열 대신 희소 bin과 두 장의 count 버퍼로 프레임을 순차 출력한다.

## 좌표·시간 경계

G=c=M=1, signature (-,+,+,+). BL exterior chart만 지원한다.
Source/Detector는 T=t_BL를 유지한 oblate Cartesian-like 좌표에서 시간에 따라 유지되는 고정 사각형이다.
유클리드 기저는 좌표면을 정의할 뿐 실제 proper distance·area를 뜻하지 않는다.
교차 root의 독립변수는 affine parameter이며 t_hit도 그 상태에서 평가한다.
프레임 번호나 APNG FPS로 t_hit를 추정하지 않는다. 자세한 식과 한계는 README에 있다.

## 다음 단계

먼저 같은 광선 집합의 순차/병렬 결과·상태·진단 일치를 검증하며 CPU 작업 풀을 추가할 수 있다.
wgpu/egui에서는 UI와 CPU/GPU 작업을 분리하고 generation ID로 오래된 결과 반영을 막는다.
카메라·재생·표시 변경은 불변 PhysicsResult를 읽고 재적분하지 않는다.
GPU는 ZAMO 초기조건부터 CPU f64와 비교하고 h,h/2,h/4 수렴, 동일 affine 및 hit 사건을 검증한다.
실측 전에 f32 허용오차, FPS, CUDA 성능 이점을 가정하지 않는다.
정칙 좌표계, 관측자 proper time/area, 검출 에너지 가중치는 별도 물리 확장이다.
