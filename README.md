# filerotate 

![rust](https://github.com/WLBF/filerotate/actions/workflows/rust.yml/badge.svg)

A log rotate tool written in rust.

## Usage

Program will rotate log file or iterate through all log files in a directory and rotate them. Rotate tasks is configured
in a yaml or json config file. See [example.yaml](example.yaml) for more details.

### Explanation of the config file

| Param           | Description                                                                                                                                                                                                       |
|-----------------|-------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| path (Required) |  Absolute path to rotate, can be a regular file or directory.                                                                                                                                           |
| keep (Required) |  File or directory keep num, including origin file or directory. (delete file if 0, truncate file if 1)                                                                                                 |
| mode (Required) |  Rotate mode can be `MoveCreate` or `CopyTruncate`.                                                                                                                                                     |
| depth           | Recursive depth if path is directory. Depth is infinite if not set.                                                                                                                                               |
| size            | Only rotate file who's size grow bigger then configured size. Byte size suffix is supported e.g. `KiB, mb, G`. Note size is counted as storage size here, may different from apparent file size listed by `ls -l` |
| regex           | Only rotate file who's name match regex.                                                                                                                                                                          |
| precmd          | Execute command before rotate.                                                                                                                                                                                    |
| postcmd         | Execute command after rotate.                                                                                                                                                                                     |

## Example

```yaml
- path: /foo/bar/dir
  keep: 3
  depth: 3
  regex: ".+\\.log$"
  mode: MoveCreate
```

<table>
<tr>
<th>Initial</th>
<th>Round #1</th>
<th>Round #2</th>
</tr>

<tr>
<td>

```
bar
└── dir
    ├── a.log
    ├── b.conf
    └── dir1
        ├── c.conf
        ├── dir2
        │   └── e.log
        ├── dir3
        └── d.log
```

</td>
<td>

```
bar
├── dir
│   ├── a.log
│   ├── b.conf
│   └── dir1
│       ├── c.conf
│       ├── dir2
│       │   └── e.log
│       ├── dir3
│       └── d.log
└── dir.1
    ├── a.log
    └── dir1
        ├── dir2
        ├── dir3
        └── d.log
```

</td>
<td>

```
bar
├── dir
│   ├── a.log
│   ├── b.conf
│   └── dir1
│       ├── c.conf
│       ├── dir2
│       │   └── e.log
│       ├── dir3
│       └── d.log
├── dir.1
│   ├── a.log
│   └── dir1
│       ├── dir2
│       ├── dir3
│       └── d.log
└── dir.2
    ├── a.log
    └── dir1
        ├── dir2
        ├── dir3
        └── d.log
```

</td>
</tr>
</table>





