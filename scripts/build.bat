call cd ..

:: use everything find fxc.exe path
call set GPUI_FXC_PATH=C:\\Program Files (x86)\\Windows Kits\\10\\bin\\10.0.22621.0\\x64\\fxc.exe
call cargo build --release

@pause