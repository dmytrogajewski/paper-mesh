# UI resources
blueprints = custom_target('blueprints',
  input: files(
    'window.blp',
    'connect_page/connect_page.blp',
    'components/message_entry.blp',
    'session/add_channel_dialog.blp',
    'session/mod.blp',
    'session/sidebar.blp',
    'session/sidebar_row.blp',
    'session/node_row.blp',
    'session/map_view.blp',
    'session/content.blp',
    'session/message_row.blp',
  ),
  output: '.',
  command: [find_program('blueprint-compiler'), 'batch-compile', '@OUTPUT@', '@CURRENT_SOURCE_DIR@', '@INPUT@'],
)

ui_resources = gnome.compile_resources(
  'ui-resources',
  'ui-resources.gresource.xml',
  gresource_bundle: true,
  source_dir: meson.current_build_dir(),
  install: true,
  install_dir: pkgdatadir,
  dependencies: blueprints,
)
