//! https://datasheets.maximintegrated.com/en/ds/MAX5532-MAX5535.pdf

use core::marker::PhantomData;

use embedded_hal::blocking::spi::Write;

pub struct Shutdown;
pub struct Standby;
pub struct Normal;

trait SendCommand {
  fn send_command(&mut self, command: u8, data_bits: u16); 
}

macro_rules! impl_max {
  ($max_ty:ident) => {
    pub struct $max_ty<W, MODE> {
      writer: W,
      _mode: PhantomData<MODE>,
    }
  }
}

impl_max!(Max5532);
impl_max!(Max5533);
impl_max!(Max5534);
impl_max!(Max5535);

// 12-bit mask
const DATA_BITS: u16 = 12;
const DATA_MASK: u16 = (1 << DATA_BITS) - 1;

impl<W, MODE> Max5532<W, MODE> 
where
  W: SendCommand
{
  pub fn new(writer: W) -> Max5532<W, Shutdown> {
    Max5532 { writer, _mode: PhantomData }
  }  

  /// Set input register A.
  #[inline]
  pub fn set_input_reg_a(&mut self, value: u16) {
    self.writer.send_command(0b0001, value & DATA_MASK)
  }

  /// Set input register B.
  #[inline]
  pub fn set_input_reg_b(&mut self, value: u16) {
    self.writer.send_command(0b0010, value & DATA_MASK)
  }
  
  /// Enter normal operation mode.
  pub fn into_normal(mut self, vref: Vref) -> Max5532<W, Normal> {
    self.writer_send_command(0b1101, vref.to_data_bits());
    Max5532 { writer: self.writer, _mode: PhantomData }
  }
  
  /// Enter shutdown mode.
  pub fn into_shutdown(mut self, vref: Vref) -> Max5532<W, Shutdown> {
    self.writer.send_command(0b1110, vref.to_data_bits());
    Max5532 { writer: self.writer, _mode: PhantomData }
  }
}

impl<W> Max5532<W, Normal> {
  /// Load DAC registers A and B from respective input registers 
  /// and update respective DAC outputs.
  #[inline]
  pub fn dac_ab(&mut self, value: u16) {
    self.writer.send_command(0b1000, value & DATA_MASK);
  }  

  /// Load input and DAC register A from shift register A and
  /// load DAC register B from input register B.
  pub fn input_a_dac_ab(&mut self, value: u16) {
    self.writer.send_command(0b1001, value & DATA_MASK);
  }

  /// Load input and DAC register B from shift register B and
  /// load DAC register A from input register A.
  pub fn input_b_dac_ab(&mut self, value: u16) {
    self.writer.send_command(0b1010, value & DATA_MASK);
  }
}

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

impl Vref {
  fn to_data_bits(self) -> u16 {
     self as u16 << 10
  }
}

macro_rules! impl_standby {
  (Max5533) => { impl_standby!(@inner Max5533); };
  (Max5535) => { impl_standby!(@inner Max5535); };
  ($max_ty:ident) => {};
  (@inner $max_ty:ident) => {
    pub fn into_standby(mut self, voltage: Vref) -> $max_ty<W, Standby> {
       self.writer.send_command(0b1100, vref.to_data_bits())
    }
  };
}

impl_standby_shutdown!(Max5532, Shutdown);

macro_rules! impl_standby_shutdown {
  ($max_ty:ident, $mode_ty:ident) => {
    impl<W> $max_ty<W, $mode_ty> 
    where
      W: SendCommand
    {
      /// Load DAC registers A and B from respective input registers 
      /// and update respective DAC outputs.
      #[inline]
      pub fn dac_ab(mut self, value: u16) -> Max5532<W, Normal> {
        let mut max_553x = Max5532 { writer: self.writer, _mode: PhantomData };
        max_553x.load_dac(value);
        max_553x
      }

      /// Load input and DAC register A from shift register A and
      /// load DAC register B from input register B.
      pub fn input_a_dac_ab(mut self, value: u16) -> Max5532<W, Normal> {
        let mut max_553x = Max5532 { writer: self.writer, _mode: PhantomData };
        max_553x.input_a_dac_ab(value);
        max_553x      
      }

      /// Load input and DAC register B from shift register A and
      /// load DAC register A from input register A.
      pub fn input_b_dac_ab(mut self, value: u16) -> Max5532<W, Normal> {
        let mut max_553x = Max5532 { writer: self.writer, _mode: PhantomData };
        max_553x.input_a_dac_ab(value);
        max_553x      
      }
    }
  }
}
