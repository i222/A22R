# Multi-item relationship lifecycle / Жизненный цикл нескольких элементов и связей

## English

The scenario creates three items, edits a specific non-selected list row,
creates two outgoing relationships, changes one relationship type, removes the
other relationship, and saves the draft. It then deletes the remaining related
item and verifies that the relationship is removed through the normal
application worker/database cascade.

After closing and restarting the real application process, the scenario
verifies that the edited item and surviving item remain while the deleted item
and both relationship targets are absent from the editor.

## Русский

Сценарий создаёт три элемента, редактирует конкретную строку списка, не
являющуюся текущей выбранной, создаёт две исходящие связи, изменяет тип одной
связи, удаляет другую и сохраняет черновик. Затем он удаляет оставшийся
связанный элемент и проверяет, что связь удалена обычным каскадом через worker и
базу данных.

После закрытия и повторного запуска настоящего процесса приложения сценарий
проверяет, что изменённый и оставшийся элементы сохранились, а удалённый
элемент и обе цели связей отсутствуют в редакторе.
