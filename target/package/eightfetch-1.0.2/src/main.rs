use std::env;
use std::fmt::Write as FmtWrite;
use std::fs;
use std::io::{self, BufRead, BufReader};
use std::path::Path;
use std::process::Command;



#[allow(dead_code)]
struct Colors {
    red: &'static str,
    s_red: &'static str,
    steelblue: &'static str,
    steelblue_lt: &'static str,
    s_yellow: &'static str,
    s_black: &'static str,
    s_brightblue: &'static str,
    purple: &'static str,
    blue: &'static str,
    s_blue: &'static str,
    teal: &'static str,
    teal_dim: &'static str,
    white: &'static str,
    orange: &'static str,
    s_orange: &'static str,
    green: &'static str,
    s_green: &'static str,
    reset_c: &'static str,
    label_color: &'static str,
    info_color: &'static str,
}

impl Colors {
    fn grey() -> Self {
        Self {
            red: "", s_red: "", steelblue: "", steelblue_lt: "",
            s_yellow: "", s_black: "", s_brightblue: "", purple: "",
            blue: "", s_blue: "", teal: "", teal_dim: "", white: "",
            orange: "", s_orange: "", green: "", s_green: "", reset_c: "",
            label_color: "", info_color: "",
        }
    }

    fn custom(hex: &str) -> Option<Self> {
        let h = hex.strip_prefix('#').unwrap_or(hex);
        if h.len() != 6 { return None; }
        let r = u8::from_str_radix(&h[0..2], 16).ok()?;
        let g = u8::from_str_radix(&h[2..4], 16).ok()?;
        let b = u8::from_str_radix(&h[4..6], 16).ok()?;
        let c = format!("\x1b[38;2;{};{};{}m", r, g, b);
        let cb = format!("\x1b[1;38;2;{};{};{}m", r, g, b);

        let c = Box::leak(c.into_boxed_str());
        let cb = Box::leak(cb.into_boxed_str());
        Some(Self {
            red: c, s_red: c, steelblue: c, steelblue_lt: c,
            s_yellow: c, s_black: c, s_brightblue: c, purple: c,
            blue: c, s_blue: c, teal: c, teal_dim: c, white: c,
            orange: c, s_orange: c, green: c, s_green: c,
            reset_c: "\x1b[0m",
            label_color: cb,
            info_color: c,
        })
    }

    fn default() -> Self {
        Self {
            s_yellow:     "\x1b[33m",
            s_black:      "\x1b[39m",
            s_brightblue: "\x1b[38;5;153m",
            purple:       "\x1b[35m",
            blue:         "\x1b[1;34m",
            s_blue:       "\x1b[34m",
            teal:         "\x1b[38;5;43m",
            teal_dim:     "\x1b[36m",
            white:        "\x1b[1;37m",
            orange:       "\x1b[1;38;5;208m",
            s_orange:     "\x1b[38;5;208m",
            green:        "\x1b[1;32m",
            s_green:      "\x1b[32m",
            reset_c:      "\x1b[0m",
            red:          "\x1b[1;31m",
            s_red:        "\x1b[31m",
            steelblue:    "\x1b[38;5;68m",
            steelblue_lt: "\x1b[38;5;111m",
            label_color:  "",
            info_color:   "",
        }
    }
}



fn read_file(path: &str) -> io::Result<String> {
    fs::read_to_string(path).map(|s| s.trim().to_owned())
}

fn read_first_line(path: &str) -> io::Result<String> {
    let f = fs::File::open(path)?;
    let mut buf = String::new();
    BufReader::new(f).read_line(&mut buf)?;
    Ok(buf.trim().to_owned())
}

fn cmd_output(cmd: &str, args: &[&str]) -> String {
    Command::new(cmd).args(args).output()
        .ok()
        .and_then(|o| if o.status.success() {
            String::from_utf8(o.stdout).ok()
        } else { None })
        .map(|s| s.trim().to_owned())
        .unwrap_or_default()
}

fn cmd_output_sh(cmdline: &str) -> String {
    Command::new("sh").args(["-c", cmdline]).output()
        .ok()
        .and_then(|o| if o.status.success() {
            String::from_utf8(o.stdout).ok()
        } else { None })
        .map(|s| s.trim().to_owned())
        .unwrap_or_default()
}



fn os_release_fields() -> (String, String, String, String) {
    let content = read_file("/etc/os-release")
        .or_else(|_| read_file("/usr/lib/os-release"))
        .unwrap_or_default();
    let mut id = String::new();
    let mut id_like = String::new();
    let mut name = String::new();
    let mut version = String::new();
    for line in content.lines() {
        let line = line.trim();
        if let Some(val) = line.strip_prefix("ID=") {
            id = val.trim_matches('"').to_owned();
        } else if let Some(val) = line.strip_prefix("ID_LIKE=") {
            id_like = val.trim_matches('"').to_owned();
        } else if let Some(val) = line.strip_prefix("NAME=") {
            name = val.trim_matches('"').to_owned();
        } else if let Some(val) = line.strip_prefix("VERSION=") {
            version = val.trim_matches('"').to_owned();
        }
    }
    (id, id_like, name, version)
}

fn detect_proxmox() -> Option<String> {
    let out = cmd_output("pveversion", &[]);
    if out.is_empty() { return None; }
    let ver = out.split('/').nth(1)?
        .split('-').next()?;
    Some(format!("Proxmox VE {}", ver))
}

fn detect_android() -> bool {

    if let Ok(prefix) = env::var("PREFIX") {
        if prefix.contains("com.termux") { return true; }
    }

    Path::new("/system/build.prop").exists()
}

fn detect_container() -> Option<&'static str> {

    if Path::new("/.dockerenv").exists() { return Some("Docker"); }
    let cgroup = read_file("/proc/1/cgroup").unwrap_or_default();
    if cgroup.contains("docker")  { return Some("Docker"); }
    if cgroup.contains("podman")  { return Some("Podman"); }
    if cgroup.contains("lxc")     { return Some("LXC"); }
    None
}

fn detect_virt(cpuinfo: &str) -> String {
    let hv = read_first_line("/sys/hypervisor/type").unwrap_or_default();
    if !hv.is_empty() { return hv; }
    if cpuinfo.lines().any(|l| l.contains("hypervisor")) {

        let dmi = read_first_line("/sys/devices/virtual/dmi/id/product_name").unwrap_or_default();
        match dmi.as_str() {
            "KVM" => return "KVM".into(),
            "VirtualBox" => return "VirtualBox".into(),
            "VMware" | "VMware Virtual Platform" => return "VMware".into(),
            _ => return "VM".into(),
        }
    }
    String::new()
}



fn get_uname_release() -> String {
    read_first_line("/proc/sys/kernel/osrelease")
        .unwrap_or_default()
}

fn get_device_name() -> String {
    read_first_line("/sys/devices/virtual/dmi/id/product_name")
        .or_else(|_| read_first_line("/sys/firmware/devicetree/base/model"))
        .unwrap_or_else(|_| "Unknown".into())
}

fn get_uptime() -> String {
    let content = read_first_line("/proc/uptime").unwrap_or_default();
    let secs: f64 = content.split_whitespace().next()
        .and_then(|s| s.parse().ok())
        .unwrap_or(0.0);
    let days   = (secs / 86400.0) as u64;
    let hours  = ((secs % 86400.0) / 3600.0) as u64;
    let mins   = ((secs % 3600.0) / 60.0) as u64;
    let mut out = String::new();
    if days > 0 { write!(out, "{}d ", days).ok(); }
    if hours > 0 || days > 0 { write!(out, "{}h ", hours).ok(); }
    write!(out, "{}m", mins).ok();
    if out.is_empty() { "unknown".into() } else { out }
}

fn get_cpu(cpuinfo: &str) -> String {
    let mut model = String::new();
    for line in cpuinfo.lines() {
        if let Some(val) = line.strip_prefix("model name\t: ") {
            if model.is_empty() { model = val.trim().to_owned(); }
        }
    }
    if model.is_empty() { model = "Unknown".into(); }
    model
}

fn read_pci_ids() -> Option<String> {
    for path in &["/usr/share/hwdata/pci.ids", "/usr/share/misc/pci.ids"] {
        if let Ok(c) = fs::read_to_string(path) { return Some(c); }
    }
    None
}

fn pci_lookup_in<'a>(vendor: &str, device: &str, content: &'a str) -> Option<&'a str> {
    let mut in_vendor = false;
    for line in content.lines() {
        if line.is_empty() || line.starts_with('#') { continue; }
        if !line.starts_with('\t') {
            let parts: Vec<&str> = line.splitn(2, ' ').collect();
            if parts.len() == 2 && parts[0].eq_ignore_ascii_case(vendor) {
                in_vendor = true;
                continue;
            }
            in_vendor = false;
        } else if in_vendor {
            let rest = line[1..].trim_start();
            if let Some(dev_id) = rest.splitn(2, ' ').next() {
                if dev_id.eq_ignore_ascii_case(device) {
                    return rest.splitn(2, ' ').nth(1)
                        .map(|n| n.trim().trim_matches('"'));
                }
            }
        }
    }
    None
}

fn clean_gpu_name(name: &str) -> String {
    let mut s = name.to_owned();
    for pat in &["Corporation ", "Technologies "] { s = s.replace(pat, ""); }
    if let Some(open) = s.find('[') {
        if let Some(close) = s[open+1..].find(']').map(|p| p + open + 1) {
            s = s[open+1..close].to_owned();
        }
    }
    s.trim().to_owned()
}

const GPU_DRIVERS: &[&str] = &[
    "i915", "nvidia", "nvidia_drm", "nouveau",
    "amdgpu", "radeon", "mgag200", "ast",
    "vmwgfx", "bochs", "hyperv_fb",
];

fn get_gpu_from_sysfs() -> Vec<(String, String)> {
    let mut gpus: Vec<(String, String)> = Vec::new();
    let ids = read_pci_ids();


    for drv in GPU_DRIVERS {
        if gpus.len() >= 2 { break; }
        let drv_path = format!("/sys/bus/pci/drivers/{}", drv);
        let Ok(dir) = fs::read_dir(&drv_path) else { continue; };
        for entry in dir.flatten() {
            let path = entry.path();
            if !path.is_symlink() { continue; }
            let class = read_first_line(&path.join("class").to_string_lossy()).unwrap_or_default();
            if class != "0x030000" && class != "0x038000" { continue; }
            let vid = read_first_line(&path.join("vendor").to_string_lossy())
                .unwrap_or_default();
            let did = read_first_line(&path.join("device").to_string_lossy())
                .unwrap_or_default();
            let vid_clean = vid.strip_prefix("0x").unwrap_or("");
            let did_clean = did.strip_prefix("0x").unwrap_or("");
            let name = ids.as_ref()
                .and_then(|c| pci_lookup_in(vid_clean, did_clean, c))
                .map(|n| clean_gpu_name(n))
                .unwrap_or_else(|| "Unknown".into());
            let driver = drv.to_string();
            gpus.push((name, driver));
            if gpus.len() >= 2 { break; }
        }
    }


    if gpus.is_empty() {
        if let Ok(entries) = fs::read_dir("/sys/bus/pci/devices") {
            for entry in entries.flatten() {
                if gpus.len() >= 2 { break; }
                let p = entry.path();
                let class = read_first_line(&p.join("class").to_string_lossy()).unwrap_or_default();
                if class != "0x030000" && class != "0x038000" { continue; }
                let vid = read_first_line(&p.join("vendor").to_string_lossy())
                    .unwrap_or_default();
                let did = read_first_line(&p.join("device").to_string_lossy())
                    .unwrap_or_default();
                let vid_clean = vid.strip_prefix("0x").unwrap_or("");
                let did_clean = did.strip_prefix("0x").unwrap_or("");
                let name = ids.as_ref()
                    .and_then(|c| pci_lookup_in(vid_clean, did_clean, c))
                    .map(|n| clean_gpu_name(n))
                    .unwrap_or_else(|| "Unknown".into());
                let driver = p.join("driver").read_link().ok()
                    .and_then(|l| l.file_name().map(|f| f.to_string_lossy().into_owned()))
                    .unwrap_or_default();
                gpus.push((name, driver));
            }
        }
    }

    gpus
}

fn get_gpu() -> (String, String) {
    let mut gpus = get_gpu_from_sysfs();


    if gpus.is_empty() {
        let raw = cmd_output_sh("lspci 2>/dev/null | grep -iE 'vga|3d|display'");
        let clean = |s: &str| -> String {
            let mut s = s.to_owned();
            if let Some(pos) = s.find(" (rev ") { s.truncate(pos); }
            for pat in &["Corporation ", "Technologies ", "Inc.", "Inc ", "Co.", "Co "] {
                s = s.replace(pat, "");
            }
            if let Some(open) = s.find('[') {
                if let Some(close) = s[open+1..].find(']').map(|p| p + open + 1) {
                    s = s[open+1..close].to_owned();
                }
            }
            s.trim().to_owned()
        };
        for line in raw.lines() {
            if let Some(name) = line.split(": ").nth(1).map(clean) {
                if !name.is_empty() { gpus.push((name, String::new())); }
            }
        }
    }


    let prime = env::var("DRI_PRIME").ok();
    let swapped = prime.as_deref().map(|p| p != "0").unwrap_or(false);

    let gpu_a = gpus.first().map(|g| g.0.clone()).unwrap_or_else(|| "Unknown".into());
    let gpu_b = gpus.get(1).map(|g| g.0.clone()).unwrap_or_default();

    if swapped && !gpu_b.is_empty() { (gpu_b, gpu_a) } else { (gpu_a, gpu_b) }
}

fn get_ram() -> String {
    let content = read_file("/proc/meminfo").unwrap_or_default();
    let parse_kb = |prefix: &str| -> Option<u64> {
        content.lines()
            .find(|l| l.starts_with(prefix))?
            .split_whitespace()
            .nth(1)?
            .parse().ok()
    };
    let total_kb = parse_kb("MemTotal:").unwrap_or(0);
    let avail_kb = parse_kb("MemAvailable:").unwrap_or(0);
    let used_kb = total_kb.saturating_sub(avail_kb);

    fn human(kb: u64) -> String {
        let bytes = kb as f64 * 1024.0;
        if bytes >= 1_000_000_000.0 {
            format!("{:.1} GiB", bytes / 1_073_741_824.0)
        } else if bytes >= 1_000_000.0 {
            format!("{:.0} MiB", bytes / 1_048_576.0)
        } else {
            format!("{:.0} KiB", kb)
        }
    }
    format!("{} / {}", human(used_kb), human(total_kb))
}

fn get_shell() -> String {
    env::var("SHELL").ok()
        .and_then(|s| {
            Path::new(&s).file_name()
                .map(|f| f.to_string_lossy().into_owned())
        })
        .unwrap_or_else(|| "unknown".into())
}

fn get_terminal() -> String {
    env::var("TERM").unwrap_or_else(|_| "unknown".into())
}

fn get_wm_de() -> (String, String) {
    let mut wm = String::new();
    let mut de = String::new();


    if env::var("HYPRLAND_INSTANCE_SIGNATURE").is_ok() {
        wm = "Hyprland".into();
    } else if env::var("SWAYSOCK").is_ok() {
        wm = "Sway".into();
    } else if env::var("I3SOCK").is_ok() || env::var("i3SOCK").is_ok() {
        wm = "i3".into();
    } else if env::var("NIRI_SOCKET").is_ok() {
        wm = "Niri".into();
    } else if env::var("RIVER_SOCKET").is_ok() {
        wm = "River".into();
    } else if env::var("QTILE_SOCKET").is_ok() {
        wm = "Qtile".into();
    } else if env::var("WAYFIRE_SOCKET").is_ok() {
        wm = "Wayfire".into();
    } else if env::var("LABWC_PID_FILE").is_ok() || Path::new("/tmp/labwc-").exists() {
        wm = "LabWC".into();
    }


    if let Ok(de_var) = env::var("XDG_CURRENT_DESKTOP") {
        if !de_var.is_empty() { de = de_var; }
    } else if let Ok(ds) = env::var("DESKTOP_SESSION") {
        if !ds.is_empty() { de = ds; }
    }

    if de.is_empty() {
        if env::var("GNOME_DESKTOP_SESSION_ID").is_ok() {
            de = "GNOME".into();
        } else if env::var("KDE_FULL_SESSION").is_ok() {
            de = "KDE".into();
        } else if env::var("MATE_DESKTOP_SESSION_ID").is_ok() {
            de = "MATE".into();
        } else if env::var("CINNAMON_VERSION").is_ok() {
            de = "Cinnamon".into();
        } else if env::var("XFCE_SESSION").is_ok() || env::var("XFCONF").is_ok() {
            de = "XFCE".into();
        } else if env::var("LXQT_SESSION_DIR").is_ok() {
            de = "LXQt".into();
        } else if Path::new("/usr/bin/lxsession").exists() || env::var("LXDE").is_ok() {
            de = "LXDE".into();
        } else if Path::new("/usr/bin/budgie-wm").exists() {
            de = "Budgie".into();
        } else if Path::new("/usr/bin/deepin-wm").exists() {
            de = "Deepin".into();
        } else if Path::new("/usr/bin/muffin").exists() {
            de = "Cinnamon".into();
        }
    }


    if wm.is_empty() && env::var("DISPLAY").is_ok() {
        let out = cmd_output_sh(
            "xprop -id $(xprop -root _NET_SUPPORTING_WM_CHECK 2>/dev/null | awk '{print $NF}') _NET_WM_NAME 2>/dev/null | grep -oP '(?<=\")[^\"]+(?=\")' | head -1"
        );
        if !out.is_empty() {
            wm = out;
        } else {

            let out = cmd_output_sh("wmctrl -m 2>/dev/null | grep 'Name:' | awk '{$1=\"\"; print $0}' | xargs");
            if !out.is_empty() { wm = out; }
        }
    }


    if wm.is_empty() {
        for wm_bin in &["Xorg", "X", "weston", "dwl", "awesome", "bspwm",
                        "herbstluftwm", "openbox", "fluxbox", "jwm", "pekwm",
                        "fvwm", "2bwm", "dwm", "cwm", "qtile", "wmii"] {
            if !Path::new(&format!("/usr/bin/{}", wm_bin)).exists() &&
               !Path::new(&format!("/usr/local/bin/{}", wm_bin)).exists() { continue; }
            if cmd_output("pidof", &[wm_bin]).trim().is_empty() { continue; }
            wm = wm_bin.to_string();
            break;
        }
    }

    if de.is_empty() && wm.is_empty() {

        if let Ok(home) = env::var("HOME") {
            if Path::new(&format!("{}/.xinitrc", home)).exists() {

            }
        }
    }

    (de, wm)
}

fn get_display_resolutions() -> (String, String) {
    let mut disp1 = String::new();
    let mut disp2 = String::new();


    if env::var("HYPRLAND_INSTANCE_SIGNATURE").is_ok() {
        let out = cmd_output("hyprctl", &["monitors"]);
        let mut nd = 0;
        for line in out.lines() {
            if let Some(at) = line.trim().strip_prefix("Monitor ") {
                if let Some(res) = at.split_whitespace().next() {

                    if let Some(paren) = res.find('(') {
                        let inner = &res[paren+1..];
                        if let Some(res_str) = inner.split('@').next() {
                            if nd == 0 { disp1 = res_str.to_owned(); }
                            else       { disp2 = res_str.to_owned(); }
                            nd += 1;
                        }
                    }
                    if nd >= 2 { break; }
                }
            }
        }
    }

    else if env::var("DISPLAY").is_ok() {
        let out = cmd_output("xrandr", &["--nograb"]);
        if !out.is_empty() {
            let mut res: Vec<(&str, bool)> = Vec::new();
            for line in out.lines() {
                if !line.contains(" connected") { continue; }
                let primary = line.contains(" primary");
                for part in line.split_whitespace() {
                    if let Some(res_str) = part.split(|c| c == '+' || c == '@').next() {
                        if res_str.contains('x') && res_str.chars().all(|c| c.is_digit(10) || c == 'x') {
                            res.push((res_str, primary));
                            break;
                        }
                    }
                }
            }
            if !res.is_empty() {
                let pri_idx = res.iter().position(|r| r.1).unwrap_or(0);
                disp1 = res[pri_idx].0.to_owned();
                if res.len() > 1 {
                    for (i, r) in res.iter().enumerate() {
                        if i != pri_idx { disp2 = r.0.to_owned(); break; }
                    }
                }
            }
        }
    }

    if disp1.is_empty() {
        if let Ok(entries) = fs::read_dir("/sys/class/drm") {
            let mut nd = 0;
            for entry in entries.flatten() {
                let status_path = entry.path().join("status");
                let modes_path = entry.path().join("modes");
                if !status_path.exists() || !modes_path.exists() { continue; }
                let status = read_first_line(&status_path.to_string_lossy()).unwrap_or_default();
                if status != "connected" { continue; }
                let mode = read_first_line(&modes_path.to_string_lossy()).unwrap_or_default();
                if mode.is_empty() { continue; }
                if nd == 0 { disp1 = mode; }
                else       { disp2 = mode; }
                nd += 1;
                if nd >= 2 { break; }
            }
        }
    }

    if disp1.is_empty() { disp1 = "unknown".into(); }
    (disp1, disp2)
}

fn count_packages_dir(path: &str) -> usize {
    fs::read_dir(path).ok()
        .map(|e| e.flatten().filter(|e| e.file_type().map(|t| t.is_dir()).unwrap_or(false)).count())
        .unwrap_or(0)
}

fn count_packages_file(path: &str, suffix: &str) -> usize {
    let dir = match fs::read_dir(path) { Ok(d) => d, Err(_) => return 0 };
    dir.flatten().filter(|e| {
        e.file_name().to_string_lossy().ends_with(suffix)
    }).count()
}

fn get_packages(arch_based: bool, deb_based: bool) -> String {
    let mut counts: Vec<String> = Vec::new();

    if arch_based {
        let n = count_packages_dir("/var/lib/pacman/local");
        if n > 0 { counts.push(format!("{} (pacman)", n)); }
    } else if deb_based {
        let n = count_packages_file("/var/lib/dpkg/info", ".list");
        if n > 0 { counts.push(format!("{} (dpkg)", n)); }
        else {
            let out = cmd_output_sh("dpkg-query -f '${db:Status-Status}' -W 2>/dev/null | grep -c 'installed'");
            let n: usize = out.trim().parse().unwrap_or(0);
            if n > 0 { counts.push(format!("{} (dpkg)", n)); }
        }
    }
    if counts.is_empty() && Path::new("/var/lib/rpm").exists() {
        let out = cmd_output_sh("rpm -qa 2>/dev/null | wc -l");
        let n: usize = out.trim().parse().unwrap_or(0);
        if n > 0 { counts.push(format!("{} (rpm)", n)); }
    }

    if Path::new("/usr/bin/flatpak").exists() {
        let out = cmd_output("flatpak", &["list"]);
        let n = out.lines().filter(|l| !l.is_empty()).count();
        if n > 0 { counts.push(format!("{} (flatpak)", n)); }
    }
    if Path::new("/usr/bin/snap").exists() {
        let out = cmd_output("snap", &["list"]);
        let n = out.lines().filter(|l| !l.is_empty()).count();
        let actual = n.saturating_sub(1);
        if actual > 0 { counts.push(format!("{} (snap)", actual)); }
    }

    counts.join(", ")
}

extern "C" {
    fn statvfs(path: *const i8, buf: *mut libc_statvfs) -> i32;
}

#[repr(C)]
struct libc_statvfs {
    f_bsize: u64,
    f_frsize: u64,
    f_blocks: u64,
    f_bfree: u64,
    f_bavail: u64,
    f_files: u64,
    f_ffree: u64,
    f_favail: u64,
    f_fsid: u64,
    f_flag: u64,
    f_namemax: u64,
    __f_spare: [i32; 6],
}

fn get_disk_usage() -> String {

    let mut buf = std::mem::MaybeUninit::<libc_statvfs>::zeroed();
    let path = "/\0";
    let ret = unsafe { statvfs(path.as_ptr() as *const i8, buf.as_mut_ptr()) };
    if ret != 0 { return String::new(); }
    let s = unsafe { buf.assume_init() };
    let total = s.f_blocks.saturating_mul(s.f_frsize);
    let avail = s.f_bavail.saturating_mul(s.f_frsize);
    let used = total.saturating_sub(avail);
    if total == 0 { return String::new(); }
    let pct = used as f64 / total as f64 * 100.0;

    fn fmt_size(bytes: u64) -> String {
        const UNITS: &[&str] = &["B", "KiB", "MiB", "GiB", "TiB", "PiB"];
        let mut v = bytes as f64;
        let mut i = 0usize;
        while v >= 1024.0 && i < UNITS.len() - 1 {
            v /= 1024.0;
            i += 1;
        }
        format!("{:.0}{}", v, UNITS[i])
    }

    format!("{} / {} ({:.0}%)", fmt_size(used), fmt_size(total), pct)
}



fn choose_label_color(distro_id: &str, distro_base: &str, colors: &Colors, is_proxmox: bool, is_android: bool) -> &'static str {
    if is_proxmox { return colors.orange; }
    if is_android { return colors.green; }

    let id = distro_id;
    if matches!(id, "arch" | "endeavouros" | "manjaro" | "garuda") { return colors.blue; }
    if id == "artix"               { return colors.blue; }
    if id == "fedora"              { return colors.blue; }
    if id == "ubuntu"              { return colors.orange; }
    if id == "linuxmint"           { return colors.green; }
    if id == "pop"                 { return colors.teal; }
    if id == "cachyos"             { return colors.teal; }
    if id == "void"                { return colors.green; }
    if id == "nixos"               { return colors.blue; }
    if id == "debian"              { return colors.s_red; }
    if id == "gentoo"              { return colors.steelblue; }
    if id.contains("opensuse")     { return colors.green; }
    if id == "alpine"              { return colors.s_blue; }
    if id == "slackware"           { return colors.s_black; }
    if id == "solus"               { return colors.s_brightblue; }
    if id == "rhel" || id == "centos" { return colors.red; }

    if distro_base.contains("arch")  { return colors.blue; }
    if distro_base.contains("suse") || distro_base.contains("opensuse") { return colors.green; }
    if distro_base.contains("debian") || distro_base.contains("ubuntu") { return colors.s_red; }

    colors.purple
}



type AsciiFn = fn(&Colors, &mut Vec<String>);

fn ascii_proxmox(c: &Colors, out: &mut Vec<String>) {
    let (o, w, r) = (c.orange, c.white, c.reset_c);
    out.push(format!("             {o}@@@@{r}   {o}@@@@{r}", o=o, r=r));
    out.push(format!("          {w}++++{r} {o}@@@@@@@{r} {w}++++{r}", w=w, o=o, r=r));
    out.push(format!("            {w}++++{r} {o}@@@{r} {w}++++{r}", w=w, o=o, r=r));
    out.push(format!("              {w}++++ ++++{r}", w=w, r=r));
    out.push(format!("             {w}++++{r}{o}@@@{r}{w}++++{r}", w=w, o=o, r=r));
    out.push(format!("           {w}++++{r} {o}@@@@@{r} {w}++++{r}", w=w, o=o, r=r));
    out.push(format!("          {w}+++{r} {o}@@@@{r} {o}@@@@{r} {w}+++{r}", w=w, o=o, r=r));
    out.push(format!("             {o}@@@{r}     {o}@@@{r}", o=o, r=r));
    out.push(String::new());
}

fn ascii_android(c: &Colors, out: &mut Vec<String>) {
    let (g, r) = (c.s_green, c.reset_c);
    out.push(format!("         {g}-o          o-{r}", g=g, r=r));
    out.push(format!("          {g}+hydNNNNdyh+{r}", g=g, r=r));
    out.push(format!("        {g}+mMMMMMMMMMMMMm+{r}", g=g, r=r));
    out.push(format!("       {g}dMMm:NMMMMMMN:mMMd{r}", g=g, r=r));
    out.push(format!("      {g}hMMMMMMMMMMMMMMMMMMh{r}", g=g, r=r));
    out.push(format!("  {g}..  yyyyyyyyyyyyyyyyyyyy  ..{r}", g=g, r=r));
    out.push(format!("{g}.mMMm MMMMMMMMMMMMMMMMMMMM mMMm.{r}", g=g, r=r));
    out.push(format!("{g}:MMMM-MMMMMMMMMMMMMMMMMMMM-MMMM:{r}", g=g, r=r));
    out.push(format!("{g}:MMMM-MMMMMMMMMMMMMMMMMMMM-MMMM:{r}", g=g, r=r));
    out.push(format!("{g}:MMMM-MMMMMMMMMMMMMMMMMMMM-MMMM:{r}", g=g, r=r));
    out.push(format!("{g}:MMMM-MMMMMMMMMMMMMMMMMMMM-MMMM:{r}", g=g, r=r));
    out.push(format!("{g}-MMMM-MMMMMMMMMMMMMMMMMMMM-MMMM-{r}", g=g, r=r));
    out.push(format!(" {g}+yy+ MMMMMMMMMMMMMMMMMMMM +yy+{r}", g=g, r=r));
    out.push(format!("      {g}mMMMMMMMMMMMMMMMMMMm{r}", g=g, r=r));
    out.push(format!("       {g}/++MMMMh++hMMMM++/{r}", g=g, r=r));
    out.push(format!("          {g}MMMMo  oMMMM{r}", g=g, r=r));
    out.push(format!("          {g}MMMMo  oMMMM{r}", g=g, r=r));
    out.push(format!("          {g}oNMm-  -mMNs{r}", g=g, r=r));
}

fn ascii_arch(_c: &Colors, out: &mut Vec<String>) {
    let b = "\x1b[34m"; let r = "\x1b[0m";
    out.push(format!("                   {b}-'{r}", b=b, r=r));
    out.push(format!("                  {b}.o+'{r}", b=b, r=r));
    out.push(format!("                 {b}'ooo/{r}", b=b, r=r));
    out.push(format!("                {b}'+oooo:{r}", b=b, r=r));
    out.push(format!("               {b}'+oooooo:{r}", b=b, r=r));
    out.push(format!("               {b}-+oooooo+:{r}", b=b, r=r));
    out.push(format!("             {b}'/:-:++oooo+:{r}", b=b, r=r));
    out.push(format!("            {b}'/++++/+++++++:{r}", b=b, r=r));
    out.push(format!("           {b}'/++++++++++++++:{r}", b=b, r=r));
    out.push(format!("          {b}'/+++ooooooooooooo/'{r}", b=b, r=r));
    out.push(format!("         {b}./ooosssso++osssssso+'{r}", b=b, r=r));
    out.push(format!("        {b}.oossssso-''''/ossssss+'{r}", b=b, r=r));
    out.push(format!("       {b}-osssssso.      :ssssssso.{r}", b=b, r=r));
    out.push(format!("      {b}:osssssss/        osssso+++.{r}", b=b, r=r));
    out.push(format!("     {b}/ossssssss/        +ssssooo/-{r}", b=b, r=r));
    out.push(format!("   {b}'/ossssso+/:-        -:/+osssso+-{r}", b=b, r=r));
    out.push(format!("  {b}'+sso+:-'                 '.-/+oso:{r}", b=b, r=r));
    out.push(format!(" {b}'++:.                           '-/+/{r}", b=b, r=r));
    out.push(format!(" {b}.'                                 '{r}", b=b, r=r));
}

fn ascii_artix(_c: &Colors, out: &mut Vec<String>) {
    let b = "\x1b[34m"; let r = "\x1b[0m";
    out.push(String::new());
    out.push(format!("                  {b}+{r}", b=b, r=r));
    out.push(format!("                 {b}=++{r}", b=b, r=r));
    out.push(format!("                {b}=+=+={r}", b=b, r=r));
    out.push(format!("               {b}-+===+-{r}", b=b, r=r));
    out.push(format!("              {b}-++====+-{r}", b=b, r=r));
    out.push(format!("             {b}:+++-====+:{r}", b=b, r=r));
    out.push(format!("             {b}=+***+===++.{r}", b=b, r=r));
    out.push(format!("                {b}:****==++.{r}", b=b, r=r));
    out.push(format!("          {b}.+.      .***=++.{r}", b=b, r=r));
    out.push(format!("          {b}+++=++-      +*++{r}", b=b, r=r));
    out.push(format!("        {b}.=++=====++++     -+.{r}", b=b, r=r));
    out.push(format!("        {b}++++=======++++++.{r}", b=b, r=r));
    out.push(format!("       {b}=+++=======-+++++++=.{r}", b=b, r=r));
    out.push(format!("      {b}=+++======+****+:     :+={r}", b=b, r=r));
    out.push(format!("     {b}-+++====+***=.      -++==+-{r}", b=b, r=r));
    out.push(format!("    {b}-+++==***:        =+++++===+-{r}", b=b, r=r));
    out.push(format!("   {b}:+++*+.                :+**+=+.{r}", b=b, r=r));
    out.push(format!("  {b}:+-                          .=+:{r}", b=b, r=r));
    out.push(String::new());
}

fn ascii_ubuntu(_c: &Colors, out: &mut Vec<String>) {
    let o = "\x1b[38;5;208m"; let r = "\x1b[0m";
    out.push(String::new());
    out.push(format!("              {o}==========={r}", o=o, r=r));
    out.push(format!("          {o}==================={r}", o=o, r=r));
    out.push(format!("        {o}======================={r}", o=o, r=r));
    out.push(format!("      {o}=================     ====={r}", o=o, r=r));
    out.push(format!("     {o}==========       =     ======{r}", o=o, r=r));
    out.push(format!("    {o}========= -=        ==========={r}", o=o, r=r));
    out.push(format!("   {o}========    .=======.    ========{r}", o=o, r=r));
    out.push(format!("  {o}========    ===========    ========{r}", o=o, r=r));
    out.push(format!("  {o}====   =   =============   :======={r}", o=o, r=r));
    out.push(format!("  {o}===     =  ========================{r}", o=o, r=r));
    out.push(format!("  {o}====   =   =============   :======={r}", o=o, r=r));
    out.push(format!("  {o}========    ===========    ========{r}", o=o, r=r));
    out.push(format!("   {o}========    .=======:    ========{r}", o=o, r=r));
    out.push(format!("    {o}========= -=        ==========={r}", o=o, r=r));
    out.push(format!("     {o}==========       =     ======{r}", o=o, r=r));
    out.push(format!("      {o}=================     ====={r}", o=o, r=r));
    out.push(format!("        {o}======================={r}", o=o, r=r));
    out.push(format!("          {o}==================={r}", o=o, r=r));
    out.push(format!("              {o}==========={r}", o=o, r=r));
    out.push(String::new());
    out.push(String::new());
}

fn ascii_mint(_c: &Colors, out: &mut Vec<String>) {
    let g = "\x1b[32m"; let r = "\x1b[0m";
    out.push(String::new());
    out.push(format!("            {g}==============={r}", g=g, r=r));
    out.push(format!("         {g}====================={r}", g=g, r=r));
    out.push(format!("       {g}========================={r}", g=g, r=r));
    out.push(format!("      {g}==={r}...{g}====================={r}", g=g, r=r));
    out.push(format!("     {g}===={r}...{g}====={r}.....{g}={r}.....{g}======{r}", g=g, r=r));
    out.push(format!("    {g}====={r}...{g}==={r}...............{g}====={r}", g=g, r=r));
    out.push(format!("   {g}======{r}...{g}==={r}...{g}==={r}...{g}==={r}...{g}======{r}", g=g, r=r));
    out.push(format!("   {g}======{r}...{g}==={r}...{g}==={r}...{g}==={r}...{g}======{r}", g=g, r=r));
    out.push(format!("   {g}======{r}...{g}==={r}...{g}==={r}...{g}==={r}...{g}======{r}", g=g, r=r));
    out.push(format!("   {g}======{r}...{g}==={r}...{g}==={r}...{g}==={r}...{g}======{r}", g=g, r=r));
    out.push(format!("   {g}======{r}...{g}==={r}...{g}==={r}...{g}==={r}...{g}======{r}", g=g, r=r));
    out.push(format!("    {g}====={r}...{g}==============={r}...{g}====={r}", g=g, r=r));
    out.push(format!("     {g}====={r}...................{g}====={r}", g=g, r=r));
    out.push(format!("      {g}======{r}...............{g}======{r}", g=g, r=r));
    out.push(format!("       {g}========================={r}", g=g, r=r));
    out.push(format!("         {g}====================={r}", g=g, r=r));
    out.push(format!("            {g}==============={r}", g=g, r=r));
    out.push(String::new());
    out.push(String::new());
}

fn ascii_pop(_c: &Colors, out: &mut Vec<String>) {
    let b = "\x1b[38;5;153m"; let r = "\x1b[0m";
    out.push(String::new());
    out.push(format!("            {b}.++++++++++++*..{r}", b=b, r=r));
    out.push(format!("         {b}.++++++++++++++*****.{r}", b=b, r=r));
    out.push(format!("       {b}.+++{r}.........{b}++*********.{r}", b=b, r=r));
    out.push(format!("      {b}+++{r}.....{b}+-{r}.....{b}+***********{r}", b=b, r=r));
    out.push(format!("     {b}++++{r}.....{b}+++{r}.....{b}************{r}", b=b, r=r));
    out.push(format!("    {b}++++++{r}.....{b}+**{r}....{b}***{r}....{b}******{r}", b=b, r=r));
    out.push(format!("   {b}.+++++++{r}.....{b}*={r}...{b}+**{r}.....{b}******.{r}", b=b, r=r));
    out.push(format!("   {b}+++++++++{r}........{b}****{r}....{b}********{r}", b=b, r=r));
    out.push(format!("   {b}++++++++**{r}.....{b}******{r}...{b}*********{r}", b=b, r=r));
    out.push(format!("   {b}++++++*****{r}....{b}******{r}..{b}**********{r}", b=b, r=r));
    out.push(format!("   {b}.+++********{r}....{b}****************.{r}", b=b, r=r));
    out.push(format!("    {b}+***********{r}....{b}***{r}..{b}**********{r}", b=b, r=r));
    out.push(format!("     {b}*****************************{r}", b=b, r=r));
    out.push(format!("      {b}*****{r}.................{b}*****{r}", b=b, r=r));
    out.push(format!("       {b}.***{r}.................{b}***.{r}", b=b, r=r));
    out.push(format!("         {b}.*******************.{r}", b=b, r=r));
    out.push(format!("           {b}..*************..{r}", b=b, r=r));
    out.push(String::new());
    out.push(String::new());
    out.push(String::new());
}

fn ascii_cachyos(_c: &Colors, out: &mut Vec<String>) {
    let t = "\x1b[38;5;43m"; let d = "\x1b[36m"; let r = "\x1b[0m";
    out.push(format!("        {t}**{d}++++++++++++++++{t}*{r}", t=t, d=d, r=r));
    out.push(format!("       {d}=++++{t}*{d}+++++++++++++      {d}={r}", d=d, t=t, r=r));
    out.push(format!("      {d}==+++==++{t}*{d}+++++++++      {d}=--{r}", d=d, t=t, r=r));
    out.push(format!("     {d}===++++====++++++++        {d}={r}", d=d, r=r));
    out.push(format!("    {d}====+++++++++++++++{r}", d=d, r=r));
    out.push(format!("   {d}=====++++                {d}+++{r}", d=d, r=r));
    out.push(format!("  {d}===+++{t}*{d}++                {d}====={r}", d=d, t=t, r=r));
    out.push(format!(" {d}+++++++{t}*{d}+                  {d}---{r}", d=d, t=t, r=r));
    out.push(format!("{d}========{t}*{r}", d=d, t=t, r=r));
    out.push(format!(" {d}===++++++                        {d}++++{r}", d=d, r=r));
    out.push(format!("  {d}+{t}*{d}+++++++                      {d}======{r}", d=d, t=t, r=r));
    out.push(format!("   {d}+++{t}***{d}+++=                     {d}----{r}", d=d, t=t, r=r));
    out.push(format!("     {d}+++++===+++================={r}", d=d, r=r));
    out.push(format!("      {d}++++==+++++{t}*{d}+============{r}", d=d, t=t, r=r));
    out.push(format!("       {d}+++=+++++++++++========{r}", d=d, r=r));
    out.push(format!("        {d}+++++++++++++++++===={r}", d=d, r=r));
    out.push(format!("         {d}+++++++++++++++++++{r}", d=d, r=r));
}

fn ascii_void(_c: &Colors, out: &mut Vec<String>) {
    out.push("                -------".into());
    out.push("              -----------".into());
    out.push("               -----------".into());
    out.push("            *   --   ------".into());
    out.push("            **         ----".into());
    out.push("           ****    -    ----".into());
    out.push("         @@****@@*++=-@@#+++@@".into());
    out.push("         @@@**@@@%%+@@=@@@@+#@@".into());
    out.push("          @@%%%%@@@=-@@#@@@%%-#@@".into());
    out.push("          @@#* @@*%%@+@@@@%%#@@@".into());
    out.push("           **** @*=--*@@%%++=".into());
    out.push("           ****    -    ----".into());
    out.push("            ****         --".into());
    out.push("            ******    *   -".into());
    out.push("             ***********".into());
    out.push("              ***********".into());
    out.push("                *******".into());
    out.push(String::new());
}

fn ascii_nixos(_c: &Colors, out: &mut Vec<String>) {
    let b = "\x1b[1;34m"; let w = "\x1b[1;37m"; let r = "\x1b[0m";
    out.push(format!("         {b}+++{r}     {w}-----   ---{r}", b=b, w=w, r=r));
    out.push(format!("         {b}+++*{r}     {w}----- ----{r}", b=b, w=w, r=r));
    out.push(format!("          {b}++**{r}     {w}--------{r}", b=b, w=w, r=r));
    out.push(format!("     {b}+++++++********{r}{w}---==={r}     {b}+{r}", b=b, w=w, r=r));
    out.push(format!("    {b}++++++++*********{r}{w}-===={r}    {b}+++{r}", b=b, w=w, r=r));
    out.push(format!("          {w}====={r}       {w}====={r}  {b}+++++{r}", w=w, b=b, r=r));
    out.push(format!("         {w}===={r}           {w}===={r}{b}++++{r}", w=w, b=b, r=r));
    out.push(format!(" {w}--=-======={r}             {w}=={r}{b}**+++++++{r}", w=w, b=b, r=r));
    out.push(format!("{w}-------===={r}               {b}******+++++{r}", w=w, b=b, r=r));
    out.push(format!(" {w}--------={r}{b}**{r}             {b}**********+{r}", w=w, b=b, r=r));
    out.push(format!("     {w}----{r}{b}****{r}           {b}****{r}", w=w, b=b, r=r));
    out.push(format!("    {w}----{r}  {b}*****{r}       {b}*****{r}", w=w, b=b, r=r));
    out.push(format!("    {w}---{r}    {b}****+{r}{w}========={r}{w}--------{r}", w=w, b=b, r=r));
    out.push(format!("     {w}-{r}     {b}*****+{r}{w}======={r}{w}--------{r}", w=w, b=b, r=r));
    out.push(format!("          {b}**++++++{r}     {w}=---{r}", b=b, w=w, r=r));
    out.push(format!("         {b}++++ +++++{r}     {w}----{r}", b=b, w=w, r=r));
    out.push(format!("         {b}+++   +++++{r}     {w}---{r}", b=b, w=w, r=r));
}

fn ascii_debian(_c: &Colors, out: &mut Vec<String>) {
    let rd = "\x1b[31m"; let r = "\x1b[0m";
    out.push(format!("               {rd}#####  #{r}", rd=rd, r=r));
    out.push(format!("           {rd}##################{r}", rd=rd, r=r));
    out.push(format!("        {rd}########        ########{r}", rd=rd, r=r));
    out.push(format!("       {rd}#####                #####{r}", rd=rd, r=r));
    out.push(format!("     {rd}#####                   ######{r}", rd=rd, r=r));
    out.push(format!("    {rd}####                       ## #{r}", rd=rd, r=r));
    out.push(format!("   {rd}###             #####       ###{r}", rd=rd, r=r));
    out.push(format!("   {rd}###           #              ##{r}", rd=rd, r=r));
    out.push(format!("  {rd}###          ##               ###{r}", rd=rd, r=r));
    out.push(format!("  {rd}##           #                ##{r}", rd=rd, r=r));
    out.push(format!("  {rd}##           #                ##{r}", rd=rd, r=r));
    out.push(format!("  {rd}##           ##             ###{r}", rd=rd, r=r));
    out.push(format!("  {rd}###         # ##           ##{r}", rd=rd, r=r));
    out.push(format!("  {rd}###           # ##      ###{r}", rd=rd, r=r));
    out.push(format!("   {rd}##              #  ###{r}", rd=rd, r=r));
    out.push(format!("    {rd}###{r}", rd=rd, r=r));
    out.push(format!("    {rd}####{r}", rd=rd, r=r));
    out.push(format!("      {rd}##{r}", rd=rd, r=r));
    out.push(format!("       {rd}###{r}", rd=rd, r=r));
    out.push(format!("         {rd}##{r}", rd=rd, r=r));
    out.push(format!("           {rd}##{r}", rd=rd, r=r));
    out.push(format!("              {rd}##{r}", rd=rd, r=r));
    out.push(String::new());
}

fn ascii_gentoo(_c: &Colors, out: &mut Vec<String>) {
    let s = "\x1b[38;5;68m"; let l = "\x1b[38;5;111m"; let d = "\x1b[34m"; let r = "\x1b[0m";
    out.push(format!("            {l}%%%%%%%{r}", l=l, r=r));
    out.push(format!("        {l}%%{r}{s}##########{r}{l}%%%{r}", l=l, s=s, r=r));
    out.push(format!("      {l}%{r}{s}################{r}{l}%%%{r}", l=l, s=s, r=r));
    out.push(format!("    {l}%{r}{s}####################{r}{l}%%%{r}", l=l, s=s, r=r));
    out.push(format!("   {l}%{r}{s}###############{r}{l}%%{r}{s}####{r}{l}%%%%%{r}", l=l, s=s, r=r));
    out.push(format!("  {l}%{r}{s}#############{r}{l}%%%%%{r}{d}@{r}{s}###{r}{l}%%%%%{r}{s}##{r}", l=l, s=s, r=r, d=d));
    out.push(format!("  {l}%%{r}{s}###########{r}{l}%%{r} {l}%%%{r}{d}@{r}{s}###{r}{l}%%%%%%%{r}{s}#{r}{l}%{r}", l=l, s=s, r=r, d=d));
    out.push(format!("  {l}%%%%%{r}{s}###########{r}{l}%%{r}{s}####{r}{l}%%%%%%%%%{r}{s}##{r}", l=l, s=s, r=r));
    out.push(format!("    {l}%%%%{r}{s}################{r}{l}%%%%%%%%%{r}{s}##{r}{l}%{r}", l=l, s=s, r=r));
    out.push(format!("       {l}%%%%{r}{s}#############{r}{l}%%%%%%%%{r}{s}##{r}{l}%%{r}", l=l, s=s, r=r));
    out.push(format!("         {s}##############{r}{l}%%%%%%%{r}{s}###{r}{l}%%{r}", s=s, l=l, r=r));
    out.push(format!("       {s}################{r}{l}%%%%%{r}{s}###{r}{l}%%%{r}", s=s, l=l, r=r));
    out.push(format!("     {s}#################{r}{l}%%%%{r}{s}###{r}{l}%%%{r}", s=s, l=l, r=r));
    out.push(format!("   {s}##################{r}{l}%%%{r}{s}###{r}{l}%%%{r}", s=s, l=l, r=r));
    out.push(format!("  {s}##################{r}{l}%{r}{s}####{r}{l}%%%{r}", s=s, l=l, r=r));
    out.push(format!(" {l}%{r}{s}####################{r}{l}%%%%{r}", l=l, s=s, r=r));
    out.push(format!(" {l}%{r}{s}################{r}{l}%%%%%{r}", l=l, s=s, r=r));
    out.push(format!("  {l}%{r}{s}###########{r}{l}%%%%%%{r}", l=l, s=s, r=r));
    out.push(format!("   {l}%%%%%%%%{r}{d}@@@@@@{r}", l=l, d=d, r=r));
    out.push(format!("      {d}@@@@@@{r}", d=d, r=r));
}

fn ascii_opensuse(_c: &Colors, out: &mut Vec<String>) {
    let g = "\x1b[1;32m"; let sg = "\x1b[32m"; let r = "\x1b[0m";
    out.push(String::new());
    out.push(format!("              {g}#########{r}", g=g, r=r));
    out.push(format!("          {g}#####{r}{sg}*-----*{r}{g}#####{r}", g=g, sg=sg, r=r));
    out.push(format!("       {g}####{r}{sg}---------------{r}{g}####{r}", g=g, sg=sg, r=r));
    out.push(format!("      {g}###{r}{sg}-------------------{r}{g}###{r}", g=g, sg=sg, r=r));
    out.push(format!("    {g}#####{r}{sg}--{r}{g}########{r}{sg}-----------{r}{g}###{r}", g=g, sg=sg, r=r));
    out.push(format!("    {g}####################{r}{sg}+------{r}{g}##{r}", g=g, sg=sg, r=r));
    out.push(format!("   {g}###############{r}{sg}+--{r}{g}#{r}{sg}=-={r}{g}#{r}{sg}*-----{r}{g}##{r}", g=g, sg=sg, r=r));
    out.push(format!("  {g}###############{r}{sg}+-{r}{g}##{r}{sg}--{r}{g}#{r}{sg}-={r}{g}#{r}{sg}=----{r}{g}###{r}", g=g, sg=sg, r=r));
    out.push(format!("  {g}###############{r}{sg}=-{r}{g}#####{r}{sg}--{r}{g}##{r}{sg}-----{r}{g}##{r}", g=g, sg=sg, r=r));
    out.push(format!("  {g}###########{r}{sg}-{r}{g}####{r}{sg}---{r}{g}#{r}{sg}=--{r}{g}####{r}{sg}----{r}{g}##{r}", g=g, sg=sg, r=r));
    out.push(format!("  {g}############{r}{sg}----{r}{g}#########{r}{sg}+----{r}{g}###{r}", g=g, sg=sg, r=r));
    out.push(format!("   {g}###############{r}{sg}=--------{r}{g}#{r}{sg}+---{r}{g}##{r}", g=g, sg=sg, r=r));
    out.push(format!("    {g}#######################{r}{sg}=---{r}{g}##{r}", g=g, sg=sg, r=r));
    out.push(format!("    {g}###################{r}{sg}-------{r}{g}###{r}", g=g, sg=sg, r=r));
    out.push(format!("      {g}###{r}{sg}-------------------{r}{g}###{r}", g=g, sg=sg, r=r));
    out.push(format!("       {g}####{r}{sg}---------------{r}{g}####{r}", g=g, sg=sg, r=r));
    out.push(format!("          {g}#####{r}{sg}*-----*{r}{g}#####{r}", g=g, sg=sg, r=r));
    out.push(format!("              {g}#########{r}", g=g, r=r));
    out.push(String::new());
}

fn ascii_fedora(_c: &Colors, out: &mut Vec<String>) {
    let b = "\x1b[1;34m"; let r = "\x1b[0m";
    out.push(format!("             {b}###########{r}", b=b, r=r));
    out.push(format!("         {b}###################{r}", b=b, r=r));
    out.push(format!("      {b}################....#####{r}", b=b, r=r));
    out.push(format!("    {b}##############..........**###{r}", b=b, r=r));
    out.push(format!("   {b}##############..........****###{r}", b=b, r=r));
    out.push(format!("  {b}##############....########****###{r}", b=b, r=r));
    out.push(format!(" {b}##############....*########*****###{r}", b=b, r=r));
    out.push(format!(" {b}##############....#########*****###{r}", b=b, r=r));
    out.push(format!("{b}###############....#########****#####{r}", b=b, r=r));
    out.push(format!("{b}###############.....#####******######{r}", b=b, r=r));
    out.push(format!("{b}#####*****...............*****#######{r}", b=b, r=r));
    out.push(format!("{b}####******..............***##########{r}", b=b, r=r));
    out.push(format!("{b}###****########....#################{r}", b=b, r=r));
    out.push(format!("{b}##****#########....#################{r}", b=b, r=r));
    out.push(format!("{b}##****#########....################{r}", b=b, r=r));
    out.push(format!("{b}##*****########....###############{r}", b=b, r=r));
    out.push(format!("{b}###******###+.....###############{r}", b=b, r=r));
    out.push(format!("{b}####***..........##############{r}", b=b, r=r));
    out.push(format!("{b}#######*......##############{r}", b=b, r=r));
    out.push(format!("  {b}######################{r}", b=b, r=r));
    out.push(String::new());
}

fn ascii_alpine(_c: &Colors, out: &mut Vec<String>) {
    let b = "\x1b[34m"; let r = "\x1b[0m";
    out.push(format!("        {b}       .hdddddddddddddddddddddddh.{r}", b=b, r=r));
    out.push(format!("       {b}     .dddddddddddddddddddddddddddd.{r}", b=b, r=r));
    out.push(format!("      {b}    .dddddddddddddddddddddddddddddd.{r}", b=b, r=r));
    out.push(format!("     {b}   .dddddddddddddddddddddddddddddddd.{r}", b=b, r=r));
    out.push(format!("    {b}  .dddddddddddddddddddddddddddddddddd.{r}", b=b, r=r));
    out.push(format!("   {b} .dddddddddddddddddddddddddddddddddddd.{r}", b=b, r=r));
    out.push(format!("  {b} .ddddddddddddddddddddddddddddddddddddd.{r}", b=b, r=r));
    out.push(format!(" {b} .dddddddddddddddddddddddddddddddddddddd.{r}", b=b, r=r));
    out.push(format!("{b} .dddddddddddddddddddddddddddddddddddddddd.{r}", b=b, r=r));
    out.push(format!("{b} .dddddddddddddddddddddddddddddddddddddddd.{r}", b=b, r=r));
    out.push(format!("{b} .dddddddddddddddddddddddddddddddddddddddd.{r}", b=b, r=r));
    out.push(format!("{b} .dddddddhs++++++++++++++++++syddddddddddd.{r}", b=b, r=r));
    out.push(format!("{b} .dddddd/                      :dddddddddd.{r}", b=b, r=r));
    out.push(format!("{b} .dddddd/                      :dddddddddd.{r}", b=b, r=r));
    out.push(format!("{b} .dddddd/                      :dddddddddd.{r}", b=b, r=r));
    out.push(format!("{b} .dddddd/                      :dddddddddd.{r}", b=b, r=r));
    out.push(format!("{b} .dddddd/                      :dddddddddd.{r}", b=b, r=r));
    out.push(format!("{b}  'yyyyy+'                     :yyyyyyyyyy.{r}", b=b, r=r));
    out.push(format!("{b}                                    .{r}", b=b, r=r));
}

fn ascii_linux(_c: &Colors, out: &mut Vec<String>) {
    let b = "\x1b[39m"; let r = "\x1b[0m";
    out.push(format!("              {b}a8888b.{r}", b=b, r=r));
    out.push(format!("             {b}d888888b.{r}", b=b, r=r));
    out.push(format!("             {b}8P{r}\"{b}YP{r}\"{b}Y88{r}", b=b, r=r));
    out.push("             8|o||o|88".into());
    out.push("             8'    .88".into());
    out.push("             8'._.' Y8.".into());
    out.push("            d/      '8b.".into());
    out.push("          .dP   .     Y8b.".into());
    out.push("         d8:'   \"   '::88b.".into());
    out.push("        d8\"           'Y88b".into());
    out.push("       :8P     '       :888".into());
    out.push("        8a.    :      _a88P".into());
    out.push("      ._/\"Yaa_ :    .| 88P|".into());
    out.push("      \\    YP\"      '| 8P  '.".into());
    out.push("      /     \\._____.d|    .'".into());
    out.push("      '--..__)888888P'._.'".into());
}



fn choose_ascii(distro_id: &str, distro_base: &str, is_proxmox: bool, is_android: bool) -> AsciiFn {
    if is_proxmox { return ascii_proxmox; }
    if is_android { return ascii_android; }

    match distro_id {
        "arch" | "endeavouros" | "manjaro" | "garuda" => return ascii_arch,
        "artix" => return ascii_artix,
        "fedora" => return ascii_fedora,
        "ubuntu" => return ascii_ubuntu,
        "linuxmint" => return ascii_mint,
        "pop" => return ascii_pop,
        "cachyos" => return ascii_cachyos,
        "void" => return ascii_void,
        "nixos" => return ascii_nixos,
        "debian" => return ascii_debian,
        "gentoo" => return ascii_gentoo,
        "alpine" => return ascii_alpine,
        _ => {}
    }

    if distro_id.contains("opensuse") { return ascii_opensuse; }

    if distro_base.contains("suse") || distro_base.contains("opensuse") { return ascii_opensuse; }
    if distro_base.contains("arch") { return ascii_arch; }
    if distro_base.contains("debian") || distro_base.contains("ubuntu") { return ascii_debian; }

    ascii_linux
}



fn build_info(
    info: &SysInfo, colors: &Colors, label_color: &str
) -> Vec<String> {
    let mut lines: Vec<String> = Vec::new();
    let lc = label_color;
    let rc = colors.reset_c;
    let ic = colors.info_color;

    macro_rules! il {
        ($label:expr, $val:expr) => {
            let val = $val;
            let label = $label;
            if !val.is_empty() {
                lines.push(format!("{lc}{label}:{rc} {ic}{val}{rc}"));
            }
        };
    }

    il!("OS",        &info.os);
    il!("Kernel",    &info.kernel);
    il!("Device",    &info.device);
    il!("Uptime",      &info.uptime);
    if !info.packages.is_empty() {
        il!("Packages", &info.packages);
    }
    il!("Shell",       &info.shell);

    if !info.de.is_empty() {
        il!("DE", &info.de);
    } else if !info.wm.is_empty() {
        il!("WM", &info.wm);
    }

    il!("Terminal",  &info.terminal);
    il!("Resolution", &info.disp1);
    if !info.disp2.is_empty() {
        il!("Resolution-2", &info.disp2);
    }

    il!("CPU",       &info.cpu);
    il!("GPU",       &info.gpu);
    if !info.gpu2.is_empty() {
        il!("GPU-2", &info.gpu2);
    }
    il!("RAM",       &info.ram);

    if !info.disk.is_empty() {
        il!("Disk (/)", &info.disk);
    }
    if !info.virt.is_empty() {
        il!("Virtualization", &info.virt);
    }
    if !info.container.is_empty() {
        il!("Container", &info.container);
    }

    lines
}



struct SysInfo {
    os: String,
    kernel: String,
    device: String,
    uptime: String,
    shell: String,
    terminal: String,
    de: String,
    wm: String,
    disp1: String,
    disp2: String,
    cpu: String,
    gpu: String,
    gpu2: String,
    ram: String,
    packages: String,
    disk: String,
    virt: String,
    container: String,
    distro_id: String,
    distro_base: String,
    is_proxmox: bool,
    is_android: bool,
}



fn vis_len(s: &str) -> usize {
    let mut len = 0;
    let mut esc = false;
    for b in s.bytes() {
        if esc && b == b'm' { esc = false; continue; }
        if b == 0x1b { esc = true; continue; }
        if esc { continue; }
        len += 1;
    }
    len
}



fn main() {
    let args: Vec<String> = env::args().collect();
    let mut grey = false;
    let mut color_hex: Option<&str> = None;

    for arg in &args[1..] {
        if arg == "--grey" { grey = true; }
        if let Some(hex) = arg.strip_prefix("--color:") { color_hex = Some(hex); }
    }

    let colors = if grey {
        Colors::grey()
    } else if let Some(hex) = color_hex {
        Colors::custom(hex).unwrap_or_else(Colors::default)
    } else {
        Colors::default()
    };



    let (distro_id, distro_base, os_name, os_ver) = os_release_fields();

    let proxmox_ver = detect_proxmox();
    let is_proxmox = proxmox_ver.is_some();
    let is_android = detect_android();

    let os = if is_proxmox {
        proxmox_ver.unwrap()
    } else if is_android {
        "Android (Termux)".into()
    } else {
        let name = if os_name.is_empty() { "Linux".into() } else { os_name };
        if os_ver.is_empty() { name } else { format!("{} {}", name, os_ver) }
    };

    let kernel = {
        let mut k = get_uname_release();
        if !k.is_empty() { k = format!("Linux {}", k); }
        k
    };

    let cpuinfo = read_file("/proc/cpuinfo").unwrap_or_default();
    let (de, wm) = get_wm_de();
    let (disp1, disp2) = get_display_resolutions();
    let (gpu1, gpu2) = get_gpu();

    let arch_based = matches!(distro_id.as_str(),
        "arch" | "endeavouros" | "manjaro" | "garuda" | "artix" | "cachyos")
        || distro_base.contains("arch");
    let deb_based = !arch_based && (matches!(distro_id.as_str(),
        "debian" | "ubuntu" | "linuxmint" | "pop")
        || distro_base.contains("debian"));

    let info = SysInfo {
        os,
        kernel,
        device: get_device_name(),
        uptime: get_uptime(),
        shell: get_shell(),
        terminal: get_terminal(),
        de,
        wm,
        disp1,
        disp2,
        cpu: get_cpu(&cpuinfo),
        gpu: gpu1,
        gpu2,
        ram: get_ram(),
        packages: get_packages(arch_based, deb_based),

        disk: get_disk_usage(),

        virt: detect_virt(&cpuinfo),
        container: detect_container().unwrap_or("").to_string(),
        distro_id,
        distro_base,
        is_proxmox,
        is_android,
    };

    let label_color = choose_label_color(
        &info.distro_id, &info.distro_base, &colors, info.is_proxmox, info.is_android
    );

    let final_label_color = if color_hex.is_some() && !colors.label_color.is_empty() {
        colors.label_color
    } else {
        label_color
    };


    let mut ascii_lines: Vec<String> = Vec::new();
    let ascii_fn = choose_ascii(&info.distro_id, &info.distro_base, info.is_proxmox, info.is_android);
    ascii_fn(&colors, &mut ascii_lines);
    let n_ascii = ascii_lines.len();


    let info_lines = build_info(&info, &colors, final_label_color);
    let n_info = info_lines.len();


    let max_vlen = info_lines.iter().map(|l| vis_len(l)).max().unwrap_or(0);
    let box_w = max_vlen;
    let inner = box_w + 2;


    let border: String = std::iter::repeat('\u{2500}').take(inner).collect();
    let ic  = colors.info_color;
    let rst = if ic.is_empty() { "" } else { colors.reset_c };

    let box_h = n_info + 2;
    let rows = if n_ascii > box_h { n_ascii } else { box_h };

    for i in 0..rows {
        let left = ascii_lines.get(i).map(|s| s.as_str()).unwrap_or("");

        let mut right = String::new();
        if i < box_h {
            if i == 0 {
                let _ = write!(right, "{ic}\u{256d}{border}\u{256e}{rst}", ic=ic, rst=rst);
            } else if i == box_h - 1 {
                let _ = write!(right, "{ic}\u{2570}{border}\u{256f}{rst}", ic=ic, rst=rst);
            } else {
                let content = &info_lines[i - 1];
                let cvlen = vis_len(content);
                let spaces = box_w.saturating_sub(cvlen);
                let _ = write!(right, "{ic}\u{2502}{rst} {content}{spc} {ic}\u{2502}{rst}",
                    ic=ic, rst=rst, content=content,
                    spc = " ".repeat(spaces));
            }
        }

        let left_vlen = vis_len(left);
        let pad = LEFT_WIDTH.saturating_sub(left_vlen);
        println!("{}{:pad$}{}", left, "", right, pad=pad);
    }
    println!();
}

const LEFT_WIDTH: usize = 40;
