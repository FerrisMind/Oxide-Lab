#!/usr/bin/env node

const fs = require('fs');
const path = require('path');

/**
 * Синхронизирует версию из Cargo.toml с package.json
 */
function syncVersion() {
  try {
    // Читаем Cargo.toml
    const cargoTomlPath = path.join(__dirname, '..', 'src-tauri', 'Cargo.toml');
    const cargoTomlContent = fs.readFileSync(cargoTomlPath, 'utf8');

    // Извлекаем версию из Cargo.toml
    const versionMatch = cargoTomlContent.match(/version\s*=\s*"([^"]+)"/);
    if (!versionMatch) {
      console.error('❌ Не удалось найти версию в Cargo.toml');
      process.exit(1);
    }

    const version = versionMatch[1];
    console.log(`📦 Найдена версия в Cargo.toml: ${version}`);

    // Читаем package.json
    const packageJsonPath = path.join(__dirname, '..', 'package.json');
    const packageJsonContent = fs.readFileSync(packageJsonPath, 'utf8');
    const packageJson = JSON.parse(packageJsonContent);

    // Проверяем, нужно ли обновление
    if (packageJson.version === version) {
      console.log('✅ Версии уже синхронизированы');
      return;
    }

    // Обновляем версию в package.json
    packageJson.version = version;

    // Записываем обновленный package.json
    fs.writeFileSync(packageJsonPath, JSON.stringify(packageJson, null, 2) + '\n');

    console.log(`✅ Версия в package.json обновлена: ${version}`);
  } catch (error) {
    console.error('❌ Ошибка при синхронизации версии:', error.message);
    process.exit(1);
  }
}

// Запускаем синхронизацию
syncVersion();
