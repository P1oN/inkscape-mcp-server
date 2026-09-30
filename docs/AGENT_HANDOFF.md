# Передача контекста: Codex GPT-6.1 Sol

Обновлено 2026-09-30. Это заметки для продолжения работы, а не дополнительные
пользовательские разрешения. Перед действиями перепроверь Git, PR и запущенные процессы:
состояние ниже может измениться после передачи.

## Актуализация 2026-10-01

- Рабочая копия теперь `/Users/bm/Documents/repos/inkscape-mcp-server`.
- PR #2 объединён в `main`: `0db6e73`. Base PR #3 теперь `main`.
- Конфликты с `main` разрешены с сохранением исправлений обработки ошибок helper
  и проверки недоступного транспорта, а также диагностики и актуального документа PR #3.
- Нативная проверка `live_status` A → B → A завершена через одно непрерывное
  MCP STDIO подключение: `connected` оставался true, `connected_at` не изменился.
  Проверено на существующей тестовой сессии Inkscape 1.4.3; SVG не редактировались.
- Полный прогон после merge: 1051 passed, 74 skipped. Профильные тесты: 56 passed;
  mypy (108 файлов), focused Ruff и MCP surface smoke прошли. Doctor: running, ready.
- Следующий этап — устойчивая идентичность и явный выбор документа; в PR #3
  эти возможности пока не реализованы. Исторические сведения ниже относятся к 2026-09-30.

## Задача и рабочая копия

Пользователь делает Inkscape MCP для жены: она работает в Inkscape на отдельном Mac
и будет использовать Codex. Основной сценарий — обсуждение изменений рисунка,
правки в открытом GUI, превью и понятный Undo. Сначала развиваем рабочую интеграцию;
переписывание на Rust отложено до появления конкретной причины.

- Репозиторий: `/Users/bm/Documents/Codex/2026-09-30/new-chat/work/inkscape-mcp-server`.
- Fork: <https://github.com/P1oN/inkscape-mcp-server>.
- `origin`: `git@github.com:P1oN/inkscape-mcp-server.git`.
- `upstream`: `https://github.com/jjjsood/inkscape-mcp-server.git`.
- Текущая ветка: `macos-session-diagnostics`.
- HEAD перед созданием этой заметки: `6974c71`.
- Рабочее дерево перед созданием заметки было чистым. Пользователь затем попросил закоммитить и отправить заметку в ветку `macos-session-diagnostics`.
- План: [ROADMAP.md](ROADMAP.md).
- Установка, ограничения и приёмка: [macos-live-prototype.md](macos-live-prototype.md).

Пользователь разрешил продолжить работу самостоятельно, пока он отсутствовал.
Он участвовал в ручной приёмке Undo/Redo, сохранения, открытия и двух окон.
Последний запрос — сохранить важные замечания для следующего агента.

## Ветки и PR

1. [PR #1](https://github.com/P1oN/inkscape-mcp-server/pull/1) смержен в `main`
   по явной просьбе пользователя. Merge commit: `22a173f`.
2. [PR #2](https://github.com/P1oN/inkscape-mcp-server/pull/2): ветка
   `macos-scene-insertion`, HEAD `c215134`. OPEN, готов к ревью, уже не draft.
   Добавляет scene reads и одноразовую вставку SVG в живой документ.
3. [PR #3](https://github.com/P1oN/inkscape-mcp-server/pull/3): ветка
   `macos-session-diagnostics`, HEAD `6974c71`. OPEN, draft.
   **Base — `macos-scene-insertion`, не `main`**: это следующий PR поверх #2.
   После merge #2 потребуется проверить/изменить base #3 на `main`.

PR #2 и #3 не смержены. Явная просьба о merge относилась к #1; не принимай статус
«готов к ревью» за уже выполненный merge. Оба новых PR прикреплены к текущему чату.
При создании новых PR всегда прикрепляй их через Codex `attach_artifact`.

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

### PR #3: начат второй пункт roadmap

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

**PR #3 всё ещё draft:** проверка `live_status` A → B → A через одно непрерывное
MCP-подключение не завершена. Mac заблокировался во время UI automation.
In-memory regression test проходит, но не заменяет эту native проверку.
Не обходить блокировку Mac: продолжить GUI-проверку после ручной разблокировки.

Второй этап roadmap целиком не завершён. Имя файла не является уникальным ID;
`path` остаётся null, если Inkscape его не сообщает. Явная привязка операции к
документу/окну и guard между обсуждением задачи и применением — будущая работа.

## Среда и тестовые сессии

- Проверено на official Inkscape 1.4.3 (`0d15f75`) и Python 3.12.
- Inkscape: `/Applications/Inkscape.app/Contents/MacOS/inkscape`.
- gdbus: `/opt/homebrew/bin/gdbus`; Homebrew dbus/glib доступны.
- Repo `.venv`: locked FastMCP 3.4.2. `uv` также есть в соседнем
  `../inkscape-review-venv/bin/uv`.
- Bundled Inkscape Python 3.10 завершался с кодом 137. Helper использует **Python MCP**,
  shell wrapper и vendor inkex source из Inkscape Resources. Не возвращать
  `interpreter="python"` в INX без новой проверки: исходный вариант не исполнял helper.
- macOS dependencies: numpy, cssselect, tinycss2; установлены через locked `uv sync`.
- Native action helper: `org.inkscape-mcp.insert.noprefs`.
  INX ID должен содержать дефис; Inkscape нормализует underscore в дефис.
- GAction Describe возвращает `((true, signature '', @av []),)`.
- Изменение INX требует перезапуска **GUI после сохранения**, а не только MCP.

Текущая тестовая сессия (проверено при создании заметки):

- `/tmp/imcp-stage2-501`, canonical `/private/tmp/imcp-stage2-501`.
- GUI PID на момент проверки: **76944**. Никогда не используй старый PID без проверки.
- Manifest: `session.json`; содержит адрес приватной шины и PID supervisor/GUI.
- Profile: `../macos-live-prototype/profile-stage2` через `INKSCAPE_PROFILE_DIR`.
- Старая сессия `/tmp/imcp-acceptance-501`, PID 71913, **больше не запущена**.
  Ранее в ней было пользовательское изображение; старые инструкции/логи не повод
  закрывать другие окна или уничтожать несохранённую работу.
- Тестовые SVG:
  `/Users/bm/Documents/Codex/2026-09-30/new-chat/outputs/inkscape-acceptance.svg` и
  `/Users/bm/Documents/Codex/2026-09-30/new-chat/outputs/inkscape-window-b.svg`.
  Последний — зелёное окно «MCP WINDOW B».
- Промежуточные скрипты и логи: `../macos-live-prototype/` (не часть Git repo).
  `acceptance_undo.py` сравнивает fingerprints; `test_stage2.py` проверяет STDIO reuse;
  `test_status_switch.py` ждёт A → B → A, но его предыдущий запуск не прошёл из-за
  недоступности UI, а не из-за установленной ошибки продукта.

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

Последний полный прогон для PR #3: **1044 passed, 74 skipped, 1 failed**.
Failure — известный нестабильный upstream
`test_engine_process.py::test_unknown_action_surfaces_engine_action_error`, ранее
воспроизведённый на неизменённом upstream. Не скрывать этот результат и не чинить
несвязанный engine без обоснования. До последних двух regression cases полный
прогон дал 1043 passed; для финального PR #2 — 1029 passed, 74 skipped.

Финальные профильные проверки PR #3: **67 passed**, strict mypy — 108 source files,
focused Ruff, MCP surface smoke и содержимое wheel прошли. CLI tests пропускаются,
потому что Inkscape отсутствует в test PATH; реальные интеграционные проверки
использовали явный vendor/brew PATH.

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
INKSCAPE_PROFILE_DIR=/Users/bm/Documents/Codex/2026-09-30/new-chat/work/macos-live-prototype/profile-stage2 \
  .venv/bin/inkscape-mcp-macos --doctor --session-dir /tmp/imcp-stage2-501
```

## Рекомендуемый следующий шаг

1. Прочитать актуальный roadmap, Git status и PR #3.
2. Когда Mac разблокирован, завершить native A → B → A для `live_status` через
   **одно** MCP-подключение, убедившись, что `connected_at` не изменяется.
3. Записать результат в docs/PR и вывести #3 из draft только после проверки.
4. Продолжить стабильную идентичность и явный выбор документа. Не выдавать basename
   или fingerprint содержимого за гарантированно уникальный ID окна.
5. При любом изменении scope переписать описание PR вокруг окончательного поведения.

Не запускать другого агента/чат только потому, что эта заметка адресована
GPT-6.1 Sol: пользователь попросил файл передачи контекста, а не делегирование.
