Rust TRC Autotuner
==================

A Rust port of [Transient Response Auto Tuner](https://github.com/lnayman/transient-response-auto-tuner/), mostly for grins.

You can specify different devices in the `devices/` folder as toml files (see the example device). If you have rust installed, you can run it using, for example:

```
$ cargo build
$ ./target/debug/trc-tuner --bandwidth 20e3 --capacitance 100e-9 -d test -range 100u
```

You may select the device, the loop bandwidth, the capacitance of the device-under-test (DUT), and the current range.
