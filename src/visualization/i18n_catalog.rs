//! English catalog keys match existing UI text; numbered Korean placeholders keep values opaque.
pub(super) const CATALOG: &[(&str, &str)] = &[
    (
        "KERR_RAY_THREADS must be a positive integer",
        "KERR_RAY_THREADS는 양의 정수여야 합니다",
    ),
    (
        "ray worker disconnected",
        "광선 계산 작업 스레드와의 연결이 끊겼습니다",
    ),
    (
        "ray worker panicked",
        "광선 계산 작업 스레드에서 panic이 발생했습니다",
    ),
    (
        "numerical failures recorded; see ray_diagnostics.csv (raw events preserved)",
        "수치 계산 실패가 기록됐습니다. ray_diagnostics.csv를 확인하세요 (원본 이벤트 유지)",
    ),
    (
        "raw event checksum differs from run metadata",
        "원본 이벤트의 checksum이 실행 메타데이터와 다릅니다",
    ),
    (
        "{} | {:?} | {:?} | vendor {:#x}",
        "{0} | {1} | {2} | 제조사 {3}",
    ),
    ("Auto-save FAILED: ", "자동 저장 실패: "),
    ("; calculated result retained", "; 계산 결과는 유지됨"),
    (
        "; legacy calculation retained",
        "; 이전 방식의 계산 결과는 유지됨",
    ),
    ("OS seed generation failed: ", "OS 시드 생성 실패: "),
    (
        "cannot spawn ray worker: ",
        "광선 계산 작업 스레드를 시작할 수 없음: ",
    ),
    ("cell count overflow", "셀 개수가 표현 범위를 넘었습니다"),
    (
        "invalid continuous detector event",
        "연속시간 검출 이벤트가 유효하지 않습니다",
    ),
    (
        "finite nonzero vector required",
        "유한한 0이 아닌 벡터가 필요합니다",
    ),
    (
        "non-finite Cartesian coordinate",
        "Cartesian 좌표가 유한하지 않습니다",
    ),
    (
        "time bins need finite width>0, end>start and representable edges",
        "시간 bin에는 유한한 양의 너비, end>start 및 표현 가능한 경계가 필요합니다",
    ),
    (
        "time frame count must be 1..100000",
        "시간 프레임 수는 1..100000이어야 합니다",
    ),
    (
        "invalid output resolution",
        "출력 해상도가 유효하지 않습니다",
    ),
    (
        "duplicate ray_id in absorbing detector event list",
        "흡수형 검출기 이벤트 목록에 중복 ray_id가 있습니다",
    ),
    (
        "non-finite ZAMO angular velocity",
        "ZAMO 각속도가 유한하지 않습니다",
    ),
    (
        "stationary plane must lie entirely outside r=2 (static timelike region)",
        "정지 평면 전체가 r=2 바깥의 정적 시간꼴 영역에 있어야 합니다",
    ),
    ("duplicate trajectory ray_id", "궤적 ray_id가 중복됩니다"),
    ("invalid ray coordinates", "광선 좌표가 유효하지 않습니다"),
    (
        "invalid or unordered trajectory times",
        "궤적 시간이 유효하지 않거나 시간순이 아닙니다",
    ),
    ("event/ray identity mismatch", "이벤트 / 광선 식별자 불일치"),
    (
        "Detected/event count mismatch",
        "Detected / 이벤트 개수 불일치",
    ),
    (
        "cancelled by a newer generation",
        "새 요청으로 인해 취소됐습니다",
    ),
    (
        "ray direction must be nonzero",
        "광선 방향은 0벡터일 수 없습니다",
    ),
    (
        "at least one ray initial condition is required",
        "광선 초기조건이 하나 이상 필요합니다",
    ),
    (
        "every ray position, direction and t_emit must be finite",
        "모든 광선의 위치, 방향, t_emit은 유한해야 합니다",
    ),
    (
        "ray must start between capture cutoff and escape radius",
        "광선은 포획 종료 경계와 탈출 반지름 사이에서 출발해야 합니다",
    ),
    (
        "non-finite detector distance",
        "검출기까지의 거리가 유한하지 않습니다",
    ),
    (
        "detector root did not converge within coordinate tolerance",
        "검출기 교차점의 근이 좌표 허용오차 이내로 수렴하지 않았습니다",
    ),
    (
        "detector root state exceeds ODE tolerance",
        "검출기 교차 상태가 ODE 허용오차를 초과합니다",
    ),
    (
        "KERR_RAY_THREADS is not valid Unicode",
        "KERR_RAY_THREADS가 유효한 Unicode가 아닙니다",
    ),
    (
        "KERR_RAY_THREADS must be positive",
        "KERR_RAY_THREADS는 양수여야 합니다",
    ),
    (
        "ray thread count must be positive",
        "광선 작업 스레드 수는 양수여야 합니다",
    ),
    (
        "cancelled before first ray",
        "첫 광선 계산 전에 취소됐습니다",
    ),
    ("cancelled after ray", "광선 계산 후 취소됐습니다"),
    (
        "cancelled before next ray",
        "다음 광선 계산 전에 취소됐습니다",
    ),
    (
        "cancelled while rays were running",
        "광선 계산 중 취소됐습니다",
    ),
    (
        "ray workers disconnected",
        "광선 작업 스레드와 연결이 끊어졌습니다",
    ),
    ("missing ray result", "광선 결과가 누락됐습니다"),
    (
        "trajectory ray_id has no diagnostic record",
        "궤적 ray_id에 해당하는 진단 기록이 없습니다",
    ),
    (
        "legacy ray ID out of source range",
        "과거 광선 ID가 광원 범위를 벗어납니다",
    ),
    (
        "unsupported physical/coordinate convention",
        "지원하지 않는 물리/좌표 규약입니다",
    ),
    (
        "missing simulator provenance",
        "시뮬레이터 버전 이력이 누락됐습니다",
    ),
    (
        "batch requires seeded binary format",
        "batch에는 seed 기반 binary 형식이 필요합니다",
    ),
    ("missing ray_generator", "ray_generator가 누락됐습니다"),
    (
        "unsupported binary format specification",
        "지원하지 않는 binary 형식 사양입니다",
    ),
    (
        "unsupported data_format_version",
        "지원하지 않는 data_format_version입니다",
    ),
    ("missing seed metadata", "seed 메타데이터가 누락됐습니다"),
    (
        "mixed present/missing ray_id values",
        "ray_id가 있는 값과 없는 값이 섞여 있습니다",
    ),
    ("ray_id out of range", "ray_id가 범위를 벗어납니다"),
    ("duplicate ray_id", "중복 ray_id가 있습니다"),
    (
        "only local energy=1 is supported",
        "국소 energy=1만 지원합니다",
    ),
    (
        "status/hit nullability mismatch",
        "상태 / 검출값의 null 여부가 일치하지 않습니다",
    ),
    (
        "invalid detector hit",
        "검출기 도달 데이터가 유효하지 않습니다",
    ),
    (
        "canonical input association failed",
        "정준 순서의 초기조건 연결에 실패했습니다",
    ),
    (
        "simulation numbering exhausted",
        "사용 가능한 시뮬레이션 번호가 없습니다",
    ),
    (
        "new archives require solver-generated ray_id on every row",
        "새 저장 데이터의 모든 행에는 solver가 생성한 ray_id가 필요합니다",
    ),
    (
        "expected sim_N.parquet",
        "sim_N.parquet 파일명이 필요합니다",
    ),
    ("expected sim_N.bin", "sim_N.bin 파일명이 필요합니다"),
    (
        "simulation number must be positive without padding",
        "시뮬레이션 번호는 앞에 0이 없는 양의 정수여야 합니다",
    ),
    (
        "unsupported common.json version/convention",
        "지원하지 않는 common.json 버전/규약입니다",
    ),
    (
        "non-finite canonical state",
        "정준 상태가 유한하지 않습니다",
    ),
    ("missing Parquet column", "Parquet 열이 누락됐습니다"),
    ("missing ray_id", "ray_id가 누락됐습니다"),
    ("missing file metadata", "파일 메타데이터가 누락됐습니다"),
    (
        "unsupported Parquet schema: exact uint64/float64/UTF8/nullable columns required",
        "지원하지 않는 Parquet schema입니다. 정확한 uint64/float64/UTF8/nullable 열이 필요합니다",
    ),
    (
        "unsupported Parquet metadata version/type",
        "지원하지 않는 Parquet 메타데이터 버전/형식입니다",
    ),
    (
        "expected non-null uint64 ray_id",
        "null이 아닌 uint64 ray_id가 필요합니다",
    ),
    (
        "expected non-null float64",
        "null이 아닌 float64가 필요합니다",
    ),
    (
        "invalid nullable hit type",
        "nullable 검출값의 자료형이 유효하지 않습니다",
    ),
    ("partial detector hit nulls", "검출값 일부만 null입니다"),
    ("invalid status type", "상태 자료형이 유효하지 않습니다"),
    ("Parquet row count mismatch", "Parquet 행 수 불일치"),
    (
        "non-finite RK derivative or step",
        "RK 미분값 또는 스텝이 유한하지 않습니다",
    ),
    ("non-finite RK stage", "RK 단계 상태가 유한하지 않습니다"),
    ("non-finite RK derivative", "RK 미분값이 유한하지 않습니다"),
    (
        "BL metric requires r > r_plus and 0 < theta < pi",
        "BL 계량에는 r > r_plus 및 0 < theta < pi가 필요합니다",
    ),
    (
        "cannot associate loaded physical input with computed ray",
        "불러온 물리 초기조건을 계산된 광선에 연결할 수 없습니다",
    ),
    ("missing generator", "생성기가 누락됐습니다"),
    ("missing seed/hash", "seed/hash가 누락됐습니다"),
    (
        "INITIAL CONDITIONS HASH MISMATCH: initial-condition reproduction failed; geodesic integration not started",
        "INITIAL CONDITIONS HASH MISMATCH: 초기조건 재생성 실패; 측지선 적분을 시작하지 않았습니다",
    ),
    (
        "seeded result row count mismatch",
        "seed 기반 결과의 행 수 불일치",
    ),
    (
        "seeded canonical row association mismatch",
        "seed 기반 정준 행 연결 불일치",
    ),
    (
        "invalid binary status code",
        "binary 상태 코드가 유효하지 않습니다",
    ),
    ("missing seed", "seed가 누락됐습니다"),
    (
        "binary file requires seeded common.json",
        "binary 파일에는 seed 기반 common.json이 필요합니다",
    ),
    (
        "batch number overflow",
        "batch 번호가 표현 범위를 넘었습니다",
    ),
    (
        "binary chi does not match batch numbering/specification",
        "binary의 chi가 batch 번호/사양과 일치하지 않습니다",
    ),
    (
        "binary length mismatch (truncated/trailing data)",
        "binary 길이 불일치 (잘렸거나 뒤에 불필요한 데이터가 있음)",
    ),
    (
        "non-finite null constraint",
        "영조건 값이 유한하지 않습니다",
    ),
    (
        "non-finite Carter constant",
        "Carter 상수가 유한하지 않습니다",
    ),
    (
        "finite coordinates, unit ZAMO direction and positive energy required",
        "유한한 좌표, 단위 ZAMO 방향 및 양의 에너지가 필요합니다",
    ),
    (
        "ZAMO conversion violates null tolerance 1e-10",
        "ZAMO 변환이 영조건 허용오차 1e-10을 초과합니다",
    ),
    (
        "initial state must be finite and outside BL horizon",
        "초기 상태는 유한하며 BL 지평선 바깥에 있어야 합니다",
    ),
    (
        "ray must be future-directed with positive ZAMO energy",
        "광선은 양의 ZAMO 에너지를 가지며 미래 방향이어야 합니다",
    ),
    (
        "initial state violates null constraint",
        "초기 상태가 영조건을 위반합니다",
    ),
    (
        "required step below minimum or affine precision",
        "필요한 스텝이 최솟값 또는 affine parameter 정밀도보다 작습니다",
    ),
    (
        "non-finite error estimate",
        "오차 추정값이 유한하지 않습니다",
    ),
    (
        "accepted state exceeds energy-scaled null tolerance",
        "수락된 상태가 에너지로 정규화한 영조건 허용오차를 초과합니다",
    ),
    (
        "maximum attempted steps reached; escape has NOT been established",
        "최대 시도 스텝 수에 도달했습니다. 탈출로 판정하지 않습니다",
    ),
    (
        "grid count must be 1..100000",
        "격자 개수는 1..100000이어야 합니다",
    ),
    (
        "invalid emission list",
        "광선 방출 목록이 유효하지 않습니다",
    ),
    (
        "source must be between capture cutoff and escape radius",
        "광원은 포획 종료 경계와 탈출 반지름 사이에 있어야 합니다",
    ),
    (
        "invalid integration settings: positive finite values, ordered steps, exterior escape radius required",
        "적분 설정이 유효하지 않습니다. 양의 유한한 값, 순서가 맞는 스텝, 외부 탈출 반지름이 필요합니다",
    ),
    (
        "invalid visualization/camera/sampling settings",
        "시각화/카메라/표본 설정이 유효하지 않습니다",
    ),
    (
        "free-fall extent 3..100, density 3..17, speed 0.01..100 required",
        "자유낙하 범위 3..100, 밀도 3..17, 속도 0.01..100이 필요합니다",
    ),
    ("missing diagnostic {name}", "진단 항목 누락: {0}"),
    ("short diagnostic {name}", "진단 항목 부족: {0}"),
    (
        "no sim_<positive integer>{extension} files in folder",
        "폴더에 sim_<양의 정수>{0} 파일이 없습니다",
    ),
    ("unknown status {s}", "알 수 없는 상태: {0}"),
    (
        "missing or duplicate metadata {name}",
        "메타데이터 누락 또는 중복: {0}",
    ),
    ("null metadata {name}", "null 메타데이터: {0}"),
    (
        "chi must be finite and in [-{MAX_SPIN},{MAX_SPIN}]; extremal Kerr is excluded",
        "chi는 유한하며 [-{0},{1}] 범위여야 합니다. 극단적 Kerr는 제외합니다",
    ),
    ("Settings", "설정"),
    ("Language", "언어"),
    (
        "Drag to edit or click to enter a value.\nPress Shift while dragging for better control.",
        "드래그하여 조절하거나 클릭하여 값을 입력하세요.\nShift를 누른 채 드래그하면 더 세밀하게 조절할 수 있습니다.",
    ),
    (
        "Korean system font unavailable. Install a Hangul-capable system font and restart.",
        "한글 시스템 글꼴이 없습니다. 한글 글꼴을 설치한 후 다시 실행하세요.",
    ),
    ("KERR / GEODESIC LAB", "KERR / 측지선 연구실"),
    (
        "CPU f64 physics  /  wgpu 3D",
        "CPU f64 물리 계산  /  wgpu 3D",
    ),
    (
        "G = c = M = 1   |   +Z spin   |   Cartesian-like BL coordinates, not a Euclidean embedding or Kerr-Schild time",
        "G = c = M = 1  |  +Z 회전축  |  Cartesian-like BL 좌표: 유클리드 공간의 임베딩이나 Kerr-Schild 시간이 아님",
    ),
    ("Pause", "일시정지"),
    ("Play", "재생"),
    ("Stop", "정지"),
    ("Reset time", "시간 초기화"),
    (
        "Loading / calculating on the reference worker",
        "기준 계산 작업 스레드에서 불러오기 / 계산 중",
    ),
    (
        "Auto-save pending calculation completion",
        "계산이 끝나면 자동 저장합니다",
    ),
    (
        "CPU f64 reference calculation in background",
        "백그라운드에서 CPU f64 기준 계산 중",
    ),
    ("ERROR: ", "오류: "),
    ("Load ERROR: ", "불러오기 오류: "),
    ("Export failed: ", "내보내기 실패: "),
    ("Detector display: ", "검출기 표시 오류: "),
    ("UI settings error: ", "UI 설정 저장/불러오기 오류: "),
    (
        "Export worker disconnected",
        "내보내기 작업 스레드와 연결이 끊어졌습니다",
    ),
    ("Configuration saved", "설정을 저장했습니다"),
    (
        "Current detector PNG + count CSV saved",
        "현재 검출기의 PNG 이미지와 개수 CSV를 저장했습니다",
    ),
    (
        "An export is already running",
        "내보내기가 이미 진행 중입니다",
    ),
    (
        "Exporting trajectories, events, PNG/APNG on a worker",
        "작업 스레드에서 궤적, 이벤트, PNG/APNG를 내보내는 중",
    ),
    (
        "Reproducing saved initial conditions on CPU worker; no auto-save",
        "CPU 작업 스레드에서 저장된 초기조건을 재현 계산 중: 자동 저장하지 않습니다",
    ),
    (
        "SourcePlane is not stored. Source controls define a NEW experiment only.",
        "SourcePlane이 저장되지 않았습니다. 광원 설정은 새 실험에만 적용됩니다.",
    ),
    (
        "Recompute with current physical settings",
        "현재 물리 설정으로 다시 계산",
    ),
    ("Export (new filenames only)", "내보내기 (새 파일명만 사용)"),
    ("Screenshot", "화면 캡처"),
    ("Save visualization config", "시각화 설정 저장"),
    (
        "Export current detector image + CSV",
        "현재 검출기 이미지 + CSV 내보내기",
    ),
    (
        "Save complete run + frame sequences",
        "전체 실행 결과 + 프레임 시퀀스 저장",
    ),
    ("Trajectory export stride ", "궤적 내보내기 간격 "),
    ("Observation", "관측 정보"),
    ("Active", "ACT - 진행 중"),
    ("Detected", "DET - 검출"),
    ("Captured", "CAP - 포획"),
    ("Escaped", "ESC - 탈출"),
    ("NumericalFailure", "NUM - 수치 계산 실패"),
    (
        "TRAJECTORY UNAVAILABLE\nEvents and detector counts remain valid.",
        "궤적 데이터 없음\n이벤트와 검출기 개수 데이터는 유효합니다.",
    ),
    ("Ray ID", "광선 ID"),
    ("Select", "선택"),
    ("Clear", "선택 해제"),
    ("No detector hit", "검출기에 도달하지 않음"),
    ("Conservation / GPU cache", "보존량 / GPU 캐시"),
    ("max |null|", "최대 |null|"),
    ("max rel E", "E 최대 상대오차"),
    ("max rel Lz", "Lz 최대 상대오차"),
    ("max |delta Q|", "최대 |delta Q|"),
    ("max rel Q", "Q 최대 상대오차"),
    (
        "u right / v up; coordinate pixel counts. White level configurable.",
        "u는 오른쪽, v는 위쪽입니다. 좌표 픽셀별 개수이며 흰색 기준을 조절할 수 있습니다.",
    ),
    (
        "Physical settings changed; waiting 450 ms before reference solve",
        "물리 설정 변경됨: 450 ms 대기 후 기준 계산을 시작합니다",
    ),
    ("Fit scene", "장면에 맞추기"),
    ("Pan drag", "이동 드래그"),
    ("Reset camera", "카메라 초기화"),
    ("Kerr center", "Kerr 중심"),
    ("Source", "광원"),
    ("Detector", "검출기"),
    (
        "Drag: orbit / Pan drag  |  Wheel: zoom  |  Arrows: orbit  |  Shift+arrows: pan  |  +/-: zoom",
        "드래그: 회전 / 이동  |  휠: 확대·축소  |  방향키: 회전  |  Shift+방향키: 이동  |  +/-: 확대·축소",
    ),
    (
        "No detector image available",
        "사용 가능한 검출기 이미지가 없습니다",
    ),
    ("Center (coordinate M)", "중심 (좌표 단위 M)"),
    ("Normal (XYZ)", "법선 (XYZ)"),
    ("Local v / up (XYZ)", "국소 v / 위쪽 방향 (XYZ)"),
    ("Width / height", "너비 / 높이"),
    ("Scene", "장면"),
    ("Visibility", "표시 항목"),
    ("Detector world overlay", "3D 검출기 오버레이"),
    ("Detector image panel", "검출기 이미지 패널"),
    (
        "Display settings (no new physics)",
        "표시 설정 (물리 재계산 없음)",
    ),
    (
        "Rendered trajectories (physical rays unchanged)",
        "표시할 궤적 수 (물리 광선 수는 유지)",
    ),
    ("Line px", "선 두께 (px)"),
    ("Grid auto-fit", "좌표격자 자동 맞춤"),
    ("Extent ", "범위 "),
    ("Spacing ", "간격 "),
    (
        "At most 40 grid intervals/axis; effective spacing shown below.",
        "축마다 최대 40개 격자 구간입니다. 실제 간격은 진단 영역에 표시됩니다.",
    ),
    ("Field samples/axis", "장 표본 수 / 축"),
    (
        "3D field (otherwise z=0 slice)",
        "3D 장 표시 (해제 시 z=0 단면)",
    ),
    ("Field extent ", "장 표시 범위 "),
    ("Arrow time scale M ", "화살표 시간 간격 M "),
    (
        "Arrows: omega * (-Y,X,0) * display interval. Coordinate angular velocity, not a force.",
        "화살표: omega × (-Y,X,0) × 표시 시간 간격. 힘이 아닌 좌표 각속도를 나타냅니다.",
    ),
    (
        "Simulation (background recomputation)",
        "시뮬레이션 (백그라운드 재계산)",
    ),
    ("Stratified ray generator", "층화 표본 광선 생성기"),
    ("Launch region center (M)", "발사 영역 중심 (M)"),
    ("Unit axis 1 (XYZ)", "단위축 1 (XYZ)"),
    ("Unit axis 2 (XYZ)", "단위축 2 (XYZ)"),
    ("Cell count 1 / 2", "셀 개수 1 / 2"),
    ("Cell size (M)", "셀 크기 (M)"),
    (
        "Fixed CartesianSpatial direction (no noise)",
        "고정 CartesianSpatial 방향 (잡음 없음)",
    ),
    (
        "New calculation: fresh OS 256-bit seed; cell size stays fixed when count changes.",
        "새 계산은 OS에서 새 256-bit seed를 생성합니다. 셀 개수를 바꿔도 셀 크기는 유지됩니다.",
    ),
    (
        "SourcePlane (legacy preset)",
        "광원면 SourcePlane (기존 프리셋)",
    ),
    ("Ray grid X / Y", "광선 격자 X / Y"),
    ("Use rectangular launch grid", "직사각형 발사 격자 사용"),
    (
        "16x16 is the preserved 256-ray regression preset.",
        "16×16은 기존 256개 광선 회귀 검증 프리셋입니다.",
    ),
    (
        "Direction in local ZAMO basis",
        "국소 ZAMO 기저에서 방향 지정",
    ),
    ("(radial, polar, azimuthal)", "(반지름, 극각, 방위각 방향)"),
    ("Coordinate spatial direction (XYZ)", "좌표 공간 방향 (XYZ)"),
    ("DetectorPlane", "검출면 DetectorPlane"),
    ("Reference accuracy", "기준 계산 정확도"),
    ("Detector / playback", "검출기 / 재생"),
    ("Static", "전체 궤적"),
    ("Propagation", "광선 전파"),
    ("Full trajectories", "전체 궤적"),
    ("Ray propagation", "광선 전파"),
    ("Detector playback", "검출기 재생"),
    ("Accumulated", "전체 누적"),
    ("Instantaneous", "현재 시간 구간"),
    ("Cumulative", "현재 시각까지 누적"),
    ("All accumulated hits", "모든 검출 이벤트 누적"),
    ("Instantaneous time bin", "현재 시간 bin"),
    ("Cumulative to current time", "현재 시각까지 누적"),
    ("Physical M / wall s ", "물리 M / 실제 초 "),
    ("Counts / white ", "흰색 기준 개수 "),
    ("Pixels X / Y", "픽셀 수 X / Y"),
    (
        "Time controls and detector binning reuse every stored event.",
        "시간 조절과 검출기 binning은 저장된 모든 이벤트를 재사용합니다.",
    ),
    ("Cartesian Reference Grid", "Cartesian 기준 좌표격자"),
    ("XYZ axes", "XYZ 좌표축"),
    ("Kerr spin axis (+Z)", "Kerr 회전축 (+Z)"),
    ("Event horizon", "사건의 지평선"),
    ("Ergosurface", "에르고면"),
    ("SourcePlane", "광원면 SourcePlane"),
    ("Launch points", "광선 발사점"),
    ("Trajectories", "광선 궤적"),
    ("Detector hit points", "검출기 도달점"),
    ("Frame dragging", "프레임 끌림"),
    ("Frame Dragging", "프레임 끌림"),
    ("Current ray positions", "현재 광선 위치"),
    (
        "Reading archive (no physics calculation)...",
        "저장 데이터 읽는 중 (물리 계산 없음)...",
    ),
    (
        "Load worker disconnected",
        "불러오기 작업 스레드와 연결이 끊어졌습니다",
    ),
    ("Standard simulation archive", "표준 시뮬레이션 저장 데이터"),
    (
        "Auto-save root (new calculations only)",
        "자동 저장 위치 (새 계산에만 적용)",
    ),
    ("Simulation folder path", "시뮬레이션 폴더 경로"),
    ("Load Folder", "폴더 불러오기"),
    (
        "Reproduce selected simulation",
        "선택한 시뮬레이션 재현 계산",
    ),
    ("Reproduction PASS", "재현 검증 PASS"),
    ("Reproduction FAIL", "재현 검증 FAIL"),
    (
        "Initial conditions SHA-256: PASS; seed regenerated; no stored ray IDs",
        "초기조건 SHA-256: PASS; seed로 재생성; 저장된 ray_id 사용 안 함",
    ),
    (
        "Stored IDs NOT VERIFIED (legacy file)",
        "저장된 ID 검증 안 됨 (과거 파일)",
    ),
    ("Stored / reproduced", "저장 결과 / 재현 결과"),
    ("Full comparison", "전체 비교 결과"),
    (
        "free-fall worker disconnected",
        "자유낙하 작업 스레드와 연결이 끊어졌습니다",
    ),
    (
        "Kerr Free-Fall Grid (visualization only)",
        "Kerr 자유낙하 격자 (시각화 전용)",
    ),
    ("Kerr Free-Fall Grid", "Kerr 자유낙하 격자"),
    (
        "E=1, Lz=0 rain observers; connected markers, not moving space or a Euclidean embedding.",
        "E=1, Lz=0인 자유낙하 관측자들을 연결합니다. 공간 자체의 이동이나 유클리드 임베딩이 아닙니다.",
    ),
    ("Nodes/axis ", "격자점 / 축 "),
    ("Grid M / wall s ", "격자 M / 실제 초 "),
    ("Pause grid", "격자 일시정지"),
    ("Play grid", "격자 재생"),
    ("Reset grid", "격자 초기화"),
    (
        "Precomputing free-fall cache (not ray physics)...",
        "자유낙하 캐시 계산 중 (광선 물리 계산 아님)...",
    ),
    ("Free-fall metric diagnostics", "자유낙하 계량 진단"),
    (
        "Initial probe removed at numerical horizon margin.",
        "초기 검사점이 지평선 수치 종료 여유 구간에서 제거됐습니다.",
    ),
    (
        "BL margin 0.02 M; axis excluded. Grid clock is independent of photon playback; hidden grid pauses. Reset restores ALL nodes.",
        "BL 여유 구간 0.02 M, 회전축 제외. 격자 시간은 광선 재생과 독립적이며 숨기면 정지합니다. 초기화하면 모든 격자점이 복원됩니다.",
    ),
    (
        "all accepted f64 reference samples retained in memory",
        "수락된 모든 f64 기준 표본을 메모리에 보존합니다",
    ),
    (
        "Canonical CPU trajectories; standard auto-save enabled",
        "정준 순서의 CPU 궤적; 표준 자동 저장 활성화",
    ),
    (
        "Fresh reproduction trajectories. SourcePlane/UV were not stored and are unavailable.",
        "새로 재현 계산한 궤적입니다. 저장되지 않은 SourcePlane/UV는 제공되지 않습니다.",
    ),
    (
        "Legacy event-only data: trajectories are absent. Recompute explicitly to obtain paths; original files remain unchanged.",
        "과거 이벤트 전용 데이터: 궤적이 없습니다. 경로를 얻으려면 명시적으로 재계산해야 하며 원본 파일은 유지됩니다.",
    ),
    (
        "Cannot locate UI preferences beside executable",
        "실행 파일 옆 UI 설정 위치를 확인할 수 없습니다",
    ),
    // Formatted display text. The original numeric precision is kept verbatim.
    ("{} / {} rays", "광선 {0} / {1}"),
    ("Saved complete run to {}", "전체 실행 결과 저장 위치: {0}"),
    ("Screenshot saved: {}", "화면 캡처 저장: {0}"),
    ("{:.1} FPS  |  {:.2} ms mean", "{0} FPS  |  평균 {1} ms"),
    ("chi {:.4} | physical rays {}", "chi {0} | 물리 광선 {1}"),
    ("Rendered trajectories {}", "표시 궤적 {0}"),
    ("Reference calculation: {:.3} s", "기준 계산: {0} s"),
    ("Ray {} / {}", "광선 {0} / {1}"),
    ("Launch u,v: {:.4?}", "발사 u,v: {0}"),
    ("Launch XYZ: {:.4?}", "발사 XYZ: {0}"),
    ("Cartesian direction: {direction:?}", "Cartesian 방향: {0}"),
    ("Direction: {:?}", "방향: {0}"),
    ("Stop: {}", "종료 이유: {0}"),
    (
        "Steps {} accepted / {} rejected",
        "적분 스텝: 수락 {0} / 거부 {1}",
    ),
    (
        "Ray uploads: {}\nGeometry uploads: {}",
        "광선 업로드: {0}\n기하 데이터 업로드: {1}",
    ),
    (
        "Grid extent {:.2}, spacing {:.2}",
        "좌표격자 범위 {0}, 간격 {1}",
    ),
    ("max sampled omega {:.6e} / M", "표본 omega 최댓값 {0} / M"),
    ("p95 frame {:.2} ms", "프레임 시간 p95 {0} ms"),
    ("Detector {:?}", "검출기 {0}"),
    ("Shown {} / total {} hits", "표시 {0} / 전체 도달 {1}"),
    (
        "Extent {:?} M; one uniform point per cell",
        "영역 크기 {0} M; 셀마다 균일 표본 점 하나",
    ),
    (
        "{} rays / chi {}\nStored D/C/E/F: {:?}\nNo trajectories stored. Use Reproduce for fresh paths.",
        "광선 {0} / chi {1}\n저장된 DET/CAP/ESC/NUM: {2}\n궤적은 저장되지 않습니다. 재현 계산으로 새 경로를 얻으세요.",
    ),
    ("Loaded folder: {}", "불러온 폴더: {0}"),
    (
        "Rays: {} / {}\nray_id mismatches: {}\nstatus mismatches: {}",
        "광선: {0} / {1}\nray_id 불일치: {2}\n상태 불일치: {3}",
    ),
    ("Stored IDs checked: {} / {}", "검사한 저장 ID: {0} / {1}"),
    ("{name} max error: {:.3e}", "{0} 최대 오차: {1}"),
    (
        "Grid t_BL/M {:.3} / {DURATION}; cache builds {}",
        "격자 t_BL/M {0} / {1}; 캐시 생성 {2}회",
    ),
    (
        "chi {}; visible nodes {}/{}; initial exclusions {}; cache {:.3}s",
        "chi {0}; 표시 격자점 {1}/{2}; 초기 제외 {3}; 캐시 {4}s",
    ),
    (
        "max sampled |g(u,u)+1| {:.3e}",
        "표본 |g(u,u)+1| 최댓값 {0}",
    ),
    (
        "Probe {i}: r {:.5}, theta {:.5}, phi {:.5}",
        "검사점 {0}: r {1}, theta {2}, phi {3}",
    ),
    (
        "t_BL slice gamma_rr/theta/phi: {:.5?}",
        "t_BL 단면 gamma_rr/theta/phi: {0}",
    ),
    (
        "loaded f64 trajectories; stored stride {}",
        "f64 궤적 불러옴; 저장 표본 간격 {0}",
    ),
    ("Auto-saved: {}", "자동 저장됨: {0}"),
    (
        "Auto-save FAILED: {}; calculated result retained",
        "자동 저장 실패: {0}; 계산 결과는 유지됩니다",
    ),
    (
        "Auto-save FAILED: {}; legacy calculation retained",
        "자동 저장 실패: {0}; 기존 방식의 계산 결과는 유지됩니다",
    ),
    (
        "Reproduced {} / {} (not auto-saved)",
        "재현 계산 완료: {0} / {1} (자동 저장 안 함)",
    ),
    // Detailed reproduction display; the source Comparison and CLI summary are unchanged.
    ("Reproduction check", "재현 검증"),
    ("rays: {} / {}", "광선: {0} / {1}"),
    ("canonical mismatch: {}", "정준 순서 불일치: {0}"),
    (
        "regenerated ray_id mismatch: {}",
        "재생성한 ray_id 불일치: {0}",
    ),
    (
        "stored ray_id checked: {} / {} (missing IDs in legacy archives are NOT VERIFIED)",
        "검사한 저장 ray_id: {0} / {1} (과거 파일에서 누락된 ID는 검증 안 됨)",
    ),
    (
        "stored ray_id: not applicable (seeded format stores no IDs)",
        "저장 ray_id: 해당 없음 (seed 형식은 ID를 저장하지 않음)",
    ),
    (
        "stored vs regenerated ray_id mismatch: {}",
        "저장 / 재생성 ray_id 불일치: {0}",
    ),
    (
        "reconstructed initial State vs solver start (bit mismatch): {}",
        "복원 초기 State / solver 시작 상태의 비트 불일치: {0}",
    ),
    (
        "historical initial State: not stored; original-vs-loaded covered by round-trip tests",
        "과거 초기 State: 저장 안 됨; 원본 / 불러오기 일치는 왕복 시험으로 검증",
    ),
    ("status mismatch: {}", "상태 불일치: {0}"),
    ("hit presence mismatch: {}", "검출 여부 불일치: {0}"),
    (
        "counts [Active,Detected,Captured,Escaped,NumericalFailure]: {:?} / {:?}",
        "개수 [ACT,DET,CAP,ESC,NUM]: {0} / {1}",
    ),
    (
        "hit max abs error [u,v,t]: {:?}",
        "검출 최대 절대오차 [u,v,t]: {0}",
    ),
    (
        "hit max rel error [u,v,t]: {:?}",
        "검출 최대 상대오차 [u,v,t]: {0}",
    ),
    (
        "hit bit mismatches [u,v,t]: {:?}",
        "검출 비트 불일치 [u,v,t]: {0}",
    ),
    (
        "hit tolerance: abs <= {} + {} * max(abs(original),abs(new))",
        "검출 허용오차: abs <= {0} + {1} * max(abs(original),abs(new))",
    ),
    ("hit tolerance mismatches: {}", "검출 허용오차 초과: {0}"),
    ("same source/version: {}", "동일 소스/버전: {0}"),
    (
        "reproduction calculation seconds: {}",
        "재현 계산 시간 (초): {0}",
    ),
    ("result: {}", "결과: {0}"),
    (
        "Initial conditions SHA-256: PASS (seed regenerated before integration; no stored IDs)",
        "초기조건 SHA-256: PASS (적분 전 seed로 재생성; 저장 ID 사용 안 함)",
    ),
    // Application validation diagnostics translated only at the GUI boundary.
    (
        "plane needs finite values and positive sizes",
        "평면 값은 유한해야 하며 크기는 양수여야 합니다",
    ),
    (
        "plane e_u, e_v, normal must be orthonormal within 1e-12",
        "평면의 e_u, e_v, normal은 오차 1e-12 이내에서 정규직교해야 합니다",
    ),
    (
        "plane basis must be right handed",
        "평면 기저는 오른손 좌표계여야 합니다",
    ),
    (
        "invalid detector root tolerance/resolution",
        "검출기의 근 찾기 허용오차 또는 해상도가 유효하지 않습니다",
    ),
    (
        "generator requires 1..1048576 cells",
        "생성기에는 1..1048576개의 셀이 필요합니다",
    ),
    (
        "unsupported ray generator specification/version",
        "지원하지 않는 광선 생성기 사양/버전입니다",
    ),
    (
        "invalid finite generator geometry/direction/time",
        "생성기의 기하/방향/시간 값이 유효하지 않거나 유한하지 않습니다",
    ),
    (
        "generator basis must be orthonormal within 1e-12",
        "생성기 기저는 오차 1e-12 이내에서 정규직교해야 합니다",
    ),
    (
        "generator coordinate overflow",
        "생성기 좌표가 표현 범위를 넘었습니다",
    ),
    (
        "free-fall marker outside supported BL exterior chart",
        "자유낙하 표식이 지원하는 BL 외부 좌표 영역을 벗어났습니다",
    ),
    (
        "invalid inward timelike normalization",
        "안쪽 방향 시간꼴 정규화가 유효하지 않습니다",
    ),
    (
        "invalid free-fall 4-velocity / normalization",
        "자유낙하 4-속도 / 정규화가 유효하지 않습니다",
    ),
    (
        "positive finite advection step required",
        "이류 스텝은 양의 유한한 값이어야 합니다",
    ),
    (
        "free-fall cache cancelled",
        "자유낙하 캐시 계산이 취소됐습니다",
    ),
    (
        "export needs a new empty directory; saved data is immutable",
        "내보내기에는 새 빈 폴더가 필요합니다. 저장 데이터는 변경하지 않습니다",
    ),
    ("raw event checksum mismatch", "원본 이벤트 checksum 불일치"),
    ("scene/event list mismatch", "장면 / 이벤트 목록 불일치"),
    (
        "trajectory checksum/filename mismatch",
        "궤적 checksum / 파일명 불일치",
    ),
    (
        "trajectory requires 10 columns",
        "궤적에는 10개 열이 필요합니다",
    ),
    ("cancelled before auto-save", "자동 저장 전에 취소됐습니다"),
    ("Horizon", "Horizon (지평선 종료)"),
    ("Escape", "Escape (탈출)"),
    ("DetectorHit", "DetectorHit (검출기 도달)"),
    ("AffineLimit", "AffineLimit (affine parameter 한도)"),
    ("Failure", "Failure (실패)"),
    ("HorizonCutoff", "HorizonCutoff (지평선 수치 종료)"),
    ("EscapeRadius", "EscapeRadius (탈출 반지름 도달)"),
    ("StepLimit", "StepLimit (스텝 수 한도)"),
    ("StepUnderflow", "StepUnderflow (스텝 크기 하한)"),
    (
        "InvalidInitialState",
        "InvalidInitialState (유효하지 않은 초기 상태)",
    ),
    ("NullViolation", "NullViolation (영조건 위반)"),
    ("NonFinite", "NonFinite (유한하지 않은 값)"),
    ("IntegratorFailure", "IntegratorFailure (적분기 실패)"),
];
