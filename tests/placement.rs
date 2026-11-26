use magpie::placement::{self, Placement, Rect, Screen, State};

const LAPTOP: Screen = Screen {
    work_area: Rect {
        x: 0.0,
        y: 0.0,
        width: 1536.0,
        height: 816.0,
    },
    primary: true,
};

const MONITOR: Screen = Screen {
    work_area: Rect {
        x: 1536.0,
        y: -200.0,
        width: 2560.0,
        height: 1400.0,
    },
    primary: false,
};

fn windowed(x: f32, y: f32, width: f32, height: f32) -> Placement {
    Placement {
        bounds: Rect {
            x,
            y,
            width,
            height,
        },
        state: State::Windowed,
    }
}

#[test]
fn a_window_stays_on_the_screen_that_shows_most_of_it() {
    let straddling = windowed(1300.0, 100.0, 1200.0, 800.0);

    let (screen, fitted) = straddling.fit(&[LAPTOP, MONITOR]).unwrap();

    assert_eq!(screen, 1);
    assert_eq!(fitted, windowed(1536.0, 100.0, 1200.0, 800.0));
}

#[test]
fn a_window_left_on_a_disconnected_screen_comes_back_whole_on_the_primary_one() {
    let on_the_monitor = Placement {
        state: State::Maximized,
        ..windowed(2400.0, -100.0, 1800.0, 1000.0)
    };

    let (screen, fitted) = on_the_monitor.fit(&[LAPTOP]).unwrap();

    assert_eq!(screen, 0);
    assert_eq!(
        fitted,
        Placement {
            state: State::Maximized,
            ..windowed(0.0, 0.0, 1536.0, 816.0)
        }
    );
    assert_eq!(on_the_monitor.fit(&[]), None);
}

#[test]
fn the_saved_placement_survives_a_restart_and_broken_bounds_are_ignored() {
    let folder = tempfile::tempdir().unwrap();
    let saved = Placement {
        state: State::Maximized,
        ..windowed(-8.5, 40.0, 1360.0, 860.0)
    };

    placement::save(folder.path(), &saved).unwrap();
    assert_eq!(placement::load(folder.path()).unwrap(), Some(saved));

    std::fs::write(
        folder.path().join("window.json"),
        r#"{"bounds":{"x":0,"y":0,"width":-1,"height":600},"state":"windowed"}"#,
    )
    .unwrap();
    assert_eq!(placement::load(folder.path()).unwrap(), None);
}
