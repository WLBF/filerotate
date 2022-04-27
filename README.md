# filerotate

A log rotate tool written in rust.

## Usage

Process will rotate log file or iterate through all log files in a directory and rotate them.

| Param   | Description                                                                                                                                                                                                           |
|---------|-----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| path    | Absolute path to rotate, can be a regular file or directory.                                                                                                                                                          |
| keep    | File or directory Rotate times before being removed. (Simply delete file if 0, truncate file if 1)                                                                                                                    |
| depth   | Recursive depth if path is directory. Depth is infinite if not set.                                                                                                                                                   |
| size    | Only rotate files who's size grow bigger then configured size. Byte size suffix is supported e.g. `KiB, MiB, GiB`. Note size is counted as storage size here, may different from apparent file size listed by `ls -l` |
| regex   | Only rotate files who's name match regex.                                                                                                                                                                             |
| precmd  | Execute command before rotate.                                                                                                                                                                                        |
| postcmd | Execute command after rotate.                                                                                                                                                                                         |
| mode    | Rotate mode can be `MoveCreate` or `CopyTruncate`.                                                                                                                                                                    |


## Todo
- [ ] support gz compress
- [ ] rotate dir overlap check
