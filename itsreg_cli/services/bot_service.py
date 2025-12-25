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
        start_state = self._input_state_number(
            nodes_by_state, default_state=1, prompt_title="Номер стартового состояния"
        )
        self._build_node_recursive(nodes_by_state, start_state)

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
                "Стартовое состояние:", default=str(start_state)
            ).ask() or str(start_state)
            try:
                start = int(start_str)
            except ValueError:
                start = start_state
            if start == 0 or start not in nodes_by_state:
                raise ValueError("Указано несуществующее состояние для entry")
            entries.append({"key": key, "start": start})
            if not questionary.confirm("Добавить ещё entry?", default=False).ask():
                break
        if not entries:
            entries.append({"key": "start", "start": start_state})

        nodes = list(nodes_by_state.values())
        script = Script(nodes=nodes, entries=entries)
        bot = Bot(id=bot_id, token=token, enabled=True, status="idle", script=script)
        return self.client.create_bot(bot)

    def _input_state_number(
        self, nodes_by_state: dict[int, Node], default_state: int, prompt_title: str
    ) -> int:
        while True:
            state_str = questionary.text(
                prompt_title + ":", default=str(default_state)
            ).ask() or str(default_state)
            try:
                state = int(state_str)
            except ValueError:
                continue
            if state == 0:
                state = 1
            if state in nodes_by_state:
                continue
            return state

    def _build_node_recursive(
        self, nodes_by_state: dict[int, Node], state: int
    ) -> Node:
        if state in nodes_by_state:
            return nodes_by_state[state]
        title = (
            questionary.text(f"Узел {state}: Название:", default=f"state-{state}").ask()
            or f"state-{state}"
        )
        messages: List[Message] = []
        while True:
            message_text = questionary.text(f"Узел {state}: Текст сообщения:").ask()
            if message_text:
                messages.append(Message(text=message_text))
            if not questionary.confirm(
                f"Узел {state}: Добавить ещё сообщение?", default=False
            ).ask():
                break
        if not messages:
            messages.append(Message(text=""))
        options: List[str] = []
        while questionary.confirm(
            f"Узел {state}: Добавить кнопку-опцию?", default=False
        ).ask():
            option_text = questionary.text(f"Узел {state}: Текст опции:").ask()
            if option_text:
                options.append(option_text)
        edges: List[Edge] = []
        edge_index = 1
        while questionary.confirm(
            f"Узел {state}: Добавить исходящее ребро?", default=False
        ).ask():
            predicate_type = (
                questionary.select(
                    f"Узел {state}, Ребро #{edge_index}: Тип условия:",
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
                    questionary.text(
                        f"Узел {state}, Ребро #{edge_index}: Regex-паттерн:",
                        default=".*",
                    ).ask()
                    or ".*"
                )
                predicate = {"type": "regex", "pattern": pattern}
            else:
                text_value = (
                    questionary.text(
                        f"Узел {state}, Ребро #{edge_index}: Ожидаемый текст:",
                        default="Далее",
                    ).ask()
                    or "Далее"
                )
                predicate = {"type": "exact", "text": text_value}
            to_state_str = questionary.text(
                f"Узел {state}, Ребро #{edge_index}: Целевое состояние:",
                default=str(state + 1),
            ).ask() or str(state + 1)
            try:
                to_state = int(to_state_str)
            except ValueError:
                to_state = state + 1
            if to_state == 0:
                to_state = 1
            operation = (
                questionary.select(
                    f"Узел {state}, Ребро #{edge_index}: Операция:",
                    choices=["noop", "save", "append"],
                    default="noop",
                ).ask()
                or "noop"
            )
            edges.append(Edge(predicate=predicate, to=to_state, operation=operation))
            if to_state not in nodes_by_state:
                self._build_node_recursive(nodes_by_state, to_state)
            edge_index += 1
        node = Node(
            state=state, title=title, messages=messages, edges=edges, options=options
        )
        nodes_by_state[state] = node
        return node

    def enable_bot(self, bot_id: str) -> None:
        self.client.enable_bot(bot_id)

    def disable_bot(self, bot_id: str) -> None:
        self.client.disable_bot(bot_id)

    def get_bot_answers(self, bot_id: str) -> str:
        return self.client.get_bot_answers(bot_id)
