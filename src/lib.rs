//! Driver for MAX5532/MAX5533/MAX5534/MAX5535 DACs.
//!
//! Implemented according to <https://datasheets.maximintegrated.com/en/ds/MAX5532-MAX5535.pdf>.

use core::marker::PhantomData;

use embedded_hal::blocking::spi::Write;

/// Marks a DAC in normal operation mode.
pub enum Normal {}

/// Marks a DAC in standby mode.
pub enum Standby {}

/// Marks a DAC in shutdown mode.
pub enum Shutdown {}

pub trait SendCommand {
  type Error;

  fn send_command(&mut self, command: [u8; 2]) -> Result<(), Self::Error>;
}

impl<T> SendCommand for T
where
  T: Write<u8>
{
  type Error = T::Error;

  fn send_command(&mut self, command: [u8; 2]) -> Result<(), Self::Error> {
    self.write(&command)?;
    Ok(())
  }
}

const fn command_bytes(control_bits: u8, data_bits: u16) -> [u8; 2] {
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

macro_rules! impl_into_normal_shutdown {
  (Max5533) => { impl_into_normal_shutdown!(@with_vref Max5533); };
  (Max5535) => { impl_into_normal_shutdown!(@with_vref Max5535); };
  ($max_ty:ident) => {};
  (@without_vref $max_ty:ident) => {
    /// Enter normal operation mode.
    pub fn into_normal(mut self) -> Result<$max_ty<W, Normal>, W::Error> {
      self.writer.send_command(command_bytes(0b1101, 0))?;
      Ok($max_ty { writer: self.writer, _mode: PhantomData })
    }

    /// Enter shutdown mode.
    pub fn into_shutdown(mut self) -> Result<$max_ty<W, Shutdown>, W::Error> {
      self.writer.send_command(command_bytes(0b1110, 0))?;
      Ok($max_ty { writer: self.writer, _mode: PhantomData })
    }
  };
  (@with_vref $max_ty:ident) => {
    /// Enter normal operation mode and set internal voltage reference.
    pub fn into_normal(mut self, vref: Vref) -> Result<$max_ty<W, Normal>, W::Error> {
      self.writer.send_command(vref_command_bytes(0b1101, vref))?;
      Ok($max_ty { writer: self.writer, _mode: PhantomData })
    }

    /// Enter shutdown mode and set internal voltage reference.
    pub fn into_shutdown(mut self, vref: Vref) -> Result<$max_ty<W, Shutdown>, W::Error> {
      self.writer.send_command(vref_command_bytes(0b1110, vref))?;
      Ok($max_ty { writer: self.writer, _mode: PhantomData })
    }
  };
}


macro_rules! impl_max {
  ($(#[$outer:meta]),* $max_ty:ident) => {


    $(
      #[$outer]
    )*
    #[derive(Debug)]
    pub struct $max_ty<W, MODE> {
      writer: W,
      _mode: PhantomData<MODE>,
    }

    impl<W, MODE> $max_ty<W, MODE> {
      /// Create a new DAC with the given writer.
      pub fn new(writer: W) -> $max_ty<W, Shutdown> {
        $max_ty { writer, _mode: PhantomData }
      }

      /// Release the contained writer.
      pub fn release(self) -> W {
        self.writer
      }
    }

    impl<W, MODE> $max_ty<W, MODE>
    where
      W: SendCommand
    {
      /// Load input register A from shift register.
      #[inline]
      pub fn input_a(&mut self, value: u16) -> Result<(), W::Error> {
        self.writer.send_command(command_bytes(0b0001, value))
      }

      /// Load input register B from shift register.
      #[inline]
      pub fn input_b(&mut self, value: u16) -> Result<(), W::Error> {
        self.writer.send_command(command_bytes(0b0010, value))
      }

      impl_into_normal_shutdown!($max_ty);
    }
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

macro_rules! impl_standby {
  (Max5533) => { impl_standby!(@inner Max5533); };
  (Max5535) => { impl_standby!(@inner Max5535); };
  ($max_ty:ident) => {};
  (@inner $max_ty:ident) => {
    /// Enter standby mode and set internal voltage reference.
    pub fn into_standby(mut self, vref: Vref) -> Result<$max_ty<W, Standby>, W::Error> {
      self.writer.send_command(vref_command_bytes(0b1100, vref))?;
      Ok($max_ty { writer: self.writer, _mode: PhantomData })
    }
  };
}

macro_rules! impl_normal {
  ($max_ty:ident) => {
    impl<W> $max_ty<W, Normal>
    where
      W: SendCommand
    {
      /// Load DAC registers A and B from respective input registers
      /// and update respective DAC outputs.
      #[inline]
      pub fn dac_ab(&mut self, value: u16) -> Result<(), W::Error> {
        self.writer.send_command(command_bytes(0b1000, value))
      }

      /// Load input and DAC register A from shift register A and
      /// load DAC register B from input register B.
      pub fn input_a_dac_ab(&mut self, value: u16) -> Result<(), W::Error> {
        self.writer.send_command(command_bytes(0b1001, value))
      }

      /// Load input and DAC register B from shift register B and
      /// load DAC register A from input register A.
      pub fn input_b_dac_ab(&mut self, value: u16) -> Result<(), W::Error> {
        self.writer.send_command(command_bytes(0b1010, value))
      }

      impl_standby!($max_ty);
    }
  }
}

impl_normal!(Max5532);
impl_normal!(Max5533);
impl_normal!(Max5534);
impl_normal!(Max5535);

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
      W: SendCommand
    {
      /// Load DAC registers A and B from respective input registers, update
      /// respective DAC outputs and enter normal operation mode.
      #[inline]
      pub fn dac_ab(self, value: u16) -> Result<$max_ty<W, Normal>, W::Error> {
        let mut max_553x: $max_ty<W, Normal> = $max_ty { writer: self.writer, _mode: PhantomData };
        max_553x.dac_ab(value)?;
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
        max_553x.input_a_dac_ab(value)?;
        Ok(max_553x)
      }
    }
  }
}

impl_standby_shutdown!(Max5532);
impl_standby_shutdown!(Max5533);
impl_standby_shutdown!(Max5534);
impl_standby_shutdown!(Max5535);
