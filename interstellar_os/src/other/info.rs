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

pub struct BootInfo {
    // Cannot use memory regions as this is not safe to send across threads
    pub api_version: ApiVersion,
    pub framebuffer_info: Option<FrameBufferInfo>,
    pub physical_memory_offset: Option<u64>,
    pub recursive_index: Option<u16>,
    pub rsdp_addr: Option<u64>,
    pub tls_template: Option<TlsTemplate>,
    pub ramdisk_addr: Option<u64>,
    pub ramdisk_len: u64,
}

pub struct Info<'a> {
    pub os_version: &'a str,
}
