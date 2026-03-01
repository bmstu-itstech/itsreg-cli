VENV=.venv
PYTHON=$(VENV)/bin/python
PIP=$(VENV)/bin/pip
OPENAPI_CLIENT=$(VENV)/bin/openapi-python-client

$(VENV):
	python3 -m venv $(VENV)

$(OPENAPI_CLIENT): $(VENV) requirements.txt
	$(PIP) install --upgrade pip
	$(PIP) install -r requirements.txt

install: $(VENV)
	$(PIP) install --upgrade pip
	$(PIP) install -r requirements.txt
	$(VENV)/bin/pre-commit install

generate-client: $(OPENAPI_CLIENT)
	rm -rf its_reg_api_client itsreg_api_client openapi_client itsreg_api
	$(OPENAPI_CLIENT) generate \
		--url https://raw.githubusercontent.com/bmstu-itstech/itsreg/main/api/openapi/bots.yaml \
		--config openapi-config.json \
		--meta none

run: generate-client
	$(PYTHON) -m itsreg_cli

clean:
	rm -rf $(VENV) itsreg_api
