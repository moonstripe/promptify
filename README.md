# Promptify: Format Plaintext Directories for LLMs

Promptify is a command-line utility that transforms plaintext directories into a format suitable for processing by Large Language Models (LLMs).
**Key Features:**

- **Recursive Directory Listing:** Promptify traverses directories recursively, ensuring all relevant files are processed.
- **Plaintext File Identification:** It intelligently identifies plaintext files based on file extensions (.html.twig) and MIME type analysis using the `mime_guess` crate. Supports JSON files as well.
- **LLM-Friendly Formatting:** Files are formatted with code blocks for clear separation and structure, making them readily consumable by LLMs.

**Example Usage:**

````bash
promptify -d /path/to/directory -p \"This prompt will be applied to the end of the formatted text.\"```
````

- `-d /path/to/directory`: Specifies the directory to process.
- `-p \"This prompt will be applied to the end of the formatted text.\"`: Adds a prompt to the formatted text.

**How it Works:**

1. Promptify scans the specified directory and its subdirectories.
2. It identifies plaintext files (`.html.twig` and MIME type `text/plain`) and JSON files.
3. Each file's content is read and formatted within code blocks, along with an optional prompt if requested.

**Contributing:**
Feel free to contribute to Promptify by reporting bugs,suggesting improvements, or expanding its capabilities.

**License:**
This project is licensed under the MIT License - see the LICENSE file for details.
