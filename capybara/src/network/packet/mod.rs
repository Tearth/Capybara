use anyhow::{bail, Result};
use std::{
    mem::{self, MaybeUninit},
    ptr, slice,
};

pub mod from;
pub mod into;

const PING_CID: u8 = 0x00;
const PONG_CID: u8 = 0x01;
const OBJECT_CID: u8 = 0x02;
const ARRAY_CID: u8 = 0x03;

#[derive(Debug, Clone)]
pub enum Packet {
    Ping { timestamp: u64 },
    Pong { timestamp: u64 },
    Object { oid: u16, data: Vec<u8> },
    Array { aid: u16, length: u32, data: Vec<u8> },
    Unknown,
}

impl Packet {
    pub fn from_object<T>(oid: u16, object: &T) -> Self
    where
        T: Clone,
    {
        unsafe {
            let size = mem::size_of::<T>();
            let data = slice::from_raw_parts((object as *const T) as *const u8, size);

            Packet::Object { oid, data: data.to_vec() }
        }
    }

    pub fn from_array<T>(aid: u16, array: &[T]) -> Self
    where
        T: Clone,
    {
        unsafe {
            let data = slice::from_raw_parts(array.as_ptr() as *const u8, mem::size_of_val(array));
            Packet::Array { aid, length: array.len() as u32, data: data.to_vec() }
        }
    }

    pub fn to_object<T>(&self) -> Result<T>
    where
        T: Clone,
    {
        unsafe {
            match self {
                Packet::Object { oid: _, data } => {
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
        unsafe {
            match self {
                Packet::Array { aid: _, length, data } => {
                    let size = mem::size_of::<T>();
                    let length = *length as usize;

                    if size * length != data.len() {
                        bail!("Size of array is incorrect");
                    }

                    let array = slice::from_raw_parts(data.as_ptr() as *const T, length);
                    Ok(array.to_vec())
                }
                _ => bail!("Packet is not a valid array"),
            }
        }
    }
}
