@echo off

pyinstaller pyinstaller.spec
move dist\main.exe etternagraph.exe
