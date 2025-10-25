use crate::utils::error::{AppError, AppResult};
use serde_json;

pub fn generate_html(prompts: &[crate::core::data::Prompt]) -> AppResult<String> {
    let prompts_json = serde_json::to_string(prompts)
        .map_err(|e| AppError::System(format!("Failed to serialize prompts to JSON: {}", e)))?;

    let html = format!(
        r#"
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Promptheus - Prompt Collection</title>
    <style>
        /* CSS will be embedded here */
        * {{
            margin: 0;
            padding: 0;
            box-sizing: border-box;
        }}

        body {{
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
            line-height: 1.6;
            color: #333;
            background-color: #f5f5f5;
        }}

        .container {{
            max-width: 1200px;
            margin: 0 auto;
            padding: 20px;
        }}

        .header {{
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            color: white;
            padding: 2rem;
            border-radius: 10px;
            margin-bottom: 2rem;
            text-align: center;
        }}

        .header h1 {{
            font-size: 2.5rem;
            margin-bottom: 0.5rem;
        }}

        .controls {{
            background: white;
            padding: 1.5rem;
            border-radius: 10px;
            margin-bottom: 2rem;
            box-shadow: 0 2px 10px rgba(0,0,0,0.1);
            display: flex;
            gap: 1rem;
            flex-wrap: wrap;
            align-items: center;
        }}

        .search-box {{
            flex: 1;
            min-width: 200px;
            padding: 0.75rem;
            border: 2px solid #ddd;
            border-radius: 8px;
            font-size: 1rem;
        }}

        .search-box:focus {{
            outline: none;
            border-color: #667eea;
        }}

        .filter-group {{
            display: flex;
            gap: 0.5rem;
            align-items: center;
        }}

        .filter-select {{
            padding: 0.75rem;
            border: 2px solid #ddd;
            border-radius: 8px;
            font-size: 1rem;
            background: white;
        }}

        .btn {{
            padding: 0.75rem 1.5rem;
            background: #667eea;
            color: white;
            border: none;
            border-radius: 8px;
            cursor: pointer;
            font-size: 1rem;
            transition: background 0.3s ease;
        }}

        .btn:hover {{
            background: #5a6fd8;
        }}

        .stats {{
            background: white;
            padding: 1rem;
            border-radius: 10px;
            margin-bottom: 2rem;
            box-shadow: 0 2px 10px rgba(0,0,0,0.1);
            display: flex;
            gap: 2rem;
            justify-content: center;
            flex-wrap: wrap;
        }}

        .stat-item {{
            text-align: center;
        }}

        .stat-number {{
            font-size: 2rem;
            font-weight: bold;
            color: #667eea;
        }}

        .stat-label {{
            color: #666;
            font-size: 0.9rem;
        }}

        .prompt-grid {{
            display: grid;
            grid-template-columns: repeat(auto-fill, minmax(400px, 1fr));
            gap: 1.5rem;
        }}

        .prompt-card {{
            background: white;
            border-radius: 10px;
            padding: 1.5rem;
            box-shadow: 0 2px 10px rgba(0,0,0,0.1);
            transition: transform 0.3s ease, box-shadow 0.3s ease;
        }}

        .prompt-card:hover {{
            transform: translateY(-5px);
            box-shadow: 0 5px 20px rgba(0,0,0,0.15);
        }}

        .prompt-header {{
            margin-bottom: 1rem;
        }}

        .prompt-title {{
            font-size: 1.25rem;
            font-weight: bold;
            color: #333;
            margin-bottom: 0.5rem;
        }}

        .prompt-meta {{
            display: flex;
            gap: 0.5rem;
            flex-wrap: wrap;
            margin-bottom: 1rem;
        }}

        .tag {{
            background: #e3f2fd;
            color: #1976d2;
            padding: 0.25rem 0.5rem;
            border-radius: 15px;
            font-size: 0.8rem;
        }}

        .category {{
            background: #f3e5f5;
            color: #7b1fa2;
            padding: 0.25rem 0.5rem;
            border-radius: 15px;
            font-size: 0.8rem;
        }}

        .prompt-content {{
            background: #f8f9fa;
            padding: 1rem;
            border-radius: 8px;
            font-family: 'Courier New', monospace;
            font-size: 0.9rem;
            white-space: pre-wrap;
            max-height: 200px;
            overflow-y: auto;
            margin-bottom: 1rem;
            border: 1px solid #e9ecef;
        }}

        .prompt-actions {{
            display: flex;
            gap: 0.5rem;
            flex-wrap: wrap;
        }}

        .btn-small {{
            padding: 0.5rem 1rem;
            font-size: 0.8rem;
            background: #28a745;
        }}

        .btn-small:hover {{
            background: #218838;
        }}

        .btn-secondary {{
            background: #6c757d;
        }}

        .btn-secondary:hover {{
            background: #5a6268;
        }}

        .modal {{
            display: none;
            position: fixed;
            z-index: 1000;
            left: 0;
            top: 0;
            width: 100%;
            height: 100%;
            background-color: rgba(0,0,0,0.5);
        }}

        .modal-content {{
            background-color: white;
            margin: 5% auto;
            padding: 2rem;
            border-radius: 10px;
            width: 90%;
            max-width: 800px;
            max-height: 80vh;
            overflow-y: auto;
        }}

        .close {{
            color: #aaa;
            float: right;
            font-size: 28px;
            font-weight: bold;
            cursor: pointer;
        }}

        .close:hover {{
            color: #000;
        }}

        .edit-textarea {{
            width: 100%;
            height: 300px;
            padding: 1rem;
            border: 2px solid #ddd;
            border-radius: 8px;
            font-family: 'Courier New', monospace;
            font-size: 0.9rem;
            resize: vertical;
        }}

        .edit-textarea:focus {{
            outline: none;
            border-color: #667eea;
        }}

        .form-group {{
            margin-bottom: 1rem;
        }}

        .form-label {{
            display: block;
            margin-bottom: 0.5rem;
            font-weight: bold;
        }}

        .form-input {{
            width: 100%;
            padding: 0.75rem;
            border: 2px solid #ddd;
            border-radius: 8px;
            font-size: 1rem;
        }}

        .form-input:focus {{
            outline: none;
            border-color: #667eea;
        }}

        .hidden {{
            display: none;
        }}

        @media (max-width: 768px) {{
            .container {{
                padding: 10px;
            }}

            .controls {{
                flex-direction: column;
            }}

            .search-box {{
                width: 100%;
            }}

            .prompt-grid {{
                grid-template-columns: 1fr;
            }}
        }}
    </style>
</head>
<body>
    <div class="container">
        <header class="header">
            <h1>🚀 Promptheus</h1>
            <p>Interactive Prompt Collection Viewer & Editor</p>
        </header>

        <div class="controls">
            <input type="text" class="search-box" id="searchBox" placeholder="🔍 Search prompts...">

            <div class="filter-group">
                <label for="categoryFilter">Category:</label>
                <select class="filter-select" id="categoryFilter">
                    <option value="">All Categories</option>
                </select>
            </div>

            <div class="filter-group">
                <label for="tagFilter">Tag:</label>
                <select class="filter-select" id="tagFilter">
                    <option value="">All Tags</option>
                </select>
            </div>

            <button class="btn" onclick="addNewPrompt()">➕ Add New</button>
            <button class="btn btn-secondary" onclick="exportTomlWithSuccess()">📥 Export TOML</button>
        </div>

        <div id="saveInstructions" style="background: #fff3cd; border: 1px solid #ffeaa7; border-radius: 8px; padding: 1rem; margin-bottom: 1rem; display: none;">
            <div style="display: flex; justify-content: space-between; align-items: center;">
                <div>
                    <h4 style="margin: 0 0 0.5rem 0; color: #856404;">💾 Changes Detected</h4>
                    <p style="margin: 0; color: #856404;">Click "📥 Export TOML" to download updated prompts.toml file</p>
                </div>
                <button onclick="hideSaveInstructions()" style="background: #856404; color: white; border: none; padding: 0.5rem 1rem; border-radius: 4px; cursor: pointer; font-size: 0.9rem;">Dismiss</button>
            </div>
        </div>

        <div class="stats">
            <div class="stat-item">
                <div class="stat-number" id="totalCount">0</div>
                <div class="stat-label">Total Prompts</div>
            </div>
            <div class="stat-item">
                <div class="stat-number" id="categoryCount">0</div>
                <div class="stat-label">Categories</div>
            </div>
            <div class="stat-item">
                <div class="stat-number" id="tagCount">0</div>
                <div class="stat-label">Tags</div>
            </div>
        </div>

        <div class="prompt-grid" id="promptGrid">
            <!-- Prompt cards will be generated here -->
        </div>
    </div>

    <!-- Edit Modal -->
    <div id="editModal" class="modal">
        <div class="modal-content">
            <span class="close" onclick="closeModal()">&times;</span>
            <h2 id="modalTitle">Edit Prompt</h2>
            <form id="editForm">
                <div class="form-group">
                    <label class="form-label" for="editDescription">Description:</label>
                    <input type="text" class="form-input" id="editDescription" required>
                </div>

                <div class="form-group">
                    <label class="form-label" for="editCategory">Category:</label>
                    <input type="text" class="form-input" id="editCategory" placeholder="e.g., programming, writing">
                </div>

                <div class="form-group">
                    <label class="form-label" for="editTags">Tags (comma-separated):</label>
                    <input type="text" class="form-input" id="editTags" placeholder="e.g., rust, cli, productivity">
                </div>

                <div class="form-group">
                    <label class="form-label" for="editContent">Content:</label>
                    <textarea class="edit-textarea" id="editContent" required></textarea>
                </div>

                <div style="display: flex; gap: 1rem; justify-content: flex-end;">
                    <button type="button" class="btn btn-secondary" onclick="closeModal()">Cancel</button>
                    <button type="submit" class="btn">Save Changes</button>
                </div>
            </form>
        </div>
    </div>

    <script>
        // Data embedded from Rust
        const promptsData = {prompts_json};

        let prompts = [];
        let currentEditIndex = -1;

        // Initialize the application
        function init() {{
            prompts = promptsData.map(p => ({{
                description: p.Description,
                content: p.Content,
                category: p.Category && p.Category.trim() !== '' ? p.Category : null,
                tag: p.Tag && p.Tag.length > 0 ? p.Tag : null,
                created_at: p.Created_at,
                updated_at: p.Created_at // Use created_at as fallback
            }}));
            populateFilters();
            updateStats();
            renderPrompts();
        }}

        // Populate filter dropdowns
        function populateFilters() {{
            const categories = [...new Set(prompts.map(p => p.category).filter(Boolean))];
            const tags = [...new Set(prompts.flatMap(p => p.tag || []))];

            const categoryFilter = document.getElementById('categoryFilter');
            const tagFilter = document.getElementById('tagFilter');

            categories.forEach(category => {{
                const option = document.createElement('option');
                option.value = category;
                option.textContent = category;
                categoryFilter.appendChild(option);
            }});

            tags.forEach(tag => {{
                const option = document.createElement('option');
                option.value = tag;
                option.textContent = tag;
                tagFilter.appendChild(option);
            }});
        }}

        // Update statistics
        function updateStats() {{
            document.getElementById('totalCount').textContent = prompts.length;

            const categories = [...new Set(prompts.map(p => p.category).filter(Boolean))];
            document.getElementById('categoryCount').textContent = categories.length;

            const tags = [...new Set(prompts.flatMap(p => p.tag || []))];
            document.getElementById('tagCount').textContent = tags.length;
        }}

        // Render prompts
        function renderPrompts() {{
            const grid = document.getElementById('promptGrid');
            const searchTerm = document.getElementById('searchBox').value.toLowerCase();
            const selectedCategory = document.getElementById('categoryFilter').value;
            const selectedTag = document.getElementById('tagFilter').value;

            let filteredPrompts = prompts.filter(prompt => {{
                const matchesSearch = !searchTerm ||
                    prompt.description.toLowerCase().includes(searchTerm) ||
                    prompt.content.toLowerCase().includes(searchTerm);

                const matchesCategory = !selectedCategory || prompt.category === selectedCategory;
                const matchesTag = !selectedTag || (prompt.tag && prompt.tag.includes(selectedTag));

                return matchesSearch && matchesCategory && matchesTag;
            }});

            grid.innerHTML = '';

            if (filteredPrompts.length === 0) {{
                grid.innerHTML = '<div style="text-align: center; padding: 2rem; color: #666;">No prompts found matching your criteria.</div>';
                return;
            }}

            filteredPrompts.forEach((prompt, index) => {{
                const originalIndex = prompts.indexOf(prompt);
                const card = createPromptCard(prompt, originalIndex);
                grid.appendChild(card);
            }});
        }}

        // Create prompt card
        function createPromptCard(prompt, index) {{
            const card = document.createElement('div');
            card.className = 'prompt-card';

            const tagsHtml = prompt.tag ? prompt.tag.map(tag => `<span class="tag">` + tag + `</span>`).join('') : '';
            const categoryHtml = prompt.category ? `<span class="category">` + prompt.category + `</span>` : '';

            card.innerHTML = '<div class="prompt-header"><div class="prompt-title">' + prompt.description + '</div><div class="prompt-meta">' + categoryHtml + tagsHtml + '</div></div><div class="prompt-content">' + prompt.content + '</div><div class="prompt-actions"><button class="btn btn-small" onclick="editPrompt(' + index + ')">✏️ Edit</button><button class="btn btn-small btn-secondary" onclick="copyToClipboard(' + index + ')">📋 Copy</button><button class="btn btn-small btn-secondary" onclick="deletePrompt(' + index + ')">🗑️ Delete</button></div>';

            return card;
        }}

        // Edit prompt
        function editPrompt(index) {{
            currentEditIndex = index;
            const prompt = prompts[index];

            document.getElementById('modalTitle').textContent = 'Edit Prompt';
            document.getElementById('editDescription').value = prompt.description;
            document.getElementById('editCategory').value = prompt.category || '';
            document.getElementById('editTags').value = prompt.tag ? prompt.tag.join(', ') : '';
            document.getElementById('editContent').value = prompt.content;

            document.getElementById('editModal').style.display = 'block';
        }}

        // Add new prompt
        function addNewPrompt() {{
            currentEditIndex = -1;

            document.getElementById('modalTitle').textContent = 'Add New Prompt';
            document.getElementById('editForm').reset();
            document.getElementById('editModal').style.display = 'block';
        }}

        // Delete prompt
        function deletePrompt(index) {{
            if (confirm('Are you sure you want to delete this prompt?')) {{
                prompts.splice(index, 1);
                populateFilters();
                updateStats();
                renderPrompts();
                showSaveInstructions(); // Show save instructions after deletion
            }}
        }}

        // Copy to clipboard
        function copyToClipboard(index) {{
            const prompt = prompts[index];
            const text = 'Description: ' + prompt.description + '\\n' +
                        'Category: ' + (prompt.category || 'N/A') + '\\n' +
                        'Tags: ' + (prompt.tag ? prompt.tag.join(', ') : 'N/A') + '\\n' +
                        'Content:\\n' + prompt.content;

            navigator.clipboard.writeText(text).then(() => {{
                alert('Prompt copied to clipboard!');
            }});
        }}

        // Close modal
        function closeModal() {{
            document.getElementById('editModal').style.display = 'none';
        }}

        // Show save instructions
        function showSaveInstructions() {{
            const dismissedTime = localStorage.getItem('saveInstructionsDismissedTime');
            const now = Date.now();
            const oneDayMs = 24 * 60 * 60 * 1000; // 24 hours in milliseconds

            // Check if dismissed state has expired (older than 1 day)
            if (!dismissedTime || (now - parseInt(dismissedTime)) > oneDayMs) {{
                // Expired or never dismissed, show the instructions
                document.getElementById('saveInstructions').style.display = 'block';
            }}
        }}

        // Hide save instructions and remember preference with timestamp
        function hideSaveInstructions() {{
            document.getElementById('saveInstructions').style.display = 'none';
            localStorage.setItem('saveInstructionsDismissedTime', Date.now().toString());
        }}

        // Reset save instructions preference (for future use)
        function resetSaveInstructionsPreference() {{
            localStorage.removeItem('saveInstructionsDismissedTime');
        }}

        // Export TOML with success message
        function exportTomlWithSuccess() {{
            const tomlContent = generateToml();
            downloadFile('prompts.toml', tomlContent);

            // Show success message
            showSuccessMessage('✅ prompts.toml downloaded! Replace your original file to save changes.');
        }}

        // Show success message
        function showSuccessMessage(message) {{
            const successDiv = document.createElement('div');
            successDiv.id = 'successMessage';
            successDiv.style.cssText = 'position: fixed; top: 20px; right: 20px; background: #d4edda; border: 1px solid #c3e6cb; color: #155724; padding: 1rem; border-radius: 8px; z-index: 1001; box-shadow: 0 4px 12px rgba(0,0,0,0.15);';
            successDiv.innerHTML = message;

            document.body.appendChild(successDiv);

            // Auto-remove after 4 seconds
            setTimeout(() => {{
                if (document.getElementById('successMessage')) {{
                    document.body.removeChild(successDiv);
                }}
            }}, 4000);
        }}

        // Generate TOML content
        function generateToml() {{
            const tomlLines = [];

            prompts.forEach((prompt, index) => {{
                tomlLines.push('[[prompts]]');
                tomlLines.push('Description = "' + escapeToml(prompt.description) + '"');
                tomlLines.push('Content = """' + prompt.content + '"""');

                if (prompt.category) {{
                    tomlLines.push('Category = "' + escapeToml(prompt.category) + '"');
                }}

                if (prompt.tag && prompt.tag.length > 0) {{
                    const tagsArray = prompt.tag.map(tag => '"' + escapeToml(tag) + '"').join(', ');
                    tomlLines.push('Tag = [' + tagsArray + ']');
                }}

                tomlLines.push(''); // Empty line for readability
            }});

            return tomlLines.join('\\n');
        }}

        // Escape TOML strings
        function escapeToml(str) {{
            return str.replace(/"/g, '\\\\"').replace(/\\n/g, '\\\\n');
        }}

        // Download file
        function downloadFile(filename, content) {{
            const blob = new Blob([content], {{ type: 'text/plain' }});
            const url = window.URL.createObjectURL(blob);
            const a = document.createElement('a');
            a.href = url;
            a.download = filename;
            document.body.appendChild(a);
            a.click();
            document.body.removeChild(a);
            window.URL.revokeObjectURL(url);
        }}

        // Event listeners
        document.getElementById('searchBox').addEventListener('input', renderPrompts);
        document.getElementById('categoryFilter').addEventListener('change', renderPrompts);
        document.getElementById('tagFilter').addEventListener('change', renderPrompts);

        document.getElementById('editForm').addEventListener('submit', function(e) {{
            e.preventDefault();

            const description = document.getElementById('editDescription').value.trim();
            const category = document.getElementById('editCategory').value.trim();
            const tagsInput = document.getElementById('editTags').value.trim();
            const content = document.getElementById('editContent').value.trim();

            const tags = tagsInput ? tagsInput.split(',').map(tag => tag.trim()).filter(Boolean) : [];

            if (currentEditIndex === -1) {{
                // Add new prompt
                const newPrompt = {{
                    description: description,
                    content: content,
                    category: category || null,
                    tag: tags.length > 0 ? tags : null,
                    created_at: new Date().toISOString(),
                    updated_at: new Date().toISOString()
                }};
                prompts.push(newPrompt);
            }} else {{
                // Update existing prompt
                prompts[currentEditIndex].description = description;
                prompts[currentEditIndex].content = content;
                prompts[currentEditIndex].category = category || null;
                prompts[currentEditIndex].tag = tags.length > 0 ? tags : null;
                prompts[currentEditIndex].updated_at = new Date().toISOString();
            }}

            populateFilters();
            updateStats();
            renderPrompts();
            closeModal();
            showSaveInstructions(); // Show save instructions after edit
        }});

        // Close modal when clicking outside
        window.onclick = function(event) {{
            const modal = document.getElementById('editModal');
            if (event.target === modal) {{
                closeModal();
            }}
        }}

        // Initialize on page load
        document.addEventListener('DOMContentLoaded', init);
    </script>
</body>
</html>
"#
    );

    Ok(html)
}

pub fn open_browser(path: &str) -> AppResult<()> {
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("cmd")
            .args(["/C", "start", path])
            .spawn()
            .map_err(|e| AppError::System(format!("Failed to open browser: {}", e)))?;
    }

    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(path)
            .spawn()
            .map_err(|e| AppError::System(format!("Failed to open browser: {}", e)))?;
    }

    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open")
            .arg(path)
            .spawn()
            .map_err(|e| AppError::System(format!("Failed to open browser: {}", e)))?;
    }

    Ok(())
}
