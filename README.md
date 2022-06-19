# High-Level OpenCL
> **WARNING**\
> hlocl is still in an alpha stage. 

hlocl is a high-level OpenCL API for Rust

# Features
| Name  | Description                                                                             | Default |
| ----- | --------------------------------------------------------------------------------------- | ------- |
| def   | Enables default contexts and command queues                                             | Yes     |
| cl2   | Enables OpenCL 2.0 features                                                             | No      |
| async | Implements ```Future``` for OpenCL events and various other utils                       | No      |
| serde | Enables [```serde```](https://crates.io/crates/serde) support for OpenCL buffers        | No      |
| rand  | Enables OpenCL accelerated random number generation                                     | No      |
| error-stack | Enables rich errors via [```error-stack```](https://crates.io/crates/error-stack) | No      |