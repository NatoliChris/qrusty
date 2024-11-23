//! # QRusty
//!
//! QRusty the qrab, a small applet to read QRs on the screen and present them
//! either as text or copied directly to the clipboard.
//!
//! Right now it's not battle tested, nor is it ready for production.
//!
//! ## Commands
//!
//! * `-s` : select with the mouse by click and dragging a bounding box.
//! * `-c` : copy the identified QR code into clipboard (note, multiple QRs
//!          will lead to a space-delimited array copied)

use std::{thread::sleep, time::Duration};

use bounding_box::BoundingBox;
use clap::{Arg, ArgAction, Command};
use copypasta::{ClipboardContext, ClipboardProvider};
use device_query::{DeviceQuery, DeviceState};
use image::{imageops, DynamicImage, RgbaImage};
use notify_rust::Notification;
use rqrr::MetaData;
use xcap::Monitor;

mod bounding_box;

#[derive(Clone, Debug)]
struct MouseBox {
    start_x: i32,
    end_x: i32,
    start_y: i32,
    end_y: i32,
}

/// Get a selection from a user
///
/// The user clicks and drags their mouse to create a bounding
/// rectangle.
/// TODO: find a way to provide a visual of the rectangle as the user
/// is drawing it.
/// TODO: find a way to act like `scrot -s`; have the tool capture the click
/// and drag so that it doesn't interact with the page, etc.
fn get_region_from_user() -> Result<MouseBox, ()> {
    // variables for tracking
    let mut init_selection = false;
    let mut is_selecting = false;

    // the screen box to be created
    let mut sc_box = MouseBox {
        start_x: 0,
        start_y: 0,
        end_x: 0,
        end_y: 0,
    };

    // get the device state
    let d_state = DeviceState::new();

    // the loop over events until necessary
    loop {
        // Get the mouse
        let m_state = d_state.get_mouse();
        let coords = m_state.coords;
        if m_state.button_pressed[1] {
            // the left button is pressed for the first time
            if !is_selecting && !init_selection {
                sc_box.start_x = coords.0;
                sc_box.start_y = coords.1;
                init_selection = true;
                is_selecting = true;
            }
        }

        // if the button is now released
        if !m_state.button_pressed[1] && is_selecting && init_selection {
            sc_box.end_x = coords.0;
            sc_box.end_y = coords.1;
            is_selecting = false;
        }

        if !is_selecting && init_selection {
            // the button click has stopped - return
            break;
        }
    }

    Ok(sc_box)
}

/// Get which monitor the selection falls on.
///
/// This attempts to find which monitor the mouse's selection falls on.
/// TODO: find a way to morph multiple monitors for screenshots over multiple
/// screens
fn find_monitor_selected(selection_bound: &BoundingBox) -> Result<Monitor, ()> {
    if let Ok(m_list) = Monitor::all() {
        for mon in m_list {
            // Create the monitor bounding box
            let monitor_box =
                BoundingBox::new(mon.x(), mon.y(), mon.width() as i32, mon.height() as i32);
            // Checking for bounding box overlaps
            // NOTE: screen Y position increases as you move down in xrandr?
            if monitor_box.intersects(selection_bound) {
                return Ok(mon);
            }
        }
    }
    Err(())
}

/// List all the monitors
///
/// List all onitors returning a list of the monitors - used in the tool
/// when default is to check QRs on all screens.
fn list_all_monitors() -> Result<Vec<Monitor>, ()> {
    if let Ok(m_list) = Monitor::all() {
        return Ok(m_list);
    }

    // Could not return any monitors
    Err(())
}

/// Grab the screen at the selected area from the mouse
///
/// This will crop the screen area to the mouse selection.
/// NOTE: there are a few potential bugs here with crossing monitors, still
/// have to fix them up.
fn get_screen_shot(selection: &MouseBox) -> Result<RgbaImage, ()> {
    let selection_bound = BoundingBox::new_from_coords(
        selection.start_x,
        selection.start_y,
        selection.end_x,
        selection.end_y,
    );

    let maybe_monitor = find_monitor_selected(&selection_bound);
    if let Ok(monitor) = maybe_monitor {
        let maybe_ss = monitor.capture_image();

        if let Ok(mut ss) = maybe_ss {
            // Create a bounding box relative to the screenshot
            let relative_box = BoundingBox::new_from_coords(
                selection_bound.top_left.x - monitor.x(),
                selection_bound.top_left.y - monitor.y(),
                selection_bound.bottom_right.x - monitor.x(),
                selection_bound.bottom_right.y - monitor.y(),
            );

            // Crop the picture to the selection
            let sub = imageops::crop(
                &mut ss,
                relative_box.top_left.x as u32,
                relative_box.top_left.y as u32,
                (relative_box.top_right.x - relative_box.top_left.x) as u32,
                (relative_box.bottom_left.y - relative_box.top_left.y) as u32,
            );

            // Return the array
            return Ok(sub.to_image());
        }
    }
    Err(())
}

/// Screen grab all the monitors available
///
/// Will take a screenshot of all monitors
fn screenshot_all_monitors() -> Result<Vec<RgbaImage>, ()> {
    let maybe_monitors = list_all_monitors();

    if let Ok(monitors) = maybe_monitors {
        monitors
            .into_iter()
            .map(|m| m.capture_image())
            .map(|r| if let Ok(img) = r { Ok(img) } else { Err(()) })
            .collect::<Result<Vec<RgbaImage>, ()>>()
    } else {
        Err(())
    }
}

/// Finds all valid QRs from the list of screenshots passed in.
///
/// From all screenshots, it will transform into a grayscale luma image
/// then it will run the qr reader (from `rqrr`) to prepare and read all
/// QR grids it can locate.
///
/// NOTE: this doesn't find a lot of variants of QRs, maybe worth looking into
/// other libraries that can identify and decode more -- even barcodes?
fn find_all_qrs(imgs: Vec<RgbaImage>) -> Vec<String> {
    imgs.into_iter()
        .flat_map(|i| {
            let luma_img = DynamicImage::ImageRgba8(i).into_luma8();
            let mut dec = rqrr::PreparedImage::prepare(luma_img);

            let grids = dec.detect_grids();
            grids
                .iter()
                .map(|x| {
                    x.decode().unwrap_or((
                        // Test dummy data
                        MetaData {
                            version: rqrr::Version(1),
                            ecc_level: 1,
                            mask: 1,
                        },
                        String::from(""),
                    ))
                })
                .map(|x| x.1)
                .collect::<Vec<String>>()
        })
        .collect::<Vec<String>>()
}

// Send it to the clipboard context
fn send_to_clipboard(qrs: Vec<String>) {
    let maybe_ctx = ClipboardContext::new();

    if let Ok(mut ctx) = maybe_ctx {
        println!("setting: {:?}", qrs);
        // todo: fix
        if let Err(e) = ctx.set_contents(qrs.join(" ").to_string().to_owned()) {
            println!("Failed to set clipboard");
            return;
        }
        let _ = ctx.get_contents();

        // TODO: look at persistence for the clipboard
        sleep(Duration::from_secs(5));
    } else {
        eprintln!("Failed to get clipboard context, falling back to print");
        println!("{:?}", qrs.join(" "));
    }
}

/// Main QRusty
fn main() {
    // Set up the command line arguments
    let arg_matches = Command::new("qrusty")
        .version("0.1")
        .about("Read QR codes from an area of the screen, or the full screen")
        .arg(
            Arg::new("select")
                .short('s')
                .long("select")
                .action(ArgAction::SetTrue)
                .help("Capture part of the display"),
        )
        .arg(
            Arg::new("clipboard")
                .short('c')
                .long("clip")
                .action(ArgAction::SetTrue)
                .help("Copy output directly to the clipboard"),
        )
        .get_matches();

    let mut found_qrs: Vec<String> = Vec::new();

    // Call the appropriate function to capture the area or the full screen.
    if let Ok(Some(true)) = arg_matches.try_get_one::<bool>("select") {
        if let Ok(reg) = get_region_from_user() {
            if let Ok(ss) = get_screen_shot(&reg) {
                found_qrs = find_all_qrs(vec![ss]);
            };
        }
    } else if let Ok(images) = screenshot_all_monitors() {
        found_qrs = find_all_qrs(images);
    }

    if let Err(_) = Notification::new()
        .summary("QRusty")
        .body(format!("QR: {}", found_qrs.join(" ")).as_str())
        .show()
    {
        eprintln!("Failed to send notification");
    }

    // Now send to clipboard if the clipboard
    if let Ok(Some(true)) = arg_matches.try_get_one::<bool>("clipboard") {
        send_to_clipboard(found_qrs);
    } else {
        // Print it out
        println!("{:?}", found_qrs);
    }
}
