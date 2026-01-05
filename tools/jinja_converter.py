import re
import argparse
import sys
import os
import json

def convert_to_minijinja(content):
    """
    Converts a potentially malformed or dialect-specific Jinja2 template
    into strict Minijinja-compatible syntax.
    """
    lines = content.split('\n')
    fixed_lines = []
    
    # Global state validation/warnings
    has_raise_exception = False
    
    for line in lines:
        original_line = line
        
        # 1. Fix "else if" -> "elif"
        # Matches: {%- else if ... %} or {% else if ... %}
        # We need to be careful not to match inside strings, but simple regex works for 99% of templates
        if re.search(r'\{%-?\s*else\s+if', line):
            line = re.sub(r'(\{%-?\s*)else\s+if', r'\1elif', line)
            
        # 2. Fix Go-style "{{- for" or "{{ for" -> "{%- for"
        # Minijinja uses {% for %} for control flow.
        # Check if it starts with {{ and contains " for " or " if " or " set " but is NOT a print e.g. {{ "string" }}
        # Heuristic: if it contains " in " (for loop) or "==" (if) inside {{ }} AND does not look like a variable print.
        # Better heuristic: strict keywords at start.
        
        # Fix {{ for ... }}
        if re.search(r'\{\{-?\s*for\s', line):
            # Replace {{ with {% and }} with %}
            line = re.sub(r'\{\{(-?\s*for\s.*?)\}\}', r'{%\1%}', line)
            
        # Fix {{ end }} or {{ endfor }} -> {% endfor %}
        if re.search(r'\{\{-?\s*end\s*\}\}', line) or re.search(r'\{\{-?\s*endfor\s*\}\}', line):
             line = re.sub(r'\{\{(-?\s*(?:end|endfor)\s*)\}\}', r'{% endfor %}', line) # Normalize to endfor
             # Note: preserve whitespace control if present in capture group?
             # Let's simple normalize to {%- endfor %} if it was {{- ...
             if '{{-' in original_line:
                 line = line.replace('{% endfor %}', '{%- endfor %}')

        # Fix {{ if ... }} -> {% if ... %}
        # Be careful not to match {{ true if ... else ... }} (expression)
        # We target lines that look like statements.
        # Only if it strictly starts the block.
        if re.search(r'\{\{-?\s*if\s', line) and 'else' not in line: # simplistic check
             # This is risky, manual review recommended for expression-ifs.
             # But usually top-level {{ if }} is a mistake for {% if %}
             pass 

        # 3. Fix "raise_exception" warning
        if 'raise_exception' in line:
            has_raise_exception = True

        fixed_lines.append(line)

    result = '\n'.join(fixed_lines)
    
    return result, has_raise_exception


def extract_stop_tokens(model_dir, template_name=None):
    """
    Extracts stop tokens from model configuration files.
    Priority:
    1. Ollama-style {name}.json with "stop" array (if template_name provided)
    2. tokenizer_config.json: eos_token, stop_strings
    3. generation_config.json: eos_token, stop_strings
    Returns a list of stop token strings.
    """
    stop_tokens = set()
    
    if not model_dir or not os.path.isdir(model_dir):
        return []
    
    # Priority 1: Ollama-style stop file (e.g., llama3-instruct.json)
    if template_name:
        ollama_stop_file = os.path.join(model_dir, f"{template_name}.json")
        if os.path.exists(ollama_stop_file):
            try:
                with open(ollama_stop_file, 'r', encoding='utf-8') as f:
                    config = json.load(f)
                if "stop" in config and isinstance(config["stop"], list):
                    sys.stderr.write(f"--- Using Ollama stop tokens from {template_name}.json ---\n")
                    return config["stop"]
            except (json.JSONDecodeError, IOError):
                pass
    
    # Priority 2: tokenizer_config.json and generation_config.json
    config_files = [
        "tokenizer_config.json",
        "generation_config.json",
    ]
    
    # Only use explicit stop token fields
    token_keys = [
        "eos_token",      # Primary EOS token
        "stop_strings",   # Explicit stop strings list
        "stop_token",     # Alternative name
    ]
    
    for config_file in config_files:
        config_path = os.path.join(model_dir, config_file)
        if not os.path.exists(config_path):
            continue
            
        try:
            with open(config_path, 'r', encoding='utf-8') as f:
                config = json.load(f)
        except (json.JSONDecodeError, IOError) as e:
            sys.stderr.write(f"Warning: Could not parse {config_file}: {e}\n")
            continue
        
        # Extract tokens from explicit fields only
        for key in token_keys:
            if key in config:
                value = config[key]
                if isinstance(value, str) and value:
                    stop_tokens.add(value)
                elif isinstance(value, dict):
                    # Some tokenizers use {"content": "token_value"} format
                    if "content" in value:
                        stop_tokens.add(value["content"])
                elif isinstance(value, list):
                    for item in value:
                        if isinstance(item, str) and item:
                            stop_tokens.add(item)
    
    return sorted(stop_tokens)


def generate_rust_struct(name, template_content, stop_tokens=None):
    """
    Wraps the template in a Rust TemplateEntry struct.
    """
    # Escape double quotes or use raw string if possible
    # We use r#"..."# style.
    # If the template contains "#, we need more hashes (e.g., r##"..."##)
    # The terminating sequence is "# (quote then hashes), so check for that
    hashes = "#"
    while f'"{hashes}' in template_content:
        hashes += "#"
    
    # Format stop tokens - each on its own line for readability
    if stop_tokens:
        tokens_lines = ",\n        ".join(f'"{t}"' for t in stop_tokens)
        stop_tokens_block = f"stop_tokens: &[\n        {tokens_lines},\n    ],"
    else:
        stop_tokens_block = "stop_tokens: &[], // TODO: Fill manually or use --model-dir"
        
    rust_code = f"""use crate::core::template_registry::TemplateEntry;

pub const TEMPLATE: TemplateEntry = TemplateEntry {{
    name: "{name}",
    template: r{hashes}"{template_content}"{hashes},
    {stop_tokens_block}
    force_bos: false,
}};
"""
    return rust_code


def verify_template(template_content):
    """
    Verifies the template using the minijinja Python bindings.
    """
    try:
        import minijinja
    except ImportError:
        sys.stderr.write("\nNOTE: 'minijinja' python package not found. Skipping verification.\n")
        sys.stderr.write("To enable verification: pip install minijinja\n")
        return

    sys.stderr.write("\n--- Verifying with Minijinja (Python bindings) ---\n")
    env = minijinja.Environment()
    
    # Register common dummy filters/functions to avoid false positives on missing features
    # (The user can extend this list if their templates use custom rust filters)
    env.add_filter("tojson", lambda x, **kwargs: str(x)) # Dummy implementation
    env.add_filter("trim", lambda x: x.strip())
    
    # DEBUG: Help user debug installation
    # sys.stderr.write(f"DEBUG: Environment dir: {dir(env)}\n")

    try:
        # 1. Syntax / Render
        # Minijinja Python API seems to validly use .render_str() for quick rendering
        # or .compile_template(name, source) ?
        
        # Based on typical usage:
        # tmpl = env.from_string(source) -> Might be missing in some versions?
        
        # Let's try rendering directly to check syntax + runtime
        # This covers both steps in one go.
        
        ctx = {
            "messages": [
                {"role": "user", "content": "Hello"},
                {"role": "assistant", "content": "Hi there"},
            ],
            "bos_token": "<s>",
            "eos_token": "</s>",
            "add_generation_prompt": True,
            "tools": [], # Mock tools
        }

        # Try render_str if available (often simplest entry point)
        if hasattr(env, 'render_str'):
             env.render_str(template_content, **ctx)
             sys.stderr.write("[OK] Syntax & Render check passed (via render_str).\n")
             return

        # Fallback to loading via add_template (if get_template failed before, maybe API is different)
        # Some versions use loader. 
        # But allow fallback to printing methods if all else fails.
        if hasattr(env, 'from_string'):
             tmpl = env.from_string(template_content)
             tmpl.render(**ctx)
             sys.stderr.write("[OK] Syntax & Render check passed.\n")
             return
             
        # If we are here, we are lost on API.
        sys.stderr.write(f"[FAIL] Could not find render method. Available: {dir(env)}\n")

    except Exception as e:
        sys.stderr.write(f"[FAIL] Verification FAILED: {e}\n")


def main():
    parser = argparse.ArgumentParser(
        description="Convert Jinja2 templates to Minijinja Rust format",
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="""
Examples:
  # Basic conversion
  python jinja_converter.py chat_template.jinja --name "llama3" --rust
  
  # With automatic stop token extraction
  python jinja_converter.py chat_template.jinja --name "llama3" --rust --model-dir /path/to/model
  
  # Output to file
  python jinja_converter.py chat_template.jinja --name "llama3" --rust --model-dir ./model > llama3.rs
"""
    )
    parser.add_argument("input_file", nargs='?', help="Input template file (or stdin)")
    parser.add_argument("--name", "-n", default=None, help="Name for the TemplateEntry (default: folder name)")
    parser.add_argument("--rust", "-r", action="store_true", help="Output as Rust struct")
    parser.add_argument("--output", "-o", default=None, help="Output file path (default: {name}.rs when --rust)")
    parser.add_argument("--no-verify", action="store_true", help="Skip minijinja validation")
    parser.add_argument("--model-dir", "-m", help="Path to model directory for extracting stop tokens and templates")
    
    args = parser.parse_args()
    
    content = ""
    
    # Auto-detect name from model_dir if not specified
    if args.name is None:
        if args.model_dir:
            args.name = os.path.basename(os.path.normpath(args.model_dir))
        elif args.input_file:
            # Use input filename without extension
            args.name = os.path.splitext(os.path.basename(args.input_file))[0]
        else:
            args.name = "unknown"
    
    # Priority 1: Explicit input file
    if args.input_file:
        with open(args.input_file, 'r', encoding='utf-8') as f:
            content = f.read()
        sys.stderr.write(f"--- Using template from: {args.input_file} ---\n")
        
    # Priority 2: Look for template files in model directory
    elif args.model_dir:
        # 2a: .jinja files
        jinja_files = [f for f in os.listdir(args.model_dir) if f.endswith('.jinja')]
        
        if jinja_files:
            # Use first .jinja file found (or chat_template.jinja if exists)
            jinja_file = "chat_template.jinja" if "chat_template.jinja" in jinja_files else jinja_files[0]
            jinja_path = os.path.join(args.model_dir, jinja_file)
            with open(jinja_path, 'r', encoding='utf-8') as f:
                content = f.read()
            sys.stderr.write(f"--- Using template from: {jinja_file} ---\n")
        else:
            # 2b: .json files with chat_template field (e.g., chat_template.json)
            json_files = [f for f in os.listdir(args.model_dir) 
                         if f.endswith('.json') and 'chat_template' in f.lower() and f != 'tokenizer_config.json']
            
            template_found = False
            for json_file in json_files:
                json_path = os.path.join(args.model_dir, json_file)
                try:
                    with open(json_path, 'r', encoding='utf-8') as f:
                        config = json.load(f)
                    if "chat_template" in config:
                        content = config["chat_template"]
                        sys.stderr.write(f"--- Using chat_template from: {json_file} ---\n")
                        template_found = True
                        break
                except (json.JSONDecodeError, IOError):
                    continue
            
            # 2c: tokenizer_config.json as fallback
            if not template_found:
                tokenizer_config_path = os.path.join(args.model_dir, "tokenizer_config.json")
                if os.path.exists(tokenizer_config_path):
                    try:
                        with open(tokenizer_config_path, 'r', encoding='utf-8') as f:
                            config = json.load(f)
                        if "chat_template" in config:
                            content = config["chat_template"]
                            sys.stderr.write(f"--- Using chat_template from tokenizer_config.json ---\n")
                        else:
                            sys.stderr.write(f"ERROR: No 'chat_template' field in {tokenizer_config_path}\n")
                            return
                    except (json.JSONDecodeError, IOError) as e:
                        sys.stderr.write(f"ERROR: Could not parse {tokenizer_config_path}: {e}\n")
                        return
                else:
                    sys.stderr.write(f"ERROR: No .jinja/.json template files and no tokenizer_config.json in {args.model_dir}\n")
                    return
                
    # Priority 4: stdin
    else:
        content = sys.stdin.read()
        
    if not content:
        sys.stderr.write("ERROR: No template content provided\n")
        return

    fixed, has_warning = convert_to_minijinja(content)
    
    # Extract stop tokens if model directory provided
    stop_tokens = []
    if args.model_dir:
        stop_tokens = extract_stop_tokens(args.model_dir, args.name)
        if stop_tokens:
            sys.stderr.write(f"\n--- Extracted stop tokens from configs ---\n")
            for token in stop_tokens:
                sys.stderr.write(f"  - {repr(token)}\n")
        else:
            sys.stderr.write(f"\n--- No stop tokens found in {args.model_dir} ---\n")
    
    if args.rust:
        output_content = generate_rust_struct(args.name, fixed, stop_tokens)
    else:
        output_content = fixed
    
    # Determine output destination
    if args.output:
        output_path = args.output
    elif args.rust:
        # Auto-generate output filename from name
        output_path = f"{args.name}.rs"
    else:
        output_path = None
    
    # Write output
    if output_path:
        with open(output_path, 'w', encoding='utf-8') as f:
            f.write(output_content)
        sys.stderr.write(f"\n--- Written to: {output_path} ---\n")
    else:
        print(output_content)
        
    if has_warning:
        sys.stderr.write("\nWARNING: Template uses 'raise_exception'. Ensure validation function is registered in Minijinja env.\n")

    if not args.no_verify:
        verify_template(fixed)

if __name__ == "__main__":
    main()

