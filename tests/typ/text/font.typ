// Test configuring font properties.

---
// Set same font size in three different ways.
#font(22pt)[A]
#font(200%)[A]
#font(size: 16.5pt + 50%)[A]

// Do nothing.
#font[Normal]

// Set style (is available).
#font(style: italic)[Italic]

// Set weight (is available).
#font(weight: bold)[Bold]

// Set stretch (not available, matching closest).
#font(stretch: 50%)[Condensed]

// Set family.
#font(family: "PT Sans")[Sans serif]

// Emoji.
Emoji: 🐪, 🌋, 🏞

// Math.
#font("Latin Modern Math")[
    ∫ 𝛼 + 3𝛽 d𝑡
]

// Colors.
#font(fill: eastern)[This is #font(fill: rgb("FA644B"))[way more] colorful.]

---
// Test top and bottom edge.

#page!(width: 170pt)
#let try(top, bottom) = rect(fill: conifer)[
    #font!(top-edge: top, bottom-edge: bottom)
    `From `#top` to `#bottom
]

#try(ascender, descender)
#try(ascender, baseline)
#try(cap-height, baseline)
#try(x-height, baseline)

---
// Test class definitions.
#font!(sans-serif: "PT Sans")
#font(family: sans-serif)[Sans-serif.] \
#font(monospace)[Monospace.] \
#font(monospace, monospace: ("Nope", "Latin Modern Math"))[Math.]

---
// Ref: false

// Error: 7-12 unexpected argument
#font(false)[]

// Error: 14-18 expected font style, found font weight
// Error: 28-34 expected font weight, found string
// Error: 43-44 expected string or array of strings, found integer
#font(style: bold, weight: "thin", serif: 0)[]

// Error: 7-27 unexpected argument
#font(something: "invalid")[]

// Error: 13-23 unexpected argument
#font(12pt, size: 10pt)[]

// Error: 16-35 unexpected argument
#font("Arial", family: "Helvetica")[]
