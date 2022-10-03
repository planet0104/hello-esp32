use std::{ffi::CString, ptr::null_mut};

use esp_idf_sys::{nvs_open_mode_t_NVS_READWRITE, nvs_handle_t, nvs_open, size_t, nvs_get_str, nvs_close, nvs_set_str};
use anyhow::{Result, anyhow};

//https://blog.csdn.net/weixin_42328389/article/details/122703875

pub fn read_string(storage_name:&str) -> Result<String>{
    let mut nvs_handle = nvs_handle_t::default();
    let mut error_msg = None;
    let mut out_value = String::new();
    unsafe{
        loop{
            let storage_name = CString::new(storage_name)?;
            let res = nvs_open(storage_name.as_ptr(), nvs_open_mode_t_NVS_READWRITE, &mut nvs_handle);
            if res != 0{
                error_msg.replace(format!("nvs_open失败:{res}"));
                break;
            }
            
            // 获取数据长度
            let mut required_size = size_t::default();
            let res = nvs_get_str(nvs_handle, storage_name.as_ptr(), null_mut(), &mut required_size);
            if res != 0{
                error_msg.replace(format!("nvs_get_str 0失败:{res}"));
                break;
            }

            // 初始化字符串
            let empty_string:String = vec![' '; required_size as usize].iter().collect();
            let cfg_data = CString::new(empty_string)?;

            // 读取字符串
            let cfg_data_ptr = cfg_data.into_raw();
            let res = nvs_get_str(nvs_handle, storage_name.as_ptr(), cfg_data_ptr, &mut required_size);
            let cfg_data = CString::from_raw(cfg_data_ptr);
            if res != 0{
                error_msg.replace(format!("nvs_get_str 1失败:{res}"));
            }else{
                out_value = cfg_data.to_str()?.to_string();
            }
            break;
        }
    }

    unsafe{ nvs_close(nvs_handle) };

    match error_msg{
        Some(err) => {
            Err(anyhow!("{err}"))
        }
        None => Ok(out_value)
    }
}

pub fn write_string(storage_name:&str, data:&str) -> Result<()>{
    let mut nvs_handle = nvs_handle_t::default();
    let mut error_msg = None;

    unsafe{
        loop{
            let storage_name = CString::new(storage_name)?;
            let res = nvs_open(storage_name.as_ptr(), nvs_open_mode_t_NVS_READWRITE, &mut nvs_handle);
            if res != 0{
                error_msg.replace(format!("nvs_open失败:{res}"));
                break;
            }
            
            //写入字符串
            let save_data = CString::new(data)?;
            
            let res = nvs_set_str(nvs_handle, storage_name.as_ptr(), save_data.as_ptr());
            if res != 0{
                error_msg.replace(format!("nvs_set_str失败:{res}"));
            }
            break;
        }
        
    }

    unsafe{ nvs_close(nvs_handle) };

    match error_msg{
        Some(err) => {
            Err(anyhow!("{err}"))
        }
        None => Ok(())
    }
}