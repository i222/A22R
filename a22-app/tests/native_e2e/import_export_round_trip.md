# Import/export round trip / Цикл импорта и экспорта

## English

The scenario creates an item, exports the complete v2 dataset through the
native save dialog, verifies the JSON file, deletes the item, and imports the
file through the native open dialog. It then verifies that the item is visible
again and closes the application cleanly.

The shared harness keeps the database in an isolated temporary profile and
writes native file-dialog artifacts to the ignored project-root
`native-e2e-artifacts/` directory.

macOS note: the Open panel is a separate native dialog. Depending on its
remembered view mode, the exported file appears as a list-row `tree_item` or
an icon-group. The harness resolves the dialog through the application's
native windows and traverses that dialog directly instead of relying on an
application-root locator.

## Русский

Сценарий создаёт элемент, экспортирует полный набор данных v2 через штатный
диалог сохранения, проверяет JSON-файл, удаляет элемент и импортирует файл
через штатный диалог открытия. Затем он проверяет, что элемент снова виден, и
штатно закрывает приложение.

Общий harness хранит базу в изолированном временном профиле, а файлы native
диалогов сохраняет в игнорируемом каталоге `native-e2e-artifacts/` в корне
проекта.

Примечание для macOS: Open panel — отдельный native-диалог. В зависимости от
сохранённого режима просмотра экспортированный файл представлен либо строкой
`tree_item`, либо группой с иконкой. Harness получает сам диалог через native
windows приложения и обходит его напрямую, не полагаясь на locator от корня
приложения.
