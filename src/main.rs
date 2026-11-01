slint::include_modules!();
use std::path::Path;
use serde::Deserialize;
use std::fs;
use std::io::{Write};
use std::env;

mod helpers;
use helpers::{sh, lipc_set, lipc_get, lipc_event, boot_enable, rootfs_writeable, rootfs_readonly, read_path, kill_process, reboot};

#[cfg(all(target_os = "linux", target_arch = "arm"))]
static INTER: &[u8] = include_bytes!("./inter.ttf");

#[cfg(all(target_os = "linux", target_arch = "arm"))]
static BASKERVILLE: &[u8] = include_bytes!("./libre-baskerville.ttf");

/*
    Kindle has no fonts, 
    on desktop just install Inter & Libre Baskerville during testing.
*/

fn main() {
    #[cfg(all(target_os = "linux", target_arch = "arm"))]
    let backend = slint_backend_kindle::install(INTER).expect("Failed to Install!");

    let app = AppWindow::new().expect("Failed to Create Window!");
    
    #[cfg(all(target_os = "linux", target_arch = "arm"))]
    backend.register_font_from_memory(BASKERVILLE).expect("Failed to Install Libre Baskerville!");

    let version = env!("CARGO_PKG_VERSION");
    app.set_jobman_version(version.into());
    match load_ssh_config() {
        Ok(config) => {
            app.set_usb_ssh_ip(config.kindle_ip.into());
        }
        Err(e) => {
            app.set_usb_ssh_ip("N/A".into());
            app.set_error(format!("Could not load config: {}", e).into());
            app.set_show_error(true);
        }
    }

    match get_ip_addr() {
        Some(ip) => {
            app.set_wifi_ssh_ip(ip.into());
        }
        None => {
            app.set_wifi_ssh_ip("N/A".into());
        }
    }

    match default_password() {
        Some(pass) => app.set_ssh_password(pass.into()),
        None => app.set_ssh_password("N/A".into()),
    };

    if let Some(reason) = recovery_mode() {
        app.set_boot_mode(true);
        app.set_boot_reason(reason.into());
    }

    app.set_ota_status(ota_status());
    app.set_wifi_status(wifi_status());
    app.set_usb_ssh_status(usb_ssh_enabled());
    app.set_wifi_ssh_status(wifi_ssh_enabled());
    app.set_koreader_status(koreader_installed());
    app.set_recovery_installation_status(recovery_installed());
    match battery_health() {
        Ok(health) => {
            app.set_battery_health(health);
        }

        Err(error_message) => {
            app.set_error(error_message.into());
            app.set_show_error(true);
        }
    }

    let app_weak = app.as_weak(); //No memory leaks
    let toggle_weak = app_weak.clone();

    app.on_toggle_ota(move || {
        let app = toggle_weak.unwrap();
        let status = app.get_ota_status();
        
        let result = if status {
            block_ota() 
        } else {
            enable_ota() 
        };

        if let Err(error_message) = result {
            app.set_error(error_message.into());
            app.set_show_error(true);
        }
    });

    let update_weak = app_weak.clone();
    app.on_update_environment(move || {
        let _app = update_weak.unwrap();
        //For redraw 

        let run_weak = update_weak.clone();
        let _ = slint::invoke_from_event_loop(move || {
            match update_env() {
                Ok(_) => {
                    std::process::exit(0); //JB.sh performs a GUI reboot.
                }
                Err(error_message) => {
                    if let Some(ui) = run_weak.upgrade() {
                        ui.set_error(error_message.into());
                        ui.set_show_error(true);
                    }
                }
            }
        });
    });

    let usb_ssh_weak = app_weak.clone();
    app.on_usb_ssh_toggle(move || {
        let run_weak = usb_ssh_weak.clone();
        let _ = slint::invoke_from_event_loop(move || {
            let Some(app) = run_weak.upgrade() else { return };
            let status = app.get_usb_ssh_status();
        
            let result = if status {
                disable_usb_ssh() 
            } else {
                enable_usb_ssh()
            };

            app.set_usb_ssh_status(usb_ssh_enabled()); //Sync

            if let Err(error_message) = result {
                app.set_error(error_message.into());
                app.set_show_error(true);
            }
        });
    });

    let wifi_ssh_weak = app_weak.clone();
    app.on_wifi_ssh_toggle(move || {
        let run_weak = wifi_ssh_weak.clone();
        let _ = slint::invoke_from_event_loop(move || {
            let Some(app) = run_weak.upgrade() else { return };
            let status = app.get_wifi_ssh_status();
        
            let result = if status {
                disable_wifi_ssh() 
            } else {
                enable_wifi_ssh()
            };

            app.set_wifi_ssh_status(wifi_ssh_enabled()); //Sync

            if let Err(error_message) = result {
                app.set_error(error_message.into());
                app.set_show_error(true);
            }
        });
    });

    let recovery_toggle_weak = app_weak.clone();
    app.on_recovery_toggle(move || {
        let run_weak = recovery_toggle_weak.clone();
        let _ = slint::invoke_from_event_loop(move || {
            let Some(app) = run_weak.upgrade() else { return };
            let status = app.get_recovery_installation_status();

            let result = if status {
                remove_recovery()
            } else {
                install_recovery()
            };

            app.set_recovery_installation_status(recovery_installed());

            if let Err(error_message) = result {
                app.set_error(error_message.into());
                app.set_show_error(true);
            }
        });
    });

    let recovery_generate_logs_weak = app_weak.clone();
    app.on_recovery_generate_logs(move || {
        let run_weak = recovery_generate_logs_weak.clone();
        let _ = slint::invoke_from_event_loop(move || {
            let Some(_app) = run_weak.upgrade() else { return };
            let app = run_weak.unwrap();

            match recovery_run_kmclog() {
                Ok(_) => {
                    app.set_recovery_generating_logs(false);
                }
                Err(error_message) => {
                    app.set_recovery_generating_logs(false);

                    app.set_error(error_message.into());
                    app.set_show_error(true);
                }
            }
        });
    });

    let koreader_flag_weak = app_weak.clone();
    app.on_create_koreader_flag(move || {
        let run_weak = koreader_flag_weak.clone();
        let _ = slint::invoke_from_event_loop(move || {
            let Some(_app) = run_weak.upgrade() else { return };
            let app = run_weak.unwrap();

            match create_koreader_flag() {
                Ok(_) => {
                    std::process::exit(0) //Progress in upstart sequence
                }
                Err(_) => {
                    app.set_error("Failed to create KOReader flag (permissions?)".into());
                    app.set_show_error(true);
                }
            }
        });
    });

    app.on_quit(|| std::process::exit(0));
    #[cfg(all(target_os = "linux", target_arch = "arm"))]
    {
        let app_weak = app.as_weak();
        slint::Timer::single_shot(std::time::Duration::ZERO, move || {
            if let Some(app) = app_weak.upgrade() {
                app.window().request_redraw();
            }
        });
    }

    app.run().expect("Event Loop Error!");
}

//UI backend
fn recovery_mode() -> Option<String> {
    let mut args = env::args().skip(1);
    while let Some(arg) = args.next() {
        if arg == "--recovery" || arg == "-r" {
            return Some(args.next().unwrap_or_else(|| "N/A".to_string()));
        }
    }

    None
}

fn ota_status() -> bool {
    Path::new("/usr/bin/otav3").try_exists().unwrap_or(false)
}

fn chattr_path() -> &'static str {
    if Path::new("/bin/chattr.e2fsprogs").exists() {
        "/bin/chattr.e2fsprogs"
    } else {
        "/bin/chattr"
    }
}

//Nothing here is ever ran in recovery!
fn block_ota() -> Result<(), String> {
    let chattr = chattr_path();

    rootfs_writeable()?;

    sh(
        &format!("{chattr} -i /usr/bin/otaupd /usr/bin/otav3"),
        "Could not make active binaries mutable"
    )?;

    sh(
        "mv /usr/bin/otaupd /usr/bin/otaupd.bck && mv /usr/bin/otav3 /usr/bin/otav3.bck",
        "Failed renaming OTA binaries to backup files"
    )?;

    sh(
        &format!("{chattr} +i /usr/bin/otaupd.bck /usr/bin/otav3.bck"),
        "Could not make backup files immutable via chattr"
    )?;

    rootfs_readonly()?;
    reboot()?;

    Ok(()) 
}

fn enable_ota() -> Result<(), String> {
    let chattr = chattr_path();

    rootfs_writeable()?;

    sh(
        &format!("{chattr} -i /usr/bin/otaupd.bck /usr/bin/otav3.bck"),
        "Could not make backup files mutable"
    )?;

    sh(
        "mv /usr/bin/otaupd.bck /usr/bin/otaupd && mv /usr/bin/otav3.bck /usr/bin/otav3",
        "Failed renaming OTA backup files to active binaries"
    )?; 

    sh(
        &format!("{chattr} +i /usr/bin/otaupd /usr/bin/otav3"),
        "Could not make active binaries immutable via chattr"
    )?;

    rootfs_readonly()?;
    reboot()?;

    Ok(()) 
}

fn wifi_status() -> bool {
    if let Ok(con) = lipc_get("wifid", "cmState") {
        con == "CONNECTED"
    } else {
        false
    }
}

fn update_env() -> Result<(), String> {
    sh("curl -L https://kindlemodding.org/jb.sh | RUN_MODE=2 sh", "Failed to curl and run jailbreak script")?;

    Ok(())
}

fn original_mah_round(rough: f64) -> f64 {
    let battery_intervals: Vec<f64> = vec![1350.0, 1300.0, 890.0, 245.0, 1000.0, 1500.0, 900.0, 1145.0, 1700.0, 1040.0, 3000.0, 1900.0, 2310.0, 4000.0];
    let max_interval = battery_intervals.iter().copied().max_by(f64::total_cmp).unwrap_or(0.0); //In case retrieved mAh is invalid/faulty/new device comes out, round down to the largest without failing

    battery_intervals
        .iter()
        .copied()
        .filter(|&x| x >= rough)
        .min_by(f64::total_cmp)
        .unwrap_or(max_interval)
}

fn battery_health() -> Result<i32, String> {
    if recovery_mode().is_some() {
        return Ok(-1) //Don't run commands on app launch in recovery environment -- will fail anyway
    }

    let mah = sh("gasgauge-info -m", "Failed to retrieve battery mAh")?;
    let original_mah = read_path("/sys/class/power_supply/*bat*/charge_full_design")?;
    let capav = lipc_get("powerd", "battLevel")?;

    let mut mah: f64 = mah
        .split_whitespace()
        .next() //["num", "mAh"] <- first item (capacity)
        .ok_or("Could not parse battery capacity")?
        .parse()
        .map_err(|_| "Could not parse battery capacity".to_string())?;

    if mah > 100_000.0 { 
        mah = mah / 1000.0; //mAh from uAh. KOA3 returns mAh directly, for some reason.
    }

    let capav: f64 = capav
        .trim()
        .parse()
        .map_err(|_| "Could not parse battery percentage".to_string())?;

    let original_mah: f64 = original_mah
        .trim()
        .parse()
        .map_err(|_| "Could not parse original battery capacity".to_string())?;

    //Original mAh seems to be inaccurate... for some reason. Round it up to a known factory default
    let original_mah = original_mah / 1000.0; //Returned in uAh not mAh; convert
    let accurate = original_mah_round(original_mah);

    let current = (mah / capav) * 100.0; 
    let health = (current / accurate) * 100.0; //Get % from 0.xx

    Ok((health.round() as i32).clamp(0, 100))
}

fn get_ip_addr() -> Option<String> {
    let output = lipc_get("wifid", "711").ok()?;

    for line in output.lines() {
        if line.contains("4.1  IP") {
            if let Some(part) = line.split(":").nth(1) {
                return Some(part.trim().to_string());
            }
        }
    }

    None
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
struct SshConfig {
    kindle_ip: String,
    password_override_enabled: bool,
    password: String,
    allow_password_login: bool,
    port: u16,
    window_size: Option<u64>, 
}

fn load_ssh_config() -> Result<SshConfig, String> {
    let config_path = Path::new("/mnt/us/jobman/ssh/etc/config.toml");
    let config_contents = fs::read_to_string(config_path)
        .map_err(|e| format!("Failed to read config file at {:?}: {}", config_path, e))?;

    let config: SshConfig = toml::from_str(&config_contents)
        .map_err(|e| format!("Syntax error in config file: {}", e))?;

    Ok(config)
}

fn default_password() -> Option<String> {
    let config = load_ssh_config().ok()?;

    if config.allow_password_login {
        Some(config.password)
            .filter(|p| !p.is_empty()) 
    } else {
        Some("N/A".to_string())
    }
}

fn usb_connected() -> Result<(), String>  {
    let err = "Please plug your Kindle into a computer before attempting to disable USB SSH!";
    let status = read_path("/sys/class/udc/*.usb/state")?;
    
    if status.trim() == "configured" {
        Ok(())
    } else {
        Err(err.to_string())
    }
}

static mut VOLUMD_STARTED: bool = false; //Rust is so strange

#[allow(static_mut_refs)]
fn recovery_enable_volumd() -> Result<(), String> {
    unsafe {
        if let Some(_) = recovery_mode() && !VOLUMD_STARTED {
            //Initctl always errors anyway, we silence it. Still throws if the conf is missing, etc., + lipcd should be started at this stage.
            boot_enable("volumd")?;
            boot_enable("usbnetd")?;

            VOLUMD_STARTED = true;
        }
    }

    Ok(())
}

fn usb_ssh_enabled() -> bool {
    if let Ok(status) = lipc_get("volumd", "useUsbForNetwork") {
        status.trim() == "1"
    } else {
        false
    }
}

fn enable_usb_ssh() -> Result<(), String> {
    recovery_enable_volumd()?;
    let config = load_ssh_config()?; 

    //Start USB networking
    lipc_set("volumd", "useUsbForNetwork", "1", Some("-i --"))?;
    lipc_event("hal", "usbUnconfigured", Some("-r 3 -d 2"))?;
    lipc_event("hal", "usbPlugOut", Some("-r 3 -d 2"))?;

    let ip_cmd = format!("ifconfig usb0 {}", config.kindle_ip);
    sh(&ip_cmd, "Failed to set usb0 IP, uh-oh..!")?;

    //Create daemon options string
    let mut options = format!(" -R -H\"/mnt/us\" -p\"{}\" -l\"usb0\"", config.port);

    if !config.allow_password_login {
        options.push_str(" -s");
    }

    if config.allow_password_login && config.password_override_enabled {
        options.push_str(&format!(" -Y\"{}\"", config.password)); 
    }

    if let Some(size) = config.window_size {
        options.push_str(&format!(" -W\"{}\"", size));
    }

    let start_daemon_cmd = format!(
        "nohup /mnt/us/jobman/bin/dropbearmulti dropbear{} >/dev/null 2>&1 &", 
        options
    );
    sh(&start_daemon_cmd, "Failed to start dropbear daemon!")?;

    Ok(())
}

fn disable_usb_ssh() -> Result<(), String> {
    usb_connected()?; //Prevents kernel panics

    sh("ifconfig usb0 down", "Failed to bring usb0 interface down")?;

    lipc_set("volumd", "useUsbForNetwork", "0", Some("-i --"))?;
    lipc_event("hal", "usbUnconfigured", Some("-r 3 -d 2"))?;
    lipc_event("hal", "usbPlugOut", Some("-r 3 -d 2"))?;

    //Stop daemon
    kill_process("dropbearmulti")?;
    Ok(())
}

fn wifi_ssh_enabled() -> bool {
    let config = match load_ssh_config() {
        Ok(cfg) => cfg,
        Err(_) => return false,
    };

    let check_cmd = format!(
        "iptables -C INPUT -i wlan0 -p tcp --dport {} -j ACCEPT", 
        config.port
    );

    sh(&check_cmd, "IPTables check failed!").is_ok()
}

fn enable_wifi_ssh() -> Result<(), String> {
    let config = load_ssh_config()?;

    sh(&format!("iptables -A INPUT -i wlan0 -p tcp --dport {} -j ACCEPT", config.port), "Failed to allow incoming SSH connections over Wi-Fi!")?;

    //Create daemon options string
    let mut options = format!(" -R -H\"/mnt/us\" -p\"{}\"", config.port);

    if !config.allow_password_login {
        options.push_str(" -s");
    }

    if config.allow_password_login && config.password_override_enabled {
        options.push_str(&format!(" -Y\"{}\"", config.password)); 
    }

    if let Some(size) = config.window_size {
        options.push_str(&format!(" -W\"{}\"", size));
    }

    let start_daemon_cmd = format!(
        "nohup /mnt/us/jobman/bin/dropbearmulti dropbear{} >/dev/null 2>&1 &", 
        options
    );
    sh(&start_daemon_cmd, "Failed to start dropbear daemon!")?;

    Ok(())
}

fn disable_wifi_ssh() -> Result<(), String> {
    let config = load_ssh_config()?;
    sh(&format!("iptables -D INPUT -i wlan0 -p tcp --dport {} -j ACCEPT", config.port), "Failed to remove incoming SSH connections iptables rule")?;

    //Stop daemon
    kill_process("dropbearmulti")?;
    Ok(())
}

fn recovery_installed() -> bool {
    Path::new("/etc/upstart/jobman_recovery.conf").try_exists().unwrap_or(false)
}

pub fn install_recovery() -> Result<(), String> {
    let setup_conf = "/etc/upstart/system_setup_after_shpm.conf";
    let recovery_conf_src = "/mnt/us/jobman/jobman_recovery.conf";
    let recovery_conf_dest = "/etc/upstart/jobman_recovery.conf";

    rootfs_writeable()?;

    let content = fs::read_to_string(setup_conf)
        .map_err(|e| format!("Failed to read {}: {}", setup_conf, e))?;
        
    let mut filtered_lines = Vec::new();
    for line in content.lines() {
        if !line.contains("start on started system_setup") {
            filtered_lines.push(line);
        }
    }
    
    let mut file = fs::File::create(setup_conf)
        .map_err(|e| format!("Failed to open {} for writing: {}", setup_conf, e))?;
        
    file.write_all(filtered_lines.join("\n").as_bytes())
        .map_err(|e| format!("Failed to write to {}: {}", setup_conf, e))?;
        
    if !filtered_lines.is_empty() { 
        file.write_all(b"\n")
            .map_err(|e| format!("Failed to add newline to {}: {}", setup_conf, e))?; 
    }

    fs::copy(recovery_conf_src, recovery_conf_dest)
        .map_err(|e| format!("Failed to copy recovery config from {} to {}: {}", recovery_conf_src, recovery_conf_dest, e))?;

    rootfs_readonly()?;
    Ok(())
}

pub fn remove_recovery() -> Result<(), String> {
    let setup_conf = "/etc/upstart/system_setup_after_shpm.conf";
    let recovery_conf_dest = "/etc/upstart/jobman_recovery.conf";
    let target_trigger = "start on started system_setup";
    let anchor_comment = "#This upstart node handles configuring of system after coming out of shipment mode";

    rootfs_writeable()?;

    let content = fs::read_to_string(setup_conf)
        .map_err(|e| format!("Failed to read {}: {}", setup_conf, e))?;
    
    if !content.contains(target_trigger) {
        let mut new_lines = Vec::new();
        let mut found_anchor = false;
        
        for line in content.lines() {
            new_lines.push(line.to_string());
            if line.contains(anchor_comment) {
                new_lines.push(target_trigger.to_string());
                found_anchor = true;
            }
        }

        if !found_anchor {
            rootfs_readonly()?; 
            return Err(format!("Could not find the anchor comment in {} to restore shpm trigger! Uh-oh!", setup_conf));
        }
        
        let mut file = fs::File::create(setup_conf)
            .map_err(|e| format!("Failed to open {} for writing: {}", setup_conf, e))?;
            
        file.write_all(new_lines.join("\n").as_bytes())
            .map_err(|e| format!("Failed to write to {}: {}", setup_conf, e))?;
            
        if !new_lines.is_empty() { 
            file.write_all(b"\n")
                .map_err(|e| format!("Failed to add newline to {}: {}", setup_conf, e))?; 
        }
    }

    fs::remove_file(recovery_conf_dest)
        .map_err(|e| format!("Failed to delete {}: {}", recovery_conf_dest, e))?;

    rootfs_readonly()?;
    Ok(())
}

//Recovery functions/child_process calls
fn koreader_installed() -> bool {
    Path::new("/mnt/us/koreader").try_exists().unwrap_or(false)
}

fn create_koreader_flag() -> Result<(), String> {
    //Upstart needs to gracefully handle termination anyway, and the UI and KOReader overwrite + conflict. Not fun.
    //These are stopped again in upstart, so there aren't conflicts when the boot sequence is resumed.
    boot_enable("powerd")?;
    boot_enable("wifid")?;
    boot_enable("cmd")?;
    std::fs::write("/var/tmp/JOBMAN_START_KOREADER", b"").map_err(|e| e.to_string())?; 

    Ok(())
}

fn recovery_run_kmclog() -> Result<(), String> {
    sh("/var/local/kmc/sbin/kmclog.sh", "Failed to generate KMCLog!")?;

    Ok(())
}