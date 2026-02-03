; ============================================
; DevPort Manager - Custom Installer Hooks
; ============================================

; 바로가기 옵션 변수
Var CreateStartMenuShortcut
Var CreateDesktopShortcut

; 초기화 - 기본값 설정
!macro NSIS_HOOK_POSTINIT
  ; 기본값: 생성함 (1 = yes)
  StrCpy $CreateStartMenuShortcut 1
  StrCpy $CreateDesktopShortcut 1
!macroend

; 설치 전 사용자에게 옵션 질문
!macro NSIS_HOOK_PREINSTALL
  ; 시작 메뉴 바로가기 질문
  MessageBox MB_YESNO|MB_ICONQUESTION "시작 메뉴에 바로가기를 추가하시겠습니까?$\n$\nAdd Start Menu shortcut?" IDYES +2
  StrCpy $CreateStartMenuShortcut 0

  ; 바탕화면 바로가기 질문
  MessageBox MB_YESNO|MB_ICONQUESTION "바탕화면에 바로가기를 추가하시겠습니까?$\n$\nAdd Desktop shortcut?" IDYES +2
  StrCpy $CreateDesktopShortcut 0
!macroend

; 설치 후 바로가기 생성 조건부 처리
!macro NSIS_HOOK_POSTINSTALL
  ; 시작 메뉴 바로가기
  StrCmp $CreateStartMenuShortcut 1 0 skip_startmenu
    CreateDirectory "$SMPROGRAMS\${PRODUCTNAME}"
    CreateShortcut "$SMPROGRAMS\${PRODUCTNAME}\${PRODUCTNAME}.lnk" "$INSTDIR\${MAINBINARYNAME}.exe" "" "$INSTDIR\${MAINBINARYNAME}.exe" 0
  skip_startmenu:

  ; 바탕화면 바로가기
  StrCmp $CreateDesktopShortcut 1 0 skip_desktop
    CreateShortcut "$DESKTOP\${PRODUCTNAME}.lnk" "$INSTDIR\${MAINBINARYNAME}.exe" "" "$INSTDIR\${MAINBINARYNAME}.exe" 0
  skip_desktop:
!macroend
