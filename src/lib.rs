//! Driver for MAX5532/MAX5533/MAX5534/MAX5535 DACs.
//!
//! Implemented according to <https://datasheets.maximintegrated.com/en/ds/MAX5532-MAX5535.pdf>.
#![no_std]

use core::marker::PhantomData;

use embedded_hal::blocking::spi::Write;

/// Marks a DAC in normal operation mode.
pub enum Normal {}

/// Marks a DAC in standby mode.
pub enum Standby {}

/// Marks a DAC in shutdown mode.
pub enum Shutdown {}

const fn command_bytes(control_bits: u8, mut data_bits: u16) -> [u8; 2] {
  if data_bits > 0b1111_1111_1111 {
    data_bits = 0b1111_1111_1111
  }

  [
    (control_bits << 4) | ((data_bits >> 8) as u8 & 0xf),
    data_bits as u8,
  ]
}

const fn vref_command_bytes(control_bits: u8, vref: Vref) -> [u8; 2] {
  [
    (control_bits << 4) | ((vref as u8) << 2),
    0
  ]
}

macro_rules! impl_into_mode {
  ($desc:expr, Max5532, Standby, $fn_name:ident, $control_bits:expr) => {
    // MAX5532 does not have standby mode.
  };
  ($desc:expr, Max5533, $mode_ty:ident, $fn_name:ident, $control_bits:expr) => {
    impl_into_mode!(@with_vref $desc, Max5533, $mode_ty, $fn_name, $control_bits);
  };
  ($desc:expr, Max5534, Standby, $fn_name:ident, $control_bits:expr) => {
    // MAX5534 does not have standby mode.
  };
  ($desc:expr, Max5535, $mode_ty:ident, $fn_name:ident, $control_bits:expr) => {
    impl_into_mode!(@with_vref $desc, Max5535, $mode_ty, $fn_name, $control_bits);
  };
  ($desc:expr, $max_ty:ident, $mode_ty:ident, $fn_name:ident, $control_bits:expr) => {
    /// Enter
    #[doc = $desc]
    /// mode.
    pub fn $fn_name(mut self) -> Result<$max_ty<W, $mode_ty>, W::Error> {
      self.writer.write(&command_bytes($control_bits, 0))?;
      Ok($max_ty { writer: self.writer, _mode: PhantomData })
    }
  };
  (@with_vref $desc:expr, $max_ty:ident, $mode_ty:ident, $fn_name:ident, $control_bits:expr) => {
    /// Enter
    #[doc = $desc]
    /// mode and set the internal voltage reference.
    pub fn $fn_name(mut self, vref: Vref) -> Result<$max_ty<W, $mode_ty>, W::Error> {
      self.writer.write(&vref_command_bytes($control_bits, vref))?;
      Ok($max_ty { writer: self.writer, _mode: PhantomData })
    }
  };
}

macro_rules! doc_imports {
  (Max5533) => {
    "max553x::{Max5533, Vref}"
  };
  (Max5535) => {
    "max553x::{Max5535, Vref}"
  };
  ($max_ty:ident) => {
    concat!("max553x::", stringify!($max_ty))
  };
}

macro_rules! doc_vref_value {
  (Max5533) => {
    0b0100
  };
  (Max5535) => {
    0b0100
  };
  ($max_ty:ident) => {
    0b0000
  };
}

macro_rules! doc_vref {
  (Max5533) => {
    "Vref::M1940"
  };
  (Max5535) => {
    "Vref::M1940"
  };
  ($max_ty:ident) => {
    ""
  };
}

macro_rules! impl_standby_shutdown {
  (Max5533) => {
    impl_standby_shutdown!(@inner Max5533, Standby);
    impl_standby_shutdown!(@inner Max5533, Shutdown);
  };
  (Max5535) => {
    impl_standby_shutdown!(@inner Max5535, Standby);
    impl_standby_shutdown!(@inner Max5535, Shutdown);
  };
  ($max_ty:ident) => {
    impl_standby_shutdown!(@inner $max_ty, Shutdown);
  };
  (@inner $max_ty:ident, $mode_ty:ident) => {
    impl<W> $max_ty<W, $mode_ty>
    where
      W: Write<u8>
    {
      /// Load DAC registers A and B from respective input registers, update
      /// respective DAC outputs and enter normal operation mode.
      #[inline]
      pub fn dac_ab(self) -> Result<$max_ty<W, Normal>, W::Error> {
        let mut max_553x: $max_ty<W, Normal> = $max_ty { writer: self.writer, _mode: PhantomData };
        max_553x.dac_ab()?;
        Ok(max_553x)
      }

      /// Load input and DAC register A from shift register A,
      /// load DAC register B from input register B
      /// and enter normal operation mode.
      pub fn input_a_dac_ab(self, value: u16) -> Result<$max_ty<W, Normal>, W::Error> {
        let mut max_553x: $max_ty<W, Normal> = $max_ty { writer: self.writer, _mode: PhantomData };
        max_553x.input_a_dac_ab(value)?;
        Ok(max_553x)
      }

      /// Load input and DAC register B from shift register A,
      /// load DAC register A from input register A
      /// and enter normal operation mode.
      pub fn input_b_dac_ab(self, value: u16) -> Result<$max_ty<W, Normal>, W::Error> {
        let mut max_553x: $max_ty<W, Normal> = $max_ty { writer: self.writer, _mode: PhantomData };
        max_553x.input_b_dac_ab(value)?;
        Ok(max_553x)
      }

      /// Load input registers A and B and DAC registers A and B from shift register
      /// and enter normal operation mode.
      pub fn input_ab_dac_ab(self, value: u16) -> Result<$max_ty<W, Normal>, W::Error> {
        let mut max_553x: $max_ty<W, Normal> = $max_ty { writer: self.writer, _mode: PhantomData };
        max_553x.input_ab_dac_ab(value)?;
        Ok(max_553x)
      }
    }
  }
}

macro_rules! impl_max {
  ($(#[$attr:meta]),* $max_ty:ident) => {
    $(
      #[$attr]
    )*
    /// # Usage
    ///
    /// ```rust
    /// # fn main() -> Result<(), embedded_hal_mock::MockError> {
    /// # use embedded_hal_mock::{spi::{Mock as SpiMock, Transaction as SpiTransaction}, delay::MockNoop as Delay};
    #[doc = concat!("use ", doc_imports!($max_ty), ";")]
    /// #
    /// # let spi = SpiMock::new(&[
    #[doc = concat!("#   SpiTransaction::write(vec![0b1101_0000 | ", doc_vref_value!($max_ty), ", 0b00000000]), // Into normal mode.")]
    /// #   SpiTransaction::write(vec![0b0001_0100, 0b11010010]), // Load input register A with value 1234.
    /// #   SpiTransaction::write(vec![0b0010_1111, 0b11111111]), // Load input register B with value 4095.
    /// #   SpiTransaction::write(vec![0b1000_0000, 0b00000000]), // Load DAC registers from input registers.
    /// #   SpiTransaction::write(vec![0b1001_0100, 0b11010010]), // Load input register A with value 1234 and load DAC register A.
    /// #   SpiTransaction::write(vec![0b1010_1111, 0b11111111]), // Load input register B with value 4095 and load DAC register B.
    #[doc = concat!("#   SpiTransaction::write(vec![0b1110_0000 | ", doc_vref_value!($max_ty), ", 0b00000000]), // Into shutdown mode.")]
    /// # ]);
    ///
    #[doc = concat!("let dac = ", stringify!($max_ty), "::new(spi);")]
    ///
    /// // Turn on.
    #[doc = concat!("let mut dac = dac.into_normal(", doc_vref!($max_ty), ")?;")]
    ///
    /// // The following two sequences have the same end result:
    /// // DAC register A set to 1234 and DAC register B set to 4095.
    ///
    /// // Set input register A.
    /// dac.input_a(1234)?;
    /// // Set input register B.
    /// dac.input_b(4095)?;
    /// // Load DAC registers from input registers.
    /// dac.dac_ab()?;
    ///
    /// // Set input register A and DAC register A and load DAC register B from input register B.
    /// dac.input_a_dac_ab(1234)?;
    /// // Set input register B and DAC register B and load DAC register A from input register A.
    /// dac.input_b_dac_ab(4095)?;
    ///
    /// // Turn off.
    #[doc = concat!("dac.into_shutdown(", doc_vref!($max_ty), ")?;")]
    /// #
    /// # Ok(())
    /// # }
    /// ```
    #[derive(Debug)]
    pub struct $max_ty<W, MODE> {
      writer: W,
      _mode: PhantomData<MODE>,
    }

    impl<W> $max_ty<W, Shutdown> {
      /// Create a new DAC with the given writer.
      pub const fn new(writer: W) -> $max_ty<W, Shutdown> {
        $max_ty { writer, _mode: PhantomData }
      }

      /// Release the contained writer.
      pub fn release(self) -> W {
        self.writer
      }
    }

    impl<W, MODE> $max_ty<W, MODE>
    where
      W: Write<u8>
    {
      /// Load input register A from shift register.
      #[inline]
      pub fn input_a(&mut self, value: u16) -> Result<(), W::Error> {
        self.writer.write(&command_bytes(0b0001, value))
      }

      /// Load input register B from shift register.
      #[inline]
      pub fn input_b(&mut self, value: u16) -> Result<(), W::Error> {
        self.writer.write(&command_bytes(0b0010, value))
      }

      impl_into_mode!("normal operation", $max_ty, Normal,   into_normal,   0b1101);
      impl_into_mode!("shutdown",         $max_ty, Shutdown, into_shutdown, 0b1110);
    }

    impl<W> $max_ty<W, Normal>
    where
      W: Write<u8>
    {
      /// Load DAC registers A and B from respective input registers
      /// and update respective DAC outputs.
      #[inline]
      pub fn dac_ab(&mut self) -> Result<(), W::Error> {
        self.writer.write(&command_bytes(0b1000, 0))
      }

      /// Load input and DAC register A from shift register A and
      /// load DAC register B from input register B.
      pub fn input_a_dac_ab(&mut self, value: u16) -> Result<(), W::Error> {
        self.writer.write(&command_bytes(0b1001, value))
      }

      /// Load input and DAC register B from shift register B and
      /// load DAC register A from input register A.
      pub fn input_b_dac_ab(&mut self, value: u16) -> Result<(), W::Error> {
        self.writer.write(&command_bytes(0b1010, value))
      }

      /// Load input registers A and B and DAC registers A and B from shift register.
      pub fn input_ab_dac_ab(&mut self, value: u16) -> Result<(), W::Error> {
        self.writer.write(&command_bytes(0b1111, value))
      }

      impl_into_mode!("standby", $max_ty, Standby, into_standby, 0b1100);
    }

    impl_standby_shutdown!($max_ty);
  }
}

impl_max!(
  /// Struct representing a MAX5532 DAC.
  Max5532
);
impl_max!(
  /// Struct representing a MAX5533 DAC.
  Max5533
);
impl_max!(
  /// Struct representing a MAX5534 DAC.
  Max5534
);
impl_max!(
  /// Struct representing a MAX5535 DAC.
  Max5535
);

/// Internal voltage reference for MAX5533/MAX5535.
#[derive(Debug, Clone, Copy)]
pub enum Vref {
  /// 1.214 V
  M1214 = 0b00,
  /// 1.940 V
  M1940 = 0b01,
  /// 2.425 V
  M2425 = 0b10,
  /// 3.885 V
  M3885 = 0b11,
}
