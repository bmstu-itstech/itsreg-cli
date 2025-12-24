from __future__ import annotations

from pydantic_settings import BaseSettings, SettingsConfigDict


class Settings(BaseSettings):
    api_url: str = "https://itsreg.itsbmstu.ru/api/v2/"
    jwt_token: str

    model_config = SettingsConfigDict(
        env_prefix="ITSREG_",
        extra="forbid",
        case_sensitive=False,
        env_file=".env",
        env_file_encoding="utf-8",
    )


def load_settings(
    cli_api_url: str | None = None, cli_jwt_token: str | None = None
) -> Settings:
    overrides: dict[str, str] = {}
    if cli_api_url:
        overrides["api_url"] = cli_api_url
    if cli_jwt_token:
        overrides["jwt_token"] = cli_jwt_token

    return Settings(**overrides)
