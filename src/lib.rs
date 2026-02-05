use std::ffi::{c_void, CStr, CString};

use bladerf_sys::*;
use num_complex::Complex;

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
    samples_per_block: usize,
}

impl BladeRfDevice {
    pub fn from_device_serial(serial: &str) -> Option<Self> {
        // Open device using given serial
        let device_string = CString::new(format!("*:serial={}", serial)).expect("failed to convert String -> CString");
        let mut devptr: *mut bladerf = std::ptr::null_mut();
        if unsafe { bladerf_open(&mut devptr, device_string.as_ptr()) } < 0 {
            return None;
        }

        // Configuration for synchronous operation
        let layout = bladerf_channel_layout_BLADERF_RX_X1;
        let format = bladerf_format_BLADERF_FORMAT_SC16_Q11;
        //let format = bladerf_format_BLADERF_FORMAT_SC8_Q7;
        let bufsize_samples = 8192;
        let ntransfers = 4;
        let nbuffers = 4 * ntransfers;
        let stream_timeout = 0;
        if unsafe { bladerf_sync_config(devptr, layout, format, nbuffers, bufsize_samples, ntransfers, stream_timeout) } < 0 {
            return None;
        }

        // Now enable channel 0???
        let channel = 0;
        if unsafe { bladerf_enable_module(devptr, channel, true) } < 0 {
            None
        } else {
            Some(BladeRfDevice { handle: devptr, samples_per_block: bufsize_samples as usize })
        }
    }

    pub fn get_devinfo(&self) -> Option<BladeRfDevInfo> {
        let mut devinfo = bladerf_devinfo {
            backend: 0,
            serial: [0; 33],
            usb_bus: 0,
            usb_addr: 0,
            instance: 0,
            manufacturer: [0; 33],
            product: [0; 33],
        };
        if unsafe { bladerf_get_devinfo(self.handle, &mut devinfo) } < 0 {
            None
        } else {
            Some(BladeRfDevInfo::from(devinfo))
        }
    }

    pub fn get_samplerate(&self, channel: i32) -> u32 {
        let mut samplerate = 0;
        let status = unsafe { bladerf_get_sample_rate(self.handle, channel, &mut samplerate) };
        samplerate
    }

    pub fn set_samplerate(&self, samplerate: u32, channel: i32) -> u32 {
        let mut actual_samplerate = 0;
        let status = unsafe { bladerf_set_sample_rate(self.handle, channel, samplerate, &mut actual_samplerate) };
        actual_samplerate
    }

    pub fn get_bias_tee(&self, channel: i32) -> bool {
        let mut enable = false;
        let status = unsafe { bladerf_get_bias_tee(self.handle, channel, &mut enable) };
        enable
    }

    pub fn set_bias_tee(&self, enable: bool, channel: i32) {
        let status = unsafe { bladerf_set_bias_tee(self.handle, channel, enable) };
    }

    pub fn get_frequency(&self, channel: i32) -> u64 {
        let mut frequency = 0;
        let status = unsafe { bladerf_get_frequency(self.handle, channel, &mut frequency) };
        frequency
    }

    pub fn set_frequency(&self, frequency: u64, channel: i32) {
        let status = unsafe { bladerf_set_frequency(self.handle, channel, frequency) };
    }

    pub fn get_bandwidth(&self, channel: i32) -> u32 {
        let mut bandwidth = 0;
        let status = unsafe { bladerf_get_bandwidth(self.handle, channel, &mut bandwidth) };
        bandwidth
    }

    pub fn set_bandwidth(&self, bandwidth: u32, channel: i32) -> u32 {
        let mut actual = 0;
        let status = unsafe { bladerf_set_bandwidth(self.handle, channel, bandwidth, &mut actual) };
        actual
    }

    pub fn recv(&mut self, num_blocks: usize) -> Vec<Complex<i16>> {
        let num_samples = num_blocks * self.samples_per_block;
        let mut samples = vec![Complex::<i16>::ZERO; num_samples];
        let mut meta = std::ptr::null_mut();
        let timeout_ms = 1000;

        let (ptr, len, cap) = samples.into_raw_parts();
        let status = unsafe { bladerf_sync_rx(self.handle, ptr as *mut c_void, num_samples as u32, meta, timeout_ms) };
        unsafe { Vec::from_raw_parts(ptr, len, cap) }
    }
}

impl Drop for BladeRfDevice {
    fn drop(&mut self) {
        //println!("dropping {:?}", self);
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

    pub fn open(&mut self) -> Option<BladeRfDevice> {
        // TODO: Can we open again if its already open? No, but this returns "successfully with a
        // handle of 0x0, which is invalid. Need to do something here...
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
            // does drop work?
            let mut brf = v[0].open().expect("failed to open");
            println!("handle: {:?}", brf);
            println!("handle.devinfo(): {:?}", brf.get_devinfo());

            let actual = brf.set_samplerate(10_000_000, 1);
            println!("actual: {}", actual);
            println!("samplerate(0): {}", brf.get_samplerate(0));
            println!("samplerate(1): {}", brf.get_samplerate(1));
            println!("samplerate(2): {}", brf.get_samplerate(2));
            println!("samplerate(3): {}", brf.get_samplerate(3));

            brf.set_bias_tee(true, 0);
            println!("bias_tee: {}", brf.get_bias_tee(0));
            brf.set_bias_tee(false, 0);
            println!("bias_tee: {}", brf.get_bias_tee(0));

            brf.set_frequency(100_000_000, 0);
            println!("frequency: {}", brf.get_frequency(0));
            brf.set_frequency(200_000_000, 0);
            println!("frequency: {}", brf.get_frequency(0));

            brf.set_bandwidth(1_000_000, 0);
            println!("bandwidth: {}", brf.get_bandwidth(0));
            brf.set_bandwidth(2_000_000, 0);
            println!("bandwidth: {}", brf.get_bandwidth(0));

            let samples = brf.recv(2);
            println!("samples.len(): {}", samples.len());
        }
        assert_eq!(4, 4);
    }
}
