# Promptheus

A Rust-based command-line prompt management tool that helps you efficiently organize, search, and execute various prompt templates.

Inspired by [knqyf263/pet: Simple command-line snippet manager](https://github.com/knqyf263/pet)

## Introduction

Promptheus is a powerful prompt management system designed for developers, content creators, and AI prompt engineers. It provides complete prompt lifecycle management, including creation, editing, searching, categorization, and synchronization features.

### Key Features

- 📝 **Prompt Management**: Create, edit, delete, and organize prompts
- 🔍 **Smart Search**: Search by tags, categories, and content
- 🏷️ **Tag System**: Flexible tag-based categorization
- 📁 **Category Management**: Organize prompts by categories
- 🔄 **Cloud Sync**: GitHub Gist synchronization support
- 🎯 **Interactive Interface**: Intuitive command-line interaction
- ⚡ **Quick Execution**: One-click copy or output of prompt content

## Installation and Usage

### Prerequisites

- Rust 1.70+
- Git (for sync functionality)

### Installation

```bash
# Build from source
git clone https://github.com/VandeeFeng/promptheus.git
cd promptheus
cargo build --release
cargo install --path .

```

### First Time Use

Promptheus automatically creates configuration files on first run:

```bash
# Show help
promptheus --help

# Create your first prompt
promptheus new
```

Configuration file location:
- Linux/macOS: `~/.config/promptheus/config.toml`
- Windows: `%APPDATA%\promptheus\config.toml`

## Command Line Interface

### Basic Commands

#### `new` - Create new prompt
```bash
# Interactive creation
promptheus new

# Create with parameters
promptheus new -T "Title" -d "Description" -t "tag" -c "category"

# Create using editor
promptheus new --editor

# Create with direct content
promptheus new --content "prompt content"
```

#### `list` - List prompts
```bash
# List all prompts
promptheus list

# Filter by tag
promptheus list -t "tag_name"

# Filter by category
promptheus list -c "category_name"

# Show detailed information
promptheus list --format detailed

# Table format display
promptheus list --format table

# JSON format output
promptheus list --format json

# Show statistics
promptheus list --stats
```

#### `search` - Search prompts
```bash
# Interactive search
promptheus search

# Search by keyword
promptheus search -q "keyword"

# Search by tag
promptheus search -t "tag_name"

# Search by category
promptheus search -c "category_name"

# Search and execute
promptheus search --execute

# Search and copy to clipboard
promptheus search --copy
```

#### `exec` - Execute prompt
```bash
# Copy to clipboard
promptheus exec "prompt_id_or_title"

# Output to console
promptheus exec "prompt_id_or_title" --output

# Use variable substitution
promptheus exec "prompt_id_or_title" --vars "var1=value1" "var2=value2"
```

### Edit Commands

#### `edit` - Edit prompt
```bash
# Interactive selection and edit
promptheus edit

# Edit specific prompt
promptheus edit "prompt_id_or_title"

# Filter by tag then edit
promptheus edit -t "tag_name"

# Filter by category then edit
promptheus edit -c "category_name"

```

#### `show` - Show prompt details
```bash
# Show prompt details
promptheus show "prompt_id_or_title"

# Show with variable substitution
promptheus show "prompt_id_or_title" --vars "var1=value1"
```

#### `delete` - Delete prompt
```bash
# Delete prompt (with confirmation)
promptheus delete "prompt_id_or_title"

# Force delete (no confirmation)
promptheus delete "prompt_id_or_title" --force
```

### Organization Commands

#### `tags` - Manage tags
```bash
# Show all tags
promptheus tags
```

#### `categories` - Manage categories
```bash
# Show all categories
promptheus categories
```

### Configuration Commands

#### `config` - Configuration management
```bash
# Show current configuration
promptheus config show

# Open configuration file in editor
promptheus config open

# Reset configuration to defaults
promptheus config reset
```

### Sync Commands

#### `sync` - Synchronize prompts
```bash
# Two-way synchronization
promptheus sync

# Upload only
promptheus sync --upload

# Download only
promptheus sync --download

# Force sync (overwrite conflicts)
promptheus sync --force
```

#### `push` - Force push
```bash
# Force upload local prompts to remote
promptheus push
```

### Global Options

```bash
# Specify configuration file
promptheus --config /path/to/config.toml <command>

# Enable debug mode
promptheus --debug <command>
```

## Usage Examples

### Workflow Examples

1. **Create a prompt**
   ```bash
   promptheus new -T "Code Review" -d "Prompt for code review" -t "programming" -c "work"
   ```

2. **Search and execute**
   ```bash
   promptheus search -q "code review" --execute
   ```

3. **Manage tags**
   ```bash
   promptheus tags  # View all tags
   promptheus list -t "programming"  # View programming-related prompts
   ```

4. **Sync to cloud**
   ```bash
   promptheus sync --upload  # Upload to GitHub Gist
   ```

### Configuration Example

Example `config.toml`:

```toml
[general]
prompt_file = "/home/user/.config/promptheus/prompts.toml"
prompt_dirs = []
editor = "vim"
select_cmd = "fzf"
default_tags = []
auto_sync = false
sort_by = "recency"
color = true
content_preview = true
search_case_sensitive = false

[gist]
file_name = "prompt.toml"
access_token = "your_github_token"
gist_id = "your_gist_id"
public = false
auto_sync = false
```

