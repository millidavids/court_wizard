//! Clipboard and prompt JS interop for WASM targets.
//!
//! Provides copy-to-clipboard and text input via `window.prompt()`.

use wasm_bindgen::prelude::*;

#[wasm_bindgen(inline_js = "
export function copy_to_clipboard(text) {
    if (navigator.clipboard && navigator.clipboard.writeText) {
        navigator.clipboard.writeText(text).catch(function(err) {
            // Fallback: create a temporary textarea
            var textarea = document.createElement('textarea');
            textarea.value = text;
            textarea.style.position = 'fixed';
            textarea.style.opacity = '0';
            document.body.appendChild(textarea);
            textarea.select();
            document.execCommand('copy');
            document.body.removeChild(textarea);
        });
    } else {
        var textarea = document.createElement('textarea');
        textarea.value = text;
        textarea.style.position = 'fixed';
        textarea.style.opacity = '0';
        document.body.appendChild(textarea);
        textarea.select();
        document.execCommand('copy');
        document.body.removeChild(textarea);
    }
}
")]
extern "C" {
    /// Copies the given text to the system clipboard.
    pub fn copy_to_clipboard(text: &str);
}

/// Prompts the user for text input using `window.prompt()`.
///
/// Returns `Some(text)` if the user entered text, or `None` if they cancelled.
pub fn prompt_for_text(message: &str) -> Option<String> {
    let window = web_sys::window()?;
    let result = window.prompt_with_message(message).ok()??;
    let trimmed = result.trim().to_string();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed)
    }
}
