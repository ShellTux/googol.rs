#!/bin/sh
set -e

address="${1:-127.0.0.1:8080}"

# shellcheck disable=SC2089
for method_endpoint in \
	'get '"$address"'/' \
	'get '"$address"'/health' \
	'post '"$address"'/enqueue url="https://spankme.dad"' \
	'post '"$address"'/enqueue url="https://shelltux.github.io/blog"' \
	'get '"$address"'/search --url-query words=vitae'
do
	echo "$method_endpoint" | grep --silent enqueue && continue

	# shellcheck disable=SC2086,SC2090
	(set -x; curlie $method_endpoint)

	echo
done
