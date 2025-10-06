# RustDash

[![English](https://img.shields.io/badge/lang-English-blue.svg)](README-en.md)

🦀 Rust로 만든 Prometheus와 Loki용 실시간 터미널 기반 모니터링 대시보드

<img width="1039" height="847" alt="Screenshot 2025-10-06 at 13 10 49" src="https://github.com/user-attachments/assets/968869c3-f153-4974-ad49-d64b7a5ec957" />

## 설치

```bash
# 저장소 클론
git clone https://github.com/yourusername/rustdash
cd rustdash

# 빌드
cargo build --release

# 실행
cargo run
```

## 설정

애플리케이션을 시작하면 엔드포인트를 입력하라는 메시지가 표시됩니다:
```
=== RustDash Configuration ===
Press Enter to use default values.

Enter Prometheus URL [default: http://localhost:9090]: 
Enter Loki URL [default: http://localhost:3100]: 
```

기본 localhost 엔드포인트를 사용하려면 Enter 키를 누르세요.

## 예제

### 커스텀 엔드포인트 사용
```bash
cargo run
# Prometheus URL 입력: http://metrics.example.com:9090
# Loki URL 입력: http://logs.example.com:3100
```

## 주요 기능

### 핵심 기능
- **Prometheus 실시간 메트릭**
  - 초당 HTTP 요청 수
  - URI별 평균 응답 시간
  - 시간 범위 선택기 (1분, 5분, 30분, 1시간, 24시간, 전체)
  - 응답 시간 바 차트 시각화

- **Loki 로그 스트리밍**
  - 5초마다 자동 로그 가져오기
  - 시간순 로그 표시 (최신 로그가 맨 아래)
  - 사용 가능한 로그 스트림 자동 감지

- **반응형 터미널 UI**
  - 다양한 터미널 크기에 적응
  - 최소 터미널 크기: 80x24
  - 터미널 높이에 따른 동적 메트릭 패널 크기 조정

### 패널 탐색
- **탭 탐색**
  - `Tab` - 로그와 메트릭 패널 간 전환
  - `ESC` - 현재 패널 비활성화 (중립 상태)
  - 활성 패널은 청록색 테두리로 강조 표시

### 로그 탐색 및 관리
- **키보드 탐색 (로그 패널 활성 시)**
  - `↑/↓` - 로그 한 줄씩 탐색
  - `Enter` - 긴 로그 메시지 펼치기/접기
  - `[/]` - 5줄씩 빠르게 이동
  - `Page Up/Down` - 페이지 단위로 탐색
  - `Home/End` - 첫 번째/마지막 로그로 이동
  - 선택된 로그는 회색 배경으로 강조 표시

- **로그 펼치기 기능**
  - 잘린 긴 로그는 `▶` 표시
  - `Enter`를 눌러 전체 메시지를 여러 줄로 확장
  - 펼쳐진 로그는 `▼` 표시
  - 다시 `Enter`를 누르면 한 줄로 축소
  - 펼쳐진 상태에서 단어 단위 줄바꿈으로 가독성 향상

- **새 로그 강조**
  - 새 로그는 노란색 화살표(→)로 표시
  - 더 새로운 로그가 도착할 때까지 강조 유지
  - 새 로그 표시를 위한 자동 스크롤 (선택 시 비활성화)

- **클립보드 지원**
  - `c` - 선택한 로그를 시스템 클립보드에 복사
  - 형식: `[LEVEL] message`

### 메트릭 탐색
- **시간 범위 선택 (메트릭 패널 활성 시)**
  - `←/→` - 시간 범위 변경 (1분 → 5분 → 30분 → 1시간 → 24시간 → 전체)
  - `↑/↓` - URI 메트릭 목록 스크롤 (목록이 긴 경우)
  - 새 데이터를 가져올 때 로딩 표시기 표시

### 표시 정보
- **헤더 섹션**
  - 현재 엔드포인트 (Prometheus & Loki URL)
  - 마지막 가져오기 시간 (서버에서 데이터를 검색한 시간)
  - 마지막 업데이트 시간 (UI가 새로 고쳐진 시간)
  - 연결 상태 및 새 로그 수

## 조작법

### 기본 조작
- `q` - 애플리케이션 종료
- `r` - 수동 새로고침
- `Tab` - 패널 간 전환
- `ESC` - 현재 패널 비활성화

### 로그 패널 (활성 시)
- `↑/↓` - 로그 탐색
- `Enter` - 긴 로그 메시지 펼치기/접기 (접힌 상태 ▶, 펼친 상태 ▼)
- `[/]` - 5줄 위/아래로 이동
- `Page Up/Down` - 페이지 단위로 탐색
- `Home/End` - 첫 번째/마지막 로그로 이동
- `c` - 선택한 로그를 클립보드에 복사

### 메트릭 패널 (활성 시)
- `←/→` - 시간 범위 변경
- `↑/↓` - 메트릭 스크롤 (목록이 긴 경우)

## 설정

애플리케이션 시작 시 설정을 입력받습니다:

**기본값** (Enter 키만 누르면 사용)
- Prometheus: `http://localhost:9090`
- Loki: `http://localhost:3100`

**커스텀 엔드포인트**
- 프롬프트가 표시되면 사용자 정의 URL 입력
- 예: `http://prometheus.example.com:9090`

## 요구사항

- Rust 1.70 이상
- 최소 크기 80x24의 터미널

참고: 서비스에 액세스할 수 없는 경우 대시보드에 "No data available"이 표시됩니다.

## 문제 해결

대시보드에 데이터가 표시되지 않는 경우:
1. Prometheus/Loki가 실행 중인지 확인
2. 입력한 URL이 올바른지 확인
3. 서비스가 컴퓨터에서 액세스 가능한지 확인
4. 원격 엔드포인트를 사용하는 경우 방화벽 설정 확인

## 라이선스

MIT