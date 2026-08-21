#[doc = "The `nvme` module contains the implementation of the `Nvme` controller, its driver, and associated components."]
pub mod allocator;
#[doc = "The `bus` module contains the implementation of the PCI bus interface for `Nvme` devices."]
pub mod bus;
#[doc = "The `driver` module contains the implementation of the `Nvme` driver, which manages the interaction between the operating system and the `Nvme` controller."]
pub mod driver;
#[doc = "The `queue` module contains the implementation of the `Nvme` submission and completion queues, which are used for sending commands to and receiving responses from the `Nvme` controller."]
pub mod queue;
#[doc = "The `registers` module contains the implementation of the `Nvme` controller's registers, which are used to configure and control the `Nvme` device."]
pub mod registers;
