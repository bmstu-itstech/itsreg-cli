# itsreg-cli

TUI клиент для управления Telegram-ботами ITS Reg.

## Установка
```bash
git clone https://github.com/bmstu-itstech/itsreg-cli
cd itsreg-cli
make install
```

## Запуск
```bash
export ITSREG_JWT_TOKEN=your_token
export ITSREG_API_URL=https://itsreg.itsbmstu.ru/api/v2/
make run
```

## Архитектура
```
itsreg-cli/
├── domain/         # Domain Models (Pydantic)
├── api/            # API Layer (openapi-codegen + wrapper)
├── services/       # Business Logic (domain only)
├── tui/            # TUI экраны (questionary + rich)
├── config/         # Конфигурация
├── main.py         # Entry point
├── requirements*.txt  # зависимости (runtime/dev)
├── Makefile        # make install, generate-client
└── README.md       # Инструкции
```

## Конфигурация (приоритет)
1. CLI флаги: `itsreg-tui --api=https://... --jwt-token=...`
2. ENV: `ITSREG_JWT_TOKEN`, `ITSREG_API_URL`
3. Если ничего нет → ошибка: `ITSREG_JWT_TOKEN не задан`

## Генерация клиента
```bash
make generate-client
```
Используется `openapi-python-client` и спецификация `api/openapi/bots.yaml`.
