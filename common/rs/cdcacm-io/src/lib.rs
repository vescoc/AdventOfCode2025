#![no_std]

pub fn split<'d, D: embassy_usb::driver::Driver<'d>>(class: embassy_usb::class::cdc_acm::CdcAcmClass<'d, D>) -> (impl embedded_io_async::Write, impl embedded_io_async::Read) {
    let (sender, receiver) = class.split();
    (Tx(sender), Rx(receiver))
}

#[derive(Debug)]
pub enum Error {
    Other
}

impl embedded_io_async::Error for Error {
    fn kind(&self) -> embedded_io_async::ErrorKind {
        embedded_io_async::ErrorKind::Other
    }
}

struct Rx<'d, D: embassy_usb::driver::Driver<'d>>(embassy_usb::class::cdc_acm::Receiver<'d, D>);

impl<'d, D: embassy_usb::driver::Driver<'d>> embedded_io_async::ErrorType for Rx<'d, D> {
    type Error = Error;
}

impl<'d, D: embassy_usb::driver::Driver<'d>> embedded_io_async::Read for Rx<'d, D> {
    async fn read(&mut self, buffer: &mut [u8]) -> Result<usize, Self::Error> {
        let len = buffer.len().min(self.0.max_packet_size() as usize);
        loop {
            match self.0.read_packet(&mut buffer[..len]).await {
                Ok(size) => return Ok(size),
                Err(embassy_usb::driver::EndpointError::Disabled) => {
                    self.0.wait_connection().await;
                }
                Err(embassy_usb::driver::EndpointError::BufferOverflow) => return Err(Error::Other),
            }
        }
    }
}

struct Tx<'d, D: embassy_usb::driver::Driver<'d>>(embassy_usb::class::cdc_acm::Sender<'d, D>);

impl<'d, D: embassy_usb::driver::Driver<'d>> embedded_io_async::ErrorType for Tx<'d, D> {
    type Error = Error;
}

impl<'d, D: embassy_usb::driver::Driver<'d>> embedded_io_async::Write for Tx<'d, D> {
    async fn write(&mut self, buffer: &[u8]) -> Result<usize, Self::Error> {
        let len = buffer.len().min(self.0.max_packet_size() as usize);
        loop {
            match self.0.write_packet(&buffer[..len]).await {
                Ok(()) => return Ok(len),
                Err(embassy_usb::driver::EndpointError::Disabled) => {
                    self.0.wait_connection().await;
                }
                Err(embassy_usb::driver::EndpointError::BufferOverflow) => return Err(Error::Other),
            }
        }
    }
}
