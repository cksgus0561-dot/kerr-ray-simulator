# 물리 검증 기록

2026-09-15, Rust 1.98.1, Windows x86_64 MSVC. **그림은 검증 기준으로 사용하지 않았다.**
Milestone 1, 2, 3 각각 fmt/clippy/test/test --release 통과 후 진행했다.
최종 시험은 18개(적분기 2, 계량 3, 영측지선 9, 실험/CLI 4)이며 debug/release 모두 통과했다.

## 재현

```powershell
. .\env.ps1
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test -- --nocapture
cargo test --release -- --nocapture
```

구체적인 수치와 assertion은 `tests/metric.rs`, `tests/geodesics.rs`, `tests/experiments.rs`에 있다.
아래 측정값은 debug와 release 중 더 큰 값(표시 자릿수 반올림)이다.
부동소수점 연산 순서 때문에 두 빌드가 완전히 같은 숫자를 주지는 않는다.

## 계량과 국소 기준계

| 검증 | 시험 영역/정의 | 허용오차 | 측정 |
|---|---|---|---|
| g_mu alpha g^alpha nu = identity | a=0,.2,.7,.99; r=r_plus+.01,3,10,10⁴; theta=.1,.7,pi/2,2.7,3.04 | 최대 절대 2e-11 | 2.274e-13 |
| Schwarzschild 극한 | a=0; r=2.1,3,10,10⁶; theta=.3,1.2,2.9; 모든 성분과 역성분 | 5e-14*max(1,성분 절댓값) | 통과 |
| 지평선/에르고면 | Delta(r_plus)=0, 적도 에르고면=2, 극에서 에르고면=지평선 | 2e-15 / 1e-15 | 통과 |
| 미분식 | 해석적 partial_r g^mu nu vs 독립 5점 중앙차분, h=1e-4*r | 상대/바닥 1e-8 척도에서 2e-7 | 1.417e-10 |
| ZAMO 정규직교성 | 에르고영역 포함 r=r_plus+.05,5,100, theta=.3,pi/2,2.4 | 절대 2e-12 | 3.553e-14 |
| 국소 광자의 영조건 | 5개 방향, a=0,.7,.99; 미래 방향 dt/dlambda>0 | 절대 2e-12 | 통과 |

## 실제 해밀턴 적분과 알려진 결과

계량 부호 (-,+,+,+), G=c=M=1. default rtol=1e-12, atol=1e-14,
epsilon=1e-3, escape=100, max_step=.5. 개별 시험에서 바뀌는 값은 소스에 명시되어 있다.

| 검증 | 방법 | 허용오차 | 측정 |
|---|---|---|---|
| DP5(4) 차수 | y'=y를 λ=1까지 h=.2/.1로 계산 | fine 절대 <1e-8, 오차비 >20 | 통과 |
| 광자 원궤도 | a=0,.5,.9,.99, 양 방향, ZAMO 접선 발사; λ=10까지 실제 적분 | 초기 abs(dr)<1e-13, abs(dp_r)<2e-11; 최대 abs(r-r_ph)<2e-7 | 반지름 1.475e-11 M; null 3.944e-13 |
| 원궤도 해석적 극한 | a=0에서 양쪽3; a=1 공식에서 pro1/retro4; 1-1e-8 근방 접근 | 정확 극한 1e-14, 근방 pro<1.0002 / retro>3.9999 | 통과 |
| 임계 b | Schwarzschild b=±(3sqrt(3)±.001), r_source=r_escape=50 | 양방향 안쪽 Captured/바깥 Escaped | 경계 b=5.196152422706632 ±.001 내 |
| a=0 반사 대칭 | ±b 결과 r와 phi 비교 | r 1e-9, phi 합 1e-7 | 통과 |
| Schwarzschild 포획 null | 위 경계 광선의 모든 승인 상태 | 절대 1e-5 | 2.581e-9 |
| E, Lz 보존 | 전체 시간발전, 초기값과 비교 | 해당 검증에서 정확 0 | 0; Hamilton 미분 자체가 0 |
| Carter Q | 적도면 축소 상태 | 정확 0 | 0; 일반 3D 검증 아님 |
| 독립 Kerr 반지름 퍼텐셜 | R=[E(r²+a²)-aLz]²-Delta(Lz-aE)², r⁴(dr/dλ)²와 비교 | 차이/(r⁴ E²)<2e-10 | 1.193e-12 |
| 허용오차 수렴 | a=.7,b=6,r_source=r_escape=40,max_step=8; rtol=1e-6/1e-9 vs 1e-12 | fine phi<2e-7이고 coarse의1/20 미만 | 2.548e-7 → 8.240e-10 rad |
| 먼 거리 직선 극한 | r=10⁶, a=0/.99, λ=1000; 시작 접선 직선과 비교 | 변위/λ<2e-8 | 1.312e-10 |
| 약한 편향 | Schwarzschild b=1000, r_source=r_escape=10⁶ | 4/b와 상대 차이 .006 미만 | 0.004011823816 rad; 상대 .002956 |
| 프레임 끌림 | Lz=0, a=.8,r=8에서 dphi/dt=2ar/A | 절대1e-15, 양의 phi 진행 | 통과 |

광자 반지름 식:
`r_pro=2[1+cos((2/3)acos(-a))]`, `r_retro=2[1+cos((2/3)acos(a))]`.
공식 극한 함수만 a=1을 허용하며 실제 `Kerr::new()` 적분은 a<=.99로 제한한다.
광자 원궤도는 불안정하므로 λ=10 시험이 임의로 긴 시간의 원궤도 안정성을 보장하지 않는다.
약한 편향에서의 차이는 고차 중력항·유한 반지름·수치 오차를 포함하며 모두 적분 오차로 해석하면 안 된다.

## 예제, 상태와 경계

- 8개 기본 시나리오, 총 222광선: 모두 Captured 또는 Escaped, NumericalFailure 0.
  최대 |C_null|=9.6392e-8, 시험 문턱 2e-6.
- 높은 스핀 a=.99, b=-6, epsilon=.01/.003/.001:
  모두 Captured, 모든 승인 r>r_plus, 종료 경계 오차 <3e-11 M.
  최대 |C_null|=3.8464e-7, 시험 문턱 2e-6.
- 수치 실패 시험: 비영 초기 상태, NaN, 비유한 RK 미분/오차,
  스텝 한도 초과가 정상 탈출로 처리되지 않는다.
- 유한 max_affine 종료는 Active+AffineLimit. CSV 실패 행과 CLI 종료 코드 1까지 시험한다.
- 임계 b 스캔에서는 표본 해상도가 경계 정확도를 제한한다. 기본 CSV의 간격 .02와
  자동 시험의 ±.001 경계 검증은 서로 다른 해상도다.

## 오차와 보존의 해석

최대값은 승인된 수치 상태에서 측정한다. 스텝 내부의 연속 최대, 임의 초기조건 전체,
지평선 내부 또는 임의로 작은 epsilon에 대한 보장은 없다.
E/Lz의 0 오차는 시간·방위각 대칭을 해밀턴 방정식에 그대로 적용한 결과이다.
이 보존만으로 올바른 궤적임을 주장하지 않고 null, 독립 퍼텐셜, 알려진 궤도와 수렴을 함께 시험했다.
Q는 상태공간에서 정확하게 0이며 비적도 운동 검증은 미완료다.

지평선 근처 BL 좌표에서는 큰 시간/운동량 항의 상쇄가 발생한다.
허용오차를 줄여도 f64 반올림 오차가 지배할 수 있다. epsilon을 바꿀 때는
포획 판정과 관심 관측량의 수렴, null 오차를 다시 확인해야 한다.
실패 문턱 `|C_null|/E_local0²=1e-5`는 허용하는 최악의 한계이며,
실측 오차나 정확도 보장값을 의미하지 않는다. 필요하면 `--null-tolerance`로 더 엄격하게 설정한다.

## CPU/GPU 비교 — 아직 미완료

GPU 구현이 없으므로 비교 결과는 없다. 다음을 통과하기 전 GPU를 기준 해로 취급하지 않는다.

1. 같은 ZAMO 초기조건을 f32로 변환한 오차를 별도 측정.
2. CPU/GPU를 같은 affine parameter에서 비교; 이벤트 상태와 불확정 경계 구분.
3. captured/escaped, r/phi, null/E/Lz 오차를 함께 기록.
4. h, h/2, h/4 수렴 및 임계/고스핀/지평선 근방 사례.
5. f32의 실측 허용범위를 문서화하고 초과 결과를 NumericalFailure로 표시.

GPU 오차 허용수치는 구현 전 임의로 만들지 않는다. 전체 3D Q, Kerr–Schild,
장시간 임계 궤도, GPU 정밀도 및 UI 성능 검증도 남아 있다.

## 수식 출처

- [Visser, The Kerr spacetime: A brief introduction](https://arxiv.org/html/0706.0622v3): 좌표, 계량, 지평선/에르고면.
- [Su et al., Photon emissions from Kerr equatorial geodesic orbits, 2024](https://link.springer.com/article/10.1140/epjc/s10052-024-13113-w): 독립 반지름 퍼텐셜과 보존량.
- [SciPy RK45 공식 문서](https://docs.scipy.org/doc/scipy/reference/generated/scipy.integrate.RK45.html): Dormand–Prince 5(4) 및 원 논문 서지.
