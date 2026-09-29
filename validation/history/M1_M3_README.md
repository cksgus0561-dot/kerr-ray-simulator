# Kerr Ray — 수정하며 배우는 영측지선 기준 계산기

**Milestone 1–3 구현 완료.** 현재는 Rust CLI/라이브러리이며 렌더링과 GPU 계산은 아직 없다.
고정된 Kerr 시공간에서 실제 해밀턴 방정식으로 적도면 빛의 궤적을 계산한다.
초기조건, 매개변수 탐색, 관측량을 학생이 직접 수정하는 것을 정상 사용 방식으로 취급한다.

## 빠른 실행

이 README가 있는 `outputs/kerr-ray`가 Cargo 프로젝트 루트다.
이번 Windows 환경에는 Rust가 없어서 작업 폴더의 `work/cargo`, `work/rustup`에
공식 Rust 1.98.1을 설치했다. 시스템 PATH는 변경하지 않았다.
PowerShell에서 프로젝트 폴더로 이동한 뒤:

```powershell
. .\env.ps1
cargo run --release -- experiment single_ray --trajectories
cargo run --release -- experiment parallel_beam --spin 0.6 --rays 41 --trajectories
cargo run --release -- experiment spin_scan
cargo run --release -- experiment critical_impact_parameter_scan --b-min 5.19 --b-max 5.20 --rays 21
cargo run --release -- --help
```

다른 컴퓨터는 [공식 Rust 설치 안내](https://rust-lang.org/tools/install/)에 따라 Rust를 설치한다.
Rust 2024 edition을 사용하며 외부 crate 의존성이 없다. Windows MSVC 빌드는 C++ 빌드 도구가 필요하다.
검증한 도구: rustc/cargo 1.98.1, x86_64-pc-windows-msvc, LLVM 22.1.8.

기본 결과 경로는 `results/<실험명>_<타임스탬프>/`다. `--output results/my_run`으로 지정할 수 있다.
**같은 출력 폴더를 다시 지정하면 같은 이름의 CSV를 덮어쓴다.**
`summary.csv`에는 모든 광선의 관측량·오차·설정·초기 정준 상태가 저장된다.
`--trajectories`는 실제 승인된 적분 상태를 `ray_0000.csv` 등으로 저장한다.
수치 실패가 하나라도 발생하면 CSV를 기록한 뒤 종료 코드 1을 반환한다.

## 무엇을 계산하는가

- 고정된 Kerr 진공 배경에서 기하광학 근사의 영측지선.
- 일반 위치의 Kerr 계량과 역계량. 운동은 **theta=pi/2, p_theta=0, Q=0**으로 제한.
- ZAMO가 측정한 국소 발사 방향을 좌표 4-운동량으로 변환.
- f64 CPU 기준 해, 적응형 Dormand–Prince 5(4), 보존량 진단, 포획/탈출 판정.

현재 동적 아인슈타인 방정식, 전자기장-중력 되먹임, Maxwell 파동장,
지평선 내부, 전체 3차원 운동, 플라스마/흡수/방출은 계산하지 않는다.
프레임 끌림은 계량의 비대각 성분으로부터 나온다. 별도 회전력이나 렌즈 효과를 넣지 않는다.

## 자연단위와 부호 규약

G=c=M=1, chi=a/M=a, 허용 회전 범위 0<=chi<=0.99.
M은 독립 조절값이 아니라 단위의 정의다. 물리 단위로 환산할 때 길이는 GM/c²,
시간은 GM/c³를 곱한다. 계량 부호는 **(-,+,+,+)**, 좌표 순서는 **(t,r,theta,phi)**.
코드의 p_mu는 **공변 정준 운동량**이며 국소 3차원 진행방향이 아니다.
lambda는 affine parameter이며 광자의 고유시간이 아니다.

## Kerr 계량, 지평선과 에르고면

Sigma=r²+a² cos²(theta), Delta=r²-2r+a²,
A=(r²+a²)²-a² Delta sin²(theta).
0이 아닌 공변 성분은 다음과 같다.

```text
g_tt       = -(1-2r/Sigma)
g_tphi     = -2ar sin²(theta)/Sigma = g_phit
g_rr       = Sigma/Delta
g_thetatheta = Sigma
g_phiphi   = A sin²(theta)/Sigma

g^tt       = -A/(Sigma Delta)
g^tphi     = -2ar/(Sigma Delta) = g^phit
g^rr       = Delta/Sigma
g^thetatheta = 1/Sigma
g^phiphi   = (Delta-a² sin²(theta))/(Sigma Delta sin²(theta))
```

선요소의 교차항은 **2 g_tphi dt dphi**이다. 계량 성분의 2배와 혼동하지 않는다.
바깥 지평선은 `r_plus=1+sqrt(1-a²)`, 바깥 에르고면은
`r_ergo(theta)=1+sqrt(1-a² cos²(theta))`. 적도면 에르고면은 항상 r=2이다.
기하 정의는 [Visser의 Kerr 시공간 소개](https://arxiv.org/html/0706.0622v3)를 참고한다.

### 좌표 특이성과 종료

Boyer–Lindquist(BL) 좌표는 Delta=0에서 좌표 특이성을 가진다. 이를 물리 특이점으로 해석하지 않는다.
이 버전의 계량 API는 r>r_plus, 0<theta<pi에서만 유효하다.
광선은 `r<=r_plus+epsilon`에서 Captured로 종료한다. 기본 epsilon=0.001 M은
**수치 종료 허용값**이며 사건의 지평선 반지름을 수정하지 않는다.
적분 단계가 지평선을 건너지 않도록 제한하고, 경계 교차는 스텝을 다시 적분해 이분 탐색한다.
저장 상태의 r이나 p를 경계값으로 강제 덮어쓰지 않는다.

CSV의 `x_bl=r cos(phi), y_bl=r sin(phi)`는 **좌표 시각화**다.
곡률 있는 공간의 유클리드 임베딩, 관측자가 본 블랙홀 사진, 광학적 렌즈 영상이 아니다.

## 영측지선과 해밀턴 방정식

```text
H = (1/2) g^mu nu p_mu p_nu = 0
dx^mu/dlambda = g^mu nu p_nu
dp_mu/dlambda = -(1/2) (partial_mu g^alpha beta) p_alpha p_beta
```

적도면의 상태는 `(t,r,phi,p_t,p_r,p_phi)`다. `partial_r g^mu nu`는 해석적인
몫 미분으로 구현했다. 유한차분은 그 미분식을 시험하는 데만 사용한다.
적도면은 반사 대칭의 불변 부분공간이므로 theta 운동을 인위적으로 누르는 힘은 없다.

에너지 `E=-p_t`와 축 각운동량 `Lz=p_phi`는 시간 병진/축 회전 대칭 때문에 보존된다.
두 운동량의 Hamilton 미분은 0이므로 코드에서도 정확히 0이다. 적분 후 보정하지 않는다.
영조건 `C_null=g^mu nu p_mu p_nu`는 실제 계산해 매 승인 상태에서 검사한다.
영입자의 Carter 상수는 `Q=p_theta²+cos²(theta)[Lz²/sin²(theta)-a² E²]`.
현재 Q=0은 **제한된 상태공간의 성질**이며 일반 3차원 Carter 보존을 검증한 것이 아니다.
독립 비교에 쓰는 반지름 퍼텐셜과 보존량 정의는
[Su 외, 2024, §2](https://link.springer.com/article/10.1140/epjc/s10052-024-13113-w)를 참고했다.

## 초기조건: ZAMO

ZAMO는 Lz=0인 국소 관측자다. `alpha=sqrt(Sigma Delta/A)`, `omega=2ar/A`로 두면
정규직교 기준벡터는 다음과 같다.

```text
e_(0) = (partial_t + omega partial_phi)/alpha
e_(r) = sqrt(Delta/Sigma) partial_r
e_(theta) = partial_theta/sqrt(Sigma)
e_(phi) = partial_phi/sqrt(g_phiphi)
```

국소 광자 운동량은 `p^(hat a)=E_local(1,cos(angle),0,sin(angle))`.
기준벡터로 좌표 반변 운동량을 만든 뒤 계량으로 지표를 내린다.
angle=0은 바깥 반지름 방향, 180도는 안쪽 방향, +90도는 +phi 방향이다.
기본 E_local=1은 affine 정규화를 고른 것이다.
임계값 실험은 ZAMO 방향을 역으로 구하여 **정확한 b=Lz/E**를 지정한다.
유한 거리의 화면 높이, ZAMO 입사각, 충돌매개변수 b는 서로 다르다.

`parallel_beam`은 BL 화면에서 x=50, y=-10..10에 둔 각 ZAMO의 국소 방향을 정렬한다.
서로 다른 접공간에서 평행은 전역적으로 유일하지 않으므로 이 정의를 명시하며,
먼 거리의 평평한 극한에서 평행 빔에 접근한다.

## 수치 적분기와 진단

CPU는 [Dormand–Prince 5(4)](https://docs.scipy.org/doc/scipy/reference/generated/scipy.integrate.RK45.html)의
5차 해와 4차 오차 추정량을 사용한다. 자체 구현의 계수·차수·수렴을 시험한다.
SciPy를 실행 시 의존성으로 사용하지 않는다. 각 성분의 오차는
`atol+rtol*max(|old|,|new|)`로 나누고 최대값이 1 이하인 스텝만 승인한다.
기본 `rtol=1e-12`, `atol=1e-14`, 초기 스텝 0.05, 최대 스텝 0.5,
최소 스텝 1e-12, 최대 시도 200000, 최대 affine 길이 1000이다.
이 값은 **국소 오차 제어**이며 전체 궤적 오차의 상한을 보장하지 않는다.

| 상태 | 의미 |
|---|---|
| Active + AffineLimit | 설정한 실험 구간 종료. 포획/탈출 여부 미확정 |
| Captured | r<=r_plus+epsilon. 지평선 안으로 적분하지 않음 |
| Escaped | r>=r_escape이며 dr/dlambda>0. 유한 경계의 종료 판정 |
| NumericalFailure | NaN, null 위반, 부적절한 초기 상태, 스텝 한도/하한 또는 적분 실패 |

기본 r_escape=100. 최대 스텝 도달을 Escaped로 바꾸지 않는다.
null 실패 문턱은 `|C_null|/E_local(initial)² > 1e-5`이며 정규화 전 오차도 저장한다.
일반적인 E/Lz 상대오차와 절대오차를 함께 저장한다. 초기 보존량의 절댓값이
`E_local*1e-14` 이하이면 상대오차 대신 초기 국소 에너지 단위의 절대오차를 쓴다(M=1).
Q 오차, 승인/거절 스텝, 경계 탐색 횟수, 실패 원인, 계산시간도 결과에 포함된다.
진단의 최대값은 초기·승인 상태에 대한 최대이며 스텝 사이 연속 최대의 보장은 아니다.
지평선 근처에서는 큰 항의 상쇄로 BL 영조건 계산의 수치 조건이 나빠진다.

## 제공 실험과 CLI 설정

| 실험 | 변경 지점 |
|---|---|
| single_ray | --source-radius, --angle-deg, --spin |
| parallel_beam | --rays, --beam-width, --source-radius, --spin |
| prograde_vs_retrograde | --comparison-b (양수 크기), --spin |
| critical_impact_parameter_scan | Schwarzschild(a=0) 기준, --b-min/--b-max/--rays |
| spin_scan | 코드의 spin 목록 0,0.2,0.4,0.6,0.8,0.9,0.95; 각각 ±comparison-b |
| impact_scan | 현재 --spin에서 b 범위 탐색 |
| position_scan | source_radius부터 1.5배까지; 범위 함수는 소스에서 변경 |
| angle_scan | angle_degrees±5도; 범위 함수는 소스에서 변경 |

공통 수치 옵션: `--rtol`, `--atol`, `--epsilon`, `--escape`, `--max-affine`,
`--max-step`, `--max-steps`, `--null-tolerance`. `--beam-width`는 y 분포의 **반폭**이다.
critical scan은 `--spin`과 무관하게 a=0, spin scan은 코드에 지정된 목록을 사용한다.
각 행의 실제 spin은 콘솔과 CSV에 표시된다.

## Where to modify the code

| 파일 | 직접 바꿀 내용 |
|---|---|
| `src/experiments/initial_rays.rs` | `local_source()`의 위치·국소 각도·에너지, `beam_member()`의 광원 배치 |
| `src/experiments/scenarios.rs` | `ExperimentSettings::default()`의 기본 실험, 새 함수와 `select()` 이름 추가 |
| `src/experiments/parameter_scan.rs` | b, 위치, 각도 범위; `linear_values(start,end,count)` 사용 |
| `src/experiments/observables.rs` | `PhysicsResult`를 읽는 새 측정 함수 추가 |
| `src/experiments/output.rs` | 새 관측량을 `SUMMARY_HEADER`, `summary_row()`에 추가하여 CSV로 내보내기 |
| `src/config.rs` | 수치 정확도, epsilon, 탈출 반지름, 스텝/광선 기본값 |
| `src/physics/kerr.rs` | Kerr 계량 핵심. 수정 시 전체 물리 검증 필수 |

예를 들어 `scenarios.rs`의 `spin_scan()` 안의 배열에 `0.99`를 추가하면
그 스핀의 두 광선이 함께 계산된다. `single_ray()`의 `local_source()` 호출을
`local_source(k, 20.0, 0.0, 160.0_f64.to_radians())`로 바꾸면 국소 발사 실험이 바뀐다.
`parameter_scan::angle_scan(p, 150.0, 180.0, 61)`로 각도 탐색을 정의할 수 있다.
`observables.rs`의 `coordinate_time_elapsed()`는 함수 하나로 측정량을 추가한 예다.

검증된 핵심 파일 `metric.rs`, `kerr.rs`, `geodesic.rs`, `integrator.rs`, `tetrad.rs`를
수정하면 화면 모양과 무관하게 다음 검사를 전부 통과해야 한다.

```powershell
.\check.ps1
# 내부 실행: cargo fmt; cargo clippy --all-targets -- -D warnings;
#            cargo test; cargo test --release
```

GitHub에 프로젝트 루트를 저장하면 `.github/workflows/physics.yml`이 같은 검사를 실행한다.
이 작업에서는 원격 저장소/CI를 실행하지 않았다. 로컬 소스 수정 자체를 잠그지는 않는다.

## 관측량의 의미

- Captured/Escaped와 종료 원인, affine 길이, 계산시간.
- `sampled_min_r`: 승인된 샘플의 최소 반지름. 정확한 근일점은 더 작을 수 있다.
- `scattering_angle_bl_rad`: 시작/종료 BL 좌표 접선 방향 차이의 주값. 탈출한 광선만 기록.
  유한 거리 진단이므로 무한대 산란각으로 사용하려면 광원/탈출 반지름 수렴을 별도 확인한다.
- `delta_phi_rad`: 부호 있는 누적 방위각. `total_abs_phi_rad`: 샘플 간 |dphi| 합.
  회전 횟수는 후자를 2pi로 나눈 값이며 좌표 의존량이다.
- 최대 null/E/Lz/Q 오차, 승인/거절 스텝과 실패 여부.

## CPU/GPU, 미리보기/정밀 계산, 향후 구조

현재 계산기는 순차 CPU f64 **정밀 기준 구현 하나**다. `RayIntegrator`는 공통 초기 상태,
설정, `PhysicsResult`를 받는 작은 backend 경계다. GPU 파일이나 GUI를 가짜 구현으로 채우지 않았다.
다중 광선 CLI는 순차 실행하며 CPU 병렬화는 다음 Milestone 4의 작업이다.

Milestone 5–6은 wgpu/egui 및 창 입력, 비동기 작업 스레드, generation ID,
입력 debounce를 추가한다. 카메라·재생·표시 변경은 저장된 궤적만 읽는다.
슬라이더 입력 중 적은 광선의 미리보기를, 입력이 멎으면 검증된 정밀 결과를 계산한다.
Milestone 7의 wgpu f32/RK4 미리보기는 CPU와 상태·궤적·보존량을 비교해야 한다.
GPU 시작 시 어댑터를 열거해 가능한 discrete NVIDIA를 우선 선택하고
이름/제조사/종류/backend/features/limits를 출력하는 기능은 **아직 미구현**이다.
CUDA는 계측 근거가 있을 때 별도 backend로만 검토한다.
전체 3차원 운동 및 Kerr–Schild 변환은 Milestone 8로 남긴다.

자세한 수치 근거는 [PHYSICS_VALIDATION.md](PHYSICS_VALIDATION.md),
측정 결과는 [BENCHMARK.md](BENCHMARK.md), 구조는 [DESIGN.md](DESIGN.md)를 읽는다.
