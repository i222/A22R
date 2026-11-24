# Many tags and overflow boundary / Границы множества тегов и переполнения

## English

The scenario creates an item and fills its tag editor to the configured
five-tag maximum. A sixth default suggestion is attempted and must not be
committed. The list filter then selects all built-in default hints, verifies
the two-row `+N hidden tags` overflow surface, and checks that an already
selected tag is not offered again as autocomplete.

The application is restarted to verify that the item tags persist while the
list filter remains transient. The test also confirms that the rejected sixth
tag was not stored.

## Русский

Сценарий создаёт элемент и заполняет редактор тегов до настроенного лимита в
пять значений. Затем он пытается добавить шестую дефолтную подсказку; она не
должна попасть в данные. После этого фильтр списка получает все встроенные
дефолтные подсказки, проверяет двухстрочное переполнение `+N hidden tags` и
убеждается, что уже выбранный тег больше не предлагается автодополнением.

Приложение перезапускается для проверки сохранения тегов элемента при
одновременной проверке того, что фильтр списка остаётся временным. Также
проверяется, что отклонённый шестой тег не был записан.
