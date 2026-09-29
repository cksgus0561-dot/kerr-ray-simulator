# 실측 성능 — CPU 기준 구현

2026-09-15 측정. GPU/렌더러는 아직 없으며 FPS, frame time, GPU 이름/backend의 실행 측정은 없다.
아래 값은 **물리 계산 배치 시간**이다. 화면 프레임 시간으로 해석하지 않는다.

## 환경과 방법

- CPU: 12th Gen Intel(R) Core(TM) i5-12400F (로컬 레지스트리에서 확인).
- Windows NT 10.0.26200.0, 논리 프로세서 12개 노출. **실제 계산은 단일 스레드 순차 실행**.
- rustc/cargo 1.98.1, x86_64-pc-windows-msvc, LLVM 22.1.8, release 프로필.
- 별도 affinity/전원/백그라운드 작업 제어 없이 사용 중인 데스크톱에서 측정.
- parallel_beam 41개; chi=.6; x_source=50, y=-10..10; E_local=1.
- rtol=1e-12, atol=1e-14, epsilon=.001, escape=100, max_step=.5.
- 제외하는 워밍업 배치 1회 후 30회 측정.
- 포함: 적분, 보존량 검사, 궤적 Vec 할당/해제, 결과 생성.
- 제외: CSV/콘솔 I/O, 초기조건 생성, 컴파일, 프로세스 시작.
- p95는 정렬된 30표본의 ceil(.95*N)번째(1-based) 값.

| 엔진 | 광선/배치 | 평균 배치 시간 | p95 배치 시간 | 평균 시간/광선 |
|---|---:|---:|---:|---:|
| CPU f64 DP5(4), 순차 | 41 | 8.439727 ms | 11.252600 ms | 0.205847 ms |

30회에 걸친 수치 실패 0, 최대 |C_null|=1.4612851373e-8.
평균 시간/광선은 배치 평균을 41로 나눈 값이며 단일 광선 latency 백분위수가 아니다.

원자료: [benchmark.csv](results/benchmark/benchmark.csv),
[benchmark.txt](results/benchmark/benchmark.txt), [inputs.csv](results/benchmark/inputs.csv).

## 재현 명령

```powershell
. .\env.ps1
cargo run --release -- benchmark --repeats 30 --output results/benchmark_new
```

실제 측정값은 실행 환경에 따라 달라진다. 이 기록은 60fps 목표 달성을 의미하지 않는다.
GPU 가속, CPU 병렬화, UI 평균/p95 frame time, CPU↔GPU 복사량,
메모리 사용량과 workgroup 최적화는 **측정하지 않았다**.
다음 Milestone 4에서는 이 순차 baseline과 같은 초기조건/보존량 검사를 유지하며 병렬화를 비교한다.
