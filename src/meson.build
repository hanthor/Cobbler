cobbler_sources = [
  'main.c',
  'cobbler-application.c',
  'cobbler-window.c',
]

cobbler_deps = [
  dependency('gtk4'),
  dependency('libadwaita-1', version: '>= 1.4'),
]

cobbler_sources += gnome.compile_resources('cobbler-resources',
  'cobbler.gresource.xml',
  c_name: 'cobbler'
)

executable('cobbler', cobbler_sources,
  dependencies: cobbler_deps,
       install: true,
)
