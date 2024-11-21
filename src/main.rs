use bounding_box::BoundingBox;
use clap::{Arg, ArgAction, Command};
use device_query::{DeviceQuery, DeviceState};
use image::{imageops, DynamicImage, RgbaImage};
use xcap::Monitor;

mod bounding_box;

#[derive(Clone, Debug)]
struct MouseBox {
    start_x: i32,
    end_x: i32,
    start_y: i32,
    end_y: i32,
}

// Get the user to select a region on the screen
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

// Find what monitor the selection was in.
fn find_monitor_box(selection_bound: &BoundingBox) -> Result<Monitor, ()> {
    // TODO: find a way to morph multiple monitors for screenshots over multiple screens!

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
        Err(())
    } else {
        Err(())
    }
}

fn get_screen_shot(selection: &MouseBox) -> Result<RgbaImage, ()> {
    let selection_bound = BoundingBox::new_from_coords(
        selection.start_x,
        selection.start_y,
        selection.end_x,
        selection.end_y,
    );

    let maybe_monitor = find_monitor_box(&selection_bound);
    if let Ok(monitor) = maybe_monitor {
        // DEBUG
        println!("{:?}", monitor.name());
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

fn find_qr(im: &RgbaImage) {
    let gray = DynamicImage::ImageRgba8(im.clone()).into_luma8();

    let mut dec = rqrr::PreparedImage::prepare(gray);
    let grids = dec.detect_grids();

    // Debug print to string for now
    println!(
        "{:?}",
        grids
            .iter()
            .map(|x| x.decode().unwrap())
            .map(|x| x.1)
            .collect::<Vec<String>>()
    );
}

fn main() {
    // set up the matches
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
        .get_matches();

    if let Ok(Some(true)) = arg_matches.try_get_one::<bool>("select") {
        if let Ok(reg) = get_region_from_user() {
            if let Ok(ss) = get_screen_shot(&reg) {
                find_qr(&ss)
            };
        }
    } else {
        println!("Capturing entire screen");
    }
}
