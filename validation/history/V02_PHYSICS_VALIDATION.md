# 물리·검출·데이터 검증 기록

2026-09-15, Rust 1.98.1, Windows x86_64 MSVC, CPU f64.
**총 37개 시험, debug/release 각각 실패 0. 그림 모양은 검증 기준으로 사용하지 않았다.**

## 재현과 로그

```powershell
. .\env.ps1
cargo fmt
cargo clippy --all-targets --offline -- -D warnings
cargo test --offline -- --nocapture
cargo test --release --offline -- --nocapture
```

최종 실행 기록: [fmt](validation/fmt_final.txt), [clippy](validation/clippy_final.txt),
[debug](validation/debug_final.txt), [release](validation/release_final.txt).
rustup의 사용자 경로 canonicalize 경고는 로그에 남아 있으나 명령은 모두 exit 0이다.
컴파일러/clippy 진단을 억제한 것은 아니다. `--offline`은 vendor 사용을 확인하기 위한 옵션이다.
이전 단계 로그도 보존했다. 원격 CI는 실행하지 않았다.

## 기존 기준점 보존

| 시험 묶음 | 기존 | 신규 | 현재 |
|---|---:|---:|---:|
| physics/integrator.rs 단위시험 | 2 | 0 | 2 |
| tests/metric.rs | 3 | 0 | 3 |
| tests/geodesics.rs | 9 | 0 | 9 |
| tests/experiments.rs | 4 | 0 | 4 |
| tests/three_dimensional.rs | 0 | 6 | 6 |
| tests/detector.rs | 0 | 7 | 7 |
| tests/dataset_output.rs | 0 | 6 | 6 |
| 합계 | **18** | **19** | **37** |

기존 3개 시험 파일은 `tests/fixtures/baseline_test_hashes.json`의 SHA-256과 동일하다.
원본 적분기 파일도 M1–3 보관 ZIP과 byte 단위로 같다. 보존 결과는
[saved_results_audit.json](validation/saved_results_audit.json)에 기록했다.
M1–3 검증 문서는 [역사 기록](validation/history/M1_M3_PHYSICS_VALIDATION.md)으로 보존했다.
그 문서의 과거 오차·미구현 항목은 현재 결과와 구분한다.

## 수학 convention

G=c=M=1, signature (-,+,+,+), 좌표/계량 순서 (t,r,theta,phi), 공변 정준 운동량 p_mu.
H=1/2 g^mu nu p_mu p_nu, dx/dlambda=partial_p H, dp/dlambda=-partial_x H.
E=-p_t, Lz=p_phi, Q=p_theta²+cos²(theta)(Lz²/sin²(theta)-a²E²).
동일한 8변수 엔진에 theta=pi/2, p_theta=0을 주면 적도면 대칭이 유지된다.
내부 State 배열의 호환 순서와 표준 from_bl API는 README에 명시했다.

아래 수치는 최종 debug/release 중 큰 값이며 표시 자릿수를 반올림했다.
"허용오차"는 시험 assertion이고, "측정"은 이번 지정 사례에서 얻은 값이다.

## Kerr 계량·역계량·Schwarzschild 극한

| 검증 | 범위/방법 | 허용오차 | 측정 |
|---|---|---|---|
| g_mu alpha g^alpha nu=identity | a=0,.2,.7,.99; r=r_plus+.01,3,10,10⁴; theta=.1,.7,pi/2,2.7,3.04 | 최대 절대 2e-11 | 2.27374e-13 |
| a=0 계량·역계량 | r=2.1,3,10,10⁶; theta=.3,1.2,2.9 | 5e-14*max(1,성분 절댓값) | 통과 |
| 지평선·에르고면 | Delta(r_plus)=0, 적도 r_ergo=2, 극에서 r_ergo=r_plus | 2e-15 / 1e-15 | 통과 |
| 적도 반지름 미분 | 독립 5점 중앙차분 | 정규화 잔차 2e-7 | 1.41646e-10 |
| 일반 r/theta 미분 | a=0,.6,.99, 4개 위치, 모든 역계량 성분 | abs 오차 <3e-10+3e-7*abs(해석값) | 허용량 대비 최대 0.00364629 |
| ZAMO 정규직교성 | 에르고영역 포함 여러 r/theta/spin | 2e-12 | 3.55271e-14 |
| 일반 3D ZAMO 영조건·좌표 roundtrip | 3개 theta×4개 방향; 비단위 방향 거부 | null/r/theta/phi 각각 1e-13 | 통과 |

실제 계량 미분은 해석식이며 finite difference는 독립 시험에서만 사용한다.
ZAMO 초기조건 API의 자체 null 실패 문턱은 초기 국소 에너지로 정규화한 1e-10이다.

## 알려진 궤도와 기존 실험

| 검증 | 방법·허용오차 | 측정 |
|---|---|---|
| DP5(4) | y'=y, λ=1, h=.2/.1; fine 오차<1e-8, 오차비>20 | 통과 |
| 광자 원궤도 | a=0,.5,.9,.99, 순행/역행, λ=10; 최대 반지름 오차<2e-7 M | 9.58123e-13 M; null 3.65930e-13 |
| 해석적 원궤도 극한 | a=0 양쪽 3M; a=1 공식 pro=M/retro=4M, 오차<1e-14 | 통과 |
| 임계 충돌매개변수 | b_c=3sqrt(3)=5.196152422706632, ±b 양쪽 ±.001 | 안쪽 Captured, 바깥 Escaped |
| Schwarzschild 순행/역행 대칭 | r 차이<1e-9, phi 합<1e-7 | 통과 |
| 임계 포획 null | 절대<1e-5 | 5.17332e-9 |
| 독립 반지름 퍼텐셜 | 적도 R=[E(r²+a²)-aLz]²-Delta(Lz-aE)², 정규화 차이<2e-10 | 1.19639e-12 |
| phi 허용오차 수렴 | rtol1e-6/1e-9 vs 1e-12; fine<2e-7 및 coarse/20 미만 | 2.54835e-7 → 8.23860e-10 rad |
| 먼 거리 직선 극한 | r=10⁶, λ=1000, a=0/.99; 변위/λ<2e-8 | 1.31201e-10 |
| 약한 Schwarzschild 편향 | b=1000; 4/b와 상대 차이<.006 | 편향 .004011823816 rad; 상대 .002956 |
| 프레임 끌림 | Lz=0, a=.8,r=8; dphi/dt=2ar/A, 오차<1e-15 | 통과 |
| 기존 예제 실행 | 기존 8개+three_dimensional_ray, 223광선; null<2e-6 | 수치 실패 0, null 8.81192e-8 |
| 높은 스핀 cutoff 민감도 | a=.99,b=-6,epsilon=.01/.003/.001; null<2e-6 | **2.801244e-7** |

광자 반지름은 r_pro=2[1+cos((2/3)acos(-a))], r_retro=2[1+cos((2/3)acos(a))].
극한 공식 시험만 a=1을 허용하며 실제 Kerr::new는 a<=.99다.
기존 예제 시험의 출력 문구 "all 8"은 파일을 보존했기 때문에 그대로이고,
동적 목록에는 새 실험 1개가 추가되어 실제로 9개·223광선을 검사한다.
원궤도는 불안정하므로 짧은 λ=10 시험은 장시간 안정성 보장이 아니다.
약한 편향·직선 극한의 차이에는 유한 M/r 물리 효과도 포함된다.

## 일반 3차원 보존량·회귀·구면대칭

기본 rtol=1e-12, atol=1e-14, max_step=.5, epsilon=.001.
3D 탈출 시험은 a=0,.6,.99 × theta=.8,1.3,2.1 × p_theta 양방향 =18광선이다.
각 광선에서 실제 theta 변화>.01을 검사한다. 9개 포획 시험도 비적도 방향을 사용한다.

| 집합 | 최대 abs(C_null) | 최대 E 상대오차 | 최대 Lz 상대오차 | 최대 abs(Q-Q0) | 최대 Q 상대 척도 |
|---|---:|---:|---:|---:|---:|
| 3D 탈출 18광선 | 1.076167e-12 | 0 | 0 | 1.423218e-11 | 3.681988e-13 |
| 3D 포획 9광선, a=.99까지 | 8.388270e-9 | 0 | 0 | 1.276535e-12 | 개별 상대 상한 시험 없음 |
| 저장 source_to_detector 256광선 | 1.139370e-8 | 0 | 0 | 2.933121e-10 | 4.821477e-12 |

3D 탈출 assertion: null<1e-9, Q 절대<1e-7, Q 상대<1e-9, E/Lz=0.
3D 포획 assertion: null<2e-6, Q 절대<1e-7, E/Lz=0.
Q 상대 척도는 abs(Q-Q0)/max(abs(Q0),E_local0²)다.
E/Lz의 정확한 0은 Hamilton RHS가 대칭으로 0이기 때문이며 이 사실만으로 궤적을 검증하지 않는다.

기존 19개 최종 상태 fixture를 같은 새 엔진으로 적분했다.
최대 phi 차이 2.088172e-9 rad(허용 2e-7), t 차이 4.819441e-9(허용 1e-5),
r 차이<1e-8, affine 차이<1e-7, 모든 상태 분류 일치, theta와 p_theta 정확 유지.
부동소수점 연산 순서 때문에 과거 엔진과 bit 단위의 동일 궤적을 요구하지 않는다.

a=0에서 초기 위치·방향을 함께 0.57rad 회전한 시험:
회전시킨 최종 Cartesian 좌표 차이 2.276579e-11 M, t 차이 2.815171e-11 M,
각 허용 1e-7. 초기 총 각운동량 제곱 Q+Lz² 차이는 1e-10 미만이다.

## 면 좌표·시간·교차

SourcePlane/DetectorPlane은 T=t_BL를 유지한 다음 좌표의 정지 사각형이다.
X=sqrt(r²+a²)sin(theta)cos(phi), Y=sqrt(r²+a²)sin(theta)sin(phi), Z=r cos(theta).
이는 BL oblate Cartesian-like 표현이며 Kerr–Schild 또는 공간의 평평한 물리 계량이 아니다.
기저·넓이는 보조 좌표량이며 고유거리·면적당 플럭스를 나타내지 않는다.
사각형 전체가 보수적인 r>2 조건을 통과해야 정지 세계선이 timelike임을 보장한다.

연속 상태의 signed-distance 부호 변화에서 affine 구간을 이분 탐색하고 재적분한다.
교차의 t_hit는 전체 상태에서 얻는 f64 BL 좌표시간, delta_t=t_hit-t_emit다.
흡수형 면의 첫 유효 횡단에서 종료한다. 면 바깥 교차는 계속 진행한다.

| 시험 | 허용오차/규칙 | 측정 |
|---|---|---|
| 면 기저·pixel·가장자리 | 정규직교 1e-12; 양쪽 경계 포함; 경계 외부 1e-10 거부 | 통과 |
| 먼 영역의 해석적 직선-평면 교차 | r≈10⁸; u,v<2e-6 M, t<1e-5 M | du=3.68449e-9, dv=8.40821e-11, dt=2.03835e-6 |
| Schwarzschild 반지름 방향의 정확한 도착시간 | t 차이는 r 차이와 2ln(r-2)의 해석적 지연; 오차<2e-9 | 2.17533e-11 |
| 발사시각 이동 | t_emit를 20 이동; t_hit도 20, 위치·delta_t 유지; 2e-8 | 통과 |
| 면 외부 통과·초기 면 위 | 외부는 미흡수, 초기 횡단은 affine=0 | 통과 |

detector 수렴 시험: coarse=(rtol1e-7,max_step4), fine=(1e-10,1),
reference=(1e-13,.25). 세 성분 각각 fine 오차<2e-7 및 coarse/5 미만을 요구한다.

| 관측량 | coarse-reference 최대 오차 | fine-reference 최대 오차 |
|---|---:|---:|
| u_hit | 3.882284e-7 M | **9.956481e-12 M** |
| v_hit | 5.244429e-7 M | **1.877707e-10 M** |
| t_hit | 7.194343e-7 M | **9.958399e-10 M** |

저장된 216개 hit의 r/theta/phi를 독립 변환해 재계산한 최대 면 잔차는
9.978862e-11 M으로 root tolerance 1e-10 이내다. root 잔차와 도착시간 수렴 오차는 다른 양이다.

## 이벤트·후처리 검증

인공 이벤트로 Δt=0.1/0.01, 정확한 bin 경계와 경계 전후, Arrival/Travel 시간 기준,
중복 ray_id, 비유한 시간, 면 바깥 이벤트를 검사한다.
반열린 구간 [start+kΔt,min(end,start+(k+1)Δt))을 사용한다.
정수 bin index에 대한 4*EPSILON*max(1,abs(index)) 경계 규칙도 검사하며 원본 시간을 바꾸지 않는다.

원본이 빈 경우·분석 창이 잘린 경우에도 명시적인 0 frame을 출력한다.
cumulative는 분석 창 이전 count를 포함하고, instantaneous는 해당 bin만 표시한다.
전체 창에서는 전체 bin count 합계=전체 이벤트 수다. 잘린 창에서는 before/window/after 합계를 검사한다.
원본을 보존하는 재처리, 기존 출력 거부, 신규 experiment_id 일관성, CLI 종료도 시험한다.

## 저장된 실제 예제의 독립 읽기 감사

영측지선을 다시 계산하지 않고 Python csv/numpy/Pillow로 저장 파일을 검사했다.
스크립트: [audit_saved_results.py](validation/audit_saved_results.py),
결과: [saved_results_audit.json](validation/saved_results_audit.json).

| 자료 | Δt | 해상도 | bin/모드 | Detected=count 합 | PNG frame 검사 | APNG frame 검사 |
|---|---:|---|---:|---:|---:|---:|
| source_to_detector | 1 | 128×128 | 81 | 216 | 162 | 162 |
| source_to_detector_dt01 | .1 | 64×64 | 805 | 216 | 1610 | 1610 |

기본 256광선: Detected216, Captured20, Escaped20, Active0, NumericalFailure0.
모든 이벤트 ID 유일, t_emit=0, delta_t=t_hit-t_emit, 모든 값 유한.
모든 공간/시간 count, 모든 PNG와 APNG 픽셀·재생 지연이 원본 이벤트에서 계산한 값과 일치한다.
최종 cumulative도 전체 누적 이미지와 같다. 한 hit=1 count이며 이미지에 추가 라벨은 없다.
검사 전후 **1,796개 저장 파일의 SHA-256 일치**. 원본 이벤트 FNV-1a64는
`6d7ce5a78b31e7e9`, SHA-256은
`919e35a580ee60c2248b7b60f31039a0f9e2ccabdccd3d7c277de7572181a3ae`.

## 현재 한계·미완료 검증

이번 요청의 자동시험 실패는 없다. 아래는 현재 지원 범위 밖이거나 추가 연구가 필요한 항목이다.

- 지평선 내부 및 극축 통과: BL chart가 특이하므로 정칙 좌표계가 필요하다. epsilon은 수치 cutoff다.
- 같은 스텝 안에서 signed-distance 양끝 부호가 같은 복수 교차, 접선 접촉·평면상 운동의 일반 검출 보장은 없다.
- 실제 검출기 고유시간, 고유면적, 스펙트럼·에너지 가중 플럭스와 파동광학을 검증한 데이터가 아니다.
- 최대 보존량 오차·최소 r는 승인 상태에서 측정한다. 스텝 사이 엄밀한 극값이나 모든 초기조건에 대한 보장이 아니다.
- null 실패 문턱 1e-5는 실측 정확도 보장이 아니다. 전체 시험 중 가장 큰 null은 기존 고스핀 cutoff 시험의 2.801244e-7이다.
- 장시간 임계 궤도, 임의로 작은 epsilon, 극축 근방, 다중 교차에는 관측량별 추가 수렴 시험이 필요하다.
- CPU 병렬화·GPU·UI 미구현. CPU/GPU 비교, f32 허용오차와 렌더링 FPS는 미측정이다.

향후 GPU 비교는 동일 ZAMO 초기조건과 affine/event 상태를 맞추고,
r/theta/phi/t, Detected/Captured/Escaped, u_hit/v_hit/t_hit, null/E/Lz/Q를 함께 검사해야 한다.
h,h/2,h/4 수렴을 고스핀·임계·포획 사례에서 통과하기 전 GPU를 기준 해로 사용하지 않는다.

## 수식 출처

- [Visser, The Kerr spacetime](https://arxiv.org/html/0706.0622v3): 계량·좌표·지평선/에르고면.
- [Su et al., 2024, §2](https://link.springer.com/article/10.1140/epjc/s10052-024-13113-w): 보존량·독립 반지름 퍼텐셜.
- [Tejeda et al., Appendix B](https://academic.oup.com/mnras/article/469/4/4483/3800690): BL oblate Cartesian-like 정·역변환.
- [SciPy RK45 공식 문서](https://docs.scipy.org/doc/scipy/reference/generated/scipy.integrate.RK45.html): Dormand–Prince 5(4)와 원 논문 서지.
