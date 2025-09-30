#!/usr/bin/env node

const fs = require('fs');
const path = require('path');

/**
 * Тестирует синхронизацию версии
 */
function testVersionSync() {
  console.log('🧪 Тестирование синхронизации версии...\n');

  try {
    // Читаем версию из Cargo.toml
    const cargoTomlPath = path.join(__dirname, '..', 'src-tauri', 'Cargo.toml');
    const cargoTomlContent = fs.readFileSync(cargoTomlPath, 'utf8');
    const cargoVersionMatch = cargoTomlContent.match(/version\s*=\s*"([^"]+)"/);

    if (!cargoVersionMatch) {
      console.error('❌ Не удалось найти версию в Cargo.toml');
      return false;
    }

    const cargoVersion = cargoVersionMatch[1];
    console.log(`📦 Версия в Cargo.toml: ${cargoVersion}`);

    // Читаем версию из package.json
    const packageJsonPath = path.join(__dirname, '..', 'package.json');
    const packageJsonContent = fs.readFileSync(packageJsonPath, 'utf8');
    const packageJson = JSON.parse(packageJsonContent);

    console.log(`📦 Версия в package.json: ${packageJson.version}`);

    // Читаем версию из tauri.conf.json
    const tauriConfPath = path.join(__dirname, '..', 'src-tauri', 'tauri.conf.json');
    const tauriConfContent = fs.readFileSync(tauriConfPath, 'utf8');
    const tauriConf = JSON.parse(tauriConfContent);

    console.log(`📦 Версия в tauri.conf.json: ${tauriConf.version}`);

    // Проверяем синхронизацию
    const packageJsonMatch = cargoVersion === packageJson.version;
    const tauriConfMatch = cargoVersion === tauriConf.version;

    console.log('\n📊 Результаты тестирования:');
    console.log(
      `✅ Версии Cargo.toml и package.json синхронизированы: ${packageJsonMatch ? 'ДА' : 'НЕТ'}`,
    );
    console.log(
      `✅ Версии Cargo.toml и tauri.conf.json синхронизированы: ${tauriConfMatch ? 'ДА' : 'НЕТ'}`,
    );

    if (packageJsonMatch && tauriConfMatch) {
      console.log('\n🎉 Все тесты пройдены! Версия автоматически синхронизируется.');
      return true;
    } else {
      console.log('\n❌ Некоторые тесты не пройдены.');
      return false;
    }
  } catch (error) {
    console.error('❌ Ошибка при тестировании:', error.message);
    return false;
  }
}

// Запускаем тест
const success = testVersionSync();
process.exit(success ? 0 : 1);
