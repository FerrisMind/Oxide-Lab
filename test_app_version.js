// Тестовый скрипт для проверки команды получения версии приложения
// Запускается в браузере через консоль разработчика

async function testAppVersion() {
  try {
    // Проверяем, что Tauri API доступен
    if (typeof window.__TAURI__ === 'undefined') {
      console.log('⚠️ Tauri API недоступен. Это может быть веб-версия приложения.');
      return;
    }

    // Импортируем и вызываем команду
    const { invoke } = await import('@tauri-apps/api/core');
    const appInfo = await invoke('get_app_info');

    console.log('✅ Команда get_app_info работает корректно');
    console.log('Информация о приложении:', appInfo);

    // Проверяем, что версия извлечена корректно
    if (appInfo.version && appInfo.version !== '0.0.0') {
      console.log('✅ Версия извлечена корректно:', appInfo.version);
    } else {
      console.log('❌ Версия не найдена или некорректная:', appInfo.version);
    }

    return appInfo;
  } catch (error) {
    console.error('❌ Ошибка при вызове команды get_app_info:', error);
    return null;
  }
}

// Автоматический тест при загрузке страницы
if (typeof window !== 'undefined') {
  window.testAppVersion = testAppVersion;

  // Запускаем тест через небольшую задержку
  setTimeout(() => {
    console.log('🚀 Запуск теста команды get_app_info...');
    testAppVersion().then((result) => {
      if (result) {
        console.log('🎉 Тест завершен успешно!');
      } else {
        console.log('💥 Тест завершен с ошибками');
      }
    });
  }, 1000);
}
