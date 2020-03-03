use rand::distributions::Alphanumeric;
use rand::{self, Rng, RngCore};
use std::convert::TryFrom;
use std::iter;
use uuid::Uuid;

use crate::extn::prelude::*;

pub mod mruby;
pub mod trampoline;

const DEFAULT_REQUESTED_BYTES: usize = 16;

#[derive(Debug, Clone, Copy)]
pub struct SecureRandom;

impl SecureRandom {
    pub fn random_bytes(interp: &mut Artichoke, len: Option<Value>) -> Result<Vec<u8>, Exception> {
        let len = if let Some(len) = len {
            match len.implicitly_convert_to_int()? {
                len if len < 0 => {
                    return Err(Exception::from(ArgumentError::new(
                        interp,
                        "negative string size (or size too big)",
                    )))
                }
                len if len == 0 => return Ok(Vec::new()),
                len => usize::try_from(len)
                    .map_err(|_| Fatal::new(interp, "positive Int must be usize"))?,
            }
        } else {
            DEFAULT_REQUESTED_BYTES
        };
        let mut rng = rand::thread_rng();
        let mut bytes = vec![0; len];
        rng.fill_bytes(&mut bytes);
        Ok(bytes)
    }

    pub fn random_number(interp: &mut Artichoke, max: Option<Value>) -> Result<Value, Exception> {
        #[derive(Debug, Clone, Copy)]
        enum Max {
            Float(Float),
            Int(Int),
            None,
        }
        let max = if let Some(max) = max {
            if let Ok(max) = max.clone().try_into::<Int>() {
                Max::Int(max)
            } else if let Ok(max) = max.clone().try_into::<Float>() {
                Max::Float(max)
            } else {
                let max = max.implicitly_convert_to_int().map_err(|_| {
                    let mut message = b"invalid argument - ".to_vec();
                    message.extend(max.inspect().as_slice());
                    ArgumentError::new_raw(interp, message)
                })?;
                Max::Int(max)
            }
        } else {
            Max::None
        };
        let mut rng = rand::thread_rng();
        match max {
            Max::Float(max) if max <= 0.0 => {
                let number = rng.gen_range(0.0, 1.0);
                Ok(interp.convert_mut(number))
            }
            Max::Float(max) => {
                let number = rng.gen_range(0.0, max);
                Ok(interp.convert_mut(number))
            }
            Max::Int(max) if max <= 0 => {
                let number = rng.gen_range(0.0, 1.0);
                Ok(interp.convert_mut(number))
            }
            Max::Int(max) => {
                let number = rng.gen_range(0, max);
                Ok(interp.convert(number))
            }
            Max::None => {
                let number = rng.gen_range(0.0, 1.0);
                Ok(interp.convert_mut(number))
            }
        }
    }

    pub fn hex(interp: &mut Artichoke, len: Option<Value>) -> Result<String, Exception> {
        Self::random_bytes(interp, len).map(hex::encode)
    }

    pub fn base64(interp: &mut Artichoke, len: Option<Value>) -> Result<String, Exception> {
        Self::random_bytes(interp, len).map(|bytes| base64::encode(bytes.as_slice()))
    }

    pub fn alphanumeric(interp: &mut Artichoke, len: Option<Value>) -> Result<String, Exception> {
        let len = if let Some(len) = len {
            match len.implicitly_convert_to_int()? {
                len if len < 0 => {
                    return Err(Exception::from(ArgumentError::new(
                        interp,
                        "negative string size (or size too big)",
                    )))
                }
                len if len == 0 => return Ok(String::new()),
                len => usize::try_from(len)
                    .map_err(|_| Fatal::new(interp, "positive Int must be usize"))?,
            }
        } else {
            DEFAULT_REQUESTED_BYTES
        };
        let mut rng = rand::thread_rng();
        let string = iter::repeat(())
            .map(|_| rng.sample(Alphanumeric))
            .take(len)
            .collect::<String>();
        Ok(string)
    }

    pub fn uuid(interp: &mut Artichoke) -> String {
        let _ = interp;
        let uuid = Uuid::new_v4();
        let mut buf = Uuid::encode_buffer();
        let enc = uuid.to_hyphenated().encode_lower(&mut buf);
        enc.to_owned()
    }
}