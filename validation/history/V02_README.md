# Kerr Ray — 수정하며 실험하는 3+1차원 영측지선 데이터 생성기

Rust **v0.2.0**. 검증된 Milestone 1–3의 CPU f64 엔진을 일반적인
Boyer–Lindquist(BL) 3+1차원 운동과 광원면·흡수형 검출면으로 확장했다.
원본 연속시간 검출 이벤트에서 count 배열, 누적 PNG, 시간분해 PNG와 APNG를 만든다.
실험 코드를 직접 수정하는 것이 정상적인 사용 방식이다.

기존 18개 시험을 보존했고 새 시험 19개를 추가했다. 총 **37개, 실패 0**이며
debug/release 및 clippy를 실제 실행했다. 수치 근거는
[PHYSICS_VALIDATION.md](PHYSICS_VALIDATION.md), 완료 조건 대조는
[COMPLETION_CHECKLIST.md](COMPLETION_CHECKLIST.md), 최종 보고는 [FINAL_REPORT.md](FINAL_REPORT.md)에 있다.

## 빠른 실행

이 파일이 있는 `outputs/kerr-ray`가 Cargo 프로젝트 루트다. 이 Windows 작업 환경에서는:

```powershell
. .\env.ps1
cargo run --release --offline -- experiment source_to_detector --output results/my_source_run
cargo run --release --offline -- rebin --input results/my_source_run --output results/my_dt01 --dt 0.1 --resolution 64x64 --fps 60
cargo run --release --offline -- rebin --input results/my_source_run --output results/my_dt001 --dt 0.01 --time-start 168 --time-end 180 --resolution 64x64
cargo run --release --offline -- experiment source_to_detector --help
```

`env.ps1`은 설치된 cargo를 우선 사용하고, 없으면 이 작업의 `work/cargo`, `work/rustup`을
현재 PowerShell 세션에서 사용한다. 다른 컴퓨터는 [Rust 설치 안내](https://rust-lang.org/tools/install/)를 따른다.
검증 환경은 Rust 1.98.1 / LLVM 22.1.8 / Windows MSVC이며 Windows C++ 빌드 도구가 필요하다.
Rust 2024 edition, serde/serde_json/csv/png 의존성은 `Cargo.lock`과 `vendor/`에 고정했다.
`.cargo/config.toml`을 포함하면 crate 다운로드 없이 빌드할 수 있다. Python은 Rust 실행에 필요 없다.

`source_to_detector`의 기본 출력은 `results/source_to_detector/`다.
**동일한 원본·파생 파일이 있으면 덮어쓰지 않고 실패한다.** 매번 새 `--output`을 사용한다.
저장된 예제를 다시 탐색하려면 위 `rebin`의 `--input`을 `results/source_to_detector`로 바꾸면 된다.
재분류 명령은 원본 checksum을 확인하고 이벤트만 읽으며 영측지선을 다시 적분하지 않는다.

## 계산 범위

- 고정된 Kerr 진공 시공간에서 기하광학 근사의 영측지선: t, r, theta, phi와 네 정준 운동량.
- 일반 3차원 ZAMO 초기조건, 해석적 계량 미분, 적응형 CPU f64 Dormand–Prince 5(4).
- 매 승인 상태의 null/E/Lz/Q 진단, 수치 실패와 포획·탈출·검출 분리.
- 좌표로 정의된 광원면·검출면, 스텝 내부 교차 탐색, 연속적인 t_hit와 delta_t.
- 한 hit를 한 count로 취급하는 공간·시간 histogram과 영상 후처리.

동적인 Einstein 방정식, Maxwell 장 시간발전, 전자기장-중력 되먹임, 파동 간섭,
머신러닝, GPU, GUI, 실시간 렌더러는 현재 범위에 없다. 프레임 끌림은 Kerr 계량에서 나온다.
광선을 휘는 별도 힘이나 영상용 궤적 보정은 없다.

## 자연단위와 convention

**G=c=M=1**, chi=a/M=a, **0<=chi<=0.99**, 계량 부호 **(-,+,+,+)**.
좌표·계량 인덱스 순서는 `(t,r,theta,phi)`이며 p_mu는 공변 정준 운동량이다.
길이의 물리 단위는 GM/c², 시간은 GM/c³를 곱해 복원한다. M은 단위 정의이며 독립 가변 질량이 아니다.
lambda는 affine parameter로, 광자의 고유시간이 아니다.

## Kerr 계량, 사건의 지평선, 에르고면

```text
Sigma = r² + a² cos²(theta)
Delta = r² - 2r + a²
A = (r²+a²)² - a² Delta sin²(theta)

g_tt = -(1-2r/Sigma)              g^tt = -A/(Sigma Delta)
g_tphi = -2ar sin²(theta)/Sigma   g^tphi = -2ar/(Sigma Delta)
g_rr = Sigma/Delta               g^rr = Delta/Sigma
g_thetatheta = Sigma             g^thetatheta = 1/Sigma
g_phiphi = A sin²(theta)/Sigma   g^phiphi = (Delta-a² sin²(theta))/(Sigma Delta sin²(theta))

r_plus = 1 + sqrt(1-a²)
r_ergo(theta) = 1 + sqrt(1-a² cos²(theta))
```

교차 성분은 대칭이고 선요소의 항은 `2 g_tphi dt dphi`이다. 표시를 위해 이 값을 변경하지 않는다.
정의는 [Visser, Kerr spacetime](https://arxiv.org/html/0706.0622v3)를 참고한다.

BL 좌표는 지평선 Delta=0과 극축에 좌표 특이성을 갖는다. 이를 물리 특이점으로 해석하지 않는다.
구현은 외부 영역만 다루며 `r<=r_plus+epsilon`에서 Captured로 분류한다.
기본 epsilon=0.001 M은 수치 종료 허용값이며 실제 지평선 반지름의 보정이 아니다.
극축 통과나 지평선 내부 계산을 계속할 정칙 좌표계는 향후 과제다.

## 영측지선과 보존량

```text
H = (1/2) g^mu nu p_mu p_nu = 0
dx^mu/dlambda = g^mu nu p_nu
dp_mu/dlambda = -(1/2) (partial_mu g^alpha beta) p_alpha p_beta

C_null = g^mu nu p_mu p_nu
E = -p_t
Lz = p_phi
Q = p_theta² + cos²(theta) [Lz²/sin²(theta) - a² E²]
```

`partial_r g^mu nu`와 `partial_theta g^mu nu`는 해석적 몫 미분이다.
유한차분은 미분식 시험에서만 사용한다. t·phi 대칭 때문에 dp_t=dp_phi=0이며,
E/Lz를 적분 후 보정하지 않는다. Q는 일반 비적도 궤적에서도 실제 계산·검증한다.
적도면 초기조건은 같은 8변수 엔진에서 대칭 불변 부분공간으로 유지된다.

**호환성:** 내부 `State.0` 순서는 기존 여섯 슬롯을 유지한
`(t,r,phi,p_t,p_r,p_phi,theta,p_theta)`이다.
새 실험에서는 `State::from_bl([t,r,theta,phi],[pt,pr,ptheta,pphi])` 또는
ZAMO 함수를 사용하고, 직접 배열 인덱스를 추측하지 않는다.

## 초기조건: 3차원 ZAMO

ZAMO는 축 각운동량이 0인 국소 관측자다. `alpha=sqrt(Sigma Delta/A)`, `omega=2ar/A`:

```text
e_(0) = (partial_t + omega partial_phi)/alpha
e_(r) = sqrt(Delta/Sigma) partial_r
e_(theta) = partial_theta/sqrt(Sigma)
e_(phi) = partial_phi/sqrt(g_phiphi)
p^(hat a) = E_local (1,n_r,n_theta,n_phi), |n_local|=1
```

`physics::tetrad::ray_3d`는 기준벡터로 반변 운동량을 만든 뒤 지표를 내린다.
E_local>0 및 변환 후 영조건을 검사한다. 방향 성분은 국소 물리량이며 좌표 운동량이 아니다.
`LaunchDirection::LocalZamo([nr,ntheta,nphi])`로 직접 지정한다.
기본 E_local=1은 affine 정규화를 정한다.

## 광원면·검출면의 좌표적 의미

두 면은 다음 **BL oblate Cartesian-like 좌표**에 고정된 사각형으로 정의한다.

```text
X = sqrt(r²+a²) sin(theta) cos(phi)
Y = sqrt(r²+a²) sin(theta) sin(phi)
Z = r cos(theta)
T = t_BL
```

이 표현은 점근적으로 Cartesian에 대응하지만, **Kerr–Schild 좌표가 아니며
곡률 공간의 유클리드 등거리 임베딩도 아니다.** 시간·방위각에 Kerr–Schild 변환을 적용하지 않는다.
변환은 [Tejeda et al., Appendix B](https://academic.oup.com/mnras/article/469/4/4483/3800690)의
BL oblate 표현을 사용한다. 코드의 `coordinates.rs`가 정·역변환과 Jacobian을 담당한다.
기존 적도 CSV의 `x_bl=r cos(phi), y_bl=r sin(phi)`는 별도의 레거시 좌표 그림이며 이 면 좌표와 혼용하지 않는다.

면은 `center + u e_u + v e_v`, signed distance는 `(X-center) dot normal`로 정의한다.
기저는 이 보조 좌표에서 정규직교·오른손 방향이어야 한다(허용오차 1e-12).
가로·세로 길이, 면적, 픽셀 면적은 **좌표량**이다. 검출기의 고유 면적이나 측정 플럭스가 아니다.
모든 면 점의 정지 세계선이 timelike이도록 사각형 전체에 보수적인 r>2 조건을 검사한다.
기본 면은 X=±80 M로 충분히 멀지만 여전히 유한 거리이며 중력 효과가 0은 아니다.

`CartesianSpatial` 발사 방향은 t_BL 일정 절편의 접벡터다.
Jacobian과 공간 계량으로 ZAMO 국소 방향으로 변환·정규화한다.
ZAMO의 시간 기준벡터를 더할 때 omega shift가 생기므로 지정한 벡터를 좌표 광속으로 강제하지 않는다.
서로 떨어진 접공간에서 평행성은 유일하지 않으므로 이 convention을 실험 metadata와 함께 유지한다.

## 검출과 시간

연속 승인 상태의 signed distance 부호가 바뀌면 스텝 내부를 재적분하며 이분 탐색한다.
좌표거리 root tolerance 기본 1e-10 M, 최대 64회. 교차의 전체 상태에서
u_hit, v_hit, t_hit와 4-운동량을 얻는다. 위치나 시간을 평면으로 강제 덮어쓰지 않는다.
사각형 내부의 첫 유효 횡단 교차에서 `Detected`로 종료하는 흡수형 검출기다.
면 바깥 통과는 계속 적분한다. 면의 양쪽 가장자리는 포함한다.
초기점이 면 위에 있고 횡단 방향이면 affine=0에서 검출한다.

원본의 `t_hit`는 적분한 **연속 f64 BL 좌표시간**, `delta_t=t_hit-t_emit`는 이동 좌표시간이다.
검출기 고유시간이나 광자 고유시간이 아니다. 기본 모든 광선의 t_emit=0이며 광선별 변경도 지원한다.
영상 FPS는 APNG 재생 지연에만 적용된다. 물리적 시간 간격 Δt와 독립적이다.

## 기본 실험과 수정 예

`SourceDetectorExperiment::default()`의 설정:

| 항목 | 기본값 |
|---|---|
| spin | 0.6 |
| 광원 | 중심 (80,0,0), 크기 32×32, 16×16 셀 중심 격자, 256광선 |
| 방향·시각 | CartesianSpatial=(-1,0,0), E_local=1, t_emit=0 |
| 검출면 | 중심 (-80,0,0), 크기 300×300, 해상도 128×128 |
| 면의 방향 | normal=+X, e_u=+Y, e_v=+Z |
| 적분 | rtol=1e-12, atol=1e-14, max_step=0.5, epsilon=0.001 |
| 종료 | escape_radius=400, max_affine=1500, max_steps=200000 |
| 후처리 | Δt=1, t_hit 기준, 전체 도착시간 자동 범위, 30fps, counts_per_white=4 |

예를 들어 `source_detector.rs`의 `RectangularGrid { nu: 16, nv: 16 }`을
`RectangularGrid { nu: 32, nv: 8 }`로 바꾸면 256광선의 배치가 바뀐다.
면 위치·크기는 `plane(...)` 인수, 회전은 `Plane::from_normal_up(center, normal, up, width, height)`로 바꾼다.
방향·시각이 광선마다 다르면 `SourcePattern::Explicit(Vec<Emission>)`의
u, v, `t_emit: Some(...)`, `direction: Some(...)`을 지정한다.
`None`은 SourcePlane 기본값을 사용한다. Single, SquareGrid, RectangularGrid도 지원한다.

코드를 재컴파일하지 않고 전체 설정을 수정하려면 [examples/source_detector.json](examples/source_detector.json)을 복사·편집한다.

```powershell
cargo run --release --offline -- experiment source_to_detector --config examples/source_detector.json --output results/my_config_run
```

CLI 물리 옵션: `--spin`, `--nu`, `--nv`, `--rays`(정사각 개수),
`--source-center x,y,z`, `--detector-center x,y,z`, `--direction x,y,z`,
`--local-direction n_r,n_theta,n_phi`, `--t-emit`, `--rtol`, `--atol`,
`--max-step`, `--max-steps`, `--max-affine`, `--escape`, `--epsilon`.
면의 기저·크기와 명시적 광선 목록은 Rust/JSON에서 편집한다.

후처리 옵션: `--dt`, `--time-start`, `--time-end`, `--time-basis t_hit|delta_t`,
`--resolution WIDTHxHEIGHT`, `--fps`, `--counts-per-white`, `--output`.
rebin에 물리 옵션을 넘겨도 궤적 변경을 수행하지 않으며 허용하지 않은 옵션은 오류로 보고한다.

## 원본 → 수치 자료 → 영상

| 파일 | 의미 |
|---|---|
| `detector_events.csv` | 원본: ray_id,u_source,v_source,t_emit,u_hit,v_hit,t_hit,delta_t,status |
| `detector_hit_states.csv` | 물리 진단: 교차 affine,t,r,theta,phi,운동량,null,Q,root residual |
| `ray_diagnostics.csv` | 검출되지 않은 광선을 포함한 모든 상태·종료 원인·보존량·설정 |
| `detector_accumulated.csv` | 전체 이벤트의 정확한 u64 count 배열, 행 우선·헤더 없음 |
| `detector_accumulated.png` | 위 배열의 16bit grayscale 표시 |
| `detector_time_bins.csv` | 희소 frame,x_pixel,y_pixel,count; 없는 셀은 0 |
| `frame_index.csv` | 모든 bin의 시간 경계, 새 count와 누적 count |
| `detector_before_window.csv` | 분석 시작 이전 count 배열 |
| `detector_frames/instantaneous/frame_000000.png` 등 | 해당 bin에서 새로 도착한 hit |
| `detector_frames/cumulative/frame_000000.png` 등 | 현재 bin 끝 이전에 도착한 모든 hit |
| `detector_instantaneous.apng`, `detector_cumulative.apng` | 두 모드의 실제 애니메이션 |
| `run_started.json`, `run_metadata.json` | 설정, 단위, convention, 버전·소스 fingerprint·UTC 실행 시각·결과 |
| `derived_metadata.json`, `output_timing.json` | 후처리 설정·원본 checksum·범위별 count, 실행 시간 |

원본 CSV를 먼저 기록하고 매 hit마다 flush한다. 최종 원본 metadata는 이미지 생성 전에 저장한다.
신규 실행의 `experiment_id`는 실험명과 UTC Unix 시작 시각으로 구성한다.
코드 provenance는 버전과 소스 FNV-1a64 fingerprint이며 git commit이나 암호학적 해시는 아니다.
기존 저장 예제는 ID 필드 추가 전 생성했으며 경로·실행 시각의 대응을 [FINAL_REPORT.md](FINAL_REPORT.md)에 기록했다.

픽셀에서 +u는 오른쪽, 첫 행은 +v 방향이다. 상단/우측 면 경계는 마지막 유효 픽셀로 배정한다.
이미지 값은 `floor(65535 * min(count,counts_per_white)/counts_per_white)`이며
같은 설정을 모든 프레임에 적용한다. 기본 4 count부터 흰색으로 포화되지만 u64 CSV count는 보존한다.
이미지에는 spin 텍스트, 좌표축, UI, 지평선 그림, 진단값을 그리지 않는다. spin은 metadata에 있다.
물리량 분석은 PNG를 역변환하지 말고 원본 또는 count 배열을 사용한다.

시간 bin은 `[start+k*Δt, min(end,start+(k+1)*Δt))`이다.
bin 인덱스가 정수에서 `4*EPSILON*max(1,abs(index))` 이내이면 경계로 배정한다.
이는 0.3/0.1 같은 부동소수점 경계 계산의 convention이며 원본 시간을 수정하지 않는다.
instantaneous는 해당 bin만, cumulative는 분석 시작 이전 hit까지 포함한다.
전체 시간 범위를 포함하면 bin 합계=전체 Detected count이며,
시간 창을 자르면 분석 전/후 count를 별도 기록한다. 빈 bin도 프레임을 출력한다.
한 번의 파생 출력은 100000프레임, 16777216픽셀/영상, 모드별 총 500M픽셀 한도를 명시적으로 검사한다.

## 수치 오차와 종료 상태

DP5(4)의 성분별 오차 척도는 `atol+rtol*max(abs(old),abs(new))`이며
최대 정규화 오차가 1 이하인 스텝만 승인한다. 국소 허용오차는 전역 궤적 오차의 보장이 아니다.
초기 스텝 0.05, 최소 스텝 1e-12. 지평선·극축 근처에서는 스텝 크기를 제한한다.

| 상태 | 의미 |
|---|---|
| Active / AffineLimit | 설정한 affine 구간 종료, 최종 운명 미확정 |
| Detected / DetectorHit | 첫 유효 검출 교차에서 흡수 |
| Captured / HorizonCutoff | 외부 영역의 수치 지평선 cutoff 도달 |
| Escaped / EscapeRadius | r>=r_escape 이면서 dr/dlambda>0 |
| NumericalFailure | 비유한 값, null 위반, 부적절한 초기조건, 스텝 한도·하한, 적분 실패 |

최대 스텝 초과를 Escaped로 해석하지 않는다. null 실패 문턱 기본값은
`abs(C_null)/E_local0² > 1e-5`다. 측정 오차·허용오차·실패 문턱을 구분한다.
E/Lz 상대오차의 분모가 거의 0이면 초기 국소 에너지 단위의 절대오차를 사용한다.
Q 상대 척도는 `abs(Q-Q0)/max(abs(Q0),E_local0²)`다. 절대오차도 함께 저장한다.

## 기존 실험 호환성

```powershell
cargo run --release --offline -- experiment single_ray --trajectories
cargo run --release --offline -- experiment parallel_beam --spin 0.6 --rays 41 --trajectories
cargo run --release --offline -- experiment prograde_vs_retrograde
cargo run --release --offline -- experiment spin_scan
cargo run --release --offline -- experiment critical_impact_parameter_scan --b-min 5.19 --b-max 5.20 --rays 21
cargo run --release --offline -- experiment three_dimensional_ray --trajectories
```

기존 impact_scan, position_scan, angle_scan도 유지한다. critical scan은 a=0을 사용하고,
spin_scan은 코드의 목록을 사용한다. 각 CSV에 실제 spin을 기록한다.
기존 summary.csv/trajectory의 앞부분 형식은 보존하며 theta/p_theta/Q 진단을 확장했다.
레거시 실험 CLI는 타임스탬프 출력 폴더가 기본이고 명시적으로 같은 폴더를 주면 CSV를 덮어쓸 수 있다.
source/detector 원본 보존 정책과 구분하고 레거시에서도 새 출력 폴더를 사용한다.

## Where to modify the code

| 파일 | 직접 수정할 내용 |
|---|---|
| `src/config.rs` | 전역 기본 수치 허용오차, epsilon, r_escape, 스텝·광선 설정 |
| `src/experiments/source_detector.rs` | SourceDetectorExperiment의 spin, 광원·검출면 위치/기저/크기, 광선 목록·방향·시각, 정확도, 출력 설정 |
| `examples/source_detector.json` | 같은 실험 설정을 재컴파일 없이 변경; `--config`로 읽기 |
| `src/experiments/initial_rays.rs` | 기존 실험의 초기 위치·국소 방향 |
| `src/experiments/scenarios.rs` | 새 실험 함수와 select() 등록, spin_scan 목록 |
| `src/experiments/parameter_scan.rs` | 충돌매개변수·위치·각도 범위 탐색 |
| `src/experiments/observables.rs` | PhysicsResult를 읽는 새 관측량 함수 |
| `src/experiments/output.rs` | 추가 관측량을 기존 요약 CSV로 출력 |
| `src/detector/` | 평면 기하, 교차 root, 이벤트 검증, 시간 bin 계산 |
| `src/output/` | 원본 입출력 및 파생 수치·PNG/APNG, metadata |
| `src/physics/kerr.rs` | Kerr 계량 핵심. 수정 시 전체 물리 검증 필요 |
| `src/physics/{metric,geodesic,integrator,tetrad,coordinates,validation}.rs` | 핵심 convention·운동·검증. 수정 시 전체 시험 실행 |

예를 들어 `observables.rs`에 `fn my_measurement(result: &PhysicsResult) -> f64`를 추가하고
`experiments/output.rs`의 헤더·행에 결과를 연결한다. 계량·물리 상수에 실험 설정을 숨기지 않는다.

```powershell
.\check.ps1
# cargo fmt
# cargo clippy --all-targets -- -D warnings
# cargo test
# cargo test --release
```

물리·검출 수렴 시험이 실패하면 이미지가 그럴듯해도 검증된 결과로 취급하지 않는다.
`.github/workflows/physics.yml`도 같은 검사를 정의하지만 원격 CI 실행은 이번 검증에 포함하지 않았다.

## 물리적·수치적 한계와 다음 단계

현재 BL 외부 chart만 사용하므로 극축·지평선 통과를 일반적으로 지원하지 않는다.
검출은 스텝 양 끝의 횡단 부호 변화에 기반한다. 접촉·평면상 운동, 한 스텝 내부의 부호 변화 없는
복수 교차는 보장하지 않으므로 관심 실험에서 max_step 및 hit 수렴을 검증해야 한다.
정지 좌표면의 count는 검출기의 고유시간·고유면적당 플럭스나 에너지 가중 밝기가 아니다.
최소 접근 반지름과 최대 보존량 오차는 승인 샘플의 값이며 연속 구간의 엄밀한 최대/최소가 아니다.
지평선 근방의 큰 항 상쇄, 장시간 불안정 임계 궤도에는 별도 수렴 연구가 필요하다.

현재 CPU는 순차 실행하며 정확도 기준 구현을 유지한다. 측정은 [BENCHMARK.md](BENCHMARK.md)에 있다.
wgpu/egui 단계에서는 작업 스레드·generation ID·입력 debounce로 UI와 적분을 분리하고,
카메라·재생 변경은 저장된 결과만 사용한다. GPU f32 미리보기는 같은 초기조건과 affine/event 시점에서
CPU f64의 궤적·상태·t_hit·Q/null/E/Lz 및 스텝 수렴과 비교한 뒤 사용한다.
어댑터 검사, discrete NVIDIA 우선 선택, 기능·제한 출력, 60fps 계측은 향후 구현·검증 대상이다.
CUDA는 별도 backend의 실측 이점이 있을 때 검토한다. 현재 GPU 정확도나 FPS 수치는 없다.
