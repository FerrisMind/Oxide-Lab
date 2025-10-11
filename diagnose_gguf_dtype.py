#!/usr/bin/env python3
"""
Диагностический скрипт для анализа GGUF файла и определения типов данных тензоров.
Помогает выявить неподдерживаемые типы данных в моделях.
"""

import sys
import struct
from pathlib import Path

def get_gguf_type_size(type_id):
    """Возвращает размер в байтах для типа GGUF."""
    type_sizes = {
        0: 1,   # uint8
        1: 1,   # int8
        2: 2,   # uint16
        3: 2,   # int16
        4: 4,   # uint32
        5: 4,   # int32
        6: 4,   # float32
        7: 1,   # bool
        8: None,  # string (переменный размер)
        9: None,  # array (переменный размер)
        10: 8,  # uint64
        11: 8,  # int64
        12: 8,  # float64
    }
    return type_sizes.get(type_id, 0)

def read_gguf_header(file_path):
    """Читает заголовок GGUF файла и возвращает основную информацию."""
    f = open(file_path, 'rb')

    # Читаем magic number
    magic = f.read(4)
    if magic != b'GGUF':
        print(f"[!] Не является GGUF файлом. Magic: {magic}")
        f.close()
        return None

    # Определяем версию и порядок байтов
    version_bytes = f.read(4)
    version = struct.unpack('<I', version_bytes)[0] if len(version_bytes) == 4 else 0

    # Читаем количество тензоров и метаданных
    tensor_count_bytes = f.read(8)
    metadata_count_bytes = f.read(8)

    if len(tensor_count_bytes) != 8 or len(metadata_count_bytes) != 8:
        print("[!] Поврежденный заголовок GGUF файла")
        f.close()
        return None

    tensor_count = struct.unpack('<Q', tensor_count_bytes)[0]
    metadata_count = struct.unpack('<Q', metadata_count_bytes)[0]

    print("[+] GGUF файл валиден")
    print(f"   Версия: {version}")
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
    max_key_len = 1024 * 1024  # Максимум 1MB для ключа
    max_value_len = 10 * 1024 * 1024  # Максимум 10MB для строковых значений

    for i in range(count):
        # Читаем ключ метаданных
        key_len_bytes = file_handle.read(8)
        if len(key_len_bytes) != 8:
            break

        key_len = struct.unpack('<Q', key_len_bytes)[0]

        # Валидация размера ключа
        if key_len > max_key_len or key_len == 0:
            print(f"[!] Неверный размер ключа метаданных: {key_len} байт (макс: {max_key_len})")
            break

        try:
            key_bytes = file_handle.read(key_len)
            if len(key_bytes) != key_len:
                print("[!] Недостаточно данных для чтения ключа")
                break
            key = key_bytes.decode('utf-8', errors='ignore')
        except MemoryError:
            print(f"[!] MemoryError при чтении ключа размером {key_len} байт")
            break

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
            str_len_bytes = file_handle.read(8)
            if len(str_len_bytes) != 8:
                break
            str_len = struct.unpack('<Q', str_len_bytes)[0]

            # Валидация размера строки
            if str_len > max_value_len or str_len == 0:
                print(f"[!] Неверный размер строкового значения: {str_len} байт (макс: {max_value_len})")
                value = f"<truncated string, len={str_len}>"
                file_handle.seek(str_len, 1)  # Пропускаем данные
            else:
                try:
                    str_bytes = file_handle.read(str_len)
                    if len(str_bytes) != str_len:
                        print("[!] Недостаточно данных для чтения строки")
                        break
                    value = str_bytes.decode('utf-8', errors='ignore')
                except MemoryError:
                    print(f"[!] MemoryError при чтении строки размером {str_len} байт")
                    value = f"<memory error, len={str_len}>"
                    file_handle.seek(str_len, 1)  # Пропускаем данные
        elif value_type == 9:  # array
            array_type_bytes = file_handle.read(4)
            if len(array_type_bytes) != 4:
                break
            array_type = struct.unpack('<I', array_type_bytes)[0]

            array_len_bytes = file_handle.read(8)
            if len(array_len_bytes) != 8:
                break
            array_len = struct.unpack('<Q', array_len_bytes)[0]

            # Валидация размера массива
            max_array_len = 1000000  # Максимум 1M элементов
            if array_len > max_array_len:
                print(f"[!] Слишком большой массив: {array_len} элементов (макс: {max_array_len})")
                value = f"Array[type={array_type}, len={array_len}] <truncated>"
                # Для больших массивов просто пропускаем, не пытаемся рассчитать размер
                # Это небезопасно, но лучше чем MemoryError
                break
            else:
                value = f"Array[type={array_type}, len={array_len}]"
                # Пропускаем данные массива
                element_size = get_gguf_type_size(array_type)
                if element_size is not None and element_size > 0:
                    skip_size = array_len * element_size
                    # Проверяем, что skip_size не слишком большой
                    max_skip_size = 100 * 1024 * 1024  # 100MB максимум
                    if skip_size > max_skip_size:
                        print(f"[!] Слишком большой массив для пропуска: {skip_size} байт")
                        break
                    file_handle.seek(skip_size, 1)
                else:
                    # Для строковых массивов или неизвестных типов пропускаем
                    # Это небезопасно, но предотвращает MemoryError
                    print(f"[!] Неизвестный тип массива {array_type}, пропускаем метаданные")
                    break
        elif value_type == 10:  # uint64
            value = struct.unpack('<Q', file_handle.read(8))[0]
        elif value_type == 11:  # int64
            value = struct.unpack('<q', file_handle.read(8))[0]
        elif value_type == 12:  # float64
            value = struct.unpack('<d', file_handle.read(8))[0]
        else:
            print(f"[!] Неизвестный тип метаданных: {value_type}")
            return metadata

        metadata[key] = value

    return metadata

def read_tensors_info(file_handle, count):
    """Читает информацию о тензорах из GGUF файла."""
    tensors = []
    max_name_len = 1024  # Максимум 1KB для имени тензора

    for i in range(count):
        # Читаем имя тензора
        name_len_bytes = file_handle.read(8)
        if len(name_len_bytes) != 8:
            print(f"[!] Недостаточно данных для чтения имени тензора {i}")
            break

        name_len = struct.unpack('<Q', name_len_bytes)[0]

        # Валидация размера имени
        if name_len > max_name_len or name_len == 0:
            print(f"[!] Неверный размер имени тензора {i}: {name_len} байт (макс: {max_name_len})")
            # Пропускаем этот тензор и пытаемся продолжить
            try:
                # Пропускаем имя (если возможно)
                if name_len <= 1024 * 1024:  # Не более 1MB
                    file_handle.seek(name_len, 1)
                # Пропускаем остальные поля тензора (n_dims + dimensions + dtype + offset)
                file_handle.seek(8 + 4 + 8, 1)  # Примерный размер, может быть неточным
            except:
                pass
            continue

        try:
            name_bytes = file_handle.read(name_len)
            if len(name_bytes) != name_len:
                print(f"[!] Недостаточно данных для чтения имени тензора {i}")
                break
            name = name_bytes.decode('utf-8', errors='ignore')
        except MemoryError:
            print(f"[!] MemoryError при чтении имени тензора {i} размером {name_len} байт")
            # Пропускаем этот тензор
            continue

        try:
            # Читаем количество измерений
            n_dims_bytes = file_handle.read(4)
            if len(n_dims_bytes) != 4:
                print(f"[!] Недостаточно данных для чтения n_dims тензора {i}")
                break

            n_dims = struct.unpack('<I', n_dims_bytes)[0]

            # Валидация количества измерений
            if n_dims > 10 or n_dims == 0:
                print(f"[!] Неверное количество измерений тензора {i}: {n_dims}")
                continue

            # Читаем размерности
            dimensions = []
            for j in range(n_dims):
                dim_bytes = file_handle.read(8)
                if len(dim_bytes) != 8:
                    print(f"[!] Недостаточно данных для чтения dimension {j} тензора {i}")
                    dimensions = []
                    break
                dimensions.append(struct.unpack('<Q', dim_bytes)[0])

            if not dimensions:
                continue

            # Читаем тип данных тензора
            dtype_bytes = file_handle.read(4)
            if len(dtype_bytes) != 4:
                print(f"[!] Недостаточно данных для чтения dtype тензора {i}")
                break

            dtype = struct.unpack('<I', dtype_bytes)[0]

            # Читаем смещение данных тензора
            offset_bytes = file_handle.read(8)
            if len(offset_bytes) != 8:
                print(f"[!] Недостаточно данных для чтения offset тензора {i}")
                break

            offset = struct.unpack('<Q', offset_bytes)[0]

            tensors.append({
                'name': name,
                'n_dims': n_dims,
                'dimensions': dimensions,
                'dtype': dtype,
                'offset': offset
            })

        except Exception as e:
            print(f"[!] Ошибка при чтении тензора {i}: {e}")
            continue

    return tensors

def analyze_gguf_file(file_path):
    """Полный анализ GGUF файла."""
    print(f"\n[*] Анализ GGUF файла: {file_path}")
    print("=" * 60)

    # Читаем заголовок
    header_info = read_gguf_header(file_path)
    if not header_info:
        return

    file_handle = header_info['file']

    try:
        # Читаем метаданные
        print(f"\n[+] Чтение метаданных ({header_info['metadata_count']} элементов)...")
        metadata = read_metadata(file_handle, header_info['metadata_count'])

        print(f"Найдено метаданных: {len(metadata)}")
        for key, value in metadata.items():
            if "token" in key.lower() or "model" in key.lower() or "arch" in key.lower():
                print(f"   {key}: {value}")

        # Читаем информацию о тензорах
        print(f"\n[+] Чтение информации о тензорах ({header_info['tensor_count']} элементов)...")
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

        print("\n[+] Статистика типов данных:")
        for dtype, count in sorted(dtype_counts.items()):
            print(f"   Тип {dtype}: {count} тензоров")

        # Ищем тензор 39 и анализируем проблемные тензоры
        problematic_tensors = []
        for i, tensor in enumerate(tensors):
            if tensor['dtype'] not in [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29]:
                problematic_tensors.append((i, tensor))

        if problematic_tensors:
            print(f"\n[!] Найдено {len(problematic_tensors)} тензоров с неизвестными типами данных:")
            for i, tensor in problematic_tensors[:5]:  # Показываем первые 5
                try:
                    name_preview = tensor['name'][:50] if len(tensor['name']) > 50 else tensor['name']
                    print(f"   Тензор {i}: {name_preview}... тип={tensor['dtype']}")
                except UnicodeEncodeError:
                    print(f"   Тензор {i}: <имя с некорректной кодировкой>... тип={tensor['dtype']}")
                except:
                    print(f"   Тензор {i}: <ошибка отображения имени>... тип={tensor['dtype']}")

        # Ищем тензор 39
        if len(tensors) > 39:
            tensor_39 = tensors[39]
            print(f"\n[*] Тензор 39 (вызывает ошибку в Candle):")
            print(f"   Имя: {tensor_39['name']}")
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

            if tensor_39['dtype'] >= 30:  # Совсем неизвестные типы
                print(f"   [ERROR] Этот тип данных не поддерживается даже новейшими версиями Candle")
                print(f"   [ERROR] Файл может быть поврежден или использовать экспериментальный формат")
            elif tensor_39['dtype'] >= 16:  # Новые типы IQ
                print(f"   [WARNING] Это новый тип данных IQ (Intelligent Quantization)")
                print(f"   [WARNING] Возможно, требуется обновление Candle до последней версии")

    finally:
        file_handle.close()

def main():
    if len(sys.argv) != 2:
        print("Использование: python diagnose_gguf_dtype.py <путь_к_gguf_файлу>")
        sys.exit(1)

    file_path = Path(sys.argv[1])

    if not file_path.exists():
        print(f"[!] Файл не найден: {file_path}")
        sys.exit(1)

    if not file_path.suffix.lower() == '.gguf':
        print(f"[!] Файл не является GGUF файлом: {file_path}")
        sys.exit(1)

    analyze_gguf_file(file_path)

if __name__ == "__main__":
    main()
