# nominatim-rs
A reverse geocoding API client powered by openstreetmap in Rust.
```
nominatim = "0.1.0"
```

## Examples

You can search for something and return it's address:
```rust
let e = Nominatim::search("statue of liberty").await.unwrap();

println!("{}", e.location);
```
```
Statue of Liberty, Flagpole Plaza, Manhattan Community Board 1, Manhattan, New York County, New York, 10004, United States
```

You can also search for coordinates and return it's address:
```rust
let e = Nominatim::reverse(41.87873, -87.63558).await.unwrap();

println!("{}", e.location);
```
```
Willis Tower, 233, South Wacker Drive, Printer's Row, Loop, Chicago, Cook County, Illinois, 60606, United States
```

You can also search for openstreetmap IDs:
```rust
let e = Nominatim::lookup("W39240305").await.unwrap();

println!("{}", e.location);
```
```
Legacy Parkway Trail, Woods Cross, Davis County, Utah, 84087, United States
```

## License
```
MIT License

Copyright (c) 2021 Grant Handy

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
```