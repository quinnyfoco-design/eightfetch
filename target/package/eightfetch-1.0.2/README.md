<div align="center">
  <h1>eightfetch</h1>
  <p><strong>Blazing fast system fetch tool — Rust port of <code>myfetch</code></strong></p>
  <p>
    <a href="#-benchmarks"><strong>benchmarks</strong></a> ·
    <a href="#-install"><strong>install</strong></a> ·
    <a href="#-usage"><strong>usage</strong></a>
  </p>

  <table>
    <tr>
      <td align="left">
        <pre>                    -'                    ╭────────────────────────────────────────╮
                  .o+'                    │ OS: Arch Linux                         │
                 'ooo/                    │ Kernel: Linux 6.18.32-1-lts            │
                '+oooo:                   │ Device: Alienware 16 Aurora AC16250    │
               '+oooooo:                  │ Uptime: 2h 23m                         │
               -+oooooo+:                 │ Packages: 697 (pacman)                 │
             '/:-:++oooo+:                │ Shell: fish                            │
            '/++++/+++++++:               │ DE: Hyprland                           │
           '/++++++++++++++:              │ Terminal: xterm-kitty                  │
          '/+++ooooooooooooo/'            │ Resolution: 2560x1600                  │
         ./ooosssso++osssssso+'           │ CPU: Intel(R) Core(TM) 7 240H          │
        .oossssso-''''/ossssss+'          │ GPU: Intel Graphics                    │
       -osssssso.      :ssssssso.         │ GPU-2: GeForce RTX 5060 Max-Q / Mobile │
      :osssssss/        osssso+++.        │ RAM: 3.6 GiB / 31.0 GiB                │
     /ossssssss/        +ssssooo/-        │ Disk (/): 67GiB / 937GiB (7%)          │
    '/ossssso+/:-        -:/+osssso+-     ╰────────────────────────────────────────╯
   '+sso+:-'                  '.-/+oso:
  '++:.                           '-/+/
  .'                                 '</pre>
      </td>
    </tr>
  </table>
</div>

---

## benchmarks

> tested on Arch, all sysfetches may be slower on other distros

| tool                 | time        | comparision     |
|----------------------|-------------|-----------------|
| **myfetch (C port)** | 725 ms      | **63x slower**  |
| **fastfetch**        | 27.7 ms     | **2.4x slower** |
| **eightfetch**       | **11.5 ms** | **go brr(sorry my humor is bad)** |

> Since i want to be honest, here is how it performed on Pop!_OS:

| tool                 | time        | comparision     |
|----------------------|-------------|-----------------|
| **fastfetch**        | 50.1 ms     | - |
| **eightfetch**       | **40.5 ms** | **~1.2x faster** |

```
$ hyperfine myfetch
  Time (mean ± σ):     725.4 ms ± 78.9 ms  [User: 103.9 ms, System: 164.3 ms]

$ hyperfine fastfetch
  Time (mean ± σ):      27.7 ms ±  2.8 ms  [User: 4.3 ms, System: 12.5 ms]

$ hyperfine 8fetch
  Time (mean ± σ):      11.5 ms ±  1.3 ms  [User: 6.9 ms, System: 4.4 ms]
```

eightfetch is **~63x faster** than the original `myfetch(C port)`, and **~2.4x faster** than `fastfetch`; all while using **zero external dependencies** (pure Rust stdlib)!!

---

## install

### Via cargo (from source)

```bash
cargo install eightfetch
```

Then run:

```bash
8fetch
```

### manually (build + copy)

```bash
# clone
git clone https://github.com/quinnyfoco-design/eightfetch.git
cd eightfetch

# build
cargo build --release

# copy to PATH
cp target/release/eightfetch ~/.cargo/bin/8fetch
# or: sudo cp target/release/eightfetch /usr/local/bin/8fetch

# run it
8fetch
```

---

## usage

```bash
8fetch

# grey monochrome
8fetch --grey

# custom hex color
8fetch --color:5f5f5f
```

---

## how it works

eightfetch avoids subprocess spam(like the deprecated `myfetch(C port)` is doing) by reading directly from sysfs and procfs:

- **OS info** - `/etc/os-release` (one read, 4 fields parsed)
- **Kernel** - `/proc/sys/kernel/osrelease` (file read, no `uname`)
- **GPU** - driver-based sysfs scan via `/sys/bus/pci/drivers/*/` (no `lspci`)
- **CPU** - `/proc/cpuinfo` (already read for virt detection, shared)
- **RAM** - `/proc/meminfo`
- **Disk** - `statvfs()` syscall (no `df` subprocess)
- **Packages** - `readdir("/var/lib/pacman/local/")` (no `pacman -Q`)
- **Shell/Terminal/DE** - Environment variables only
- **Resolution** - `hyprctl monitors` (fastest available for Wayland)

Only **one** subprocess per run: `hyprctl monitors` :)

---

## build requirements

- Rust 1.70+ (edition 2021)
- Linux (uses sysfs, procfs — no Windows/macOS support)

---

## license

MIT
