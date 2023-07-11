This little program copies the contents of a Space Engineers script to the directory where the game looks for scripts when you click "Browse Scripts" in the editor interface of a Programmable Block. I might add some more functionality to it in the future.

This is intended to be used with my [script template](https://github.com/Butt4cak3/SETemplate).

## Usage

Simply navigate to a directory with a Script.cs file and run `sebuild.exe` in a terminal. It will look for a `#region Script`, de-indent it and copy it to where the game will find it.
