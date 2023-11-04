$NAME = "Test/Test"
$VERSION = "1.0.0"

Start-Process -FilePath "butler" -ArgumentList "push .\target\deploy\windows.zip $NAME`:windows --userversion $VERSION" -Wait -NoNewWindow
Start-Process -FilePath "butler" -ArgumentList "push .\target\deploy\linux.zip $NAME`:linux --userversion $VERSION" -Wait -NoNewWindow
Start-Process -FilePath "butler" -ArgumentList "push .\target\deploy\web.zip $NAME`:web --userversion $VERSION" -Wait -NoNewWindow