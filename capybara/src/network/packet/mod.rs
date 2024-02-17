use anyhow::bail;
use anyhow::Result;
use std::mem;
use std::mem::MaybeUninit;
use std::ptr;
use std::slice;

pub mod from;
pub mod into;

const PING_CID: u8 = 0x00;
const PONG_CID: u8 = 0x01;
const OBJECT_CID: u8 = 0x02;
const ARRAY_CID: u8 = 0x03;

#[derive(Clone, Debug, PartialEq)]
pub enum Packet {
    Ping { timestamp: u64 },
    Pong { timestamp: u64 },
    Object { id: u16, data: Vec<u8> },
    Array { id: u16, data: Vec<u8> },
    Unknown,
}

pub struct PacketEmptyHeader;

impl Packet {
    pub fn get_id(&self) -> Option<u16> {
        match &self {
            Packet::Object { id, data: _ } => Some(*id),
            Packet::Array { id, data: _ } => Some(*id),
            _ => None,
        }
    }

    pub fn from_object<T>(id: u16, object: &T) -> Self
    where
        T: Clone,
    {
        unsafe {
            let data = slice::from_raw_parts((object as *const T) as *const u8, mem::size_of::<T>());
            Packet::Object { id, data: data.to_vec() }
        }
    }

    pub fn from_array<T>(id: u16, array: &[T]) -> Self
    where
        T: Clone,
    {
        Self::from_array_with_header::<PacketEmptyHeader, T>(id, &PacketEmptyHeader {}, array)
    }

    pub fn from_array_with_header<H, T>(id: u16, header: &H, array: &[T]) -> Self
    where
        T: Clone,
    {
        unsafe {
            let header_data = slice::from_raw_parts((header as *const H) as *const u8, mem::size_of::<H>());
            let array_data = slice::from_raw_parts(array.as_ptr() as *const u8, mem::size_of_val(array));

            let mut data = Vec::default();
            data.extend_from_slice(header_data);
            data.extend_from_slice(array_data);

            Packet::Array { id, data: data.to_vec() }
        }
    }

    pub fn to_object<T>(&self) -> Result<T>
    where
        T: Clone,
    {
        unsafe {
            match self {
                Packet::Object { id: _, data } => {
                    let size = mem::size_of::<T>();
                    if size != data.len() {
                        bail!("Size of object is incorrect");
                    }

                    let mut object = MaybeUninit::<T>::uninit();
                    let ptr = object.as_mut_ptr() as *mut u8;
                    ptr::copy_nonoverlapping(data.as_ptr(), ptr, size);

                    Ok(object.assume_init())
                }
                _ => bail!("Packet is not a valid object"),
            }
        }
    }

    pub fn to_array<T>(&self) -> Result<Vec<T>>
    where
        T: Clone,
    {
        self.to_array_with_header::<PacketEmptyHeader, T>().map(|p| p.1)
    }

    pub fn to_array_with_header<H, T>(&self) -> Result<(H, Vec<T>)>
    where
        T: Clone,
    {
        unsafe {
            match self {
                Packet::Array { id: _, data } => {
                    let header_size = mem::size_of::<H>();
                    let item_size = mem::size_of::<T>();

                    if header_size > data.len() {
                        bail!("Size of array is incorrect");
                    }

                    let array_size = data.len() - header_size;
                    let array_length = array_size / item_size;

                    if item_size * array_length != array_size {
                        bail!("Size of array is incorrect");
                    }

                    let mut header = MaybeUninit::<H>::uninit();
                    let ptr = header.as_mut_ptr() as *mut u8;
                    ptr::copy_nonoverlapping(data.as_ptr(), ptr, header_size);

                    let data_array_slice = &data[header_size..];
                    let array = slice::from_raw_parts(data_array_slice.as_ptr() as *const T, array_length);

                    Ok((header.assume_init(), array.to_vec()))
                }
                _ => bail!("Packet is not a valid array"),
            }
        }
    }
}
