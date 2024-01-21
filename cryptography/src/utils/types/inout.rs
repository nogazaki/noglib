//! Type representing input and output, which can be either two different buffer, or the same mutable one

use core::marker::PhantomData;

/// Type representing input and output, which can be either two different buffer, or the same mutable one
#[derive(Debug)]
pub struct InOut<'input, 'output, T> {
    /// Pointer to input data
    in_ptr: *const T,
    /// Pointer to output data
    out_ptr: *mut T,
    /// Enforce lifetime of input and outputs
    _pd: PhantomData<(&'input T, &'output mut T)>,
}

impl<'input, 'output, T> From<(&'input T, &'output mut T)> for InOut<'input, 'output, T> {
    fn from((in_ref, out_ref): (&'input T, &'output mut T)) -> Self {
        InOut {
            in_ptr: in_ref as *const T,
            out_ptr: out_ref as *mut T,
            _pd: PhantomData,
        }
    }
}

impl<'output, T> From<&'output mut T> for InOut<'output, 'output, T> {
    fn from(inout_ref: &'output mut T) -> Self {
        InOut {
            in_ptr: inout_ref as *const T,
            out_ptr: inout_ref as *mut T,
            _pd: PhantomData,
        }
    }
}

impl<T> InOut<'_, '_, T> {
    /// Get the input reference
    pub const fn get_in(&self) -> &T {
        unsafe { &*self.in_ptr }
    }
    /// Get the output mutable reference
    pub fn get_out(&mut self) -> &mut T {
        unsafe { &mut *self.out_ptr }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_inout_ref() {
        let data = [128; 10];
        let mut mut_data = [0_u8; 10];

        let mut inout: InOut<_> = (&data, &mut mut_data).into();
        *inout.get_out() = *inout.get_in();
        assert_eq!(mut_data, data);

        mut_data.fill(0);
        let mut same_inout: InOut<_> = (&mut mut_data).into();
        *same_inout.get_out() = data;
        assert_eq!(mut_data, data);
    }
}
