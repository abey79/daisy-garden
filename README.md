# daisy-garden

> _“Great patch to grow daisies, innit?”_ 🌼

WIP collection of firmware for the Electrosmith [patch.Init()](https://electro-smith.com/products/patch-init).

The first project will be a Moog DFAM companion: 8x clock multiplier/quantizer combo.

## TODO

- [ ] `daisy-garden` should be a lib and have examples
- [ ] move `dg-trait::*` to `daisy-garden`, and move the hw impl
- [ ] support tuple of GateOut
- [ ] add `linked_gates()-> (impl GateIn, impl GateOut)`
- [ ] add support for CVout with voltage support
- [ ] introduce `dg-fhx` and `impl GateOut`, etc. around it.
- [ ] `dg-noise` should really be `dg-sample-hold` and support V/oct
  