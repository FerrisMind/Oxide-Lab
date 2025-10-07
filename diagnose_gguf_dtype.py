#!/usr/bin/env python3
"""
Диагностический скрипт для анализа GGUF файла и определения типов данных тензоров.
Помогает выявить неподдерживаемые типы данных в моделях.
"""

import sys
import struct
from pathlib import Path

def read_gguf_header(file_path):
    """Читает заголовок GGUF файла и возвращает основную информацию."""
    with open(file_path, 'rb') as f:
        # Читаем magic number
        magic = f.read(4)
        if magic != b'GGUF':
            print(f"❌ Не является GGUF файлом. Magic: {magic}")
            return None

        # Определяем версию и порядок байтов
        version_bytes = f.read(4)
        version = struct.unpack('<I', version_bytes)[0] if len(version_bytes) == 4 else 0

        # Читаем количество тензоров и метаданных
        tensor_count_bytes = f.read(8)
        metadata_count_bytes = f.read(8)

        if len(tensor_count_bytes) != 8 or len(metadata_count_bytes) != 8:
            print("❌ Поврежденный заголовок GGUF файла")
            return None

        tensor_count = struct.unpack('<Q', tensor_count_bytes)[0]
        metadata_count = struct.unpack('<Q', metadata_count_bytes)[0]

        print("✅ GGUF файл валиден"        print(f"   Версия: {version}")
        print(f"   Количество тензоров: {tensor_count}")
        print(f"   Количество метаданных: {metadata_count}")

        return {
            'version': version,
            'tensor_count': tensor_count,
            'metadata_count': metadata_count,
            'file': f
        }

def read_metadata(file_handle, count):
    """Читает метаданные GGUF файла."""
    metadata = {}

    for i in range(count):
        # Читаем ключ метаданных
        key_len_bytes = file_handle.read(8)
        if len(key_len_bytes) != 8:
            break

        key_len = struct.unpack('<Q', key_len_bytes)[0]
        key = file_handle.read(key_len).decode('utf-8', errors='ignore')

        # Читаем тип значения
        value_type_bytes = file_handle.read(4)
        if len(value_type_bytes) != 4:
            break

        value_type = struct.unpack('<I', value_type_bytes)[0]

        # Читаем значение в зависимости от типа
        if value_type == 0:  # uint8
            value = struct.unpack('<B', file_handle.read(1))[0]
        elif value_type == 1:  # int8
            value = struct.unpack('<b', file_handle.read(1))[0]
        elif value_type == 2:  # uint16
            value = struct.unpack('<H', file_handle.read(2))[0]
        elif value_type == 3:  # int16
            value = struct.unpack('<h', file_handle.read(2))[0]
        elif value_type == 4:  # uint32
            value = struct.unpack('<I', file_handle.read(4))[0]
        elif value_type == 5:  # int32
            value = struct.unpack('<i', file_handle.read(4))[0]
        elif value_type == 6:  # float32
            value = struct.unpack('<f', file_handle.read(4))[0]
        elif value_type == 7:  # bool
            value = bool(struct.unpack('<B', file_handle.read(1))[0])
        elif value_type == 8:  # string
            str_len = struct.unpack('<Q', file_handle.read(8))[0]
            value = file_handle.read(str_len).decode('utf-8', errors='ignore')
        elif value_type == 9:  # array
            array_type = struct.unpack('<I', file_handle.read(4))[0]
            array_len = struct.unpack('<Q', file_handle.read(8))[0]
            # Для простоты пропускаем массивы
            value = f"Array[type={array_type}, len={array_len}]"
            file_handle.seek(array_len, 1)  # Пропускаем данные массива
        elif value_type == 10:  # uint64
            value = struct.unpack('<Q', file_handle.read(8))[0]
        elif value_type == 11:  # int64
            value = struct.unpack('<q', file_handle.read(8))[0]
        elif value_type == 12:  # float64
            value = struct.unpack('<d', file_handle.read(8))[0]
        else:
            print(f"❌ Неизвестный тип метаданных: {value_type}")
            return metadata

        metadata[key] = value

    return metadata

def read_tensors_info(file_handle, count):
    """Читает информацию о тензорах из GGUF файла."""
    tensors = []

    for i in range(count):
        # Читаем имя тензора
        name_len_bytes = file_handle.read(8)
        if len(name_len_bytes) != 8:
            break

        name_len = struct.unpack('<Q', name_len_bytes)[0]
        name = file_handle.read(name_len).decode('utf-8', errors='ignore')

        # Читаем количество измерений
        n_dims_bytes = file_handle.read(4)
        if len(n_dims_bytes) != 4:
            break

        n_dims = struct.unpack('<I', n_dims_bytes)[0]

        # Читаем размерности
        dimensions = []
        for _ in range(n_dims):
            dim_bytes = file_handle.read(8)
            if len(dim_bytes) != 8:
                break
            dimensions.append(struct.unpack('<Q', dim_bytes)[0])

        # Читаем тип данных тензора
        dtype_bytes = file_handle.read(4)
        if len(dtype_bytes) != 4:
            break

        dtype = struct.unpack('<I', dtype_bytes)[0]

        # Читаем смещение данных тензора
        offset_bytes = file_handle.read(8)
        if len(offset_bytes) != 8:
            break

        offset = struct.unpack('<Q', offset_bytes)[0]

        tensors.append({
            'name': name,
            'n_dims': n_dims,
            'dimensions': dimensions,
            'dtype': dtype,
            'offset': offset
        })

    return tensors

def analyze_gguf_file(file_path):
    """Полный анализ GGUF файла."""
    print(f"\n🔍 Анализ GGUF файла: {file_path}")
    print("=" * 60)

    # Читаем заголовок
    header_info = read_gguf_header(file_path)
    if not header_info:
        return

    file_handle = header_info['file']

    # Читаем метаданные
    print(f"\n📋 Чтение метаданных ({header_info['metadata_count']} элементов)...")
    metadata = read_metadata(file_handle, header_info['metadata_count'])

    print(f"Найдено метаданных: {len(metadata)}")
    for key, value in metadata.items():
        if "token" in key.lower() or "model" in key.lower() or "arch" in key.lower():
            print(f"   {key}: {value}")

    # Читаем информацию о тензорах
    print(f"\n🧮 Чтение информации о тензорах ({header_info['tensor_count']} элементов)...")
    tensors = read_tensors_info(file_handle, header_info['tensor_count'])

    print(f"Найдено тензоров: {len(tensors)}")

    # Анализируем типы данных
    dtype_counts = {}
    for tensor in tensors:
        dtype = tensor['dtype']
        if dtype in dtype_counts:
            dtype_counts[dtype] += 1
        else:
            dtype_counts[dtype] = 1

    print("\n📊 Статистика типов данных:")
    for dtype, count in sorted(dtype_counts.items()):
        print(f"   Тип {dtype}: {count} тензоров")

    # Ищем тензор 39
    if len(tensors) > 39:
        tensor_39 = tensors[39]
        print("
🎯 Тензор 39:"        print(f"   Имя: {tensor_39['name']}")
        print(f"   Размерности: {tensor_39['dimensions']}")
        print(f"   Тип данных: {tensor_39['dtype']}")
        print(f"   Смещение: {tensor_39['offset']}")

        # Определяем тип данных
        dtype_names = {
            0: "F32", 1: "F16", 2: "Q4_0", 3: "Q4_1", 4: "Q4_2 (deprecated)",
            5: "Q4_3 (deprecated)", 6: "Q5_0", 7: "Q5_1", 8: "Q8_0", 9: "Q8_1",
            10: "Q2_K", 11: "Q3_K", 12: "Q4_K", 13: "Q5_K", 14: "Q6_K", 15: "Q8_K",
            16: "IQ2_XXS", 17: "IQ2_XS", 18: "IQ3_XXS", 19: "IQ1_S", 20: "IQ4_NL",
            21: "IQ3_S", 22: "IQ2_S", 23: "IQ4_XS", 24: "I8", 25: "I16", 26: "I32",
            27: "I64", 28: "F64", 29: "IQ1_M"
        }

        dtype_name = dtype_names.get(tensor_39['dtype'], f"Неизвестный тип {tensor_39['dtype']}")
        print(f"   Тип данных: {dtype_name}")

        if tensor_39['dtype'] >= 16:  # Новые типы IQ
            print(f"   ⚠️  Это новый тип данных IQ (Intelligent Quantization)")
            print(f"   ⚠️  Возможно, требуется обновление Candle до последней версии")

    file_handle.close()

def main():
    if len(sys.argv) != 2:
        print("Использование: python diagnose_gguf_dtype.py <путь_к_gguf_файлу>")
        sys.exit(1)

    file_path = Path(sys.argv[1])

    if not file_path.exists():
        print(f"❌ Файл не найден: {file_path}")
        sys.exit(1)

    if not file_path.suffix.lower() == '.gguf':
        print(f"❌ Файл не является GGUF файлом: {file_path}")
        sys.exit(1)

    analyze_gguf_file(file_path)

if __name__ == "__main__":
    main()
