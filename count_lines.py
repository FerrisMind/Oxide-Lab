#!/usr/bin/env python3
"""
Скрипт для подсчета строк кода в проекте Oxide-Lab.
Подсчитывает строки кода на различных языках программирования.
"""

import os
import sys
from pathlib import Path
from typing import Dict, List, Tuple


class CodeCounter:
    """Класс для подсчета строк кода в проекте."""

    # Расширения файлов для различных языков
    LANGUAGE_EXTENSIONS = {
        'Rust': ['.rs'],
        'TypeScript': ['.ts'],
        'JavaScript': ['.js'],
        'Svelte': ['.svelte'],
        'Python': ['.py'],
        'HTML': ['.html'],
        'CSS': ['.css'],
        'TOML': ['.toml'],
        'JSON': ['.json'],
        'Markdown': ['.md'],
        'YAML': ['.yml', '.yaml'],
        'Shell': ['.sh', '.bash'],
        'PowerShell': ['.ps1'],
        'SQL': ['.sql'],
    }

    # Директории, которые нужно исключить из подсчета
    EXCLUDE_DIRS = {
        'node_modules',
        'models',
        'example',
        'target',
        '.git',
        '.github',
        '__pycache__',
        '.cache',
        '.cursor',
        'dist',
        'build',
        '.next',
        '.nuxt',
        '.output',
        '.vercel',
        '.netlify',
        'coverage',
        '.nyc_output',
        '.pytest_cache',
        '.tox',
        '.eggs',
        '*.egg-info',
        '*.lock',
        '.DS_Store',
        'Thumbs.db',
        'schemas',
        'package-lock.json',
    }

    def __init__(self, root_path: str):
        self.root_path = Path(root_path).resolve()
        self.stats: Dict[str, Dict[str, int]] = {}
        self.total_lines = 0
        self.total_files = 0

    def should_exclude_dir(self, dir_path: Path) -> bool:
        """Проверяет, нужно ли исключить директорию из подсчета."""
        dir_name = dir_path.name
        return dir_name in self.EXCLUDE_DIRS or dir_name.startswith('.')

    def get_language_from_extension(self, file_path: Path) -> str:
        """Определяет язык программирования по расширению файла."""
        extension = file_path.suffix.lower()
        for language, extensions in self.LANGUAGE_EXTENSIONS.items():
            if extension in extensions:
                return language
        return 'Other'

    def count_lines_in_file(self, file_path: Path) -> Tuple[int, int]:
        """
        Подсчитывает строки в файле.
        Возвращает: (общее_количество_строк, количество_непустых_строк)
        """
        try:
            with open(file_path, 'r', encoding='utf-8', errors='ignore') as f:
                lines = f.readlines()
                total_lines = len(lines)
                non_empty_lines = sum(1 for line in lines if line.strip())
                return total_lines, non_empty_lines
        except (IOError, OSError) as e:
            print(f"Warning: Could not read {file_path}: {e}", file=sys.stderr)
            return 0, 0

    def count_lines_in_directory(self, dir_path: Path) -> None:
        """Рекурсивно подсчитывает строки кода в директории."""
        if not dir_path.exists():
            print(f"Error: Directory {dir_path} does not exist", file=sys.stderr)
            return

        for root, dirs, files in os.walk(dir_path):
            root_path = Path(root)

            # Исключаем нежелательные директории
            dirs[:] = [d for d in dirs if not self.should_exclude_dir(root_path / d)]

            for file in files:
                file_path = root_path / file

                # Пропускаем скрытые файлы
                if file.startswith('.'):
                    continue

                language = self.get_language_from_extension(file_path)
                total_lines, non_empty_lines = self.count_lines_in_file(file_path)

                if total_lines > 0:
                    if language not in self.stats:
                        self.stats[language] = {'files': 0, 'total_lines': 0, 'code_lines': 0}

                    self.stats[language]['files'] += 1
                    self.stats[language]['total_lines'] += total_lines
                    self.stats[language]['code_lines'] += non_empty_lines

                    self.total_files += 1
                    self.total_lines += non_empty_lines

    def print_report(self) -> None:
        """Выводит отчет о подсчете строк кода."""
        print("=" * 70)
        print(f"АНАЛИЗ КОДА ПРОЕКТА: {self.root_path.name}")
        print("=" * 70)
        print()

        if not self.stats:
            print("Файлы кода не найдены!")
            return

        # Сортируем языки по количеству строк кода (убывание)
        sorted_languages = sorted(
            self.stats.items(),
            key=lambda x: x[1]['code_lines'],
            reverse=True
        )

        print(f"{'Язык':<15} {'Файлов':<8} {'Всего строк':<12} {'Код строк':<12} {'Процент':<8}")
        print("-" * 70)

        for language, stats in sorted_languages:
            files = stats['files']
            total_lines = stats['total_lines']
            code_lines = stats['code_lines']
            percentage = (code_lines / self.total_lines * 100) if self.total_lines > 0 else 0

            print(f"{language:<15} {files:<8} {total_lines:<12} {code_lines:<12} {percentage:<8.1f}%")

        print("-" * 70)
        print(f"{'ИТОГО':<15} {self.total_files:<8} {'-':<12} {self.total_lines:<12} {'100.0%':<8}")
        print()

        # Дополнительная информация
        print("ДОПОЛНИТЕЛЬНАЯ ИНФОРМАЦИЯ:")
        print(f"- Проанализировано файлов: {self.total_files}")
        print(f"- Общее количество строк кода: {self.total_lines}")
        print(f"- Среднее количество строк на файл: {self.total_lines / self.total_files:.1f}" if self.total_files > 0 else "- Среднее количество строк на файл: 0")
        print()

        # Топ-5 файлов по размеру
        print("ТОП-5 НАИБОЛЬШИХ ФАЙЛОВ:")
        file_sizes = []
        for root, dirs, files in os.walk(self.root_path):
            root_path = Path(root)
            dirs[:] = [d for d in dirs if not self.should_exclude_dir(root_path / d)]

            for file in files:
                file_path = root_path / file
                if file.startswith('.'):
                    continue

                language = self.get_language_from_extension(file_path)
                if language != 'Other':
                    total_lines, code_lines = self.count_lines_in_file(file_path)
                    if code_lines > 0:
                        file_sizes.append((file_path, code_lines))

        file_sizes.sort(key=lambda x: x[1], reverse=True)
        for file_path, lines in file_sizes[:5]:
            relative_path = file_path.relative_to(self.root_path)
            print(f"  {relative_path} ({lines} строк)")


def main():
    """Главная функция скрипта."""
    if len(sys.argv) > 1:
        project_path = sys.argv[1]
    else:
        project_path = "."  # Текущая директория по умолчанию

    project_path = Path(project_path).resolve()

    if not project_path.exists():
        print(f"Ошибка: Путь {project_path} не существует!", file=sys.stderr)
        sys.exit(1)

    if not project_path.is_dir():
        print(f"Ошибка: {project_path} не является директорией!", file=sys.stderr)
        sys.exit(1)

    print(f"Анализирую проект: {project_path}")
    print("Пожалуйста, подождите...")
    print()

    counter = CodeCounter(str(project_path))
    counter.count_lines_in_directory(project_path)
    counter.print_report()


if __name__ == "__main__":
    main()
