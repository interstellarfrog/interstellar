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

super::stream_processor_task!(u8, 2048);

pub async fn process() {
    let mut stream = TaskStream::new();
    let mouse = crate::drivers::hid::mouse::get().expect("mouse should be initialized by now");
    while let Some(packet) = stream.next().await {
        mouse.add(packet).await;
    }
}
