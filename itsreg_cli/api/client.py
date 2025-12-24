from __future__ import annotations

from typing import List

import httpx

from itsreg_cli.config.settings import Settings
from itsreg_cli.domain.models import Bot, Script


class ApiError(RuntimeError):
    pass


class ItsRegClient:
    def __init__(self, settings: Settings):
        self.settings = settings
        base_url = settings.api_url.rstrip("/") + "/"
        self._client = httpx.Client(
            base_url=base_url,
            headers={"Authorization": settings.jwt_token},
            timeout=10.0,
        )

    def list_bots(self) -> List[Bot]:
        response = self._client.get("bots")
        self._raise_if_error(response)
        payload = response.json()
        raw_items = payload.get("bots") if isinstance(payload, dict) else payload
        return [Bot.model_validate(item) for item in raw_items or []]

    def create_bot(self, bot: Bot) -> Bot:
        payload = {
            "id": bot.id,
            "token": bot.token or "",
            "script": bot.script.model_dump()
            if bot.script
            else {"nodes": [], "entries": []},
        }
        response = self._client.put("bots", json=payload)
        self._raise_if_error(response, payload)
        try:
            data = response.json()
            return Bot.model_validate(data)
        except Exception:
            fetch = self._client.get(f"bots/{bot.id}")
            self._raise_if_error(fetch)
            return Bot.model_validate(fetch.json())

    def delete_bot(self, bot_id: str) -> None:
        response = self._client.delete(f"bots/{bot_id}")
        self._raise_if_error(response)

    def get_bot(self, bot_id: str) -> Bot:
        response = self._client.get(f"bots/{bot_id}")
        self._raise_if_error(response)
        return Bot.model_validate(response.json())

    def update_script(self, bot_id: str, script: Script) -> Bot:
        response = self._client.post(f"bots/{bot_id}/script", json=script.model_dump())
        self._raise_if_error(response)
        return Bot.model_validate(response.json())

    def enable_bot(self, bot_id: str) -> None:
        response = self._client.post(f"bots/{bot_id}/enable")
        self._raise_if_error(response)

    def disable_bot(self, bot_id: str) -> None:
        response = self._client.post(f"bots/{bot_id}/disable")
        self._raise_if_error(response)

    def close(self) -> None:
        self._client.close()

    def _raise_if_error(
        self, response: httpx.Response, request_payload: dict | None = None
    ) -> None:
        try:
            response.raise_for_status()
        except httpx.HTTPStatusError as exc:
            detail = exc.response.text
            payload_info = f" | Request: {request_payload}" if request_payload else ""
            raise ApiError(
                f"API error {exc.response.status_code}: {detail}{payload_info}"
            ) from exc

    def __enter__(self) -> "ItsRegClient":
        return self

    def __exit__(self, *_) -> None:
        self.close()
