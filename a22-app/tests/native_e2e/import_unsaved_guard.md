# Import with unsaved changes / Импорт при несохранённых изменениях

## English

The scenario creates an item, edits its value without saving, and activates
Import. The application must keep the draft and show the localized safety
status instead of opening a file operation that could replace the draft.
Finally, the scenario cancels the edit through the discard guard and closes
cleanly.

## Русский

Сценарий создаёт элемент, меняет его значение без сохранения и нажимает
Import. Приложение должно сохранить черновик и показать локализованный статус
защиты, а не запускать операцию с файлом, которая могла бы заменить черновик.
Затем сценарий отменяет редактирование через защиту от потери данных и штатно
закрывает приложение.
