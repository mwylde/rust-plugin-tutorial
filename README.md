# Building a plugin system in Rust

<a href="https://www.arroyo.dev/blog/rust-plugin-systems"><img 
src="https://www.arroyo.dev/_next/image?url=%2Fposts%2Frust-plugins%2Fplugins-web.webp&w=1920&q=90"
alt="A crab plugging in a plug" /></a>

This repo is an example of a simple, dynamic plugin system in Rust, built around
dynamically loading shared libraries at runtime. This is a simplified 
version of the plugin system we built at [Arroyo](https://arroyo.dev),
to support dynamic user-defined functions (UDFs) for our
streaming SQL engine.

A full blog post explaining the background behind Rust plugins and
the code here, can be found on the [Arroyo blog](https://arroyo.dev/blog/rust-plugin-systems).

## Crate makeup

The repo is split into two crates:
* `plugin` -- an example Plugin that implements the Rust `String::repeat` function
* `host` -- the host application that loads plugins and passes CLI arguments to them

## Building and running

To build and run the example, you can use the following commands:

```shellsession
$ cd plugin && cargo build
$ cd ../host && cargo build
$ target/debug/host ../plugin/target/debug/libplugin.dylib cool 3
Loaded plugin repeat
Plugin returned: coolcoolcool
```

(note that the extension of the plugin library may vary depending on your platform; on Linux it's `.so`, on macOS it's
`.dylib`, and on Windows it's `.dll`).
