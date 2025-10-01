#!/usr/bin/env python3
"""Remove output_schema from MCP tools"""

import re

def remove_output_schemas(file_path):
    with open(file_path, 'r', encoding='utf-8') as f:
        content = f.read()
    
    # Pattern to match output_schema with nested JSON
    # We need to match balanced braces
    lines = content.split('\n')
    result_lines = []
    i = 0
    
    while i < len(lines):
        line = lines[i]
        
        # Check if this line starts with output_schema: Some(json!({
        if 'output_schema: Some(json!({' in line:
            # Count braces to find the end
            brace_count = line.count('{') - line.count('}')
            
            # Skip this line and continue until braces are balanced
            i += 1
            while i < len(lines) and brace_count > 0:
                brace_count += lines[i].count('{') - lines[i].count('}')
                i += 1
            
            # Add the replacement
            result_lines.append('                    output_schema: None,')
        else:
            result_lines.append(line)
            i += 1
    
    # Write back
    with open(file_path, 'w', encoding='utf-8') as f:
        f.write('\n'.join(result_lines))
    
    print(f"OK: Removed output_schema from {file_path}")

if __name__ == '__main__':
    remove_output_schemas('src/mcp_service.rs')

