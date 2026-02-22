$Source = "C:\Users\ishan\.gemini\tmp\sublime-rust-cpu\chats"
$Destination = ".gemini"

Copy-Item -Path "$Source\*" -Destination $Destination -Recurse -Force
