from __future__ import annotations

import csv
import io
import json
from datetime import datetime
from typing import List

import questionary
from rich.console import Console
from rich.progress import Progress, SpinnerColumn, TextColumn
from rich.table import Table

from itsreg_cli.domain.models import Bot
from itsreg_cli.services.bot_service import BotService

console = Console()


def main_menu(service: BotService) -> None:
    while True:
        choice = questionary.select(
            "Выберите опцию:",
            choices=[
                "Создать бота",
                "Мои боты",
                questionary.Separator(),
                "Выход",
            ],
        ).ask()

        if choice == "Мои боты":
            show_bots_list(service)
        elif choice == "Создать бота":
            try:
                bot = service.create_bot_interactive()
                console.print(f"Бот '{bot.id}' создан!", style="green")
            except Exception as e:
                console.print(f"Ошибка: {e}", style="red")
        elif choice == "Выход":
            break


def show_bots_list(service: BotService) -> None:
    bots = service.get_my_bots()
    if not bots:
        console.print("Ботов пока нет.", style="yellow")
        return

    _render_bots_table(bots)
    selected = questionary.select(
        "Выберите бота для деталей:",
        choices=[*(bot.id for bot in bots), "Назад"],
    ).ask()
    if selected and selected != "Назад":
        show_bot_details(service, selected)


def show_bot_details(service: BotService, bot_id: str) -> None:
    while True:
        bot = service.get_bot(bot_id)
        console.print(f"Детали бота {bot.id}", style="cyan")
        console.print(f"Статус: {bot.status}, Включен: {bot.enabled}")
        nodes_count = len(bot.script.nodes) if bot.script and bot.script.nodes else 0
        entries_count = (
            len(bot.script.entries) if bot.script and bot.script.entries else 0
        )
        console.print(f"Узлов в сценарии: {nodes_count}, Точек входа: {entries_count}")
        action = questionary.select(
            "Действие:",
            choices=[
                "Показать сценарий",
                "Включить автозапуск",
                "Выключить автозапуск",
                "Экспорт ответов",
                "Удалить бота",
                "Назад",
            ],
        ).ask()
        if action == "Показать сценарий":
            console.print("Сценарий:", style="cyan")
            console.print_json(json.dumps(bot.script.model_dump(), ensure_ascii=False))
            input("\nНажмите Enter для продолжения...")
        elif action == "Включить автозапуск":
            try:
                service.enable_bot(bot_id)
                console.print("Включено в автозапуск.", style="green")
            except Exception as e:
                console.print(f"Ошибка: {e}", style="red")
        elif action == "Выключить автозапуск":
            try:
                service.disable_bot(bot_id)
                console.print("Удалён из автозапуска.", style="yellow")
            except Exception as e:
                console.print(f"Ошибка: {e}", style="red")
        elif action == "Экспорт ответов":
            export_bot_answers(service, bot_id)
        elif action == "Удалить бота":
            if questionary.confirm(f"Удалить бота '{bot_id}'?", default=False).ask():
                try:
                    service.delete_bot(bot_id)
                    console.print(f"Бот '{bot_id}' удален.", style="red")
                    break
                except Exception as e:
                    console.print(f"Ошибка: {e}", style="red")
        else:
            break


def export_bot_answers(service: BotService, bot_id: str) -> None:
    try:
        with Progress(
            SpinnerColumn(),
            TextColumn("[progress.description]{task.description}"),
            console=console,
        ) as progress:
            task = progress.add_task(
                "Загрузка ответов с сервера (может занять несколько минут)...",
                total=None,
            )
            csv_data = service.get_bot_answers(bot_id)
            progress.update(task, completed=True)

        if not csv_data.strip():
            console.print("Нет ответов для экспорта.", style="yellow")
            return

        reader = csv.reader(io.StringIO(csv_data))
        rows = list(reader)

        if not rows:
            console.print("Нет ответов для экспорта.", style="yellow")
            return

        table = Table(title=f"Ответы на бота {bot_id}")
        headers = rows[0] if rows else []
        for header in headers:
            table.add_column(header, overflow="fold")

        for row in rows[1:]:
            table.add_row(*row)

        console.print(table)

        if questionary.confirm("Сохранить в CSV файл?", default=True).ask():
            timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
            filename = f"{bot_id}_answers_{timestamp}.csv"
            with open(filename, "w", encoding="utf-8") as f:
                f.write(csv_data)
            console.print(f"Сохранено в {filename}", style="green")

    except Exception as e:
        error_msg = str(e)
        if "504" in error_msg or "Gateway Time-out" in error_msg:
            console.print(
                "Ошибка: Сервер не успел обработать запрос (504 Gateway Timeout).",
                style="red",
            )
            console.print("Это серверная проблема. Возможные решения:", style="yellow")
            console.print("1. Попробуйте позже, когда на сервере меньше нагрузки")
            console.print(
                "2. Обратитесь к администраторам для увеличения timeout на сервере"
            )
            console.print("3. Используйте веб-интерфейс для экспорта")
        else:
            console.print(f"Ошибка при экспорте: {e}", style="red")


def _render_bots_table(bots: List[Bot]) -> None:
    table = Table(title="Мои боты")
    table.add_column("ID")
    table.add_column("Статус")
    table.add_column("Включен")
    table.add_column("Узлов")

    for bot in bots:
        nodes_count = (
            str(len(bot.script.nodes)) if bot.script and bot.script.nodes else "0"
        )
        table.add_row(bot.id, bot.status, "Да" if bot.enabled else "Нет", nodes_count)

    console.print(table)
