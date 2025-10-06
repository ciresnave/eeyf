# EEYF FFI Integration Guide

## Overview

EEYF (Expeditiously Ergonomic Yahoo Finance) is a high-performance Yahoo Finance API client written in Rust. This guide explains how to create language bindings for EEYF using Foreign Function Interface (FFI).

## Architecture

The EEYF library follows a clean FFI architecture:

```
┌─────────────────────────────────────┐
│    Language-Specific Bindings       │
│   (Python, Node.js, Go, Ruby, etc)  │
├─────────────────────────────────────┤
│         FFI C Interface             │
│    (Rust extern "C" functions)      │
├─────────────────────────────────────┤
│        EEYF Core Library            │
│     (Pure Rust Implementation)      │
└─────────────────────────────────────┘
```

## Why Separate Repositories?

Language bindings should be maintained in **separate repositories** from the main EEYF crate:

### Benefits

1. **Language Ecosystem Integration**
   - Publish to language-specific package managers (PyPI, npm, crates.io, etc.)
   - Follow language-specific conventions and best practices
   - Independent versioning aligned with language community

2. **Development Independence**
   - Separate CI/CD pipelines for each language
   - Language-specific testing frameworks
   - Independent release cycles
   - Focused issue tracking per language

3. **Community Contributions**
   - Language experts can maintain bindings
   - Lower barrier to contribution (no Rust knowledge required for binding improvements)
   - Language-specific documentation and examples

4. **Build Simplification**
   - Each binding has its own build system
   - No cross-language build dependencies in main repo
   - Cleaner dependency management

### Recommended Repository Structure

```
Main Repository:
  github.com/yourusername/eeyf              (Rust core + FFI layer)

Binding Repositories:
  github.com/yourusername/eeyf-python       (PyPI: eeyf)
  github.com/yourusername/eeyf-node         (npm: @eeyf/client)
  github.com/yourusername/eeyf-go           (Go modules: github.com/.../eeyf-go)
  github.com/yourusername/eeyf-ruby         (RubyGems: eeyf)
  github.com/yourusername/eeyf-java         (Maven: com.eeyf:eeyf)
```

## FFI Layer Design

### Core Principles

1. **C ABI Compatibility** - Use C-compatible types for cross-language support
2. **Manual Memory Management** - Caller is responsible for freeing allocated memory
3. **Error Handling** - Return error codes, use output parameters for results
4. **Opaque Pointers** - Hide Rust internals behind void pointers
5. **Thread Safety** - Document thread-safety guarantees

### Required FFI Functions

To create a complete EEYF binding, implement these FFI categories:

#### 1. Client Lifecycle

```rust
// Create a new client
#[no_mangle]
pub extern "C" fn eeyf_client_new(
    cache_ttl: u64,
    max_retries: u32,
    timeout_seconds: u64
) -> *mut c_void;

// Free a client
#[no_mangle]
pub extern "C" fn eeyf_client_free(client: *mut c_void);
```

#### 2. Quote Operations

```rust
// Get a single quote
#[no_mangle]
pub extern "C" fn eeyf_get_quote(
    client: *mut c_void,
    symbol: *const c_char,
    out_quote: *mut FFIQuote,
    out_error: *mut FFIError
) -> i32;

// Get multiple quotes
#[no_mangle]
pub extern "C" fn eeyf_get_quotes(
    client: *mut c_void,
    symbols: *const *const c_char,
    symbols_len: usize,
    out_quotes: *mut *mut FFIQuote,
    out_len: *mut usize,
    out_error: *mut FFIError
) -> i32;
```

#### 3. Historical Data

```rust
// Get historical data
#[no_mangle]
pub extern "C" fn eeyf_get_historical(
    client: *mut c_void,
    symbol: *const c_char,
    start_timestamp: i64,
    end_timestamp: i64,
    interval: *const c_char,
    out_data: *mut *mut FFIHistoricalPoint,
    out_len: *mut usize,
    out_error: *mut FFIError
) -> i32;
```

#### 4. Memory Management

```rust
// Free a quote
#[no_mangle]
pub extern "C" fn eeyf_quote_free(quote: *mut FFIQuote);

// Free an array of quotes
#[no_mangle]
pub extern "C" fn eeyf_quotes_free(
    quotes: *mut FFIQuote,
    len: usize
);

// Free historical data
#[no_mangle]
pub extern "C" fn eeyf_historical_free(
    data: *mut FFIHistoricalPoint,
    len: usize
);

// Free error
#[no_mangle]
pub extern "C" fn eeyf_error_free(error: *mut FFIError);

// Free a C string returned by the library
#[no_mangle]
pub extern "C" fn eeyf_string_free(s: *mut c_char);
```

#### 5. Server Operations (Optional)

```rust
// Create HTTP server
#[no_mangle]
pub extern "C" fn eeyf_server_new(
    client: *mut c_void,
    host: *const c_char,
    port: u16
) -> *mut c_void;

// Add route to server
#[no_mangle]
pub extern "C" fn eeyf_server_add_route(
    server: *mut c_void,
    path: *const c_char,
    callback: extern "C" fn(*const c_char) -> *mut c_char
) -> i32;

// Start server (blocking)
#[no_mangle]
pub extern "C" fn eeyf_server_run(
    server: *mut c_void,
    out_error: *mut FFIError
) -> i32;

// Free server
#[no_mangle]
pub extern "C" fn eeyf_server_free(server: *mut c_void);
```

### FFI Data Types

Define C-compatible structures:

```rust
#[repr(C)]
pub struct FFIQuote {
    pub symbol: *mut c_char,
    pub price: f64,
    pub change: f64,
    pub change_percent: f64,
    pub volume: u64,
    pub market_cap: u64,
    pub timestamp: i64,
}

#[repr(C)]
pub struct FFIHistoricalPoint {
    pub timestamp: i64,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: u64,
    pub adjusted_close: f64,
}

#[repr(C)]
pub struct FFIError {
    pub code: i32,
    pub message: *mut c_char,
}

// Error codes
pub const EEYF_SUCCESS: i32 = 0;
pub const EEYF_ERROR_NULL_POINTER: i32 = -1;
pub const EEYF_ERROR_INVALID_SYMBOL: i32 = -2;
pub const EEYF_ERROR_NETWORK: i32 = -3;
pub const EEYF_ERROR_PARSE: i32 = -4;
pub const EEYF_ERROR_RATE_LIMIT: i32 = -5;
pub const EEYF_ERROR_UNKNOWN: i32 = -100;
```

## Language-Specific Binding Patterns

### Python (with ctypes or cffi)

**Repository**: Create `eeyf-python` repository

**Structure**:
```
eeyf-python/
├── eeyf/
│   ├── __init__.py
│   ├── client.py          # Client wrapper
│   ├── quote.py           # Quote models
│   ├── server.py          # Server wrapper (optional)
│   └── _ffi.py            # FFI bindings
├── tests/
├── setup.py
├── pyproject.toml
└── README.md
```

**Example Implementation** (`eeyf/client.py`):

```python
from ctypes import CDLL, c_void_p, c_char_p, c_uint64, c_uint32, POINTER, Structure
import platform
import os

# Load the shared library
def _load_library():
    lib_name = {
        'Linux': 'libeeyf.so',
        'Darwin': 'libeeyf.dylib',
        'Windows': 'eeyf.dll'
    }[platform.system()]
    
    lib_path = os.path.join(os.path.dirname(__file__), 'lib', lib_name)
    return CDLL(lib_path)

_lib = _load_library()

# Define structures
class FFIQuote(Structure):
    _fields_ = [
        ('symbol', c_char_p),
        ('price', c_double),
        ('change', c_double),
        ('change_percent', c_double),
        ('volume', c_uint64),
        ('market_cap', c_uint64),
        ('timestamp', c_int64),
    ]

class FFIError(Structure):
    _fields_ = [
        ('code', c_int32),
        ('message', c_char_p),
    ]

# Configure function signatures
_lib.eeyf_client_new.argtypes = [c_uint64, c_uint32, c_uint64]
_lib.eeyf_client_new.restype = c_void_p

_lib.eeyf_client_free.argtypes = [c_void_p]
_lib.eeyf_client_free.restype = None

_lib.eeyf_get_quote.argtypes = [
    c_void_p, 
    c_char_p, 
    POINTER(FFIQuote),
    POINTER(FFIError)
]
_lib.eeyf_get_quote.restype = c_int32

# Python wrapper
class EEYFClient:
    def __init__(self, cache_ttl=300, max_retries=3, timeout=30):
        self._handle = _lib.eeyf_client_new(cache_ttl, max_retries, timeout)
        if not self._handle:
            raise RuntimeError("Failed to create EEYF client")
    
    def __del__(self):
        if hasattr(self, '_handle') and self._handle:
            _lib.eeyf_client_free(self._handle)
    
    def get_quote(self, symbol):
        ffi_quote = FFIQuote()
        ffi_error = FFIError()
        
        result = _lib.eeyf_get_quote(
            self._handle,
            symbol.encode('utf-8'),
            ctypes.byref(ffi_quote),
            ctypes.byref(ffi_error)
        )
        
        if result != 0:
            error_msg = ffi_error.message.decode('utf-8')
            _lib.eeyf_error_free(ctypes.byref(ffi_error))
            raise RuntimeError(f"Failed to get quote: {error_msg}")
        
        # Convert to Python object
        quote = {
            'symbol': ffi_quote.symbol.decode('utf-8'),
            'price': ffi_quote.price,
            'change': ffi_quote.change,
            'change_percent': ffi_quote.change_percent,
            'volume': ffi_quote.volume,
            'market_cap': ffi_quote.market_cap,
            'timestamp': ffi_quote.timestamp,
        }
        
        _lib.eeyf_quote_free(ctypes.byref(ffi_quote))
        return quote
```

**Publishing to PyPI**:
```bash
# In eeyf-python repo
python -m build
python -m twine upload dist/*
```

### Node.js (with napi-rs or node-ffi-napi)

**Repository**: Create `eeyf-node` repository

**Structure**:
```
eeyf-node/
├── src/
│   ├── index.ts           # Main exports
│   ├── client.ts          # Client wrapper
│   ├── types.ts           # TypeScript types
│   └── ffi.ts             # FFI bindings
├── lib/                   # Compiled shared library
├── test/
├── package.json
├── tsconfig.json
└── README.md
```

**Example Implementation** (`src/client.ts`):

```typescript
import ffi from 'ffi-napi';
import ref from 'ref-napi';
import path from 'path';

// Define types
const FFIQuote = ref.types.void;
const FFIError = ref.types.void;
const ClientHandle = ref.refType(ref.types.void);

// Load library
const libPath = path.join(__dirname, '..', 'lib', process.platform === 'win32' ? 'eeyf.dll' : 'libeeyf.so');

const lib = ffi.Library(libPath, {
  'eeyf_client_new': ['pointer', ['uint64', 'uint32', 'uint64']],
  'eeyf_client_free': ['void', ['pointer']],
  'eeyf_get_quote': ['int32', ['pointer', 'string', 'pointer', 'pointer']],
  'eeyf_quote_free': ['void', ['pointer']],
  'eeyf_error_free': ['void', ['pointer']],
});

export interface Quote {
  symbol: string;
  price: number;
  change: number;
  changePercent: number;
  volume: number;
  marketCap: number;
  timestamp: number;
}

export class EEYFClient {
  private handle: Buffer;

  constructor(cacheTtl: number = 300, maxRetries: number = 3, timeout: number = 30) {
    this.handle = lib.eeyf_client_new(cacheTtl, maxRetries, timeout);
    if (this.handle.isNull()) {
      throw new Error('Failed to create EEYF client');
    }
  }

  getQuote(symbol: string): Quote {
    const quotePtr = ref.alloc(FFIQuote);
    const errorPtr = ref.alloc(FFIError);

    const result = lib.eeyf_get_quote(this.handle, symbol, quotePtr, errorPtr);

    if (result !== 0) {
      const errorMsg = ref.readCString(errorPtr);
      lib.eeyf_error_free(errorPtr);
      throw new Error(`Failed to get quote: ${errorMsg}`);
    }

    // Parse the quote structure
    const quote = this.parseQuote(quotePtr);
    lib.eeyf_quote_free(quotePtr);

    return quote;
  }

  private parseQuote(ptr: Buffer): Quote {
    // Implementation to parse FFI structure to JS object
    // This would involve reading memory at specific offsets
    throw new Error('Not implemented');
  }

  dispose(): void {
    if (this.handle && !this.handle.isNull()) {
      lib.eeyf_client_free(this.handle);
    }
  }
}
```

**Publishing to npm**:
```bash
# In eeyf-node repo
npm run build
npm publish
```

### Go (with cgo)

**Repository**: Create `eeyf-go` repository

**Structure**:
```
eeyf-go/
├── eeyf/
│   ├── client.go          # Client wrapper
│   ├── types.go           # Go types
│   ├── ffi.go             # CGO bindings
│   └── server.go          # Server wrapper
├── examples/
├── go.mod
├── go.sum
└── README.md
```

**Example Implementation** (`eeyf/client.go`):

```go
package eeyf

/*
#cgo LDFLAGS: -L./lib -leeyf
#include <stdlib.h>

// Forward declarations
void* eeyf_client_new(unsigned long long cache_ttl, unsigned int max_retries, unsigned long long timeout);
void eeyf_client_free(void* client);
int eeyf_get_quote(void* client, const char* symbol, void* out_quote, void* out_error);
void eeyf_quote_free(void* quote);
void eeyf_error_free(void* error);

// FFI structures
typedef struct {
    char* symbol;
    double price;
    double change;
    double change_percent;
    unsigned long long volume;
    unsigned long long market_cap;
    long long timestamp;
} FFIQuote;

typedef struct {
    int code;
    char* message;
} FFIError;
*/
import "C"
import (
    "fmt"
    "unsafe"
)

type Quote struct {
    Symbol        string
    Price         float64
    Change        float64
    ChangePercent float64
    Volume        uint64
    MarketCap     uint64
    Timestamp     int64
}

type Client struct {
    handle unsafe.Pointer
}

func NewClient(cacheTTL uint64, maxRetries uint32, timeout uint64) (*Client, error) {
    handle := C.eeyf_client_new(C.ulonglong(cacheTTL), C.uint(maxRetries), C.ulonglong(timeout))
    if handle == nil {
        return nil, fmt.Errorf("failed to create EEYF client")
    }
    return &Client{handle: handle}, nil
}

func (c *Client) Close() {
    if c.handle != nil {
        C.eeyf_client_free(c.handle)
        c.handle = nil
    }
}

func (c *Client) GetQuote(symbol string) (*Quote, error) {
    cSymbol := C.CString(symbol)
    defer C.free(unsafe.Pointer(cSymbol))

    var ffiQuote C.FFIQuote
    var ffiError C.FFIError

    result := C.eeyf_get_quote(
        c.handle,
        cSymbol,
        unsafe.Pointer(&ffiQuote),
        unsafe.Pointer(&ffiError),
    )

    if result != 0 {
        errorMsg := C.GoString(ffiError.message)
        C.eeyf_error_free(unsafe.Pointer(&ffiError))
        return nil, fmt.Errorf("failed to get quote: %s", errorMsg)
    }

    quote := &Quote{
        Symbol:        C.GoString(ffiQuote.symbol),
        Price:         float64(ffiQuote.price),
        Change:        float64(ffiQuote.change),
        ChangePercent: float64(ffiQuote.change_percent),
        Volume:        uint64(ffiQuote.volume),
        MarketCap:     uint64(ffiQuote.market_cap),
        Timestamp:     int64(ffiQuote.timestamp),
    }

    C.eeyf_quote_free(unsafe.Pointer(&ffiQuote))
    return quote, nil
}
```

**Publishing to Go modules**:
```bash
# In eeyf-go repo
git tag v1.0.0
git push origin v1.0.0
# Go modules are automatically available via the repository
```

### Ruby (with ffi gem)

**Repository**: Create `eeyf-ruby` repository

**Structure**:
```
eeyf-ruby/
├── lib/
│   ├── eeyf.rb            # Main module
│   ├── eeyf/
│   │   ├── client.rb      # Client wrapper
│   │   ├── quote.rb       # Quote models
│   │   ├── ffi.rb         # FFI bindings
│   │   └── version.rb     # Version
├── spec/
├── eeyf.gemspec
└── README.md
```

**Example Implementation** (`lib/eeyf/client.rb`):

```ruby
require 'ffi'

module EEYF
  module FFI
    extend ::FFI::Library
    
    lib_name = ::FFI::Platform.windows? ? 'eeyf.dll' : 'libeeyf.so'
    ffi_lib File.expand_path("../../lib/#{lib_name}", __dir__)
    
    class FFIQuote < ::FFI::Struct
      layout :symbol, :pointer,
             :price, :double,
             :change, :double,
             :change_percent, :double,
             :volume, :uint64,
             :market_cap, :uint64,
             :timestamp, :int64
    end
    
    class FFIError < ::FFI::Struct
      layout :code, :int32,
             :message, :pointer
    end
    
    attach_function :eeyf_client_new, [:uint64, :uint32, :uint64], :pointer
    attach_function :eeyf_client_free, [:pointer], :void
    attach_function :eeyf_get_quote, [:pointer, :string, :pointer, :pointer], :int32
    attach_function :eeyf_quote_free, [:pointer], :void
    attach_function :eeyf_error_free, [:pointer], :void
  end
  
  class Client
    def initialize(cache_ttl: 300, max_retries: 3, timeout: 30)
      @handle = FFI.eeyf_client_new(cache_ttl, max_retries, timeout)
      raise 'Failed to create EEYF client' if @handle.null?
      
      ObjectSpace.define_finalizer(self, self.class.finalize(@handle))
    end
    
    def self.finalize(handle)
      proc { FFI.eeyf_client_free(handle) unless handle.null? }
    end
    
    def get_quote(symbol)
      ffi_quote = FFI::FFIQuote.new
      ffi_error = FFI::FFIError.new
      
      result = FFI.eeyf_get_quote(@handle, symbol, ffi_quote.pointer, ffi_error.pointer)
      
      unless result.zero?
        error_msg = ffi_error[:message].read_string
        FFI.eeyf_error_free(ffi_error.pointer)
        raise "Failed to get quote: #{error_msg}"
      end
      
      quote = {
        symbol: ffi_quote[:symbol].read_string,
        price: ffi_quote[:price],
        change: ffi_quote[:change],
        change_percent: ffi_quote[:change_percent],
        volume: ffi_quote[:volume],
        market_cap: ffi_quote[:market_cap],
        timestamp: ffi_quote[:timestamp]
      }
      
      FFI.eeyf_quote_free(ffi_quote.pointer)
      quote
    end
  end
end
```

**Publishing to RubyGems**:
```bash
# In eeyf-ruby repo
gem build eeyf.gemspec
gem push eeyf-*.gem
```

## Building the FFI Layer

### Rust Implementation

Add to `Cargo.toml`:
```toml
[lib]
name = "eeyf"
crate-type = ["cdylib", "rlib"]  # Both dynamic library and Rust library

[dependencies]
libc = "0.2"
```

Create `src/ffi.rs`:

```rust
use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int, c_void};
use std::ptr;

use crate::client::Client;
use crate::models::{Quote, HistoricalDataPoint};

// Error codes
pub const EEYF_SUCCESS: c_int = 0;
pub const EEYF_ERROR_NULL_POINTER: c_int = -1;
pub const EEYF_ERROR_INVALID_SYMBOL: c_int = -2;
pub const EEYF_ERROR_NETWORK: c_int = -3;
pub const EEYF_ERROR_PARSE: c_int = -4;
pub const EEYF_ERROR_RATE_LIMIT: c_int = -5;
pub const EEYF_ERROR_UNKNOWN: c_int = -100;

// FFI structures
#[repr(C)]
pub struct FFIQuote {
    pub symbol: *mut c_char,
    pub price: f64,
    pub change: f64,
    pub change_percent: f64,
    pub volume: u64,
    pub market_cap: u64,
    pub timestamp: i64,
}

#[repr(C)]
pub struct FFIError {
    pub code: c_int,
    pub message: *mut c_char,
}

// Helper functions
unsafe fn create_error(code: c_int, message: &str) -> FFIError {
    FFIError {
        code,
        message: CString::new(message).unwrap().into_raw(),
    }
}

unsafe fn quote_to_ffi(quote: &Quote) -> FFIQuote {
    FFIQuote {
        symbol: CString::new(quote.symbol.as_str()).unwrap().into_raw(),
        price: quote.price,
        change: quote.change,
        change_percent: quote.change_percent,
        volume: quote.volume,
        market_cap: quote.market_cap,
        timestamp: quote.timestamp,
    }
}

// Client lifecycle
#[no_mangle]
pub unsafe extern "C" fn eeyf_client_new(
    cache_ttl: u64,
    max_retries: u32,
    timeout_seconds: u64,
) -> *mut c_void {
    let client = Client::new(cache_ttl, max_retries, timeout_seconds);
    Box::into_raw(Box::new(client)) as *mut c_void
}

#[no_mangle]
pub unsafe extern "C" fn eeyf_client_free(client: *mut c_void) {
    if !client.is_null() {
        drop(Box::from_raw(client as *mut Client));
    }
}

// Quote operations
#[no_mangle]
pub unsafe extern "C" fn eeyf_get_quote(
    client: *mut c_void,
    symbol: *const c_char,
    out_quote: *mut FFIQuote,
    out_error: *mut FFIError,
) -> c_int {
    if client.is_null() || symbol.is_null() || out_quote.is_null() {
        if !out_error.is_null() {
            *out_error = create_error(EEYF_ERROR_NULL_POINTER, "Null pointer provided");
        }
        return EEYF_ERROR_NULL_POINTER;
    }

    let client = &*(client as *const Client);
    let symbol_str = match CStr::from_ptr(symbol).to_str() {
        Ok(s) => s,
        Err(_) => {
            if !out_error.is_null() {
                *out_error = create_error(EEYF_ERROR_INVALID_SYMBOL, "Invalid UTF-8 in symbol");
            }
            return EEYF_ERROR_INVALID_SYMBOL;
        }
    };

    match client.get_quote(symbol_str) {
        Ok(quote) => {
            *out_quote = quote_to_ffi(&quote);
            EEYF_SUCCESS
        }
        Err(e) => {
            if !out_error.is_null() {
                *out_error = create_error(EEYF_ERROR_NETWORK, &e.to_string());
            }
            EEYF_ERROR_NETWORK
        }
    }
}

// Memory management
#[no_mangle]
pub unsafe extern "C" fn eeyf_quote_free(quote: *mut FFIQuote) {
    if !quote.is_null() {
        let quote = &*quote;
        if !quote.symbol.is_null() {
            drop(CString::from_raw(quote.symbol));
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn eeyf_error_free(error: *mut FFIError) {
    if !error.is_null() {
        let error = &*error;
        if !error.message.is_null() {
            drop(CString::from_raw(error.message));
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn eeyf_string_free(s: *mut c_char) {
    if !s.is_null() {
        drop(CString::from_raw(s));
    }
}
```

Add to `src/lib.rs`:
```rust
pub mod ffi;
```

### Building the Shared Library

```bash
# Build for current platform
cargo build --release

# The shared library will be at:
# Linux: target/release/libeeyf.so
# macOS: target/release/libeeyf.dylib
# Windows: target/release/eeyf.dll

# Cross-compilation examples:
cargo build --release --target x86_64-unknown-linux-gnu
cargo build --release --target x86_64-apple-darwin
cargo build --release --target x86_64-pc-windows-msvc
```

## Distribution Strategy

### Option 1: Separate Binary Distribution

Each language binding repository includes pre-built shared libraries:

```
eeyf-python/
├── eeyf/
│   └── lib/
│       ├── linux-x64/
│       │   └── libeeyf.so
│       ├── darwin-x64/
│       │   └── libeeyf.dylib
│       └── win-x64/
│           └── eeyf.dll
```

### Option 2: Download on Install

Use post-install scripts to download the appropriate binary:

```python
# setup.py
from setuptools import setup
from setuptools.command.install import install
import platform
import urllib.request

class PostInstallCommand(install):
    def run(self):
        install.run(self)
        # Download appropriate binary
        system = platform.system()
        arch = platform.machine()
        url = f"https://github.com/user/eeyf/releases/download/v1.0.0/libeeyf-{system}-{arch}.tar.gz"
        # Download and extract...
```

### Option 3: System Package Managers

Distribute via system package managers:
- **Debian/Ubuntu**: `.deb` package
- **Fedora/RHEL**: `.rpm` package
- **macOS**: Homebrew formula
- **Windows**: Chocolatey package

Then language bindings can depend on the system package.

## Testing Strategy

### Unit Tests

Each binding repository should have:

1. **FFI Layer Tests** - Test the C interface directly
2. **Wrapper Tests** - Test the language-specific wrapper
3. **Integration Tests** - Test against real Yahoo Finance API
4. **Memory Leak Tests** - Verify proper cleanup

### Example Python Test

```python
import unittest
from eeyf import EEYFClient

class TestEEYFClient(unittest.TestCase):
    def setUp(self):
        self.client = EEYFClient(cache_ttl=60)
    
    def tearDown(self):
        del self.client
    
    def test_get_quote(self):
        quote = self.client.get_quote('AAPL')
        self.assertEqual(quote['symbol'], 'AAPL')
        self.assertIsInstance(quote['price'], float)
        self.assertGreater(quote['price'], 0)
    
    def test_invalid_symbol(self):
        with self.assertRaises(RuntimeError):
            self.client.get_quote('INVALID_SYMBOL_123')
    
    def test_multiple_clients(self):
        # Test that multiple clients can coexist
        client2 = EEYFClient()
        quote1 = self.client.get_quote('GOOGL')
        quote2 = client2.get_quote('MSFT')
        self.assertNotEqual(quote1['symbol'], quote2['symbol'])
```

## Documentation Requirements

Each binding repository should include:

1. **README.md** - Quick start and installation
2. **API Documentation** - Generated from code (Sphinx, JSDoc, godoc, etc.)
3. **Examples** - Working code samples
4. **Migration Guide** - If upgrading from previous versions
5. **Contributing Guide** - How to contribute to the binding

## CI/CD Pipeline

### Main EEYF Repository

```yaml
# .github/workflows/release.yml
name: Release FFI Library

on:
  release:
    types: [created]

jobs:
  build:
    strategy:
      matrix:
        os: [ubuntu-latest, macos-latest, windows-latest]
    runs-on: ${{ matrix.os }}
    steps:
      - uses: actions/checkout@v3
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      - name: Build
        run: cargo build --release
      - name: Upload artifacts
        uses: actions/upload-release-asset@v1
        with:
          upload_url: ${{ github.event.release.upload_url }}
          asset_path: target/release/libeeyf.*
```

### Binding Repository

```yaml
# .github/workflows/test.yml (eeyf-python)
name: Test Python Bindings

on: [push, pull_request]

jobs:
  test:
    runs-on: ${{ matrix.os }}
    strategy:
      matrix:
        os: [ubuntu-latest, macos-latest, windows-latest]
        python-version: ['3.8', '3.9', '3.10', '3.11', '3.12']
    steps:
      - uses: actions/checkout@v3
      - name: Set up Python
        uses: actions/setup-python@v4
        with:
          python-version: ${{ matrix.python-version }}
      - name: Download EEYF library
        run: |
          # Download from main repo releases
      - name: Install dependencies
        run: |
          python -m pip install --upgrade pip
          pip install -e .[dev]
      - name: Run tests
        run: pytest
```

## Versioning

### Semantic Versioning

Both the main library and bindings should follow semantic versioning:

- **Main EEYF**: `v1.2.3`
  - Major: Breaking FFI changes
  - Minor: New FFI functions (backwards compatible)
  - Patch: Bug fixes

- **Language Bindings**: `v1.2.3` (matches main library) or `v1.2.3-binding.1`
  - Major: Breaking API changes or major EEYF version
  - Minor: New features or minor EEYF version
  - Patch: Bug fixes in binding code

### Compatibility Matrix

Document which binding versions work with which core versions:

| EEYF Core | Python | Node.js | Go    | Ruby  |
| --------- | ------ | ------- | ----- | ----- |
| 1.0.x     | 1.0.x  | 1.0.x   | 1.0.x | 1.0.x |
| 1.1.x     | 1.1.x  | 1.1.x   | 1.1.x | 1.1.x |
| 2.0.x     | 2.0.x  | 2.0.x   | 2.0.x | 2.0.x |

## Security Considerations

1. **Input Validation** - Validate all inputs in the FFI layer
2. **Memory Safety** - Properly manage all allocations/deallocations
3. **Thread Safety** - Document thread-safety guarantees
4. **Buffer Overflows** - Use safe string handling
5. **Error Handling** - Never panic across FFI boundary

### Example Safe FFI Pattern

```rust
#[no_mangle]
pub unsafe extern "C" fn eeyf_safe_function(
    input: *const c_char,
) -> c_int {
    // Catch panics
    let result = std::panic::catch_unwind(|| {
        // Null check
        if input.is_null() {
            return EEYF_ERROR_NULL_POINTER;
        }
        
        // Safe conversion
        let input_str = match CStr::from_ptr(input).to_str() {
            Ok(s) => s,
            Err(_) => return EEYF_ERROR_INVALID_INPUT,
        };
        
        // Actual logic
        // ...
        
        EEYF_SUCCESS
    });
    
    result.unwrap_or(EEYF_ERROR_UNKNOWN)
}
```

## Performance Considerations

1. **Minimize FFI Calls** - Batch operations when possible
2. **Zero-Copy** - Pass pointers instead of copying data
3. **Caching** - Implement caching in the binding layer
4. **Connection Pooling** - Share clients across calls
5. **Async Support** - Consider async FFI for long operations

## Community Resources

### Getting Help

- **Main EEYF Discussions**: github.com/user/eeyf/discussions
- **Python Binding Issues**: github.com/user/eeyf-python/issues
- **Node.js Binding Issues**: github.com/user/eeyf-node/issues
- **Go Binding Issues**: github.com/user/eeyf-go/issues

### Contributing

1. Check existing bindings first
2. Follow the FFI patterns documented here
3. Add comprehensive tests
4. Document your code
5. Submit examples

### Example Projects

See the official examples:
- Python: github.com/user/eeyf-python/examples
- Node.js: github.com/user/eeyf-node/examples
- Go: github.com/user/eeyf-go/examples

## Conclusion

Creating language bindings for EEYF involves:

1. **Implementing the FFI layer** in the main Rust library
2. **Creating separate repositories** for each language binding
3. **Wrapping the FFI** in idiomatic language APIs
4. **Testing thoroughly** with unit and integration tests
5. **Publishing** to language-specific package managers
6. **Documenting** everything clearly
7. **Maintaining** compatibility across versions

This approach provides the best experience for developers in each language ecosystem while maintaining a clean, maintainable architecture.

## Reference Implementation Checklist

When creating a new binding:

- [ ] Create separate repository
- [ ] Implement FFI wrapper for all core functions
- [ ] Add language-idiomatic API layer
- [ ] Include comprehensive tests
- [ ] Add usage examples
- [ ] Write API documentation
- [ ] Set up CI/CD pipeline
- [ ] Configure package publishing
- [ ] Add to official binding list
- [ ] Create release announcement

---

**Last Updated**: October 2025  
**EEYF Version**: 1.0.0  
**Maintainer**: EEYF Project Team
