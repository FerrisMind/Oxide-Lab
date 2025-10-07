#!/usr/bin/env python3
"""
Тестовый скрипт для проверки токенизатора в GGUF файлах.
Этот скрипт поможет диагностировать проблемы с токенизаторами в GGUF файлах.
"""

import sys
import json
from pathlib import Path

def check_gguf_tokenizer(file_path):
    """Проверяет наличие токенизатора в GGUF файле."""
    print(f"Проверка файла: {file_path}")
    
    try:
        # Попробуем импортировать библиотеку для работы с GGUF
        try:
            import gguf
        except ImportError:
            print("❌ Библиотека gguf не установлена. Установите её командой:")
            print("   pip install gguf")
            return False
        
        # Читаем GGUF файл
        reader = gguf.GGUFReader(file_path, "r")
        
        print(f"✅ GGUF файл успешно прочитан")
        print(f"   Версия: {reader.version}")
        print(f"   Количество метаданных: {len(reader.metadata)}")
        print(f"   Количество тензоров: {len(reader.tensors)}")
        
        # Ищем токенизатор в метаданных
        tokenizer_keys = [
            "tokenizer.json",
            "general.tokenizer_json", 
            "qwen3.tokenizer_json",
            "llama.tokenizer_json",
            "gemma.tokenizer_json",
            "tokenizer.ggml.json",
            "tokenizer_json",
            "tokenizer",
            "tokenizer.ggml.tokenizer_json",
            "tokenizer.model"
        ]
        
        found_tokenizer = False
        for key in tokenizer_keys:
            if key in reader.metadata:
                print(f"✅ Найден токенизатор по ключу: {key}")
                found_tokenizer = True
                break
        
        if not found_tokenizer:
            print("❌ Токенизатор не найден в метаданных")
            
            # Проверим наличие токенов
            token_keys = [
                "tokenizer.ggml.tokens",
                "tokenizer.tokens", 
                "tokenizer.vocab",
                "tokenizer.ggml.vocab",
                "vocab",
                "tokens"
            ]
            
            found_tokens = False
            for key in token_keys:
                if key in reader.metadata:
                    tokens = reader.metadata[key]
                    if hasattr(tokens, '__len__'):
                        print(f"✅ Найдены токены по ключу: {key} (количество: {len(tokens)})")
                        found_tokens = True
                        break
            
            if not found_tokens:
                print("❌ Токены не найдены в метаданных")
                
                # Покажем все доступные ключи метаданных
                print("\n📋 Доступные ключи метаданных:")
                for key in sorted(reader.metadata.keys()):
                    if "token" in key.lower() or "vocab" in key.lower():
                        print(f"   🔍 {key}")
        
        # Проверим наличие BPE merges
        merge_keys = [
            "tokenizer.ggml.merges",
            "tokenizer.ggml.bpe_merges", 
            "tokenizer.merges",
            "merges",
            "bpe_merges"
        ]
        
        found_merges = False
        for key in merge_keys:
            if key in reader.metadata:
                merges = reader.metadata[key]
                if hasattr(merges, '__len__'):
                    print(f"✅ Найдены BPE merges по ключу: {key} (количество: {len(merges)})")
                    found_merges = True
                    break
        
        if not found_merges:
            print("ℹ️  BPE merges не найдены (это нормально для некоторых типов токенизаторов)")
        
        return found_tokenizer or found_tokens
        
    except Exception as e:
        print(f"❌ Ошибка при чтении GGUF файла: {e}")
        return False

def main():
    if len(sys.argv) != 2:
        print("Использование: python test_gguf_tokenizer.py <путь_к_gguf_файлу>")
        sys.exit(1)
    
    file_path = Path(sys.argv[1])
    
    if not file_path.exists():
        print(f"❌ Файл не найден: {file_path}")
        sys.exit(1)
    
    if not file_path.suffix.lower() == '.gguf':
        print(f"❌ Файл не является GGUF файлом: {file_path}")
        sys.exit(1)
    
    success = check_gguf_tokenizer(file_path)
    
    if success:
        print("\n✅ Токенизатор найден в GGUF файле!")
    else:
        print("\n❌ Токенизатор не найден в GGUF файле!")
        print("\n💡 Рекомендации:")
        print("   1. Убедитесь, что GGUF файл был создан с встроенным токенизатором")
        print("   2. Попробуйте переконвертировать модель с помощью llama.cpp")
        print("   3. Проверьте, что используется последняя версия инструментов конвертации")

if __name__ == "__main__":
    main()
