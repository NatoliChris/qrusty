# QRusty the qrab

Small utility to read QR codes from the screen.

_not production ready - not sure it ever will be?_


## Motivation

Watching videos and interacting with content online, wanted a simple QR scanner
that had low overhead.

Tested/Working on Linux, will need to support other OSes in future.


## Dependencies

Uses a few libraries to make things easier:

- `rqrr` to read QR codes: [https://github.com/WanzenBug/rqrr](https://github.com/WanzenBug/rqrr)
- `image` for image processing: [https://docs.rs/image/latest/image/](https://docs.rs/image/latest/image/)
- `xcap` for monitor screenshots, etc: [https://github.com/nashaofu/xcap](https://github.com/nashaofu/xcap)


## TODOs

- [ ] provide a visual bounding box when users use selection mode
- [ ] intercept mouse click / drag when users make selection
- [ ] make it possible to screenshot over multiple monitors with drawn box
- [ ] Improve compatibility and decoding of more QRs/binary - library only does very few variations of QRs
- [ ] Copy the QR code directly into clipboard.

