# RS_SEND

## TODO
- [x] Send file/files
- [x] Send folder
- [x] Receive file/files
- [x] Receive folder
- [ ] Send/Receive text
- [ ] Work on UI(Ratatui, clap etc.)
- [ ] Improve error handling

## BUGS
- [ ] Sending a folder does not send all the files in the folder

### Resources
- https://github.com/localsend/protocol
- https://www.shuttle.rs/blog/2023/12/08/clap-rust


## Installing dependicies

```sh
     sudo apt install libssl-dev # ssl libraries

     sudo apt install mingw-w64 # compiler for generating PE executables 
```

### Cross building for Windows on ubuntu

```sh
     rustup target add x86_64-pc-windows-gnu
     cargo build --target x86_64-pc-windows-gnu
```
