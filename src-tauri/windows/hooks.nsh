; Additions to the NSIS installer Tauri generates.

; Removes what Dropwheel registers for the current user while it runs: the dropwheel:
; URL scheme of its notification buttons (only while it still points at this copy, not
; at a portable one) and the name and icon of its notifications. Tauri's own steps
; already remove the "start with Windows" entry. Nothing is removed when updating.
!macro NSIS_HOOK_POSTUNINSTALL
  ${If} $UpdateMode <> 1
    ReadRegStr $R7 HKCU "Software\Classes\dropwheel\shell\open\command" ""
    ${If} $R7 == "$\"$INSTDIR\${MAINBINARYNAME}.exe$\" $\"%1$\""
      DeleteRegKey HKCU "Software\Classes\dropwheel"
    ${EndIf}
    DeleteRegKey HKCU "Software\Classes\AppUserModelId\${BUNDLEID}"
  ${EndIf}
!macroend
