# Raybaby 

[![Build](https://github.com/ryanwebber/raybaby/actions/workflows/build.yml/badge.svg)](https://github.com/ryanwebber/raybaby/actions/workflows/build.yml)

A simple raytracing renderer using [wgpu](https://wgpu.rs) and [wgsl](https://www.w3.org/TR/WGSL/). 

## Usage

```bash
$ cargo run --release -- render --scene ./examples/01-spheres.ron --skybox-color "(0.01, 0.01, 0.01)"
```

![Raytraced scene](.github/screenshot.png?raw=true)
