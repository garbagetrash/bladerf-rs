use bladerf_sys::*;

pub fn get_version() -> bladerf_version {
    let mut v = bladerf_version {
        major: 0,
        minor: 0,
        patch: 0,
        describe: std::ptr::null(),
    };
    unsafe { bladerf_version(&mut v); };
    v
}

pub fn get_devices() -> Vec<bladerf_devinfo> {
    todo!();
    let mut output: *mut *mut bladerf_devinfo = std::ptr::null_mut();
    unsafe { bladerf_get_device_list(output); };
    vec![]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_print_version() {
        let v = get_version();
        println!("BladeRF Version: {}.{}.{}", v.major, v.minor, v.patch);
        assert_eq!(4, 4);
    }

    #[test]
    fn test_print_devices() {
        let v = get_devices();
        println!("Devices: {:?}", v);
        assert_eq!(4, 4);
    }
}
