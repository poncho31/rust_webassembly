# 🚀 Enhanced Form System - Integration Guide

## Quick Start

### 1. Include WebAssembly Module

```html
<!DOCTYPE html>
<html>
<head>
    <title>My App</title>
</head>
<body>
    <!-- Your form HTML -->
    <form id="myForm">
        <input type="text" id="name" name="name" required>
        <input type="email" id="email" name="email" required>
        <button type="submit">Submit</button>
    </form>

    <!-- Include WebAssembly module -->
    <script type="module">
        import init, { form_init, FieldType } from './pkg/client.js';
        
        async function run() {
            await init();
            
            // Simple form initialization
            form_init("myForm", "/api/submit", [
                ["name", FieldType.Text],
                ["email", FieldType.Email]
            ]);
        }
        
        run();
    </script>
</body>
</html>
```

### 2. Advanced Configuration

```javascript
import init, { 
    FormHandler, 
    FormConfig, 
    FormValidator, 
    ValidationRule,
    FieldType 
} from './pkg/client.js';

async function setupAdvancedForm() {
    await init();
    
    // Create configuration
    const config = FormConfig.builder()
        .enable_validation(true)
        .retry_attempts(3)
        .show_loading(true)
        .auto_focus_error(true)
        .success_message("Form submitted successfully!")
        .build();
    
    // Create validator
    const validator = FormValidator.new()
        .add_rule("name", ValidationRule.text(2, 50))
        .add_rule("email", ValidationRule.email())
        .add_rule("age", ValidationRule.number(0, 150));
    
    // Field specifications
    const fieldSpecs = [
        ["name", FieldType.Text],
        ["email", FieldType.Email],
        ["age", FieldType.Number],
        ["message", FieldType.TextArea]
    ];
    
    // Initialize form
    const handler = FormHandler.new("myForm", "/api/submit", fieldSpecs, config)
        .with_validator(validator);
    
    handler.initialize();
}
```

## Field Types

| Type | Description | HTML Input |
|------|-------------|------------|
| `Text` | Basic text input | `<input type="text">` |
| `Email` | Email with validation | `<input type="email">` |
| `Password` | Password field | `<input type="password">` |
| `Number` | Numeric input | `<input type="number">` |
| `Date` | Date picker | `<input type="date">` |
| `Tel` | Telephone number | `<input type="tel">` |
| `Url` | URL input | `<input type="url">` |
| `TextArea` | Multi-line text | `<textarea>` |
| `File` | File upload | `<input type="file">` |
| `Select` | Dropdown select | `<select>` |
| `Checkbox` | Checkbox input | `<input type="checkbox">` |
| `Radio` | Radio button | `<input type="radio">` |

## Validation Rules

```javascript
// Pre-built validation rules
ValidationRule.required()           // Field is required
ValidationRule.email()             // Valid email format
ValidationRule.text(minLen, maxLen) // Text length validation
ValidationRule.number(min, max)     // Numeric range validation

// Custom validation rule
ValidationRule.custom()
    .required(true)
    .min_length(5)
    .max_length(100)
    .pattern("phone")  // Built-in patterns: email, phone, url
    .custom_message("Please enter a valid phone number")
```

## Configuration Options

```javascript
FormConfig.builder()
    .enable_validation(true)          // Enable client-side validation
    .retry_attempts(3)                // Retry failed submissions
    .show_loading(true)               // Show loading indicators
    .auto_focus_error(true)           // Focus first error field
    .max_file_size(10 * 1024 * 1024) // 10MB file size limit
    .success_message("Success!")      // Custom success message
    .error_message("Error occurred")  // Custom error message
    .debounce_ms(300)                // Validation debounce delay
    .build()
```

## Error Handling

The system provides comprehensive error handling:

```javascript
// Automatic error display
// - Validation errors shown in modal
// - Network errors with retry options
// - File upload errors with details
// - Server errors with custom messages

// Programmatic error handling
try {
    const handler = FormHandler.new(/* ... */);
    handler.initialize();
} catch (error) {
    console.error("Form initialization failed:", error);
}
```

## Performance Features

### Caching
- Form validation results cached
- Field configurations cached
- Performance metrics tracked

### Optimization
- Lazy validation (validate on blur/submit)
- Debounced input validation
- Efficient DOM updates
- Memory leak prevention

## Migration from Old System

### Before (Old API)
```javascript
// Old monolithic approach
client_form_init("myForm", "/submit");
```

### After (Enhanced API)
```javascript
// New modular approach (backward compatible)
form_init("myForm", "/submit", [
    ["name", FieldType.Text],
    ["email", FieldType.Email]
]);

// Or use advanced features
FormHandler.new(/* ... */).initialize();
```

## Browser Support

- ✅ Chrome 60+
- ✅ Firefox 55+
- ✅ Safari 11+
- ✅ Edge 79+
- ✅ Mobile browsers (iOS Safari, Chrome Mobile)

## File Structure

```
your-project/
├── static/
│   ├── pkg/           # WebAssembly package
│   │   ├── client.js  # JavaScript bindings
│   │   ├── client_bg.wasm # WebAssembly binary
│   │   └── client.d.ts # TypeScript definitions
│   └── index.html     # Your HTML page
└── src/
    └── main.js        # Your application code
```

## Debugging

Enable debug mode for detailed logging:

```javascript
// Enable console logging
console.log("Form system debug mode enabled");

// The system automatically logs:
// - Form submission attempts
// - Validation results
// - Performance metrics
// - Error details
```

## Production Deployment

1. **Optimize WebAssembly:**
   ```bash
   wasm-pack build --target web --release --out-dir pkg
   ```

2. **Serve with proper MIME types:**
   ```
   .wasm -> application/wasm
   .js   -> application/javascript
   ```

3. **Enable gzip compression** for `.wasm` and `.js` files

4. **Use CDN** for static assets

## Examples

- 📄 `enhanced_form_demo.html` - Complete demo with all features
- 📁 `form/examples.rs` - Rust code examples
- 📖 `form/README.md` - Detailed API documentation

## Support

- 📖 Full documentation in `form/README.md`
- 🔧 Type definitions in `client.d.ts`
- 🐛 Error messages include helpful context
- 💬 Check console for debug information
