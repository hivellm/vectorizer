; Vectorizer GUI Windows Installer Script
; Creates Windows Service for Vectorizer

!define APP_NAME "Vectorizer GUI"
!define VECTORIZER_SERVICE "VectorizerService"

; Custom install section
Section "Install Vectorizer Service"
  ; Copy vectorizer binary
  SetOutPath "$INSTDIR"
  File "${BUILD_RESOURCES_DIR}\..\..\..\..\target\release\vectorizer.exe"
  
  ; Create service
  nsExec::ExecToLog 'sc create ${VECTORIZER_SERVICE} binPath= "$INSTDIR\vectorizer.exe" start= auto DisplayName= "Vectorizer Vector Database"'
  
  ; Set service description
  nsExec::ExecToLog 'sc description ${VECTORIZER_SERVICE} "High-performance vector database for semantic search"'
  
  ; Create config directory
  CreateDirectory "$INSTDIR\config"
  CreateDirectory "$INSTDIR\data"
  CreateDirectory "$INSTDIR\backups"
  
  ; Copy default config if not exists
  IfFileExists "$INSTDIR\config\config.yml" skipconfig
  File /oname=config\config.yml "${BUILD_RESOURCES_DIR}\..\..\config.example.yml"
  skipconfig:
  
  ; Create desktop shortcut
  CreateShortCut "$DESKTOP\${APP_NAME}.lnk" "$INSTDIR\${APP_NAME}.exe" "" "$INSTDIR\${APP_NAME}.exe" 0
  
  ; Create start menu shortcut
  CreateDirectory "$SMPROGRAMS\${APP_NAME}"
  CreateShortCut "$SMPROGRAMS\${APP_NAME}\${APP_NAME}.lnk" "$INSTDIR\${APP_NAME}.exe"
  CreateShortCut "$SMPROGRAMS\${APP_NAME}\Uninstall.lnk" "$INSTDIR\Uninstall.exe"
SectionEnd

; Custom uninstall section
Section "Uninstall"
  ; Stop and delete service
  nsExec::ExecToLog 'sc stop ${VECTORIZER_SERVICE}'
  Sleep 2000
  nsExec::ExecToLog 'sc delete ${VECTORIZER_SERVICE}'
  
  ; Remove files
  Delete "$INSTDIR\vectorizer.exe"
  Delete "$DESKTOP\${APP_NAME}.lnk"
  Delete "$SMPROGRAMS\${APP_NAME}\${APP_NAME}.lnk"
  Delete "$SMPROGRAMS\${APP_NAME}\Uninstall.lnk"
  RMDir "$SMPROGRAMS\${APP_NAME}"
  
  ; Ask user if they want to keep data
  MessageBox MB_YESNO "Do you want to keep your data and configuration?" IDYES keepdata
  RMDir /r "$INSTDIR\config"
  RMDir /r "$INSTDIR\data"
  RMDir /r "$INSTDIR\backups"
  keepdata:
  
  Delete "$INSTDIR\Uninstall.exe"
  RMDir "$INSTDIR"
SectionEnd

