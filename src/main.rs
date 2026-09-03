slint::include_modules!();
use std::path::Path;
use std::process::Command;

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

    app.set_ota_status(ota_status());
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
    app.on_toggle_ota(move || {
        let app = app_weak.unwrap();
        
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

    app.on_quit(|| std::process::exit(0));
    app.run().expect("Event Loop Error!");
}

//UI backend
fn ota_status() -> bool {
    Path::new("/usr/bin/otav3").try_exists().unwrap_or(false)
}

fn sh(cmd: &str, err: &str) -> Result<String, String> {
    let output = Command::new("sh")
        .args(["-c", cmd])
        .output()
        .map_err(|e| format!("{err} (Process Error: {e})"))?; 

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).into_owned())
    } else {
        Err(format!("{err} (Exit Code: {})", output.status.code().unwrap_or(-1)))
    }
}

fn chattr_path() -> &'static str {
    if Path::new("/bin/chattr.e2fsprogs").exists() {
        "/bin/chattr.e2fsprogs"
    } else {
        "/bin/chattr"
    }
}

fn block_ota() -> Result<(), String> {
    let chattr = chattr_path();

    sh("mntroot rw", "Failed to mount RootFS as writeable")?;

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

    let _ = sh("mntroot ro", "");

    Command::new("sh")
        .args(["-c", "sleep 3 && reboot"])
        .spawn()
        .map_err(|e| format!("Failed to reboot Kindle: {e}"))?;

    Ok(()) 
}

fn enable_ota() -> Result<(), String> {
    let chattr = chattr_path();

    sh("mntroot rw", "Failed to mount RootFS as writeable")?;

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

    let _ = sh("mntroot ro", ""); 

    Command::new("sh")
        .args(["-c", "sleep 3 && reboot"])
        .spawn()
        .map_err(|e| format!("Failed to reboot Kindle: {e}"))?;

    Ok(()) 
}

fn battery_health() -> Result<i32, String> {
    let mah = sh("gasgauge-info -m", "Failed to retrieve battery mAh")?;
    let capav = sh("lipc-get-prop com.lab126.powerd battLevel", "Failed to retrieve battery capacity")?;
    let original_mah = sh("cat /sys/class/power_supply/bd*_bat/charge_full_design", "Failed to read battery initial capacity")?;

    let mah: f64 = mah
        .split_whitespace()
        .next() //["num", "mAh"] <- first item (capacity)
        .ok_or("Could not parse battery capacity")?
        .parse()
        .map_err(|_| "Could not parse battery capacity".to_string())?;

    let mah = mah / 1000.0;

    let capav: f64 = capav
        .trim()
        .parse()
        .map_err(|_| "Could not parse battery percentage".to_string())?;

    let original_mah: f64 = original_mah
        .trim()
        .parse()
        .map_err(|_| "Could not parse original battery capacity".to_string())?;

    let original_mah = original_mah / 1000.0;
    let current = (mah / capav) * 100.0;
    let health = (current / original_mah) * 100.0;

    Ok(health.round() as i32)
}