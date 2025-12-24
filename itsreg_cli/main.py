from __future__ import annotations

import argparse
import sys

from pydantic import ValidationError

from itsreg_cli.api.client import ItsRegClient
from itsreg_cli.config.settings import Settings, load_settings
from itsreg_cli.services.bot_service import BotService
from itsreg_cli.tui.menus import main_menu


def parse_args(argv: list[str]) -> argparse.Namespace:
    parser = argparse.ArgumentParser(description="ITS Reg TUI клиент")
    parser.add_argument("--api", dest="api_url", help="Базовый URL API")
    parser.add_argument(
        "--jwt-token", dest="jwt_token", help="JWT токен аутентификации"
    )
    return parser.parse_args(argv)


def ensure_token(settings: Settings) -> None:
    if not settings.jwt_token:
        raise SystemExit("ITSREG_JWT_TOKEN не задан")


def main(argv: list[str] | None = None) -> None:
    args = parse_args(argv or sys.argv[1:])
    try:
        settings = load_settings(cli_api_url=args.api_url, cli_jwt_token=args.jwt_token)
        ensure_token(settings)
    except ValidationError as exc:
        raise SystemExit("ITSREG_JWT_TOKEN не задан") from exc

    with ItsRegClient(settings) as client:
        service = BotService(client)
        main_menu(service)


if __name__ == "__main__":
    main()
