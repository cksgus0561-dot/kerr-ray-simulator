# 실측 성능 — CPU f64 기준 구현

2026-09-15. 아래는 실제 측정한 **물리 계산 배치 시간**이다.
GPU·UI가 없으므로 FPS 또는 렌더링 frame time으로 해석하지 않는다.

## 환경

- CPU: 12th Gen Intel(R) Core(TM) i5-12400F, 논리 프로세서 12개 노출.
- Windows NT 10.0.26200.0, Rust 1.98.1, LLVM 22.1.8, x86_64-pc-windows-msvc, release.
- 실제 계산은 단일 스레드 순차 실행. 별도 affinity·전원·백그라운드 작업 제어 없음.
- f64 reference 및 보존량 검사는 모든 배치에서 유지했다.

## 3+1차원 source/detector 배치

256광선, spin=.6, source X=80 크기32×32, detector X=-80 크기300×300.
rtol1e-12, atol1e-14, max_step.5, epsilon.001, escape400, max_affine1500.
정확한 입력 전체는 각 benchmark.json의 experiment에 있다.

| 실행 | 광선/배치 | 측정 배치 | 평균 | p95 | 평균/광선 |
|---|---:|---:|---:|---:|---:|
| 최초 저장 측정 | 256 | 10 | 220.555610 ms | 237.674400 ms | .861545 ms |
| JSON 보고 오류 수정 후 측정 | 256 | 10 | **216.868790 ms** | **239.981200 ms** | **.847144 ms** |

각각 워밍업 1배치를 제외했다. p95는 정렬한 N=10의 ceil(.95*N)번째로 이 표에서는 최대값이다.
포함: 적분·검출 root·보존량 검사·궤적 할당/해제·결과 구성.
제외: 컴파일, 프로세스 시작, 초기조건 생성, CSV/PNG/APNG 쓰기, 콘솔 I/O.
두 실행의 차이는 동일 데스크톱에서의 반복 측정 차이이며 최적화 성과로 해석하지 않는다.

매 배치 Detected216/Captured20/Escaped20/Active0/NumericalFailure0.
수정 후 기록의 최대 null=1.1393694876460359e-8, 최대 Q 절대오차=2.9331204132176936e-10.
평균/광선은 배치 평균÷256이며 단일 광선 latency 백분위수가 아니다.

원자료:
[최초 batches.csv](results/benchmark_detector/batches.csv),
[최초 benchmark.json](results/benchmark_detector/benchmark.json),
[수정 후 batches.csv](results/benchmark_detector_final/batches.csv),
[수정 후 benchmark.json](results/benchmark_detector_final/benchmark.json).
최초 JSON의 max_null은 변수명 `null`을 serde_json::json!이 리터럴로 해석해 숫자가 누락됐다.
기존 기록은 보존했고 example 변수명만 수정한 뒤 새 폴더에 다시 측정했다. 물리 엔진의 변경은 없다.

```powershell
. .\env.ps1
cargo run --release --offline --example benchmark_detector -- results/benchmark_detector_new
```

## 저장된 실제 데이터 생성 1회

`results/source_to_detector/output_timing.json`과 run_metadata.json의 실제 값:

| 구간 | 시간 |
|---|---:|
| 256광선 물리 계산 합계 | 211.686 ms |
| 물리 계산+원본 출력 | 231.0397 ms |
| 후처리:128×128,81bin,두 PNG sequence+APNG | 229.7469 ms |
| 전체 실행 | 461.2749 ms |

단일 실행 wall time이며 평균값이 아니다. 구간 측정 사이의 metadata/I/O 시간이 있어
반올림한 하위 항목의 단순 합이 전체와 같을 필요는 없다.
컴파일·프로세스 시작은 실험 내부 타이머에 포함되지 않는다.
Δt=.1 재처리 결과는 존재하지만 그 실행 시간은 따로 계측하지 않았다.

## 보존된 M1–3 이력

적도면 parallel_beam 41광선, spin=.6, x_source50, y=-10..10, escape100,
rtol1e-12, atol1e-14, max_step.5. 워밍업1 제외 후 30배치.

| 엔진 | 평균 배치 | p95 배치 | 평균/광선 |
|---|---:|---:|---:|
| 당시 CPU f64 순차 | 8.439727 ms | 11.252600 ms | .205847 ms |

수치 실패0, 최대 null1.4612851373e-8. 다른 실험·버전이므로 3D 배치와 직접 속도비를 주장하지 않는다.
[이전 원자료](results/benchmark/benchmark.csv), [이전 설명](validation/history/M1_M3_BENCHMARK.md)을 보존했다.
기존 `cargo run --release -- benchmark --repeats 30 --output results/benchmark_new`도 유지한다.

## 아직 측정하지 않은 항목

CPU 병렬화 전후, GPU 모델/backend/compute, f32 오차, CPU-GPU 복사량,
workgroup 크기, 메모리 사용량, UI 평균/p95 frame time, FPS는 미측정이다.
다음 단계에서 같은 초기조건과 검증을 유지하며 측정한다. 현재 수치로 60fps 달성을 주장하지 않는다.
