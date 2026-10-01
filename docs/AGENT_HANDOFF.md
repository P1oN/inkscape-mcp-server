# Продолжение работы

Обновлено 2026-10-02. Это индекс состояния, не дополнительные разрешения.
Перед действиями проверяй Git/PR/процессы; не закрывай GUI с несохранённой работой.

## Текущее состояние main

PR #4 (document context, `bfe9e4f`) и PR #5 (everyday edits, `e6e4e80`) объединены в `main`.
Коммит `d948a25` добавил явный запуск Inkscape и переносимое определение session directory;
текущий HEAD при начале этой задачи — `852c73e` (Add agent guidance and vector authoring rules).
PR #6 (`0f1a766`) добавил live discovery и fingerprinted previews.
Это ориентир, а не требование откатывать более новые изменения.
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
  (110 инструментов, 7 prompts, 18 resources; видимость зависит от gates).

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

Проверка инструкций 2026-10-02: **41 passed** в тестах authoring/prompts/tool descriptions
и `test_llms_txt.py` (включая сверку каталога с реестром). Ruff check/format для двух
измененных Python-файлов и `git diff --check` прошли. Полный pytest, mypy и native GUI
acceptance в этой проверке не перезапускались; результаты выше относятся к 2026-10-01.

## Сессии и следующий объем

Историческая пользовательская сессия `/tmp/imcp-stage2-501` не закрывалась в предыдущих задачах;
это не утверждение, что она сейчас работает. Не использовать сохранённые PID:
перепроверять manifest и command line. Не завершать процессы по имени Inkscape.

Этап 3 уже объединён. Пользователь выбрал следующий объем: шесть улучшений ниже,
последовательно в указанном порядке; реализация завершена для рабочих копий. Общий этап 4 и остальные
долгосрочные цели остаются в [ROADMAP.md](ROADMAP.md).

## Перед началом новой задачи

Прочитай [AGENTS.md](../AGENTS.md), README, CONTRIBUTING и
[agent usage guide](agent-usage-guide.md), затем проверь `git status` и текущие сигнатуры.
В рабочем дереве уже есть незакоммиченные изменения инструкций и документации. Сохрани их:
не делай reset/checkout/clean и не заменяй файлы целиком из HEAD.

В `overview.py` и `prompts/authoring.py` уже добавлены общие инструкции:

- семантические объекты — обычные именованные группы; для новой иллюстрации по умолчанию
  один общий слой, дополнительные слои по назначению или просьбе пользователя;
- не предлагать и не выполнять трассировку PNG, в том числе внешними трассировщиками,
  скриптами или до импорта через MCP; не подменять вектор встроенным растром;
- сохранять порядок, трансформации, стили и ссылки; проверять результат визуально.

Это уже добавленное руководство для агента, а не реализованные новые инструменты или
детектор трассировки. Работающий MCP читает overview при старте: после изменения инструкции
нужен перезапуск сервера, без закрытия пользовательского GUI.

## Шесть улучшений: реализованы для рабочих копий

Реализованы 2026-10-02; текущее состояние публикации проверяй в Git/PR.
Существующие инструкции о семантических группах и запрете трассировки сохранены.

1. **Workspace и артефакты.** `get_workspace_info`, `inkscape://workspace`, root-qualified
   artifact URIs и read-only ресурс чтения с sandbox/size проверками. `open_document` и
   `save_document_as` принимают optional `root_id`; относительные пути по умолчанию по-прежнему
   используют первый root. Абсолютные server paths не выдаются за client paths. Ошибки вне
   workspace указывают на discovery; сохранение во второй root и чтение через MCP Client проверены.
2. **Группы/слои.** `create_group(label, mode)`, существующий label-only `rename_object`,
   `set_group_mode` на том же g и `reparent_object(preserve_appearance=True)`. Последний
   компенсирует affine transforms и требует неизменного глобального paint order. Отказывает
   при stylesheets, CSS transforms, singular transforms, nested viewports, внешних ссылках
   и непустом оформлении/effects/locks на изменяемой цепочке родителей. Legacy default False
   сохранён и не обещает сохранение вида. Batch-параметры и операции синхронизированы.
3. **Редактируемость.** `quality_report(editability=...)` возвращает отдельные optional
   рекомендации и factual observations; не меняет SVG validity/score. Семантические ID задаются
   явно; thresholds настраиваются, советы отключаются и ограничены 200. Это не детектор трассировки.
4. **Детали.** `render_preview(object_id/region)` переиспользует object export или рендерит
   прямоугольник в document user units. `compare_region(snapshot_id, region)` рендерит фиксированные
   bounds/scale/background без restore; разные canvas mappings отклоняются. Resource URIs и
   inline PNG доступны; artistic score не вычисляется.
5. **Фрагменты.** HIGH-risk `replace_svg_fragment` через существующий parser/allowlist и
   approval gate. Корневые ID/tag сохраняются; внутренние ID только при явном включении.
   Конфликты/duplicate IDs, unresolved refs и удаление внешне используемых ID отклоняются.
   `allow_retained` явно разрешает изменение вида surviving references; default отвергает такие
   изменения. Один snapshot/record, no-op без записи; есть соответствующий batch member.
6. **Повторение.** `repeat_objects` по explicit polyline (два пункта — линия) или rectangle grid.
   Count/spacing, fixed/tangent orientation, ограниченные jitter/scale/rotation и seed.
   Linked use и независимые copies различаются; copies переиспользуют remap duplicate engine.
   Dry-run по умолчанию проверяет полную disposable expansion без записи. Max 1024 и предварительный
   size budget; ID group задаётся явно. Anchor — local source point в document user units;
   copies могут совместно использовать внешние defs. SVG curves/path strings не поддерживаются.

Новые изменения относятся к tracked working copies, не к native live mutation protocol.
Автоматические проверки не подтверждают новый GUI Undo; GUI acceptance в этой задаче не запускался.
Для headless edits Undo обеспечен существующим snapshot/restore pipeline.

### Проверки реализации

- Итоговый полный pytest с Inkscape **1.4.3 (0d15f75)** в PATH: **1275 passed, 6 skipped**.
  Команда: `PATH="/Applications/Inkscape.app/Contents/MacOS:$PATH" .venv/bin/pytest -q`.
- Ruff check, format check (226 files), strict mypy (121 source files), manifest regeneration
  и `git diff --check` прошли. CI surface smoke обновлён до 110/7/18 и прошёл. Discovery eval: **41/41**, 100% accuracy.
- Реальные PNG проверяют cropped red→blue snapshot, совпадение всех RGBA каналов после
  safe reparent/group-layer conversion и между linked/copies. Это настоящие CLI рендеры,
  не GUI acceptance. Контейнеры/ссылки/отказы/seed/snapshot restore проверены автоматически.
- MCP Client проверил roots, сохранение во второй root, бинарное чтение resource URI и отказ
  после удаления артефакта. Отдельный свежий процесс через `.venv/bin/inkscape-mcp` проверил
  настоящий STDIO: **110 tools**, create/save/resource readback на synthetic workspace.
- Новую native GUI acceptance и native GUI Undo/Redo не запускали. Пользовательские окна
  не запускали/не закрывали; существующий live bridge не изменяли.
- Контракты и границы: [agent usage guide](agent-usage-guide.md).

MCP нужно перезапустить/переподключить для загрузки новых tools/resources/instructions.
Не закрывать пользовательский GUI: startup/reconnect по-прежнему не запускают окно.

## CI follow-up for PR #7

The initial Linux CI installed unsupported Inkscape 1.2.2 from Ubuntu's default archive.
The full-suite job now uses Ubuntu 24.04 and the official stable PPA, with an explicit
runtime-minimum check. Windows mypy exposed unguarded POSIX APIs: managed macOS helpers
now reject Windows explicitly. Missing-directory creation and save use native Windows
no-follow handles with ancestors held against renames; POSIX safeguards remain in place.
Windows-specific tests cover nested creation, overwrite, exclusive writes, symlink refusal
before truncation/descent, and parent rename prevention. Native GUI acceptance is unchanged.

The next CI run passed the Linux full suite. Windows then exposed existing CRLF shell
framing and path separator issues; shell frames normalize CRLF, registry source paths
use portable forward slashes, and DBus export filenames use forward slashes before
GVariant validation. macOS tests requiring actual POSIX ownership/locking are explicitly
platform-gated; the launch-policy fake uses the same socket path construction as production.
