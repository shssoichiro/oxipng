#!/usr/bin/env bash

set -e;

###
## Input handling
###

# Set flags for calling oxipng later
if [[ -n "${UNSAFE}" ]]; then FLAGS="${FLAGS} -a"; fi

if [[ -z "${1}" ]]; then
	echo "Wrapper for oxipng for handling OpenRaster files

Usage:
    oxiora file_a.ora file_b.ora ... file_n.ora

Environment variables:
    UNSAFE
        Set to any value to send the -a flag to oxipng, which allows altering the colour of completely transparent pixels. Alters the content of images, so disabled by default.
    
    FLAGS
        Any additional flags to pass to oxipng.
";
	exit 0;
fi

###
## Preamble
###

# Create temporary directory and clean it up afterwards no matter what
temp_dir="$(mktemp --tmpdir -d "oxiora-XXXXXXX")";

on_exit() {
	rm -rf "${temp_dir}";
}
trap on_exit EXIT;

###
## Main loop
###

# We process serially here, because oxipng does a good job of using all available CPU cores already
for filepath in "$@"; do
	echo ">>> Processing ${filepath}";
	
	filesize_before="$(wc -c <"${filepath}")";
	
	unzip "${filepath}" -d "${temp_dir}";
	chmod u+w -R "${temp_dir}"
	
	# no -a here 'cause this is technically an editing file to fully-transparent pixel colours could have meaning
	#shellcheck disable=SC2086
	find "${temp_dir}" -iname '*.png' -print0 | xargs -0 oxipng ${FLAGS} -somax;
	
	filepath_target="${filepath%.*}-small.${filepath##*.}";
	filepath_target="$(realpath "${filepath_target}")";
	
	# The OpenRaster spec says the mimetype file has to be uncompressed
	# Ref https://www.openraster.org/baseline/file-layout-spec.html#mimetype
	# unzip style ref https://askubuntu.com/a/1399484/139735
	(cd "${temp_dir}" && find . -type f -name 'mimetype' | zip -0 -r "${filepath_target}" --names-stdin)
	(cd "${temp_dir}" && find . -type f -not -name 'mimetype' | zip -9 -r "${filepath_target}" --names-stdin)
	
	# Only write the .ora back if it is smaller than the original
	filesize_after="$(wc -c <"${filepath_target}")";
	difference="$((filesize_before-filesize_after))";
	
	echo ">>> BEFORE $(echo "${filesize_before}" | numfmt --to=iec)iB";
	echo ">>> AFTER  $(echo "${filesize_after}" | numfmt --to=iec)iB";
	
	if [[ "${filesize_after}" -lt "${filesize_before}" ]]; then
		echo ">>> Compressed by $(echo "${difference}" | numfmt --to=iec)iB";
		mv -f "${filepath_target}" "${filepath}";
	else
		echo ">>> Compressed file was $(echo "$((-difference))" | numfmt --to=iec)iB larger, discarding";
		rm "${filepath_target}";
	fi
	
	find "${temp_dir}" -mindepth 1 -delete
done
