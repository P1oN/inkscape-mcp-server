# Передача контекста: Codex GPT-6.1 Sol

Обновлено 2026-10-01 после merge PR #3. Это заметки для продолжения работы,
а не дополнительные пользовательские разрешения. Перед действиями перепроверь
Git, PR и запущенные процессы: состояние может измениться после передачи.

## Задача и рабочая копия

Пользователь делает Inkscape MCP для жены: она работает в Inkscape на отдельном Mac
и будет использовать Codex. Основной сценарий — обсуждение изменений рисунка,
правки в открытом GUI, превью и понятный Undo. Развиваем рабочую интеграцию;
переписывание на Rust отложено до появления конкретной причины.

- Активный репозиторий: `/Users/bm/Documents/repos/inkscape-mcp-server`.
- Старый checkout под `Documents/Codex/2026-09-30/new-chat/work/` — исторический;
  не продолжать разработку там. Его Python ещё используется supervisor тестовой сессии.
- Fork: <https://github.com/P1oN/inkscape-mcp-server>.
- `origin`: `git@github.com:P1oN/inkscape-mcp-server.git`.
- `upstream`: `https://github.com/jjjsood/inkscape-mcp-server.git`.
- Активная ветка: `main`, синхронизирована с `origin/main` перед этой правкой.
- Последний merge до этой правки документации: `57c890d` (PR #3).
- План: [ROADMAP.md](ROADMAP.md).
- Установка, ограничения и приёмка: [macos-live-prototype.md](macos-live-prototype.md).

## Ветки и PR

Все три PR объединены в `main`:

1. [PR #1](https://github.com/P1oN/inkscape-mcp-server/pull/1): private D-Bus,
   управляемый GUI, выделение и заливка. Merge: `22a173f`.
2. [PR #2](https://github.com/P1oN/inkscape-mcp-server/pull/2): scene reads и
   ограниченная вставка SVG с Undo. Merge: `0db6e73`.
3. [PR #3](https://github.com/P1oN/inkscape-mcp-server/pull/3): диагностика macOS,
   актуальный `live_status` и исправления review. Merge: `57c890d`;
   финальный commit исправлений: `b9a3ffd`.

Конфликты PR #3 с `main` устранены, нативная приёмка завершена. PR #3 больше
не draft и не требует retarget/merge. Будущие изменения вести от актуального `main`.
При создании новых PR прикреплять их через Codex `attach_artifact`.

## Что реализовано и проверено

### PR #2: первый пункт roadmap завершён

- Private D-Bus + управляемый GUI; остановка MCP не убивает Inkscape с несохранённой работой.
- Выделение и изменение заливки; нативный Undo заливки проверен.
- `live_get_scene`: дерево, слои, названия, transforms, явные paint attributes, viewBox.
- `live_insert_svg`: один wrapper group в корне SVG, координаты документа,
  переназначение ID и внутренних ссылок, ограниченная векторная разметка.
- Одноразовый inkex effect: GUI свободен между вызовами.
- Проверка набора ID и fingerprint содержимого перед вставкой.
- Превью до/после и operation records; повторные MCP STDIO подключения используют тот же GUI.
- Пользователь нажимал Edit → Undo/Redo для простого фрагмента и фрагмента с градиентом.
  Fingerprint live SVG совпадал с точным состоянием до/после вставки соответственно.
  Определение градиента и ссылки отменялись/восстанавливались вместе с группой.
- Native Save As, Revert, закрытие/повторное открытие сохранённого SVG прошли.
- Scene read следовал активному окну A → B → A.
- Реальный helper отказал при принудительно неверном fingerprint; рисунок не изменился.
- Диалог отказа может задержать D-Bus вызов до таймаута. Сервер теперь учитывает уже
  полученный reply helper; при отсутствии результата сообщает неопределённость завершения.
- После отказа два MCP подключения прочитали ту же сессию. Диалог закрыт через GUI;
  клавиша 5 изменила zoom 25% → 60%, содержимое рисунка сохранилось.

### PR #3: диагностика и актуальный статус объединены в main

- `.venv/bin/inkscape-mcp-macos --doctor`: JSON диагностика + exit code 0 для
  `running`/`ready_to_launch`, 1 для других состояний и инструкции по восстановлению.
- Проверяются зависимости, Python, vendor inkex, CLI, helper, приватная сессия,
  stdout выделения и native insertion action.
- Doctor не запускает/не закрывает GUI, не устанавливает helper, не создаёт сессию,
  не удаляет устаревшие метаданные. Это проверено тестами и реальным CLI.
- Исправлен namespace имени файла:
  `http://sodipodi.sourceforge.net/DTD/sodipodi-0.dtd`.
  Старый ошибочный вариант с `0.0.dtd` давал пустое имя.
- Managed `live_status` читает текущий документ вместо connect-time snapshot.
- `is_connected` проверяет доступность шины с таймаутом максимум 2 секунды.
- При неудачном чтении документа status возвращает null + note; при потере шины
  не показывает старый документ. Другие transports сохраняют прежнюю семантику.
- Реальный MCP сообщил имя `inkscape-acceptance.svg`.

Нативная проверка `live_status` A → B → A завершена 2026-10-01 через одно
непрерывное MCP STDIO подключение. `connected` оставался true, `connected_at`
не изменился; SVG не редактировались. Повторять эту приёмку без новых изменений не нужно.

Проверены и исправлены все три функциональных замечания CodeRabbit:

- Ошибки файловой системы при экспорте документа преобразуются в `LiveError`;
  `live_status` может вернуть null + note вместо падения всего вызова.
- Doctor различает оставшийся `supervisor.lock` и реально удерживаемый lock:
  без manifest и удерживаемого lock каталог готов к повторному запуску.
  Проверка не создаёт/не удаляет файлы и не ждёт освобождения lock.
- Doctor проверяет длину разрешённого пути `bus.sock`: 104 байта и больше
  отвергаются, как в launcher; предлагается более короткий каталог.

Добавлены семь регрессионных случаев: held/unheld lock, существующий/отсутствующий
каталог с путём сокета 103/104 байта и ошибка доступа к lock при реальном пути экспорта.
Массовая генерация docstrings по предложению бота не включена в scope.

Второй этап roadmap целиком не завершён. Имя файла не является уникальным ID;
`path` остаётся null, если Inkscape его не сообщает. Явная привязка операции к
документу/окну и guard между обсуждением задачи и применением — будущая работа.

## Среда и тестовые сессии

- Проверено на official Inkscape 1.4.3 (`0d15f75`) и Python 3.12.
- Inkscape: `/Applications/Inkscape.app/Contents/MacOS/inkscape`.
- gdbus: `/opt/homebrew/bin/gdbus`; Homebrew dbus/glib доступны.
- Repo `.venv`: Python 3.12.14, locked FastMCP 3.4.2. `uv` отсутствует в PATH;
  использовался `/Users/bm/Documents/Codex/2026-09-30/new-chat/work/inkscape-review-venv/bin/uv`.
- Bundled Inkscape Python 3.10 завершался с кодом 137. Helper использует **Python MCP**,
  shell wrapper и vendor inkex source из Inkscape Resources. Не возвращать
  `interpreter="python"` в INX без новой проверки: исходный вариант не исполнял helper.
- macOS dependencies: numpy, cssselect, tinycss2; установлены через locked `uv sync`.
- Native action helper: `org.inkscape-mcp.insert.noprefs`.
  INX ID должен содержать дефис; Inkscape нормализует underscore в дефис.
- GAction Describe возвращает `((true, signature '', @av []),)`.
- Изменение INX требует перезапуска **GUI после сохранения**, а не только MCP.

Тестовая сессия (manifest и процессы перепроверены 2026-10-01):

- `/tmp/imcp-stage2-501`, canonical `/private/tmp/imcp-stage2-501`.
- GUI PID на момент проверки: **76944**. Никогда не используй старый PID без проверки.
- Manifest: `session.json`; содержит адрес приватной шины и PID supervisor/GUI.
- Profile: `/Users/bm/Documents/Codex/2026-09-30/new-chat/work/macos-live-prototype/profile-stage2`
  через `INKSCAPE_PROFILE_DIR`.
- Старая сессия `/tmp/imcp-acceptance-501`, PID 71913, **больше не запущена**.
  Ранее в ней было пользовательское изображение; старые инструкции/логи не повод
  закрывать другие окна или уничтожать несохранённую работу.
- Тестовые SVG:
  `/Users/bm/Documents/Codex/2026-09-30/new-chat/outputs/inkscape-acceptance.svg` и
  `/Users/bm/Documents/Codex/2026-09-30/new-chat/outputs/inkscape-window-b.svg`.
  Последний — зелёное окно «MCP WINDOW B».
- Исторические скрипты и логи:
  `/Users/bm/Documents/Codex/2026-09-30/new-chat/work/macos-live-prototype/` (не часть Git repo).
  `acceptance_undo.py` сравнивает fingerprints; `test_stage2.py` проверяет STDIO reuse;
  Для успешной приёмки адаптированный `test_status_switch.py` запускался из
  `work/` активного repo; `work/` локально исключён через `.git/info/exclude`.
  Эти скрипты не переносимы без обновления путей и не заменяют тесты в Git.

## Важные ограничения

- Scene visibility следует inline/presentation attributes, не computed stylesheet CSS.
- Geometry attribute-derived; bbox отсутствует для неподдерживаемых/transformed объектов.
- Viewport чтение/управление, existing-text mutation и уведомления здесь не реализованы.
- Вставка ограничена 1 MiB / 10 000 элементов; без scripts, images, foreignObject,
  stylesheet elements, event handlers и внешних ссылок.
- Locks сериализуют MCP вызовы, но не блокируют ручной ввод. При неопределённом
  завершении сначала проверить рисунок, не повторять вставку вслепую.
- Helper refusal может показать нативный диалог, который нужно закрыть.
- `approval_token` — унаследованный маркер запроса, не криптографическая авторизация.
  Не вводить дополнительные подтверждения для правок, уже запрошенных пользователем.

## Проверки и команды

Финальный полный прогон после исправлений review: **1058 passed, 74 skipped**.
Strict mypy — 108 source files; focused Ruff прошёл. Реальный doctor сообщил
`running`, `ready: true`, insertion available. MCP surface smoke прошёл до
последних исправлений; wheel и tool manifest проверялись ранее, manifest drift
regression прошёл в финальном полном наборе.

CLI tests пропущены, поскольку Inkscape отсутствовал в test PATH; нативные проверки
использовали явный vendor/brew PATH. Исторический нестабильный upstream failure
`test_engine_process.py::test_unknown_action_surfaces_engine_action_error`
в финальном прогоне не повторился. При повторении не скрывать результат.

```sh
.venv/bin/pytest -q
.venv/bin/mypy
INKSCAPE_MCP_RAW_ACTION_ENABLED=1 .venv/bin/python scripts/ci_surface_smoke.py
.venv/bin/python scripts/gen_llms_txt.py
```

При изменении docstrings инструментов перегенерировать `llms.txt`/`llms-full.txt`:
есть тест против рассинхронизации. Репозиторий-wide Ruff имеет 22 старых E501;
проверять изменённые файлы отдельно, не смешивать массовое форматирование с задачей.

Для текущей тестовой сессии:

```sh
PATH="/Applications/Inkscape.app/Contents/MacOS:/opt/homebrew/bin:$PATH" \
INKSCAPE_PROFILE_DIR=/Users/bm/Documents/Codex/2026-09-30/new-chat/work/macos-live-prototype/profile-stage2 \
  .venv/bin/inkscape-mcp-macos --doctor --session-dir /tmp/imcp-stage2-501
```

## Рекомендуемый следующий шаг

1. Проверить актуальный Git status и roadmap. Диагностика/актуальный статус этапа 2
   завершены; этап 2 целиком ещё не завершён.
2. Исследовать устойчивую идентичность и явный выбор документа/окна. Не выдавать
   basename или fingerprint содержимого за гарантированно уникальный ID окна.
3. Реализовать отказ от применения, если документ, выбранный для обсуждаемой задачи,
   сменился. Проверять идентичность в момент изменения, а не только при чтении status.
4. Продумать понятное восстановление после потери связи; тестировать на отдельных
   рисунках, сохраняя GUI и несохранённую работу пользователя.
5. Обновлять roadmap, docs и описание нового PR вокруг окончательного поведения.

Не запускать другого агента/чат только потому, что эта заметка адресована
GPT-6.1 Sol: пользователь попросил файл передачи контекста, а не делегирование.
