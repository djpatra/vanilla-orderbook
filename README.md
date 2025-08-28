# vanilla-orderbook

# Overview

A simple **price-time priority order book** implemented in Rust.  

---

## Features

- **Bid/Ask order book** supporting both buy and sell sides.
- **Price-time priority matching** (FIFO within each price level).
- Automatic **trade generation** when orders match.
- Tracks **best bid and best ask** prices.
- Preserves **remaining unmatched orders** in the book.
- Lightweight: implemented with `BTreeMap` + `VecDeque`.

