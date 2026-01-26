# DevPort Manager V2

## Project Overview
XAMPP를 대체하는 차세대 통합 개발 환경 관리자.
설치, 서비스 관리, 프로젝트 관리, DB 관리까지 올인원 데스크톱 앱.

## Tech Stack
- **Desktop**: Tauri 2.x (Rust backend + WebView frontend)
- **Frontend**: React 19 + TypeScript + Vite + Tailwind CSS 4
- **State**: Zustand with immer middleware
- **Icons**: Lucide React

## Project Structure
```
devport_manager/
├── src-tauri/                 # Rust backend
│   ├── src/
│   │   ├── commands/          # Tauri IPC commands
│   │   │   ├── database.rs    # DB 자동화 커맨드
│   │   │   ├── env.rs         # .env 관리 커맨드
│   │   │   ├── hosts.rs       # hosts 파일 관리
│   │   │   ├── log.rs         # 로그 관리
│   │   │   ├── port.rs        # 포트 스캔
│   │   │   ├── process.rs     # 프로세스 관리
│   │   │   ├── project.rs     # 프로젝트 CRUD
│   │   │   └── service.rs     # 서비스 관리
│   │   ├── models/            # Data models
│   │   │   ├── project.rs     # Project 타입
│   │   │   └── service.rs     # Service 타입 (4단계 상태)
│   │   ├── services/          # Business logic
│   │   │   ├── database_manager.rs  # MariaDB 연동
│   │   │   ├── env_manager.rs       # .env 파싱/쓰기
│   │   │   ├── hosts_manager.rs     # hosts 파일 관리
│   │   │   ├── log_manager.rs       # 로그 로테이션
│   │   │   ├── port_scanner.rs      # netstat 파싱
│   │   │   ├── process_manager.rs   # 프로세스 시작/종료
│   │   │   ├── service_manager.rs   # 서비스 상태 관리
│   │   │   └── storage.rs           # JSON 저장소
│   │   ├── lib.rs             # Tauri app entry
│   │   └── state.rs           # App state
│   └── Cargo.toml
├── src/                       # React frontend
│   ├── components/
│   │   ├── layout/            # AppShell, Sidebar
│   │   ├── modals/            # AddProjectModal, EnvEditorModal
│   │   ├── services/          # ServiceCard, ServiceLogViewer
│   │   ├── views/             # Dashboard, ServicesView, etc.
│   │   └── ...
│   ├── stores/                # Zustand stores
│   ├── types/                 # TypeScript types
│   └── hooks/                 # Custom hooks
└── package.json
```

## Key Commands
```bash
npm run tauri dev          # Run in development mode
npm run tauri build        # Build for production
cargo test                 # Run Rust tests
cargo check                # Check Rust compilation
npx tsc --noEmit           # Check TypeScript
```

## Architecture Notes

### Service Status Model (4단계)
| Status | Icon | Meaning |
|--------|------|---------|
| Running | 🟢 | PID 존재 + Health Check 통과 |
| Stopped | ⚫ | PID 없음 |
| Error | 🔴 | 시작 실패/크래시 |
| Unhealthy | 🟡 | PID 존재 + Health Check 실패 |

### Process Management
- Windows: `taskkill /F /T /PID` 사용
- Port scanning via `netstat -ano` parsing
- Job Object 기반 프로세스 그룹 관리

### IPC Communication
- Tauri commands (Rust -> JS)
- Tauri events (실시간 로그 스트리밍)

### Data Persistence
- `%APPDATA%/devport-manager/` - 설정 저장
- `C:\DevPort\` - 설치 경로 (MVP 고정)

## Coding Conventions

### Rust
- snake_case for functions and variables
- PascalCase for types/structs
- Modules in separate files
- Use `thiserror` for error handling

### TypeScript
- camelCase for variables/functions
- PascalCase for components/interfaces
- Interfaces over type aliases
- One component per file

### Components
- PascalCase file names
- Export from index.ts
- Separate concerns (UI / Logic)

### Stores (Zustand)
- Use immer middleware for immutable updates
- Separate state and actions in interface
- Use selectors for performance

## Default Ports
| Service | Port |
|---------|------|
| Apache | 8080 |
| MariaDB | 3306 |
| phpMyAdmin | 8080/phpmyadmin |
| Projects | 3000-3999 |

## Security Policy
- MariaDB: `127.0.0.1` 바인딩 (localhost only)
- phpMyAdmin: `Require local` (localhost only)
- hosts 파일 편집 시 UAC 권한 요청
- DB 비밀번호: DPAPI로 암호화 저장

## MVP Features Implemented
- [x] Service 모델 (4단계 상태)
- [x] Health Check 시스템
- [x] 로그 시스템 (로테이션, 스트리밍)
- [x] 프로젝트 관리 (CRUD)
- [x] 포트 스캔/충돌 감지
- [x] .env GUI 편집기
- [x] 프로필 관리 (dev/staging/production)
- [x] hosts 파일 자동 관리
- [x] DB 자동화 (생성/덤프/복원)
- [x] Services 탭 UI
- [x] Settings 탭 UI
