use clap::{Arg, ArgAction, Command};
use device_query::{DeviceQuery, DeviceState};
use image::imageops;
use xcap::Monitor;

#[derive(Clone, Debug)]
struct ScreenBox {
    start_x: i32,
    end_x: i32,
    start_y: i32,
    end_y: i32,
}

#[derive(Clone, Debug)]
struct Coord {
    x: i32,
    y: i32,
}

fn normalize_name(file_name: &str) -> String {
    file_name
        .replace("|", "")
        .replace("\\", "")
        .replace(":", "")
        .replace("/", "")
}

#[derive(Clone, Debug)]
struct BoundingBox {
    top_left: Coord,
    top_right: Coord,
    bottom_left: Coord,
    bottom_right: Coord,
}

impl BoundingBox {
    pub fn new(x: i32, y: i32, width: i32, height: i32) -> Self {
        Self {
            top_left: Coord { x, y },
            top_right: Coord { x: x + width, y },
            bottom_left: Coord { x, y: y + height },
            bottom_right: Coord {
                x: x + width,
                y: y + height,
            },
        }
    }

    pub fn new_from_coords(x: i32, y: i32, x_2: i32, y_2: i32) -> Self {
        Self {
            top_left: Coord { x, y },
            top_right: Coord { x: x_2, y },
            bottom_left: Coord { x, y: y_2 },
            bottom_right: Coord { x: x_2, y: y_2 },
        }
    }

    pub fn intersects(&self, other: &BoundingBox) -> bool {
        return !(self.top_right.x < other.bottom_left.x
            || self.bottom_left.x > other.top_right.x
            || self.top_right.y > other.bottom_left.y
            || self.bottom_left.y < other.top_right.y);
    }
}

// Get the user to select a region on the screen
fn get_region_from_user() -> Result<ScreenBox, ()> {
    // variables for tracking
    let mut init_selection = false;
    let mut is_selecting = false;

    // the screen box to be created
    let mut sc_box = ScreenBox {
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
        if !m_state.button_pressed[1] {
            if is_selecting && init_selection {
                sc_box.end_x = coords.0;
                sc_box.end_y = coords.1;
                is_selecting = false;
            }
        }

        if !is_selecting && init_selection {
            // the button click has stopped - return
            break;
        }
    }

    return Ok(sc_box);
}

// Find what monitor the selection was in.
fn find_monitor_box(selection_box: &BoundingBox) -> Result<Monitor, ()> {
    // TODO: find a way to morph multiple monitors for screenshots over multiple screens!

    if let Ok(m_list) = Monitor::all() {
        for mon in m_list {
            // Create the monitor bounding box
            let monitor_box =
                BoundingBox::new(mon.x(), mon.y(), mon.width() as i32, mon.height() as i32);
            // Checking for bounding box overlaps
            // NOTE: screen Y position increases as you move down in xrandr?
            if monitor_box.intersects(selection_box) {
                return Ok(mon);
            }
        }
        Err(())
    } else {
        Err(())
    }
}

fn get_screen_shot(selection: &ScreenBox) -> Result<(), ()> {
    let selection_box = BoundingBox::new_from_coords(
        selection.start_x,
        selection.start_y,
        selection.end_x,
        selection.end_y,
    );

    let maybe_monitor = find_monitor_box(&selection_box);
    if let Ok(monitor) = maybe_monitor {
        println!("{:?}", monitor.name());
        let maybe_ss = monitor.capture_image();

        if let Ok(mut ss) = maybe_ss {
            // get the local coordinates of the bounding box
            let relative_box = BoundingBox::new_from_coords(
                selection_box.top_left.x - monitor.x(),
                selection_box.top_left.y - monitor.y(),
                selection_box.bottom_right.x - monitor.x(),
                selection_box.bottom_right.y - monitor.y(),
            );

            let sub = imageops::crop(
                &mut ss,
                relative_box.top_left.x as u32,
                relative_box.top_left.y as u32,
                (relative_box.top_right.x - relative_box.top_left.y) as u32,
                (relative_box.bottom_left.y - relative_box.top_left.y) as u32,
            );
            // Do something
            sub.to_image()
                .save(format!("test_{}.png", normalize_name(monitor.name())))
                .unwrap();
        }
    }
    Err(())
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
            println!("{:?}", get_screen_shot(&reg));
        }
    } else {
        println!("Capturing entire screen");
    }
}
