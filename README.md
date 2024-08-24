# Exalted

![The Exalted logo](./exalted.png)

An experiment in implementing a code editor using just a 2D graphics library and
a font shaping library (`tiny-skia` and `rustybuzz` respectively).

## Inspiration

This project is mostly inspired by Emacs and its near-limitless extensibility,
however I would be remiss if I didn't also mention Sublime Text and Helix, the
former being a blazingly (🚀🚀🚀) fast, user-friendly editor implemented using
basically just Skia, and the latter being a batteries included editor with
superb defaults.

## Planned Features

* The ability to edit text (🤯).
* Syntax highlighting and context-aware editing with Tree-Sitter.
* Modal editing, à la Kakoune/Helix (I just love the editing model too much).
* A built-in LSP client, though it (LSP) may not be a great standard, it's a
  step towards more comprehensive language support without the baggage of IDEs.
  It may be worth considering providing an interface more general than that
  which is required by LSP to accommodate any future developments.
* A system akin to major/minor modes in Emacs that allows scoping of particular
  functionality as well as composition (perhaps inheritance is better for this
  kind of thing, we shall see).
* A strongly typed configuration language, perhaps some form of Scheme or ML
  (nothing screams experimental more than an obscure config language 😎).
  Perhaps Nix can be a good source of inspiration (although the lack of types
  not so much).
* Other cool stuff (idk it's 3am and I can't think of anything more).

## Roadmap

Just making it up as I go along really.

- [x] Create a `winit` window and render a shape to its surface using
  `tiny-skia`, adjusting on resize.
- [ ] Incorporate `taffy` to render some kind of editor layout.
- [ ] Shape and render some text using `rustybuzz`.
