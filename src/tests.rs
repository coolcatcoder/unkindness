#[cfg(test)]
extern crate std;
use core::fmt::Write;

macro_rules! test_with_features {
    ($visibility:vis mod $module:ident = $($features:literal),*) => {
        $(
            #[cfg(feature = $features)]
        )*
        #[cfg(test)]
        $visibility mod $module;

        #[test]
        fn $module() {
            run_with_features(
                &std::format!("{}::main", stringify!($module)),
                [
                    $($features),*
                ],
            );
        }
    };
}

fn run_with_features<const LENGTH: usize>(module: &str, features: [&str; LENGTH]) {
    let mut features_as_string = std::string::String::new();
    for feature in features {
        write!(&mut features_as_string, "{feature},").expect("Writing to a String will work.");
    }
    let output = std::process::Command::new("cargo")
        .args(["test", module, "--features", &features_as_string])
        .output()
        .expect("The command will work.");

    let stdout = str::from_utf8(&output.stdout).expect("Valid utf8.");
    let stderr = str::from_utf8(&output.stderr).expect("Valid utf8.");

    assert!(
        !(stdout.contains("test result: FAILED")
            || stderr.contains("error: could not compile")
            || stderr.contains("warning")),
        "START STDOUT START\n{stdout}\nEND END END\nSTART STDERR START\n{stderr}\nEND END END"
    );
}

test_with_features!(mod scene_macro =);
