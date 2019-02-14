# CharlesMine-rs
A triangle minesweeper game in Rust

# Installation
This project is currently windows only. Unfortunately there're still other things that must be noticed

## GNU builds
You'll need a `windres.exe` to compile resources. It is shipped in binutils packages that comes with MinGW-w64 builds.
If everything's done correctly that will be enough. 

However if somehow you made the MINGW gcc.exe took over the rust-mingw one, you'll need various fixes to reach
a consistentency between the two environments. ([Here are some notes about this but written in Chinese](https://zhuanlan.zhihu.com/p/52524621),
and also see [this bug](https://github.com/rust-lang/rust/issues/53454)).

## MSVC builds
Theorically nothing should need to be done. However to workaround [this bug](https://github.com/nabijaczleweli/rust-embed-resource/issues/11),
you have to build under "Visual Studio Command Prompt", or run `vcvarsall.bat` manually before you invoke cargo to build the project.

# Dependencies
Some dependency projects are built alongside with this project. Most noticably, [APIW-rs](https://github.com/crlf0710/apiw-rs),
which aims to grow into a cross-platform GUI crate in a distant future. They're currently at a internal development stage, and seldomly documented.
Please contact me if you're interested in any of them.


