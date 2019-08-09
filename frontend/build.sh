#!/usr/bin/env sh

js="yagcdn.js"
min="yagcdn.min.js"
outdir="./output/"
scriptdir="${outdir}scripts/"

prepare() {
  rm -rf "${outdir}"
  mkdir -p "${outdir}/scripts"
}

build() {
  elm make --optimize src/Main.elm --output=${js}
}

minify() {
  uglifyjs ${js} --compress 'pure_funcs="F2,F3,F4,F5,F6,F7,F8,F9,A2,A3,A4,A5,A6,A7,A8,A9",pure_getters,keep_fargs=false,unsafe_comps,unsafe' | uglifyjs --mangle --output=${min}
}

sha1() {
  sha1sum ${min} | cut -d' ' -f 1
}

rename_with_hash() {
  sha1=${1}
  cp ${min} "${scriptdir}/yagcdn-${sha1}.min.js"
}

create_index() {
  sha1=${1}
  SHA1="${sha1}" envsubst < template.html > "${outdir}/index.html"
}

copy_assets() {
  cp -r assets ${outdir}
}

prepare
build
minify
sha1=$(sha1)
rename_with_hash "${sha1}"
create_index "${sha1}"
copy_assets
