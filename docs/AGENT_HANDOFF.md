# Продолжение работы

Обновлено 2026-10-01. Это индекс состояния, не дополнительные разрешения.
Перед действиями проверяй Git/PR/процессы; не закрывай GUI с несохранённой работой.

## Текущее состояние main

PR #4 (document context, `bfe9e4f`) и PR #5 (everyday edits, `e6e4e80`) объединены в `main`.
Следующий коммит `d948a25` добавил явный запуск Inkscape и переносимое определение session directory.
Origin: https://github.com/P1oN/inkscape-mcp-server.

- MCP startup и `live_connect` не открывают окно. `live_launch()` запускает или использует
  managed GUI только по явному запросу пользователя. Терминальный вариант:
  `.venv/bin/inkscape-mcp-macos --launch`; начальный SVG требует `--launch --document ...`.
- Уже открытый managed GUI и несохранённая работа сохраняются между MCP-подключениями.
  Закрытое окно не открывается заново при reconnect. Для обновления helper/native bridge:
  сохранить и закрыть старое окно, явно запустить новое, затем подключиться.
- Рабочий порядок: `live_connect(prefer="no_freeze")`, `live_list_documents`,
  `live_select_document`, проверка `live_status.ready_to_edit`, чтение сцены/выделения и правки.
  Каждый reconnect сбрасывает привязку task drawing. Окно из Finder не является managed session.
- Managed macOS поддерживает заливку/обводку/прозрачность, document-space transforms,
  простой однострочный текст и duplicate/delete/group/ungroup/raise/lower/front/back.
  Один изменяющий вызов — один Undo; неизменяющий шаг не добавляет Undo.
- Правки готовятся на копии SVG в one-shot inkex effect; проверяются контекст/выделение,
  блокировки и ссылки. Таймаут/неподтверждённый результат сообщает неопределённость:
  проверить рисунок перед повтором. Native integration остаётся экспериментальной.
- Инструкции: [macOS setup](macos-live-prototype.md), [document context](document-context.md),
  [everyday edits](everyday-edits.md). Актуальный полный manifest: [llms.txt](../llms.txt)
  (103 инструмента; видимая поверхность зависит от gates).

## Проверки и доказательства

Проверка документации 2026-10-01: полный pytest — **1142 passed, 74 skipped**
(Inkscape CLI отсутствовал в тестовом PATH). Ruff, format check и strict mypy
(111 source files) прошли. Исторические результаты этапов 2/3
в тематических документах не являются текущим статусом тестов.

Native acceptance этапа 3 после исправлений ревью прошёл на official Inkscape 1.4.3
через настоящий MCP STDIO на разблокированном Mac. Проверены семейства правок, точные
отпечатки Undo/Redo, неизменяющие вызовы, блокировки/неверный выбор текста,
guard/race/stale-binding и STDIO reuse.
Доказательство: `/private/tmp/imcp-context-r79j3lvj/acceptance.json` (`passed: true`),
`stage3-*.svg` и preview PNG в том же каталоге. Эти временные файлы могут быть уже удалены.

Команда воспроизведения: `.venv/bin/python scripts/accept_document_context.py`.
Она явно запускает отдельный тестовый GUI с `--launch`, затем проверяет MCP reconnect
без launch. Успех закрывает только два проверенных синтетических окна; ошибка сохраняет GUI.
Native GUI acceptance не запускался в ходе проверки документации.

## Сессии и следующий объем

Историческая пользовательская сессия `/tmp/imcp-stage2-501` не закрывалась в предыдущих задачах;
это не утверждение, что она сейчас работает. Не использовать сохранённые PID:
перепроверять manifest и command line. Не завершать процессы по имени Inkscape.

Этап 3 уже объединён. Следующий функциональный объем — этап 4 (понимание рисунка)
и испытания на копиях реальных иллюстраций; он не входит в текущую задачу.
Общий roadmap — [ROADMAP.md](ROADMAP.md).
