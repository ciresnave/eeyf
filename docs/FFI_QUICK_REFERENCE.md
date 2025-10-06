# EEYF FFI Quick Reference

**For developers creating or using language bindings**

## Quick Links

- **Full Guide**: [docs/FFI_GUIDE.md](FFI_GUIDE.md) (1,150+ lines)
- **Architecture**: [docs/BINDINGS_ARCHITECTURE_CHANGE.md](BINDINGS_ARCHITECTURE_CHANGE.md)
- **Main Repo**: github.com/yourusername/eeyf

## Creating a Binding (5 Steps)

### 1. Create Separate Repository

```bash
# Create new repo for your language
mkdir eeyf-python  # or eeyf-node, eeyf-go, etc.
cd eeyf-python
git init
```

### 2. Structure Your Repo

```
eeyf-python/
├── eeyf/
│   ├── __init__.py
│   ├── client.py       # Your wrapper
│   ├── _ffi.py         # FFI bindings
│   └── lib/            # Shared library
│       ├── libeeyf.so  (Linux)
│       ├── libeeyf.dylib (macOS)
│       └── eeyf.dll    (Windows)
├── tests/
├── examples/
├── setup.py
└── README.md
```

### 3. Implement FFI Bindings

See reference implementations in [FFI_GUIDE.md](FFI_GUIDE.md):
- Python: 240 lines (ctypes/cffi)
- Node.js: 180 lines (ffi-napi)
- Go: 220 lines (CGO)
- Ruby: 160 lines (FFI gem)

### 4. Add Tests

```python
# Example Python test
def test_get_quote():
    client = EEYFClient(cache_ttl=60)
    quote = client.get_quote('AAPL')
    assert quote['symbol'] == 'AAPL'
    assert quote['price'] > 0
```

### 5. Publish

```bash
# Python
python -m build
python -m twine upload dist/*

# Node.js
npm run build
npm publish

# Go
git tag v1.0.0
git push origin v1.0.0

# Ruby
gem build eeyf.gemspec
gem push eeyf-*.gem
```

## Core FFI Functions

### Client Lifecycle

```c
// Create client
void* eeyf_client_new(uint64_t cache_ttl, uint32_t max_retries, uint64_t timeout);

// Free client
void eeyf_client_free(void* client);
```

### Quote Operations

```c
// Get single quote
int32_t eeyf_get_quote(
    void* client,
    const char* symbol,
    FFIQuote* out_quote,
    FFIError* out_error
);

// Get multiple quotes
int32_t eeyf_get_quotes(
    void* client,
    const char** symbols,
    size_t symbols_len,
    FFIQuote** out_quotes,
    size_t* out_len,
    FFIError* out_error
);
```

### Historical Data

```c
// Get historical data
int32_t eeyf_get_historical(
    void* client,
    const char* symbol,
    int64_t start_timestamp,
    int64_t end_timestamp,
    const char* interval,
    FFIHistoricalPoint** out_data,
    size_t* out_len,
    FFIError* out_error
);
```

### Memory Management

```c
// Free quote
void eeyf_quote_free(FFIQuote* quote);

// Free array of quotes
void eeyf_quotes_free(FFIQuote* quotes, size_t len);

// Free historical data
void eeyf_historical_free(FFIHistoricalPoint* data, size_t len);

// Free error
void eeyf_error_free(FFIError* error);

// Free string
void eeyf_string_free(char* s);
```

## FFI Data Structures

```c
// Quote structure
typedef struct {
    char* symbol;
    double price;
    double change;
    double change_percent;
    uint64_t volume;
    uint64_t market_cap;
    int64_t timestamp;
} FFIQuote;

// Historical data point
typedef struct {
    int64_t timestamp;
    double open;
    double high;
    double low;
    double close;
    uint64_t volume;
    double adjusted_close;
} FFIHistoricalPoint;

// Error structure
typedef struct {
    int32_t code;
    char* message;
} FFIError;
```

## Error Codes

```c
#define EEYF_SUCCESS              0
#define EEYF_ERROR_NULL_POINTER  -1
#define EEYF_ERROR_INVALID_SYMBOL -2
#define EEYF_ERROR_NETWORK       -3
#define EEYF_ERROR_PARSE         -4
#define EEYF_ERROR_RATE_LIMIT    -5
#define EEYF_ERROR_UNKNOWN      -100
```

## Language-Specific Patterns

### Python (ctypes)

```python
from ctypes import CDLL, c_void_p, c_char_p, c_uint64

# Load library
lib = CDLL('libeeyf.so')

# Configure signatures
lib.eeyf_client_new.argtypes = [c_uint64, c_uint32, c_uint64]
lib.eeyf_client_new.restype = c_void_p

# Use it
class EEYFClient:
    def __init__(self, cache_ttl=300):
        self._handle = lib.eeyf_client_new(cache_ttl, 3, 30)
    
    def __del__(self):
        if self._handle:
            lib.eeyf_client_free(self._handle)
```

### Node.js (ffi-napi)

```typescript
import ffi from 'ffi-napi';

const lib = ffi.Library('libeeyf.so', {
  'eeyf_client_new': ['pointer', ['uint64', 'uint32', 'uint64']],
  'eeyf_client_free': ['void', ['pointer']],
});

export class EEYFClient {
  private handle: Buffer;

  constructor(cacheTtl: number = 300) {
    this.handle = lib.eeyf_client_new(cacheTtl, 3, 30);
  }

  dispose(): void {
    lib.eeyf_client_free(this.handle);
  }
}
```

### Go (cgo)

```go
package eeyf

/*
#cgo LDFLAGS: -L./lib -leeyf
void* eeyf_client_new(unsigned long long, unsigned int, unsigned long long);
void eeyf_client_free(void*);
*/
import "C"
import "unsafe"

type Client struct {
    handle unsafe.Pointer
}

func NewClient(cacheTTL uint64) (*Client, error) {
    handle := C.eeyf_client_new(C.ulonglong(cacheTTL), 3, 30)
    return &Client{handle: handle}, nil
}

func (c *Client) Close() {
    C.eeyf_client_free(c.handle)
}
```

### Ruby (ffi gem)

```ruby
require 'ffi'

module EEYF
  module FFI
    extend ::FFI::Library
    ffi_lib 'eeyf'
    
    attach_function :eeyf_client_new, [:uint64, :uint32, :uint64], :pointer
    attach_function :eeyf_client_free, [:pointer], :void
  end
  
  class Client
    def initialize(cache_ttl: 300)
      @handle = FFI.eeyf_client_new(cache_ttl, 3, 30)
      ObjectSpace.define_finalizer(self, self.class.finalize(@handle))
    end
    
    def self.finalize(handle)
      proc { FFI.eeyf_client_free(handle) }
    end
  end
end
```

## Memory Safety Checklist

- [ ] Always check for null pointers
- [ ] Free all allocated memory
- [ ] Use RAII/finalizers for cleanup
- [ ] Catch panics in FFI boundary
- [ ] Validate all string inputs
- [ ] Handle all error codes
- [ ] Test for memory leaks (valgrind, sanitizers)

## Testing Checklist

- [ ] Unit tests for FFI layer
- [ ] Unit tests for wrapper API
- [ ] Integration tests (real API)
- [ ] Memory leak tests
- [ ] Multi-threading tests
- [ ] Error handling tests
- [ ] Platform-specific tests (Linux, macOS, Windows)

## Publishing Checklist

- [ ] Comprehensive README
- [ ] API documentation (auto-generated)
- [ ] Usage examples
- [ ] CI/CD pipeline
- [ ] Version compatibility documented
- [ ] Security considerations addressed
- [ ] License clearly stated
- [ ] Contributing guidelines

## Common Patterns

### Safe Error Handling

```python
# Python
try:
    quote = client.get_quote('AAPL')
except RuntimeError as e:
    if 'rate limit' in str(e).lower():
        # Wait and retry
        time.sleep(60)
        quote = client.get_quote('AAPL')
    else:
        raise
```

### Connection Pooling

```python
# Python
class ClientPool:
    def __init__(self, size=10):
        self.clients = [EEYFClient() for _ in range(size)]
        self.available = Queue(maxsize=size)
        for client in self.clients:
            self.available.put(client)
    
    @contextmanager
    def get_client(self):
        client = self.available.get()
        try:
            yield client
        finally:
            self.available.put(client)
```

### Batch Operations

```python
# Python
def get_portfolio_quotes(client, symbols):
    """Get quotes for multiple symbols efficiently"""
    return client.get_quotes(symbols)  # Single FFI call
```

## Performance Tips

1. **Batch requests** - Use `get_quotes()` instead of multiple `get_quote()` calls
2. **Reuse clients** - Create once, use many times
3. **Connection pooling** - Share clients across threads
4. **Cache results** - Reduce API calls
5. **Minimize FFI crossings** - Pass arrays instead of looping

## Getting Help

- **FFI Guide Issues**: Main EEYF repo issues
- **Binding Issues**: Your binding repo issues
- **General Help**: Main EEYF discussions
- **Security**: security@eeyf-project.org (if created)

## Example Projects

Full working examples:

- **Python**: See [Python reference implementation](FFI_GUIDE.md#python-with-ctypes-or-cffi)
- **Node.js**: See [Node.js reference implementation](FFI_GUIDE.md#nodejs-with-napi-rs-or-node-ffi-napi)
- **Go**: See [Go reference implementation](FFI_GUIDE.md#go-with-cgo)
- **Ruby**: See [Ruby reference implementation](FFI_GUIDE.md#ruby-with-ffi-gem)

## Version Compatibility

Match your binding version to EEYF core version:

| EEYF Core | Binding Version |
| --------- | --------------- |
| 1.0.x     | 1.0.x           |
| 1.1.x     | 1.1.x           |
| 2.0.x     | 2.0.x           |

Breaking changes increment major version.

## Repository Naming

- **Python**: `eeyf-python` → PyPI: `eeyf`
- **Node.js**: `eeyf-node` → npm: `@eeyf/client`
- **Go**: `eeyf-go` → import: `github.com/user/eeyf-go`
- **Ruby**: `eeyf-ruby` → gem: `eeyf`
- **Java**: `eeyf-java` → Maven: `com.eeyf:eeyf`
- **C#**: `eeyf-dotnet` → NuGet: `EEYF`

## CI/CD Template

```yaml
# .github/workflows/test.yml
name: Test

on: [push, pull_request]

jobs:
  test:
    runs-on: ${{ matrix.os }}
    strategy:
      matrix:
        os: [ubuntu-latest, macos-latest, windows-latest]
    steps:
      - uses: actions/checkout@v3
      - name: Download EEYF library
        run: # Download from releases
      - name: Install dependencies
        run: # Language-specific install
      - name: Run tests
        run: # Language-specific test command
```

## License

EEYF is dual-licensed under MIT OR Apache-2.0. Your bindings should be compatible with these licenses.

## Contributing

1. Read the [FFI Guide](FFI_GUIDE.md)
2. Create your binding repository
3. Implement following the patterns
4. Add comprehensive tests
5. Write clear documentation
6. Open issue in main repo to list your binding

---

**Quick Start**: Read [FFI_GUIDE.md](FFI_GUIDE.md)  
**Questions**: Open issue in main EEYF repo  
**Updates**: Watch the main repo for FFI changes
