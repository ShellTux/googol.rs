#!/bin/sh
set -e

parallel --line-buffer --halt soon,success=1 ::: \
	'simple-http-server --index --nocache --ip 127.0.0.1 --port 9090 static' \
	'find . -name "*.md" | entr make README.pdf'
