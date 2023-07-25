//Copyright (C) <2023>  <interstellarfrog>
//
//This program is free software: you can redistribute it and/or modify
//it under the terms of the GNU General Public License as published by
//the Free Software Foundation, either version 3 of the License, or
//(at your option) any later version.
//
//This program is distributed in the hope that it will be useful,
//but WITHOUT ANY WARRANTY; without even the implied warranty of
//MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
//GNU General Public License for more details.
//
//You should have received a copy of the GNU General Public License
//along with this program.  If not, see <https://www.gnu.org/licenses/>.

use bootloader_api::{
    config::ApiVersion,
    info::{FrameBufferInfo, TlsTemplate},
};
use conquer_once::spin::OnceCell;
use spinning_top::Spinlock;

pub static BOOT_INFO: OnceCell<Spinlock<BootInfo>> = OnceCell::uninit();

pub static INFO: OnceCell<Spinlock<Info>> = OnceCell::uninit();

#[allow(clippy::too_many_arguments)]
pub fn boot_info_init(
    api_version: ApiVersion,
    framebuffer_info: FrameBufferInfo,
    physical_memory_offset: u64,
    recursive_index: Option<u16>,
    rsdp_addr: u64,
    tls_template: Option<TlsTemplate>,
    ramdisk_addr: Option<u64>,
    ramdisk_len: u64,
) {
    BOOT_INFO.init_once(|| {
        Spinlock::new(BootInfo {
            api_version,
            framebuffer_info,
            physical_memory_offset,
            recursive_index,
            rsdp_addr,
            tls_template,
            ramdisk_addr,
            ramdisk_len,
        })
    });
}

pub fn info_init() {
    INFO.init_once(|| {
        let os_version = env!("CARGO_PKG_VERSION", "Could not get OS version");

        Spinlock::new(Info { os_version })
    });
}

pub struct BootInfo {
    // Cannot use memory regions as this is not safe to send across threads
    pub api_version: ApiVersion,
    pub framebuffer_info: FrameBufferInfo, // Required
    pub physical_memory_offset: u64,       // Required
    pub recursive_index: Option<u16>,
    pub rsdp_addr: u64, // Required
    pub tls_template: Option<TlsTemplate>,
    pub ramdisk_addr: Option<u64>,
    pub ramdisk_len: u64,
}

pub struct Info<'a> {
    pub os_version: &'a str,
}
