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
- 📄 **Smart Content Preview**: Long prompts show truncated preview first, then offer full viewing in pager
- ⚡ **Quick Execution**: One-click copy or output of prompt content

### Smart Content Handling

One of Promptheus's standout features is its intelligent approach to long content. When displaying lengthy prompts, the system first shows a truncated preview (beginning and end), then asks if you'd like to view the complete content in a pager.

This feature works across all commands that display prompt content, including `search`, `exec`, and `show`.

## Installation and Usage

### Prerequisites

- Rust 1.70+
- Git (for sync functionality)

### Installation

```bash
# cargo install
cargo install promptheus

# Build from source
git clone https://github.com/VandeeFeng/promptheus.git
cd promptheus
cargo build
cargo run

# Global install
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

## Quick Start

### Core Commands

```bash
# Create a new prompt
promptheus new

# Search prompts interactively
promptheus search

# Execute a prompt (copies to clipboard)
promptheus exec

# List all prompts
promptheus list

# Show prompt details
promptheus show "prompt_name"

# Edit a prompt
promptheus edit
```

### Sync Prompts

```bash
# Two-way sync with cloud (GitHub Gist)
promptheus sync

# Upload local changes to remote
promptheus sync --upload

# Download changes from remote
promptheus sync --download
```

> 💡 **Tip**: Use `promptheus --help` to see all available commands and options.

## Configuration Example

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

