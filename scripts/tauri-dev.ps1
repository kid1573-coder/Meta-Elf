# 确保集成终端能找到 cargo（与 .vscode/settings.json 作用相同，双保险）
$env:Path = "$env:USERPROFILE\.cargo\bin;$env:Path"
Set-Location $PSScriptRoot\..
npm run tauri dev
