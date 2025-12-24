from __future__ import annotations

import json
from typing import List

import questionary
from rich.console import Console
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
        console.print("Сценарий:")
        console.print_json(json.dumps(bot.script.model_dump(), ensure_ascii=False))
        action = questionary.select(
            "Действие:",
            choices=[
                "Включить автозапуск",
                "Выключить автозапуск",
                "Удалить бота",
                "Назад",
            ],
        ).ask()
        if action == "Включить автозапуск":
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
