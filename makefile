# PANDOC_OPTS += --resource-path=docs
PANDOC_OPTS += --filter=pandoc-include
PANDOC_OPTS += --filter=mermaid-filter

%.pdf: %.md
	pandoc $(PANDOC_OPTS) --output=$@ $<
