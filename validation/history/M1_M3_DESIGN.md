# 설계와 단계별 범위

## 이번 작업

1. Milestone 1: 일반 Boyer–Lindquist Kerr 계량/역계량과 단위시험.
2. 위 시험 통과 후 Milestone 2: 적도면만의 f64 해밀턴 영측지선, ZAMO 초기조건,
   적응형 Dormand–Prince 5(4), 종료 상태와 독립적인 물리 검증.
3. 위 시험 통과 후 Milestone 3: 학생이 수정하는 실험 함수, 관측량, CSV와 CLI.

## 의존성 방향

`experiments → compute → physics`; 향후 `app → ui/rendering`을 추가한다.
렌더러의 입력은 불변 `PhysicsResult`이며 물리 엔진은 그래픽 라이브러리를 모른다.
`RayIntegrator`는 초기조건/설정/결과를 공유하는 작은 경계다.
CUDA, Maxwell 장, 동적 중력장은 이번 구현에 포함하지 않는다.

## 수학 규약

G=c=M=1, chi=a, signature (-,+,+,+), 좌표 순서 (t,r,theta,phi).
p_mu는 공변 정준 운동량, lambda는 affine parameter, E=-p_t, Lz=p_phi.
전체 위치의 계량을 시험하지만 운동 상태는 theta=pi/2, p_theta=0으로 제한한다.
적도면은 Kerr의 반사 대칭에 따른 불변 부분공간이다.
계량으로부터 해석적으로 계산한 반지름 미분으로 Hamilton 방정식을 푼다.
유한차분은 미분식의 시험에만 사용한다.

## 향후 경계

- Milestone 4: 독립 광선 CPU 병렬화와 측정.
- Milestone 5–6: wgpu/egui, 카메라와 계산 요청 분리, 작업 스레드,
  generation ID, 입력 debounce, 미리보기/정밀 계산.
- Milestone 7: wgpu RK4 compute와 CPU 비교. GPU 선택 시 discrete NVIDIA 우선,
  실제 adapter/backend/features/limits 출력. 검증 전 GPU 결과는 기준 해가 아니다.
- Milestone 8: 적도면 검증 후 theta 운동, Carter Q와 3차원 좌표 구현.

카메라/표시/재생 변경은 기존 궤적만 사용한다. 물리 요청만 재계산한다.
BL x=r cos(phi), y=r sin(phi)는 좌표 그림이며 공간의 유클리드 임베딩이 아니다.
