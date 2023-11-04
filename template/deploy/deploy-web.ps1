Write-Output "Building Web binary..."
Start-Process -FilePath "trunk" -ArgumentList "build --release" -Wait -NoNewWindow
Set-Location .\dist\

Write-Output "Replacing paths..."
Get-ChildItem -Path .\ -Filter *.html -File -Name | ForEach-Object {
	(Get-Content $_) -replace """/", """./" -replace "'/", "'./" | Out-File -encoding ASCII $_
}

Write-Output "Optimizing WASM..."
Get-ChildItem -Path .\ -Filter *.wasm -File -Name | ForEach-Object {
	Start-Process -FilePath "wasm-opt" -ArgumentList "-Os -o $_ $_" -Wait -NoNewWindow
}

Write-Output "Making archive..."
Start-Process -FilePath "7z" -ArgumentList "a -tzip web.zip *" -Wait -NoNewWindow
Set-Location ..

Write-Output "Copying files..."
New-Item -Path "." -Name "target\deploy" -ItemType Directory -Force
Copy-Item -Path ".\dist\web.zip" -Destination ".\target\deploy\web.zip"

Write-Output "Done"