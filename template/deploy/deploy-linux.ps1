Write-Output "Building Linux binary..."
Start-Process -FilePath "cross" -ArgumentList "build --release --target=x86_64-unknown-linux-gnu" -Wait -NoNewWindow

Write-Output "Preparing files..."
New-Item -Path "." -Name "target\temp" -ItemType Directory -Force
Copy-Item -Path ".\data\" -Destination ".\target\temp\data\" -Recurse
Copy-Item -Path "..\.\target\x86_64-unknown-linux-gnu\release\template" -Destination ".\target\temp\template"

Write-Output "Making archive..."
Set-Location .\target\temp\
Start-Process -FilePath "7z" -ArgumentList "a -tzip linux.zip *" -Wait -NoNewWindow
Set-Location ..\..\
Copy-Item -Path ".\target\temp\linux.zip" -Destination ".\target\deploy\linux.zip"

Write-Output "Cleaning temporary files..."
Remove-Item -Path ".\target\temp\" -Recurse

Write-Output "Done"