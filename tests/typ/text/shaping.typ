// Test complex text shaping.

---
// Test ligatures.

// This should create an "fi" ligature.
Le fira

// This should just shape nicely.
#font(family: "Noto Sans Arabic")
دع النص يمطر عليك

// This should form a three-member family.
#font(family: "Twitter Color Emoji")
👩‍👩‍👦 🤚🏿

// These two shouldn't be affected by a zero-width joiner.
🏞‍🌋

---
// Test font fallback.

#font(family: ("EB Garamond", "Noto Sans Arabic", "Twitter Color Emoji"))

// Font fallback for emoji.
A😀B

// Font fallback for entire text.
دع النص يمطر عليك

// Font fallback in right-to-left text.
ب🐈😀سم

// Multi-layer font fallback.
Aب😀🏞سمB

// Tofus are rendered with the first font.
A🐈中文B

---
// Test reshaping.

#font(family: "Noto Serif Hebrew")
#lang("he")
ס \ טֶ
