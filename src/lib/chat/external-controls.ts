import { mount, unmount } from 'svelte';
import Eye from 'phosphor-svelte/lib/Eye';
import EyeSlash from 'phosphor-svelte/lib/EyeSlash';

/**
 * Создает внешнюю кнопку просмотра исходного кода для сообщения ассистента
 * @param messageEl - элемент сообщения (.message)
 * @param mdStreamEl - элемент md-stream для переключения режима
 * @returns объект с элементами кнопки
 */
export function createExternalViewButton(
  messageEl: HTMLElement,
  mdStreamEl: HTMLElement,
): {
  button: HTMLButtonElement;
  icon: any;
  destroy: () => void;
} {
  // Проверяем, не существует ли уже кнопка
  const existingButton = messageEl.querySelector('.external-view-button');
  if (existingButton) {
    existingButton.remove();
  }

  // Создаем контейнер для кнопки
  const buttonContainer = document.createElement('div');
  buttonContainer.className = 'external-view-container';

  // Создаем кнопку
  const button = document.createElement('button');
  button.type = 'button';
  button.className = 'external-view-button';
  button.title = 'Показать исходный код';

  // Создаем контейнер для иконки
  const iconContainer = document.createElement('span');
  iconContainer.className = 'external-view-icon';

  button.appendChild(iconContainer);
  buttonContainer.appendChild(button);

  // Добавляем кнопку после bubble
  messageEl.appendChild(buttonContainer);

  // Монтируем иконку
  let icon = mount(Eye, {
    target: iconContainer,
    props: { size: 16, weight: 'regular' },
  });

  // Обработчик клика
  button.addEventListener('click', () => {
    const showingRaw = mdStreamEl.classList.toggle('show-raw');

    // Переключаем иконку
    try {
      unmount(icon);
    } catch {}

    icon = mount(showingRaw ? EyeSlash : Eye, {
      target: iconContainer,
      props: { size: 16, weight: 'regular' },
    });
  });

  // Функция для удаления кнопки
  const destroy = () => {
    try {
      unmount(icon);
    } catch {}
    buttonContainer.remove();
  };

  return { button, icon, destroy };
}

/**
 * Удаляет внешнюю кнопку просмотра из сообщения
 * @param messageEl - элемент сообщения
 */
export function removeExternalViewButton(messageEl: HTMLElement): void {
  const buttonContainer = messageEl.querySelector('.external-view-container');
  if (buttonContainer) {
    buttonContainer.remove();
  }
}
