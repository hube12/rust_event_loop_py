# Rust Event Loop pattern bindings

Example of using rust tokio threads in C/C++ and Python through the C ABI.

Here the client event loop will ping localhost:8000 every 2s just run `nc -l -p 8000 -k`

This is not a hard problem but if you do it incorrectly then you might get some surprise

This is a Minimum Viable Example to show how simple it gets but how not more simpler you can make it.

To install the python package run `pip install -r requirements-dev.txt && python setup.py install --user` (python2 and 3 supported)

To use the C header and library run `cargo build --release`
