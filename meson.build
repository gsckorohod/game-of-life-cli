project('game-of-life', 'rust', 
          version: '0.1.0',
    meson_version: '>= 0.59.0',
  default_options: [ 'warning_level=2',
                     'werror=false',
                   ],
)




cargo_sources = files(
  'Cargo.toml',
  'Cargo.lock',
)

subdir('src')

