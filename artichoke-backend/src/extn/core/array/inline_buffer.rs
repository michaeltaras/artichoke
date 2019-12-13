use arrayvec::ArrayVec;

use crate::convert::Convert;
use crate::extn::core::array::ArrayType;
use crate::extn::core::exception::RubyException;
use crate::gc::MrbGarbageCollection;
use crate::sys;
use crate::value::Value;
use crate::Artichoke;

const INLINE_CAPACITY: usize = 8;

#[derive(Clone)]
pub enum InlineBuffer {
    Dynamic(Vec<sys::mrb_value>),
    Inline(ArrayVec<[sys::mrb_value; INLINE_CAPACITY]>),
}

impl Default for InlineBuffer {
    fn default() -> Self {
        Self::Inline(ArrayVec::new())
    }
}

impl From<Vec<sys::mrb_value>> for InlineBuffer {
    fn from(values: Vec<sys::mrb_value>) -> Self {
        if values.len() <= INLINE_CAPACITY {
            let mut inline = ArrayVec::new();
            inline.extend(values);
            Self::Inline(inline)
        } else {
            Self::Dynamic(values)
        }
    }
}

impl From<Vec<Value>> for InlineBuffer {
    fn from(values: Vec<Value>) -> Self {
        Self::from(values.as_slice())
    }
}

impl<'a> From<&'a [sys::mrb_value]> for InlineBuffer {
    fn from(values: &'a [sys::mrb_value]) -> Self {
        if values.len() <= INLINE_CAPACITY {
            let mut inline = ArrayVec::new();
            inline.extend(values.iter().copied());
            Self::Inline(inline)
        } else {
            Self::Dynamic(values.to_vec())
        }
    }
}

impl<'a> From<&'a [Value]> for InlineBuffer {
    fn from(values: &'a [Value]) -> Self {
        if values.len() <= INLINE_CAPACITY {
            let mut inline = ArrayVec::new();
            inline.extend(values.iter().map(Value::inner));
            Self::Inline(inline)
        } else {
            Self::Dynamic(values.iter().map(Value::inner).collect())
        }
    }
}

impl ArrayType for InlineBuffer {
    fn box_clone(&self) -> Box<dyn ArrayType> {
        Box::new(self.clone())
    }

    fn gc_mark(&self, interp: &Artichoke) {
        match self {
            Self::Dynamic(buffer) => {
                for element in buffer {
                    interp.mark_value(&Value::new(interp, *element));
                }
            }
            Self::Inline(buffer) => {
                for element in buffer {
                    interp.mark_value(&Value::new(interp, *element));
                }
            }
        }
    }

    fn real_children(&self) -> usize {
        match self {
            Self::Dynamic(buffer) => buffer.len(),
            Self::Inline(buffer) => buffer.len(),
        }
    }

    fn len(&self) -> usize {
        match self {
            Self::Dynamic(buffer) => buffer.len(),
            Self::Inline(buffer) => buffer.len(),
        }
    }

    fn is_empty(&self) -> bool {
        match self {
            Self::Dynamic(buffer) => buffer.is_empty(),
            Self::Inline(buffer) => buffer.is_empty(),
        }
    }

    fn get(&self, interp: &Artichoke, index: usize) -> Result<Value, Box<dyn RubyException>> {
        Self::get(self, interp, index)
    }

    fn slice(
        &self,
        interp: &Artichoke,
        start: usize,
        len: usize,
    ) -> Result<Box<dyn ArrayType>, Box<dyn RubyException>> {
        match Self::slice(self, interp, start, len) {
            Ok(slice) => Ok(Box::new(slice)),
            Err(err) => Err(err),
        }
    }

    fn set(
        &mut self,
        interp: &Artichoke,
        index: usize,
        elem: Value,
        realloc: &mut Option<Vec<Box<dyn ArrayType>>>,
    ) -> Result<(), Box<dyn RubyException>> {
        let _ = realloc;
        Self::set(self, interp, index, elem)
    }

    fn set_with_drain(
        &mut self,
        interp: &Artichoke,
        start: usize,
        drain: usize,
        with: Value,
        realloc: &mut Option<Vec<Box<dyn ArrayType>>>,
    ) -> Result<usize, Box<dyn RubyException>> {
        let _ = realloc;
        Self::set_with_drain(self, interp, start, drain, with)
    }

    fn set_slice(
        &mut self,
        interp: &Artichoke,
        start: usize,
        drain: usize,
        with: Box<dyn ArrayType>,
        realloc: &mut Option<Vec<Box<dyn ArrayType>>>,
    ) -> Result<usize, Box<dyn RubyException>> {
        let _ = realloc;
        if let Ok(buffer) = with.downcast_ref::<Self>() {
            Self::set_slice(self, interp, start, drain, buffer)
        } else {
            unimplemented!("Set slice on InlineBuffer with other ArrayType");
        }
    }

    fn concat(
        &mut self,
        interp: &Artichoke,
        other: Box<dyn ArrayType>,
        realloc: &mut Option<Vec<Box<dyn ArrayType>>>,
    ) -> Result<(), Box<dyn RubyException>> {
        let _ = realloc;
        if let Ok(buffer) = other.downcast_ref::<Self>() {
            Self::concat(self, interp, buffer)
        } else {
            unimplemented!("Set slice on InlineBuffer with other ArrayType");
        }
    }

    fn pop(
        &mut self,
        interp: &Artichoke,
        realloc: &mut Option<Vec<Box<dyn ArrayType>>>,
    ) -> Result<Value, Box<dyn RubyException>> {
        let _ = realloc;
        Self::pop(self, interp)
    }

    fn reverse(&mut self, interp: &Artichoke) -> Result<(), Box<dyn RubyException>> {
        Self::reverse(self, interp)
    }
}

impl InlineBuffer {
    pub fn with_capacity(capacity: usize) -> Self {
        if capacity <= INLINE_CAPACITY {
            Self::Inline(ArrayVec::new())
        } else {
            Self::Dynamic(Vec::with_capacity(capacity))
        }
    }

    pub fn as_vec(&self, interp: &Artichoke) -> Vec<Value> {
        match self {
            Self::Dynamic(buffer) => buffer
                .iter()
                .copied()
                .map(|value| Value::new(interp, value))
                .collect(),
            Self::Inline(buffer) => buffer
                .iter()
                .copied()
                .map(|value| Value::new(interp, value))
                .collect(),
        }
    }

    pub fn as_ptr(&self) -> *const sys::mrb_value {
        match self {
            Self::Dynamic(buffer) => buffer.as_ptr(),
            Self::Inline(buffer) => buffer.as_ptr(),
        }
    }

    pub fn as_mut_ptr(&mut self) -> *mut sys::mrb_value {
        match self {
            Self::Dynamic(buffer) => buffer.as_mut_ptr(),
            Self::Inline(buffer) => buffer.as_mut_ptr(),
        }
    }

    pub unsafe fn set_len(&mut self, len: usize) {
        match self {
            Self::Dynamic(buffer) => buffer.set_len(len),
            Self::Inline(buffer) => buffer.set_len(len),
        }
    }

    pub fn clear(&mut self) {
        match self {
            Self::Dynamic(buffer) => buffer.clear(),
            Self::Inline(buffer) => buffer.clear(),
        }
    }

    pub fn get(&self, interp: &Artichoke, index: usize) -> Result<Value, Box<dyn RubyException>> {
        let elem = match self {
            Self::Dynamic(buffer) => buffer.get(index),
            Self::Inline(buffer) => buffer.get(index),
        };
        let elem = elem.copied().map(|elem| Value::new(interp, elem));
        Ok(interp.convert(elem))
    }

    pub fn slice(
        &self,
        interp: &Artichoke,
        start: usize,
        len: usize,
    ) -> Result<Self, Box<dyn RubyException>> {
        let _ = interp;
        match self {
            Self::Dynamic(buffer) => {
                let iter = buffer.iter().skip(start).take(len);
                Ok(Self::from(iter.copied().collect::<Vec<_>>()))
            }
            Self::Inline(buffer) => {
                let iter = buffer.iter().skip(start).take(len);
                Ok(Self::from(iter.copied().collect::<Vec<_>>()))
            }
        }
    }

    pub fn set(
        &mut self,
        interp: &Artichoke,
        index: usize,
        elem: Value,
    ) -> Result<(), Box<dyn RubyException>> {
        let _ = interp;
        let buflen = self.len();
        match self {
            Self::Dynamic(ref mut buffer) => {
                if index < buflen {
                    buffer[index] = elem.inner();
                } else {
                    buffer.reserve(index + 1 - buflen);
                    let nil = unsafe { sys::mrb_sys_nil_value() };
                    for _ in buflen..index {
                        buffer.push(nil);
                    }
                    buffer.push(elem.inner());
                }
            }
            Self::Inline(ref mut buffer) => {
                if index < buflen {
                    buffer[index] = elem.inner();
                } else if index < buffer.capacity() {
                    let nil = unsafe { sys::mrb_sys_nil_value() };
                    for _ in buflen..index {
                        buffer.push(nil);
                    }
                    buffer.push(elem.inner());
                } else {
                    let nil = unsafe { sys::mrb_sys_nil_value() };
                    let mut dynamic = vec![nil; index + 1];
                    dynamic[0..buffer.len()].copy_from_slice(buffer.as_slice());
                    dynamic[index] = elem.inner();
                    *self = Self::Dynamic(dynamic);
                }
            }
        }
        Ok(())
    }

    pub fn set_with_drain(
        &mut self,
        interp: &Artichoke,
        start: usize,
        drain: usize,
        with: Value,
    ) -> Result<usize, Box<dyn RubyException>> {
        let _ = interp;
        let buflen = self.len();
        let drained = std::cmp::min(buflen.checked_sub(start).unwrap_or_default(), drain);
        if start > buflen {
            set_with_drain_sparse(self, start, with);
        } else if (buflen + 1).checked_sub(drain).unwrap_or_default() <= INLINE_CAPACITY {
            set_with_drain_to_inline(self, start, drain, with);
        } else {
            match self {
                Self::Dynamic(ref mut buffer) => {
                    buffer.push(with.inner());
                }
                Self::Inline(ref mut buffer) => {
                    let nil = unsafe { sys::mrb_sys_nil_value() };
                    let mut dynamic = vec![nil; start + 1];
                    dynamic[0..buffer.len()].copy_from_slice(buffer.as_slice());
                    dynamic[start] = with.inner();
                    *self = Self::Dynamic(dynamic);
                }
            }
        }
        Ok(drained)
    }

    pub fn set_slice(
        &mut self,
        interp: &Artichoke,
        start: usize,
        drain: usize,
        with: &Self,
    ) -> Result<usize, Box<dyn RubyException>> {
        let _ = interp;
        let buflen = self.len();
        let drained = std::cmp::min(buflen.checked_sub(start).unwrap_or_default(), drain);
        let newlen = start
            + buflen
                .checked_sub(start)
                .and_then(|tail| tail.checked_sub(drain))
                .unwrap_or_default()
            + with.len();
        if start > buflen {
            set_slice_with_drain_sparse(self, start, with);
        } else if newlen <= INLINE_CAPACITY {
            set_slice_with_drain_to_inline(self, start, drain, with);
        } else {
            set_slice_with_drain_to_dynamic(self, start, drain, with);
        }
        Ok(drained)
    }

    pub fn concat(
        &mut self,
        interp: &Artichoke,
        other: &Self,
    ) -> Result<(), Box<dyn RubyException>> {
        let _ = interp;
        if self.len() + other.len() <= INLINE_CAPACITY {
            concat_to_inline(self, other);
        } else {
            concat_to_dynamic(self, other);
        }
        Ok(())
    }

    pub fn pop(&mut self, interp: &Artichoke) -> Result<Value, Box<dyn RubyException>> {
        let value = match self {
            Self::Dynamic(buffer) => buffer.pop(),
            Self::Inline(buffer) => buffer.pop(),
        };
        Ok(interp.convert(value.map(|value| Value::new(interp, value))))
    }

    pub fn reverse(&mut self, interp: &Artichoke) -> Result<(), Box<dyn RubyException>> {
        let _ = interp;
        match self {
            Self::Dynamic(ref mut buffer) => {
                buffer.reverse();
            }
            Self::Inline(ref mut buffer) if buffer.is_empty() => {}
            Self::Inline(ref mut buffer) => {
                let mut left = 0;
                let mut right = buffer.len() - 1;
                while left < right {
                    buffer.swap(left, right);
                    left += 1;
                    right -= 1;
                }
            }
        }
        Ok(())
    }
}

fn set_with_drain_sparse(ary: &mut InlineBuffer, start: usize, elem: Value) {
    let nil = unsafe { sys::mrb_sys_nil_value() };
    let buflen = ary.len();
    if start < INLINE_CAPACITY {
        match ary {
            InlineBuffer::Dynamic(buffer) => {
                let mut inline = ArrayVec::new();
                inline.extend(buffer.as_slice().iter().copied());
                for _ in buflen..start {
                    inline.push(nil);
                }
                inline.push(elem.inner());
                *ary = InlineBuffer::Inline(inline);
            }
            InlineBuffer::Inline(ref mut buffer) => {
                for _ in buflen..start {
                    buffer.push(nil);
                }
                buffer.push(elem.inner());
            }
        }
    } else {
        match ary {
            InlineBuffer::Dynamic(ref mut buffer) => {
                buffer.reserve(start + 1 - buflen);
                for _ in buflen..start {
                    buffer.push(nil);
                }
                buffer.push(elem.inner());
            }
            InlineBuffer::Inline(buffer) => {
                let mut dynamic = vec![nil; start + 1];
                dynamic[0..buflen].copy_from_slice(buffer.as_slice());
                dynamic[start] = elem.inner();
                *ary = InlineBuffer::Dynamic(dynamic);
            }
        }
    }
}

fn set_with_drain_to_inline(ary: &mut InlineBuffer, start: usize, drain: usize, elem: Value) {
    match ary {
        InlineBuffer::Dynamic(ref mut buffer) => {
            let mut inline = ArrayVec::new();
            if start < buffer.len() {
                inline.extend(buffer.drain(0..start));
            } else {
                inline.extend(buffer.drain(..));
            }
            inline.push(elem.inner());
            inline.extend(buffer.drain(drain..));
            *ary = InlineBuffer::Inline(inline);
        }
        InlineBuffer::Inline(ref mut buffer) => {
            let mut inline = ArrayVec::new();
            if start < buffer.len() {
                inline.extend(buffer.drain(0..start));
            } else {
                inline.extend(buffer.drain(..));
            }
            inline.push(elem.inner());
            inline.extend(buffer.drain(drain..));
            *ary = InlineBuffer::Inline(inline);
        }
    }
}

fn set_slice_with_drain_sparse(ary: &mut InlineBuffer, start: usize, with: &InlineBuffer) {
    let buflen = ary.len();
    let nil = unsafe { sys::mrb_sys_nil_value() };
    match ary {
        InlineBuffer::Dynamic(ref mut buffer) => {
            for _ in buflen..start {
                buffer.push(nil);
            }
            match with {
                InlineBuffer::Dynamic(with) => {
                    buffer.extend_from_slice(with.as_slice());
                }
                InlineBuffer::Inline(with) => {
                    buffer.extend_from_slice(with.as_slice());
                }
            }
        }
        InlineBuffer::Inline(ref mut buffer) if start < INLINE_CAPACITY => {
            for _ in buflen..start {
                buffer.push(nil);
            }
            match with {
                InlineBuffer::Dynamic(with) => {
                    if buffer.try_extend_from_slice(with.as_slice()).is_err() {
                        let mut dynamic = buffer.as_slice().to_vec();
                        dynamic.extend_from_slice(with.as_slice());
                        *ary = InlineBuffer::Dynamic(dynamic);
                    }
                }
                InlineBuffer::Inline(with) => {
                    if buffer.try_extend_from_slice(with.as_slice()).is_err() {
                        let mut dynamic = buffer.as_slice().to_vec();
                        dynamic.extend_from_slice(with.as_slice());
                        *ary = InlineBuffer::Dynamic(dynamic);
                    }
                }
            }
        }
        InlineBuffer::Inline(ref buffer) => {
            let mut dynamic = buffer.as_slice().to_vec();
            for _ in buflen..start {
                dynamic.push(nil);
            }
            match with {
                InlineBuffer::Dynamic(with) => {
                    dynamic.extend_from_slice(with.as_slice());
                }
                InlineBuffer::Inline(with) => {
                    dynamic.extend_from_slice(with.as_slice());
                }
            }
            *ary = InlineBuffer::Dynamic(dynamic);
        }
    }
}

fn set_slice_with_drain_to_inline(
    ary: &mut InlineBuffer,
    start: usize,
    drain: usize,
    with: &InlineBuffer,
) {
    match ary {
        InlineBuffer::Dynamic(ref mut buffer) => {
            let mut inline = ArrayVec::new();
            if start < buffer.len() {
                inline.extend(buffer.drain(0..start));
            } else {
                inline.extend(buffer.drain(..));
            }
            if drain < buffer.len() {
                buffer.drain(0..drain);
            } else {
                buffer.clear();
            }
            match with {
                InlineBuffer::Dynamic(with) => {
                    if inline.try_extend_from_slice(with.as_slice()).is_err() {
                        let mut dynamic = inline.as_slice().to_vec();
                        dynamic.extend_from_slice(with.as_slice());
                        dynamic.append(buffer);
                        *ary = InlineBuffer::Dynamic(dynamic);
                    } else {
                        inline.extend(buffer.iter().copied());
                        *ary = InlineBuffer::Inline(inline);
                    }
                }
                InlineBuffer::Inline(with) => {
                    if inline.try_extend_from_slice(with.as_slice()).is_err() {
                        let mut dynamic = inline.as_slice().to_vec();
                        dynamic.extend_from_slice(with.as_slice());
                        dynamic.append(buffer);
                        *ary = InlineBuffer::Dynamic(dynamic);
                    } else {
                        inline.extend(buffer.iter().copied());
                        *ary = InlineBuffer::Inline(inline);
                    }
                }
            }
        }
        InlineBuffer::Inline(ref mut buffer) => {
            let mut inline = ArrayVec::new();
            if start < buffer.len() {
                inline.extend(buffer.drain(0..start));
            } else {
                inline.extend(buffer.drain(..));
            }
            if drain < buffer.len() {
                buffer.drain(0..drain);
            } else {
                buffer.clear();
            }
            match with {
                InlineBuffer::Dynamic(with) => {
                    if inline.try_extend_from_slice(with.as_slice()).is_err() {
                        let mut dynamic = inline.as_slice().to_vec();
                        dynamic.extend_from_slice(with.as_slice());
                        dynamic.extend_from_slice(buffer.as_slice());
                        *ary = InlineBuffer::Dynamic(dynamic);
                    } else {
                        inline.extend(buffer.iter().copied());
                        *ary = InlineBuffer::Inline(inline);
                    }
                }
                InlineBuffer::Inline(with) => {
                    if inline.try_extend_from_slice(with.as_slice()).is_err() {
                        let mut dynamic = inline.as_slice().to_vec();
                        dynamic.extend_from_slice(with.as_slice());
                        dynamic.extend_from_slice(buffer.as_slice());
                        *ary = InlineBuffer::Dynamic(dynamic);
                    } else {
                        inline.extend(buffer.iter().copied());
                        *ary = InlineBuffer::Inline(inline);
                    }
                }
            }
        }
    }
}

fn set_slice_with_drain_to_dynamic(
    ary: &mut InlineBuffer,
    start: usize,
    drain: usize,
    with: &InlineBuffer,
) {
    match ary {
        InlineBuffer::Dynamic(ref mut buffer) => match with {
            InlineBuffer::Dynamic(with) => {
                buffer.splice(start..start + drain, with.iter().copied());
            }
            InlineBuffer::Inline(with) => {
                buffer.splice(start..start + drain, with.as_slice().iter().copied());
            }
        },
        InlineBuffer::Inline(buffer) => {
            let mut dynamic = buffer.as_slice().to_vec();
            match with {
                InlineBuffer::Dynamic(with) => {
                    dynamic.splice(start..start + drain, with.iter().copied());
                }
                InlineBuffer::Inline(with) => {
                    dynamic.splice(start..start + drain, with.as_slice().iter().copied());
                }
            }
            *ary = InlineBuffer::Dynamic(dynamic);
        }
    }
}

fn concat_to_inline(ary: &mut InlineBuffer, other: &InlineBuffer) {
    let mut inline = ArrayVec::new();
    match ary {
        InlineBuffer::Dynamic(buffer) => inline.extend(buffer.as_slice().iter().copied()),
        InlineBuffer::Inline(buffer) => inline.extend(buffer.as_slice().iter().copied()),
    }
    match other {
        InlineBuffer::Dynamic(buffer) => inline.extend(buffer.as_slice().iter().copied()),
        InlineBuffer::Inline(buffer) => inline.extend(buffer.as_slice().iter().copied()),
    }
    *ary = InlineBuffer::Inline(inline);
}

fn concat_to_dynamic(ary: &mut InlineBuffer, other: &InlineBuffer) {
    let mut dynamic = Vec::with_capacity(ary.len() + other.len());
    match ary {
        InlineBuffer::Dynamic(buffer) => dynamic.extend_from_slice(buffer.as_slice()),
        InlineBuffer::Inline(buffer) => dynamic.extend_from_slice(buffer.as_slice()),
    }
    match other {
        InlineBuffer::Dynamic(buffer) => dynamic.extend_from_slice(buffer.as_slice()),
        InlineBuffer::Inline(buffer) => dynamic.extend_from_slice(buffer.as_slice()),
    }
    *ary = InlineBuffer::Dynamic(dynamic);
}
