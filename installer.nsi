Name "Game Translator"
OutFile "installer.exe"
InstallDir "$PROGRAMFILES\Game Translator"
RequestExecutionLevel admin
SetCompress auto
SetCompressor lzma

Page directory
Page instfiles
UninstPage uninstConfirm
UninstPage instfiles

Section "Install"
    SetOutPath $INSTDIR
    File /r "C:\gtk-build\gtk\x64\release\bin\*.*"

    CreateShortCut "$DESKTOP\Game Translator.lnk" "$INSTDIR\game-translator.exe"
    CreateDirectory "$SMPROGRAMS\Game Translator"
    CreateShortCut "$SMPROGRAMS\Game Translator\Game Translator.lnk" "$INSTDIR\game-translator.exe"
    WriteUninstaller "$INSTDIR\Uninstall.exe"
SectionEnd

Section "Uninstall"
    Delete "$DESKTOP\Game Translator.lnk"
    Delete "$SMPROGRAMS\Game Translator\Game Translator.lnk"
    RMDir "$SMPROGRAMS\Game Translator"
    Delete "$INSTDIR\game-translator.exe"
    Delete "$INSTDIR\Uninstall.exe"
    RMDir /r "$INSTDIR"
SectionEnd