# Отладка системы истории чатов

## Исправленная проблема

**Проблема**: При переключении между чатами загружались только сообщения пользователя, ответы модели пропадали.

**Причина**: Циклические реактивные обновления между `Chat.svelte` и `chatHistory` store.

**Решение**: Добавлен флаг `isLoadingFromHistory` для предотвращения циклов и правильная последовательность обновлений.

## Как работает синхронизация

### 1. Загрузка сессии при переключении

```typescript
// В Chat.svelte
let isLoadingFromHistory = false;
let lastSessionId: string | null = null;

$: {
  if ($currentSession && $currentSession.id !== lastSessionId) {
    isLoadingFromHistory = true;
    messages = [...$currentSession.messages]; // Новый массив
    lastSessionId = $currentSession.id;

    setTimeout(() => {
      isLoadingFromHistory = false;
    }, 100);
  }
}
```

**Что происходит**:

1. При изменении `currentSession` проверяем, изменился ли ID
2. Устанавливаем флаг `isLoadingFromHistory = true`
3. Копируем сообщения из истории в `messages`
4. Запоминаем ID текущей сессии
5. Через 100мс сбрасываем флаг

### 2. Синхронизация обратно в историю

```typescript
$: {
  if (messages && !isLoadingFromHistory && $currentSession) {
    chatHistory.updateMessages(messages);
  }
}
```

**Что происходит**:

1. Срабатывает при изменении `messages`
2. Проверяем флаг `isLoadingFromHistory` - если он `true`, пропускаем
3. Обновляем сообщения в истории только когда флаг `false`

## Логирование для отладки

### В консоли браузера вы увидите:

При переключении сессии:

```
loadSession: загружаем сессию session-xxx с 10 сообщениями
  - Пользователь: 5, Ассистент: 5
Загружаем сессию: session-xxx Сообщений: 10
```

При обновлении сообщений:

```
Синхронизируем сообщения в историю: 10
updateMessages: обновляем сессию session-xxx с 10 сообщениями
  - Пользователь: 5, Ассистент: 5
```

## Проверка работоспособности

### Шаг 1: Создание тестового чата

1. Откройте приложение
2. Загрузите модель
3. Отправьте несколько сообщений (минимум 3)
4. Дождитесь ответов модели

### Шаг 2: Создание второго чата

1. Откройте панель истории (кнопка с иконкой списка)
2. Нажмите "+" для создания нового чата
3. Отправьте другие сообщения
4. Дождитесь ответов

### Шаг 3: Переключение между чатами

1. В панели истории кликните на первый чат
2. **Проверка**: Должны отобразиться ВСЕ сообщения (и пользователя, и модели)
3. Кликните на второй чат
4. **Проверка**: Снова все сообщения на месте
5. Переключайтесь несколько раз

### Что должно работать

✅ Все сообщения (пользователя и модели) сохраняются
✅ При переключении все сообщения восстанавливаются
✅ Новые сообщения добавляются в текущую сессию
✅ История сохраняется в localStorage

## Проверка localStorage

### Просмотр сохраненных данных

Откройте консоль браузера (F12) и выполните:

```javascript
const data = JSON.parse(localStorage.getItem('oxide-lab-chat-history'));
console.log('Всего сессий:', data.sessions.length);
console.log('Текущая сессия:', data.currentSessionId);

// Просмотр конкретной сессии
const session = data.sessions[0];
console.log('ID:', session.id);
console.log('Название:', session.title);
console.log('Сообщений:', session.messages.length);
console.log('Сообщения:', session.messages);
```

### Проверка структуры сообщений

```javascript
const data = JSON.parse(localStorage.getItem('oxide-lab-chat-history'));
const session = data.sessions[0];

// Подсчет сообщений по ролям
const userMessages = session.messages.filter((m) => m.role === 'user').length;
const assistantMessages = session.messages.filter((m) => m.role === 'assistant').length;

console.log('Пользователь:', userMessages);
console.log('Ассистент:', assistantMessages);

// Вывод всех ролей
session.messages.forEach((msg, i) => {
  console.log(`${i}: ${msg.role} - ${msg.content.substring(0, 50)}...`);
});
```

## Типичные проблемы и решения

### Проблема 1: Сообщения дублируются

**Симптомы**: При каждом переключении количество сообщений удваивается

**Причина**: Циклические обновления не предотвращаются

**Решение**: Убедитесь, что флаг `isLoadingFromHistory` работает корректно

```javascript
// В консоли проверьте
console.log('isLoadingFromHistory:', isLoadingFromHistory);
```

### Проблема 2: Сообщения не сохраняются

**Симптомы**: После перезагрузки страницы история пустая

**Причина**: localStorage заблокирован или переполнен

**Проверка**:

```javascript
try {
  localStorage.setItem('test', 'test');
  console.log('localStorage работает');
  localStorage.removeItem('test');
} catch (e) {
  console.error('localStorage недоступен:', e);
}
```

**Решение**:

- Очистите localStorage
- Проверьте настройки приватности браузера

### Проблема 3: Только сообщения пользователя

**Симптомы**: После переключения видны только вопросы, ответов нет

**Причина**: Сообщения модели не сохраняются при добавлении

**Проверка**:

```javascript
// Отследите момент добавления сообщения модели
// В Chat.svelte должен срабатывать:
console.log('Добавлено сообщение:', message.role, message.content.substring(0, 30));
```

**Решение**: Убедитесь, что при streaming ответа сообщение модели добавляется в `messages`

## Отладочные команды

### Очистка истории

```javascript
localStorage.removeItem('oxide-lab-chat-history');
location.reload();
```

### Создание тестовой истории

```javascript
const testHistory = {
  sessions: [
    {
      id: 'test-session-1',
      title: 'Тестовый чат',
      messages: [
        { role: 'user', content: 'Привет!' },
        { role: 'assistant', content: 'Здравствуйте! Чем могу помочь?' },
        { role: 'user', content: 'Расскажи о себе' },
        { role: 'assistant', content: 'Я AI ассистент, работающий на базе LLM.' },
      ],
      createdAt: Date.now() - 3600000,
      updatedAt: Date.now(),
    },
  ],
  currentSessionId: 'test-session-1',
};

localStorage.setItem('oxide-lab-chat-history', JSON.stringify(testHistory));
location.reload();
```

### Проверка синхронизации в реальном времени

```javascript
// Подписка на изменения chatHistory
import { chatHistory } from '$lib/stores/chat-history';

chatHistory.subscribe((state) => {
  console.log('История обновлена:', {
    sessions: state.sessions.length,
    currentId: state.currentSessionId,
    messages: state.sessions.find((s) => s.id === state.currentSessionId)?.messages.length,
  });
});
```

## Тестовые сценарии

### Сценарий 1: Базовая работа

1. Создать чат → Отправить 3 сообщения
2. Создать второй чат → Отправить 2 сообщения
3. Переключиться на первый → **Проверить**: 6 сообщений (3 пары)
4. Переключиться на второй → **Проверить**: 4 сообщения (2 пары)

### Сценарий 2: Streaming

1. Создать чат
2. Отправить сообщение с длинным ответом
3. Во время streaming переключиться на другой чат
4. Вернуться → **Проверить**: Полный ответ сохранен

### Сценарий 3: Перезагрузка

1. Создать чат с 5 парами сообщений
2. Перезагрузить страницу (F5)
3. **Проверить**: История восстановилась
4. **Проверить**: Все сообщения на месте

### Сценарий 4: Множественные чаты

1. Создать 5 разных чатов
2. В каждом по 2-3 пары сообщений
3. Переключаться между всеми
4. **Проверить**: Каждый чат сохраняет свои сообщения

## Метрики производительности

### Время загрузки сессии

```javascript
console.time('loadSession');
chatHistory.loadSession(sessionId);
console.timeEnd('loadSession');
// Должно быть < 50ms
```

### Размер localStorage

```javascript
const data = localStorage.getItem('oxide-lab-chat-history');
const sizeKB = new Blob([data]).size / 1024;
console.log(`Размер истории: ${sizeKB.toFixed(2)} KB`);
// Должно быть < 5MB для 50 чатов
```

## Известные ограничения

1. **Максимум 50 сессий** - старые автоматически удаляются
2. **localStorage лимит** - обычно 5-10 MB в браузере
3. **Задержка 100ms** - для предотвращения циклов при переключении
4. **Нет синхронизации** - между вкладками браузера (по дизайну)

## Поддержка

При возникновении проблем:

1. Откройте консоль браузера (F12)
2. Скопируйте все логи с префиксами `loadSession`, `updateMessages`, `Загружаем сессию`
3. Проверьте содержимое localStorage (см. выше)
4. Создайте issue с этой информацией
