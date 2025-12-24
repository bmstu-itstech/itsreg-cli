from __future__ import annotations

from typing import List

import questionary

from itsreg_cli.api.client import ItsRegClient
from itsreg_cli.domain.models import Bot, Edge, Message, Node, Script


class BotService:
    def __init__(self, client: ItsRegClient):
        self.client = client

    def get_my_bots(self) -> List[Bot]:
        return self.client.list_bots()

    def get_bot(self, bot_id: str) -> Bot:
        return self.client.get_bot(bot_id)

    def delete_bot(self, bot_id: str) -> None:
        self.client.delete_bot(bot_id)

    def create_bot_interactive(self) -> Bot:
        bot_id = questionary.text("Введите идентификатор бота:").ask()
        if not bot_id:
            raise ValueError("Идентификатор бота обязателен")

        token = questionary.text("Введите Telegram токен (BotFather):").ask()
        if not token:
            raise ValueError("Токен обязателен")

        nodes_by_state: dict[int, Node] = {}
        self._ask_node_full(nodes_by_state, prefilled_state=1)
        while questionary.confirm("Добавить ещё узел?", default=False).ask():
            self._ask_node_full(nodes_by_state)

        if not nodes_by_state:
            raise ValueError("Нужно создать хотя бы один узел")

        entries: List[dict] = []
        while True:
            key = questionary.text(
                "Entry key:", default="start" if not entries else ""
            ).ask()
            if not key:
                break
            start_str = questionary.text(
                "Стартовое состояние:", default=str(min(nodes_by_state.keys()))
            ).ask() or str(min(nodes_by_state.keys()))
            try:
                start = int(start_str)
            except ValueError:
                start = min(nodes_by_state.keys())
            if start == 0 or start not in nodes_by_state:
                raise ValueError("Указано несуществующее состояние для entry")
            entries.append({"key": key, "start": start})
            if not questionary.confirm("Добавить ещё entry?", default=False).ask():
                break
        if not entries:
            entries.append({"key": "start", "start": min(nodes_by_state.keys())})

        nodes = list(nodes_by_state.values())
        script = Script(nodes=nodes, entries=entries)
        bot = Bot(id=bot_id, token=token, enabled=True, status="idle", script=script)
        return self.client.create_bot(bot)

    def _ask_node_full(
        self, nodes_by_state: dict[int, Node], prefilled_state: int | None = None
    ) -> Node:
        default_state = (
            prefilled_state
            if prefilled_state is not None
            else (max(nodes_by_state.keys()) + 1 if nodes_by_state else 1)
        )
        state_str = questionary.text(
            "Номер состояния узла:", default=str(default_state)
        ).ask() or str(default_state)
        try:
            state = int(state_str)
        except ValueError:
            state = default_state
        if state == 0:
            state = 1
        title = (
            questionary.text(f"Название узла #{state}:", default=f"state-{state}").ask()
            or f"state-{state}"
        )
        messages: List[Message] = []
        while True:
            message_text = questionary.text("Текст сообщения узла:").ask()
            if message_text:
                messages.append(Message(text=message_text))
            if not questionary.confirm("Добавить ещё сообщение?", default=False).ask():
                break
        if not messages:
            messages.append(Message(text=""))
        edges: List[Edge] = []
        while questionary.confirm(
            "Добавить переход из этого узла?", default=False
        ).ask():
            predicate_type = (
                questionary.select(
                    "Тип условия (predicate):",
                    choices=[
                        questionary.Choice(title="Всегда", value="always"),
                        questionary.Choice(title="Точный текст", value="exact"),
                        questionary.Choice(title="Регекс", value="regex"),
                    ],
                    default="exact",
                ).ask()
                or "exact"
            )
            if predicate_type == "always":
                predicate = {"type": "always"}
            elif predicate_type == "regex":
                pattern = (
                    questionary.text("Введите regex-паттерн:", default=".*").ask()
                    or ".*"
                )
                predicate = {"type": "regex", "pattern": pattern}
            else:
                text_value = (
                    questionary.text("Ожидаемый текст:", default="Далее").ask()
                    or "Далее"
                )
                predicate = {"type": "exact", "text": text_value}
            to_state_str = questionary.text(
                "Целевое состояние (число):", default=str(state + 1)
            ).ask() or str(state + 1)
            try:
                to_state = int(to_state_str)
            except ValueError:
                to_state = state + 1
            if to_state == 0:
                to_state = 1
            operation = (
                questionary.select(
                    "Операция:", choices=["noop", "save", "append"], default="noop"
                ).ask()
                or "noop"
            )
            if to_state not in nodes_by_state:
                if questionary.confirm(
                    f"Создать узел состояния {to_state} сейчас?", default=True
                ).ask():
                    self._ask_node_full(nodes_by_state, prefilled_state=to_state)
            edges.append(Edge(predicate=predicate, to=to_state, operation=operation))
        options: List[str] = []
        while questionary.confirm(
            "Добавить вариант ответа (option)?", default=False
        ).ask():
            option_text = questionary.text("Текст опции:").ask()
            if option_text:
                options.append(option_text)
        node = Node(
            state=state, title=title, messages=messages, edges=edges, options=options
        )
        nodes_by_state[state] = node
        return node

    def enable_bot(self, bot_id: str) -> None:
        self.client.enable_bot(bot_id)

    def disable_bot(self, bot_id: str) -> None:
        self.client.disable_bot(bot_id)
