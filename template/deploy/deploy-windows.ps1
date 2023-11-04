Write-Output "Building Windows binary..."
Start-Process -FilePath "cargo" -ArgumentList "build --release --target=x86_64-pc-windows-msvc" -Wait -NoNewWindow

Write-Output "Preparing files..."
New-Item -Path "." -Name "target\temp" -ItemType Directory -Force
Copy-Item -Path ".\data\" -Destination ".\target\temp\data\" -Recurse
Copy-Item -Path "..\.\target\x86_64-pc-windows-msvc\release\template.exe" -Destination ".\target\temp\template.exe"

Write-Output "Making archive..."
Set-Location .\target\temp\
Start-Process -FilePath "7z" -ArgumentList "a -tzip windows.zip *" -Wait -NoNewWindow
Set-Location ..\..\
Copy-Item -Path ".\target\temp\windows.zip" -Destination ".\target\deploy\windows.zip"

Write-Output "Cleaning temporary files..."
Remove-Item -Path ".\target\temp\" -Recurse

Write-Output "Done"