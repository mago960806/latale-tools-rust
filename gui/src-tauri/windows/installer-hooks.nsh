!macro NSIS_HOOK_POSTINSTALL
  WriteRegStr SHCTX "Software\Classes\SystemFileAssociations\.spf\shell\LaTaleTools.Verify" "MUIVerb" "使用 LaTale Tools 验证"
  WriteRegStr SHCTX "Software\Classes\SystemFileAssociations\.spf\shell\LaTaleTools.Verify" "Icon" "$INSTDIR\latale-tools-gui.exe"
  WriteRegStr SHCTX "Software\Classes\SystemFileAssociations\.spf\shell\LaTaleTools.Verify\command" "" '$\"$INSTDIR\latale-tools-gui.exe$\" --action verify $\"%1$\"'

  WriteRegStr SHCTX "Software\Classes\SystemFileAssociations\.spf\shell\LaTaleTools.Unpack" "MUIVerb" "使用 LaTale Tools 解包"
  WriteRegStr SHCTX "Software\Classes\SystemFileAssociations\.spf\shell\LaTaleTools.Unpack" "Icon" "$INSTDIR\latale-tools-gui.exe"
  WriteRegStr SHCTX "Software\Classes\SystemFileAssociations\.spf\shell\LaTaleTools.Unpack\command" "" '$\"$INSTDIR\latale-tools-gui.exe$\" --action unpack $\"%1$\"'

  WriteRegStr SHCTX "Software\Classes\SystemFileAssociations\.ldt\shell\LaTaleTools.Convert" "MUIVerb" "使用 LaTale Tools 转换"
  WriteRegStr SHCTX "Software\Classes\SystemFileAssociations\.ldt\shell\LaTaleTools.Convert" "Icon" "$INSTDIR\latale-tools-gui.exe"
  WriteRegStr SHCTX "Software\Classes\SystemFileAssociations\.ldt\shell\LaTaleTools.Convert\command" "" '$\"$INSTDIR\latale-tools-gui.exe$\" --action convert $\"%1$\"'

  WriteRegStr SHCTX "Software\Classes\SystemFileAssociations\.stg\shell\LaTaleTools.Convert" "MUIVerb" "使用 LaTale Tools 转换"
  WriteRegStr SHCTX "Software\Classes\SystemFileAssociations\.stg\shell\LaTaleTools.Convert" "Icon" "$INSTDIR\latale-tools-gui.exe"
  WriteRegStr SHCTX "Software\Classes\SystemFileAssociations\.stg\shell\LaTaleTools.Convert\command" "" '$\"$INSTDIR\latale-tools-gui.exe$\" --action convert $\"%1$\"'
!macroend

!macro NSIS_HOOK_PREUNINSTALL
  DeleteRegKey SHCTX "Software\Classes\SystemFileAssociations\.spf\shell\LaTaleTools.Verify"
  DeleteRegKey SHCTX "Software\Classes\SystemFileAssociations\.spf\shell\LaTaleTools.Unpack"
  DeleteRegKey SHCTX "Software\Classes\SystemFileAssociations\.ldt\shell\LaTaleTools.Convert"
  DeleteRegKey SHCTX "Software\Classes\SystemFileAssociations\.stg\shell\LaTaleTools.Convert"
!macroend
