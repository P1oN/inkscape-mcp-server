# Продолжение работы

Обновлено 2026-10-01. Это краткий индекс состояния, не дополнительные разрешения.
Перед действиями проверяй Git/PR/процессы; не закрывай GUI с несохранённой работой.

## Объем текущей задачи

Пользователь просит выполнить следующие шаги прежнего handoff в одном PR:
выбор/идентичность рисунка, guard при смене документа, восстановление связи,
компактная передача состояния. Полный roadmap — отдельный план, не scope этого PR.
Миграция на Rust условная: только без проблем реализации и без потери функционала.

## Текущее состояние

- Ветка: `codex/document-context-guard`, база `main` (`9f58b16`).
- Origin: `https://github.com/P1oN/inkscape-mcp-server`.
- Один PR: [#4](https://github.com/P1oN/inkscape-mcp-server/pull/4).
  Продолжать в нем; не создавать второй PR.
- Реализованы `live_list_documents` / `live_select_document`, UUID живых GTK
  объектов окна/документа, выбор рисунка для задачи и guard внутри GUI перед GAction.
- Общий context scope связывает scene/frame, provenance, previews и изменение.
- Status сообщает выбранный/активный рисунок, готовность правки, состояние связи
  и восстановление. Reconnect сохраняет GUI и сбрасывает выбор для задачи.
- Мост — небольшой GTK/GIO/Cocoa модуль. Launcher создает приватную ad-hoc signed
  копию исполняемого файла; vendor app не меняется. Это экспериментальная схема,
  с дополнительными требованиями установки и без hardened runtime vendor executable.
- Полное описание и ограничения: [document-context.md](document-context.md).
- Rust не начат: в native integration встретились реальные сложности. Смена языка
  не устраняет ограничения Inkscape; условие пользователя для миграции не выполнено.

## Проверки и оставшаяся работа

- Strict mypy: 110 source files; focused Ruff, MCP surface smoke (101 tools), wheel
  build проходят. Focused final suite: 81 passed. Native module компилируется
  clang с `-Wall -Wextra -Werror`.
- Последний полный pytest: 1078 passed, 74 skipped, 1 failed — известный ранее
  нестабильный `test_engine_process.py::test_unknown_action_surfaces_engine_action_error`.
  Он использует fake shell, а не Inkscape. Отдельный повтор прошел; ошибка полного
  прогона остается зафиксированной. Не скрывать этот результат.
- Финальная native acceptance прошла 2026-10-01 на official Inkscape 1.4.3
  (`0d15f75`) через настоящий MCP STDIO в новой disposable session.
  Проверены разные ID одинаковых SVG, явный выбор, Undo/Redo заливки и вставки,
  native dispatch race refusal без изменения B, stale-binding refusal и повторное
  подключение с тем же GUI/ID и сбросом выбора. Только два тестовых окна закрыты.
- Во время lock Inkscape попадал в crash handler при primary-monitor initialization.
  Module теперь отказывает до загрузки рисунка при отсутствии primary monitor.
  Отказ на locked Mac и успешный новый запуск после unlock проверены.
- Локальное доказательство: `work/context-probe/acceptance-resume.log` и
  `/private/tmp/imcp-context-xyxqexpl/acceptance.json` (`passed: true`).
  Воспроизводимая команда: `.venv/bin/python scripts/accept_document_context.py`.
- Объем этого PR реализован и проверен. Следующий функциональный объем — этап 3
  roadmap; он не входит в текущий запрос. Перед новой работой сверить состояние PR.

## Сессии

Историческая пользовательская сессия `/tmp/imcp-stage2-501` не закрывалась.
Не использовать сохраненные PID: перепроверять manifest и command line.
Тестовые GUI этой задачи находятся в `/tmp/imcp-context-*` и используют синтетические
`a.svg` / `b.svg`. Часть неудачных запусков оставлена для осмотра. Не завершать процесс
по имени Inkscape и не трогать окна других сессий. Локальные логи: `work/context-probe/`
(не входят в Git). Подробная история старых PR доступна в Git, не нужна в контексте.
