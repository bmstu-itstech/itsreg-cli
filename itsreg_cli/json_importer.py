import json
import sys
from pathlib import Path

from itsreg_cli.api.client import ItsRegClient
from itsreg_cli.config.settings import load_settings
from itsreg_cli.domain.models import Bot, Edge, Message, Node, Script


def load_bot_from_json(json_path: str) -> Bot:
    with open(json_path, "r", encoding="utf-8") as f:
        data = json.load(f)

    nodes = []
    for node_data in data.get("script", {}).get("nodes", []):
        edges = []
        for edge_data in node_data.get("edges", []):
            edge = Edge(
                predicate=edge_data.get("predicate"),
                to=edge_data.get("to"),
                operation=edge_data.get("operation"),
            )
            edges.append(edge)

        messages = []
        for msg_data in node_data.get("messages", []):
            messages.append(Message(text=msg_data.get("text", "")))

        node = Node(
            state=node_data.get("state"),
            title=node_data.get("title", ""),
            messages=messages,
            edges=edges,
            options=node_data.get("options", []),
        )
        nodes.append(node)

    entries = data.get("script", {}).get("entries", [])

    script = Script(nodes=nodes, entries=entries)

    bot = Bot(
        id=data.get("id"),
        token=data.get("token"),
        enabled=data.get("enabled", True),
        status=data.get("status", "idle"),
        script=script,
    )

    return bot


def import_bot(json_path: str) -> None:
    json_file = Path(json_path)
    if not json_file.exists():
        raise FileNotFoundError(f"JSON файл не найден: {json_path}")

    bot = load_bot_from_json(json_path)
    settings = load_settings()
    client = ItsRegClient(settings)

    try:
        print(f"Загрузка бота из {json_path}...")
        print(f"ID: {bot.id}")
        print(f"Token: {bot.token[:20]}...")
        print(f"Узлов: {len(bot.script.nodes)}")
        print(f"Entries: {len(bot.script.entries)}\n")

        created_bot = client.create_bot(bot)
        print(f"✓ Бот '{created_bot.id}' успешно создан!")

    except Exception as e:
        print(f"✗ Ошибка при создании бота: {e}", file=sys.stderr)
        sys.exit(1)

    finally:
        client.close()


if __name__ == "__main__":
    if len(sys.argv) < 2:
        print("Использование: python -m itsreg_cli.json_importer <путь_к_json>")
        print("Пример: python -m itsreg_cli.json_importer bot_schema.json")
        sys.exit(1)

    import_bot(sys.argv[1])
