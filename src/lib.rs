use std::ffi::{CStr, CString};

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

#[derive(Clone, Debug)]
pub struct BladeRfDevice {
    handle: *mut bladerf,
}

impl BladeRfDevice {
    pub fn from_device_serial(serial: &str) -> Self {
        let device_string = CString::new(format!("*:serial={}", serial)).expect("failed to convert String -> CString");
        let mut devptr: *mut bladerf = std::ptr::null_mut();
        let status = unsafe { bladerf_open(&mut devptr, device_string.as_ptr()) };
        return Self { handle: devptr }
    }

    pub fn get_devinfo(&self) -> BladeRfDevInfo {
        let mut devinfo: *mut bladerf_devinfo = std::ptr::null_mut();
        let _status = unsafe { bladerf_get_devinfo(self.handle, devinfo) };
        BladeRfDevInfo::from(unsafe { *devinfo })
    }
}

impl Drop for BladeRfDevice {
    fn drop(&mut self) {
        unsafe { bladerf_close(self.handle); };
    }
}

#[derive(Clone, Debug)]
pub struct BladeRfDevInfo {
    pub backend: u32,
    pub serial: String,
    pub usb_bus: u8,
    pub usb_addr: u8,
    pub instance: u32,
    pub manufacturer: String,
    pub product: String,
}

impl BladeRfDevInfo {
    pub fn from(devinfo: bladerf_devinfo) -> Self {
        Self {
            backend: devinfo.backend as u32,
            serial: unsafe { CStr::from_ptr(devinfo.serial.as_ptr()) }.to_string_lossy().to_string(),
            usb_bus: devinfo.usb_bus,
            usb_addr: devinfo.usb_addr,
            instance: devinfo.instance,
            manufacturer: unsafe { CStr::from_ptr(devinfo.manufacturer.as_ptr()) }.to_string_lossy().to_string(),
            product: unsafe { CStr::from_ptr(devinfo.product.as_ptr()) }.to_string_lossy().to_string(),
        }
    }

    pub fn open(&mut self) -> BladeRfDevice {
        // TODO: Can we open again if its already open?
        BladeRfDevice::from_device_serial(&self.serial)
    }
}

pub fn get_devices() -> Vec<BladeRfDevInfo> {
    let mut devptr: *mut bladerf_devinfo = std::ptr::null_mut();
    let n_devices = unsafe { bladerf_get_device_list(&mut devptr) };
    let sraw = if n_devices > 0 {
        if !devptr.is_null() {
            unsafe {
                std::slice::from_raw_parts(
                    devptr,
                    n_devices as usize,
                )
            }
        } else {
            // If devptr is NULL for some reason just return empty slice
            &[]
        }
    } else {
        // If no devices then just return empty slice
        &[]
    };
    let output = sraw.iter().map(|v| BladeRfDevInfo::from(*v)).collect();
    if !devptr.is_null() {
        unsafe { bladerf_free_device_list(devptr); }
    }
    output
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
        let mut v = get_devices();
        println!("Devices: {:?}", v);
        if v.len() > 0 {
            let brf_handle = v[0].open();
            println!("handle: {:?}", brf_handle);
            println!("handle.devinfo(): {:?}", brf_handle.get_devinfo());
        }
        assert_eq!(4, 4);
    }
}
