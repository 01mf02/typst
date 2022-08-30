// General tests for set.

---
// Test that text is affected by instantiation-site bold.
#let x = [World]
Hello *{x}*

---
// Test that lists are affected by correct indents.
#let fruit = [
  - Apple
  - Orange
  #list(body-indent: 20pt, [Pear])
]

- Fruit
[#set list(indent: 10pt)
 #fruit]
- No more fruit

---
// Test that that par spacing and text style are respected from
// the outside, but the more specific fill is respected.
#set par(spacing: 4pt)
#set text(style: "italic", fill: eastern)
#let x = [And the forest #parbreak() lay silent!]
#text(fill: forest, x)

---
// Test that scoping works as expected.
{
  if true {
    set text(blue)
    [Blue ]
  }
  [Not blue]
}

---
// Test relative path resolving in layout phase.
#let choice = ("monkey.svg", "rhino.png", "tiger.jpg")
#set enum(label: n => {
  let path = "../../res/" + choice(n - 1)
  move(dy: -0.15em, image(path, width: 1em, height: 1em))
})

. Monkey
. Rhino
. Tiger

---
{ set text(blue) + set text(tracking: 1pt) }
Blue!
