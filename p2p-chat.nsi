; The name of the installer
Name "P2P Chat"

; The file to write
OutFile "p2pchat.exe"

; Request application privileges for Windows Vista and higher
RequestExecutionLevel admin

; Build Unicode installer
Unicode True

; The default installation directory
InstallDir "$PROGRAMFILES\P2P Chat"

; Registry key to check for directory (so if you install again, it will 
; overwrite the old one automatically)
InstallDirRegKey HKLM "Software\P2P Chat" "Install_Dir"

;--------------------------------

; Pages

Page components
Page directory
Page instfiles

UninstPage uninstConfirm
UninstPage instfiles

;--------------------------------

; The stuff to install
Section "Program files (required)"

  SectionIn RO
  
  ; Set output path to the installation directory.
  SetOutPath $INSTDIR
  
  ; Put file there
  File "p2p-chat.nsi"
  
  ; Write the installation path into the registry
  WriteRegStr HKLM "SOFTWARE\P2P Chat" "Install_Dir" "$INSTDIR"
  
  ; Write the uninstall keys for Windows
  WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\P2P Chat" "DisplayName" "P2P Chat"
  WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\P2P Chat" "UninstallString" '"$INSTDIR\uninstall.exe"'
  WriteRegDWORD HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\P2P Chat" "NoModify" 1
  WriteRegDWORD HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\P2P Chat" "NoRepair" 1
  WriteUninstaller "$INSTDIR\uninstall.exe"
  
SectionEnd

; Optional section (can be disabled by the user)
Section "Start Menu Shortcuts"

  CreateDirectory "$SMPROGRAMS\P2P Chat"
  CreateShortcut "$SMPROGRAMS\P2P Chat\Uninstall.lnk" "$INSTDIR\uninstall.exe"
  CreateShortcut "$SMPROGRAMS\P2P Chat\Example2 (MakeNSISW).lnk" "$INSTDIR\p2p-chat.nsi"

SectionEnd

;--------------------------------

; Uninstaller

Section "Uninstall"
  
  ; Remove registry keys
  DeleteRegKey HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\P2P Chat"
  DeleteRegKey HKLM "SOFTWARE\P2P Chat"

  ; Remove files and uninstaller
  Delete $INSTDIR\p2p-chat.nsi
  Delete $INSTDIR\uninstall.exe

  ; Remove shortcuts, if any
  Delete "$SMPROGRAMS\P2P Chat\*.lnk"

  ; Remove directories
  RMDir "$SMPROGRAMS\P2P Chat"
  RMDir "$INSTDIR"

SectionEnd
