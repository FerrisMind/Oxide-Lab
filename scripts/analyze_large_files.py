#!/usr/bin/env python3
"""
Script to find all files larger than 200 lines of code for refactoring analysis.
Focuses on TypeScript, Svelte, Rust, and JavaScript files in the Oxide-Lab project.
"""

import os
import glob
from pathlib import Path
from typing import List, Tuple, Dict

def count_lines_in_file(file_path: str) -> int:
    """Count the number of lines in a file, excluding empty lines and comments."""
    try:
        with open(file_path, 'r', encoding='utf-8', errors='ignore') as f:
            lines = f.readlines()
        
        # Count non-empty lines (basic filtering)
        non_empty_lines = [line.strip() for line in lines if line.strip()]
        return len(non_empty_lines)
    except Exception as e:
        print(f"Error reading {file_path}: {e}")
        return 0

def is_code_file(file_path: str) -> bool:
    """Check if the file is a code file we want to analyze."""
    code_extensions = {
        '.ts', '.js', '.svelte', '.rs', '.tsx', '.jsx',
        '.py', '.html', '.css', '.scss', '.sass', '.vue',
        '.go', '.java', '.cpp', '.c', '.h', '.hpp',
        '.cs', '.php', '.rb', '.swift', '.kt', '.dart'
    }
    return Path(file_path).suffix.lower() in code_extensions

def find_large_files(directory: str, min_lines: int = 200) -> List[Tuple[str, int]]:
    """Find all code files larger than min_lines across the entire project."""
    large_files = []
    
    # Directories to exclude from analysis (keep minimal exclusions)
    exclude_dirs = {
        'node_modules',
        'target',
        'dist',
        'build',
        '.git',
        '.svelte-kit',
        '.vscode',
        '__pycache__',
        '.pytest_cache',
        'example'  # Exclude external Candle framework directory
    }
    
    # Files to exclude (temporary/generated files)
    exclude_files = {
        'package-lock.json',
        'yarn.lock',
        'Cargo.lock',
        '.gitignore',
        'README.md',
        'LICENSE'
    }
    
    # Walk through entire project directory
    for root, dirs, files in os.walk(directory):
        # Remove excluded directories from dirs list to prevent walking into them
        dirs[:] = [d for d in dirs if d not in exclude_dirs]
        
        for file in files:
            file_path = os.path.join(root, file)
            
            # Skip excluded files
            if file in exclude_files:
                continue
                
            if is_code_file(file_path):
                line_count = count_lines_in_file(file_path)
                if line_count >= min_lines:
                    relative_path = os.path.relpath(file_path, directory)
                    large_files.append((relative_path, line_count))
    
    return sorted(large_files, key=lambda x: x[1], reverse=True)

def categorize_files_by_size(files: List[Tuple[str, int]]) -> Dict[str, List[Tuple[str, int]]]:
    """Categorize files by size ranges for refactoring priority."""
    categories = {
        'Critical (500+ lines)': [],
        'High Priority (300-499 lines)': [],
        'Medium Priority (200-299 lines)': []
    }
    
    for file_path, line_count in files:
        if line_count >= 500:
            categories['Critical (500+ lines)'].append((file_path, line_count))
        elif line_count >= 300:
            categories['High Priority (300-499 lines)'].append((file_path, line_count))
        else:
            categories['Medium Priority (200-299 lines)'].append((file_path, line_count))
    
    return categories

def analyze_file_complexity(file_path: str) -> Dict[str, any]:
    """Analyze file complexity for refactoring recommendations."""
    try:
        with open(file_path, 'r', encoding='utf-8', errors='ignore') as f:
            content = f.read()
        
        # Basic complexity metrics
        analysis = {
            'functions_count': content.count('function ') + content.count('const ') + content.count('let '),
            'classes_count': content.count('class ') + content.count('interface '),
            'imports_count': content.count('import '),
            'exports_count': content.count('export '),
            'comments_ratio': (content.count('//') + content.count('/*')) / max(content.count('\n'), 1)
        }
        return analysis
    except:
        return {}

def generate_refactoring_recommendations(file_path: str, line_count: int, analysis: Dict) -> List[str]:
    """Generate specific refactoring recommendations based on file analysis."""
    recommendations = []
    
    if line_count > 500:
        recommendations.append("URGENT: Split into multiple modules")
    
    if analysis.get('functions_count', 0) > 20:
        recommendations.append("Extract functions into separate utility modules")
    
    if analysis.get('classes_count', 0) > 5:
        recommendations.append("Consider splitting classes into separate files")
    
    if analysis.get('imports_count', 0) > 15:
        recommendations.append("Review dependencies - may indicate tight coupling")
    
    if file_path.endswith('.svelte') and line_count > 300:
        recommendations.append("Split Svelte component into smaller components")
    
    if file_path.endswith('.ts') and 'controller' in file_path.lower():
        recommendations.append("Apply controller pattern - separate concerns")
    
    if analysis.get('comments_ratio', 0) < 0.1:
        recommendations.append("Add more documentation and comments")
    
    return recommendations

def main():
    """Main function to analyze the Oxide-Lab project."""
    # Get the project root (parent directory of scripts)
    script_dir = os.path.dirname(os.path.abspath(__file__))
    project_root = os.path.dirname(script_dir)  # Go up one level from scripts/ to project root
    
    # Create logs directory in project root if it doesn't exist
    logs_dir = os.path.join(project_root, 'logs')
    os.makedirs(logs_dir, exist_ok=True)
    
    print(f"Analyzing entire project: {project_root}")
    print("Searching through ALL directories (excluding external 'example' folder)...")
    print("Focusing on main application code only...")
    print(f"Results will be saved to: {logs_dir}")
    print("=" * 80)
    
    # Find all large files across entire project
    large_files = find_large_files(project_root, min_lines=200)
    
    # Also show breakdown by file type
    file_types = {}
    for file_path, line_count in large_files:
        ext = Path(file_path).suffix.lower()
        if ext not in file_types:
            file_types[ext] = []
        file_types[ext].append((file_path, line_count))
    
    if not large_files:
        print("No files found with more than 200 lines!")
        return
    
    print(f"Found {len(large_files)} files with 200+ lines\n")
    
    # Categorize by priority
    categories = categorize_files_by_size(large_files)
    
    # Generate detailed report
    for category, files in categories.items():
        if files:
            print(f"\n🔴 {category}")
            print("-" * 60)
            
            for file_path, line_count in files:
                print(f"\n📁 {file_path}")
                print(f"   Lines: {line_count}")
                
                # Analyze complexity
                full_path = os.path.join(project_root, file_path)
                analysis = analyze_file_complexity(full_path)
                
                if analysis:
                    print(f"   Functions/Variables: ~{analysis.get('functions_count', 0)}")
                    print(f"   Classes/Interfaces: {analysis.get('classes_count', 0)}")
                    print(f"   Imports: {analysis.get('imports_count', 0)}")
                
                # Generate recommendations
                recommendations = generate_refactoring_recommendations(file_path, line_count, analysis)
                if recommendations:
                    print("   🔧 Refactoring suggestions:")
                    for rec in recommendations:
                        print(f"      • {rec}")
    
    # Summary statistics
    print(f"\n\n📊 SUMMARY")
    print("=" * 80)
    total_lines = sum(line_count for _, line_count in large_files)
    print(f"Total files requiring refactoring: {len(large_files)}")
    print(f"Total lines in large files: {total_lines:,}")
    print(f"Average file size: {total_lines // len(large_files) if large_files else 0} lines")
    
    # Show breakdown by file type
    print(f"\n📁 BREAKDOWN BY FILE TYPE:")
    print("-" * 40)
    for ext, files in sorted(file_types.items()):
        total_lines_for_type = sum(line_count for _, line_count in files)
        print(f"{ext}: {len(files)} files, {total_lines_for_type:,} total lines")
        # Show largest file of this type
        largest = max(files, key=lambda x: x[1])
        print(f"   └─ Largest: {largest[0]} ({largest[1]} lines)")
    
    # Top priority files
    print(f"\n🎯 TOP PRIORITY FOR REFACTORING:")
    print("-" * 40)
    for i, (file_path, line_count) in enumerate(large_files[:5], 1):
        print(f"{i}. {file_path} ({line_count} lines)")
    
    # Export to file in logs directory
    output_file = os.path.join(logs_dir, 'refactoring_analysis.txt')
    detailed_output_file = os.path.join(logs_dir, 'refactoring_detailed_analysis.txt')
    
    # Generate summary report
    with open(output_file, 'w', encoding='utf-8') as f:
        f.write("OXIDE-LAB REFACTORING ANALYSIS\n")
        f.write("=" * 50 + "\n\n")
        f.write(f"Analysis Date: {__import__('datetime').datetime.now().strftime('%Y-%m-%d %H:%M:%S')}\n")
        f.write(f"Total files requiring refactoring: {len(large_files)}\n")
        f.write(f"Total lines in large files: {total_lines:,}\n")
        f.write(f"Average file size: {total_lines // len(large_files) if large_files else 0} lines\n\n")
        
        for category, files in categories.items():
            if files:
                f.write(f"{category}\n")
                f.write("-" * len(category) + "\n")
                for file_path, line_count in files:
                    f.write(f"{file_path}: {line_count} lines\n")
                f.write("\n")
    
    # Generate detailed report with recommendations
    with open(detailed_output_file, 'w', encoding='utf-8') as f:
        f.write("OXIDE-LAB DETAILED REFACTORING ANALYSIS\n")
        f.write("=" * 60 + "\n\n")
        f.write(f"Analysis Date: {__import__('datetime').datetime.now().strftime('%Y-%m-%d %H:%M:%S')}\n\n")
        
        # Summary
        f.write("SUMMARY\n")
        f.write("-" * 20 + "\n")
        f.write(f"Total files requiring refactoring: {len(large_files)}\n")
        f.write(f"Total lines in large files: {total_lines:,}\n")
        f.write(f"Average file size: {total_lines // len(large_files) if large_files else 0} lines\n\n")
        
        # File type breakdown
        f.write("BREAKDOWN BY FILE TYPE\n")
        f.write("-" * 30 + "\n")
        for ext, files in sorted(file_types.items()):
            total_lines_for_type = sum(line_count for _, line_count in files)
            f.write(f"{ext}: {len(files)} files, {total_lines_for_type:,} total lines\n")
            largest = max(files, key=lambda x: x[1])
            f.write(f"   Largest: {largest[0]} ({largest[1]} lines)\n")
        f.write("\n")
        
        # Detailed analysis by category
        for category, files in categories.items():
            if files:
                f.write(f"{category}\n")
                f.write("=" * len(category) + "\n")
                
                for file_path, line_count in files:
                    f.write(f"\nFile: {file_path}\n")
                    f.write(f"Lines: {line_count}\n")
                    
                    # Analyze complexity
                    full_path = os.path.join(project_root, file_path)
                    analysis = analyze_file_complexity(full_path)
                    
                    if analysis:
                        f.write(f"Functions/Variables: ~{analysis.get('functions_count', 0)}\n")
                        f.write(f"Classes/Interfaces: {analysis.get('classes_count', 0)}\n")
                        f.write(f"Imports: {analysis.get('imports_count', 0)}\n")
                    
                    # Generate recommendations
                    recommendations = generate_refactoring_recommendations(file_path, line_count, analysis)
                    if recommendations:
                        f.write("Refactoring suggestions:\n")
                        for rec in recommendations:
                            f.write(f"  • {rec}\n")
                    f.write("-" * 40 + "\n")
                f.write("\n")
    
    print(f"\n💾 Summary analysis saved to: {output_file}")
    print(f"💾 Detailed analysis saved to: {detailed_output_file}")

if __name__ == "__main__":
    main()