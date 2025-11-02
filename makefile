# PANDOC_OPTS += --resource-path=docs
PANDOC_OPTS += --filter=pandoc-include
PANDOC_OPTS += --filter=mermaid-filter

GIT_FILES := $(shell git ls-files)

.PHONY: all
all: docs/relatorio.pdf archive

%.pdf: %.md
	pandoc $(PANDOC_OPTS) --output=$@ $<

.PHONY: archive
archive: googol-LuísGóis.zip

googol-LuísGóis.zip: docs/relatorio.pdf README.pdf $(GIT_FILES)
	git archive --verbose --add-file=docs/relatorio.pdf --add-file=README.pdf --output=$@ main
