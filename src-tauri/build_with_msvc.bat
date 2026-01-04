@echo off
call "C:\Program Files\Microsoft Visual Studio\2022\Community\VC\Auxiliary\Build\vcvarsall.bat" x64
cd /d H:\GitHub\Oxide-Lab\src-tauri
cargo build --features cuda,flash-attn --release
