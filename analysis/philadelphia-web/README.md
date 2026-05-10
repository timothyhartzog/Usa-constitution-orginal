# The Philadelphia Web

**A three-part forensic analysis of the United States Constitutional Convention of 1787 — its delegates, alliances, antagonisms, ratification battle, and intellectual genealogy.**

Date assembled: May 10, 2026
Author: Generated in conversation with Claude (Anthropic)

---

## Contents

This repository contains three self-contained, interactive HTML documents. Each combines a force-directed network graph (rendered with D3.js, fully interactive — drag nodes, hover for biographical tooltips, filter by relationship type) with substantial long-form analytical prose. Open any file in a modern browser; no build step or server is required.

### [Part I — Factions at the Convention](./part_1_factions_at_the_convention.html)

The summer of 1787 inside Independence Hall. Network of all fifty-five delegates plus the structural cleavages among them: Nationalists, Connecticut Brokers, Small-State Resistance, Walkouts & Refusers, the Deep South slavery phalanx, and those who drifted in body or spirit. Covers the Virginia Triumvirate, the Pennsylvania Intellectual Core, the Connecticut Compromisers, the Carolina–Georgia bloc, the Delaware Wall, and the secret Massachusetts–South Carolina commercial bargain. Catalogs the major antagonisms: Hamilton vs. his own New York delegation, Madison vs. Mason, Luther Martin vs. nearly everyone, the Pinckney cousin rivalry, the Mason–Rutledge slavery confrontation, Bedford's "foreign ally" threat. Closes with the final reckoning of who signed and who refused on September 17.

- **Nodes:** 55 · **Edges:** 84
- **Word count:** ~4,800

### [Part II — Ratification and the Great Inversion](./part_2_ratification_and_inversion.html)

The thirteen months that followed the Convention plus the four years that followed those. Expands the graph to sixty nodes (the 55 delegates plus five consequential non-delegates: John Jay, Patrick Henry, George Clinton, Samuel Adams, Thomas Jefferson). Covers the *Federalist Papers* collaboration; the state-by-state ratification battles with full vote tallies; the three critical reversals (Randolph's flip in Virginia, Madison's pivot on the Bill of Rights, Henry's eventual reconciliation); and the Great Inversion of 1789–1795, when the Madison–Hamilton co-authorship dissolved into the leadership of two opposing parties. Closes with a chronology of the fates of the fifty-five — two presidents, four Justices, one duel, one expulsion, one disappearance.

- **Nodes:** 60 · **Edges:** 92
- **Word count:** ~5,400

### [Part III — The Intellectual Genealogy](./part_3_intellectual_genealogy.html)

The books and teachers behind the men. A three-layered network of delegates, educational institutions, and intellectual sources, traced across six identifiable traditions: Reformed and federal theology (Calvin → Althusius → the Westminster Confession → Witherspoon), Scottish Common Sense (Hutcheson → Reid → Smith → Witherspoon), classical republicanism (Polybius → Cicero → Plutarch → Tacitus), Enlightenment liberalism (Locke → Montesquieu → Hume), English common law (Coke → Blackstone), and the Hebrew Republic tradition (Cunaeus → Harrington → Selden). Documents the Witherspoon phenomenon (nine of the fifty-five were his Princeton students), the Calvinist anthropology underlying the architecture of restraint, the etymological lineage of *federal* from *foedus*, the Althusian foundation of nested covenantal sovereignty, the Lutz citation data, and what was conspicuously absent (Rousseau, Filmer, the Continental civil-law tradition).

- **Nodes:** 56 · **Edges:** 96
- **Word count:** ~5,300

---

## Notes on the Visual Design

All three documents share a consistent design language: parchment background, Cormorant Garamond and Libre Caslon Text typography, JetBrains Mono for metadata, and historically-rooted color tokens (oxblood, navy, ochre, forest green, sepia, slate). The graphs are force-directed via D3 v7. Nodes can be dragged; hovering exposes biographical tooltips; toggle buttons filter the visible edge types.

Each part is self-contained — no external dependencies beyond Google Fonts and the D3.js CDN — and renders identically on desktop and mobile.

---

## Sources

The analytical prose draws on, among others:

- Max Farrand, *The Records of the Federal Convention of 1787* (1911)
- James Madison, *Notes of Debates in the Federal Convention of 1787*
- Robert Yates, *Secret Proceedings and Debates of the Convention*
- William Pierce, *Character Sketches of Delegates to the Federal Convention*
- Pauline Maier, *Ratification: The People Debate the Constitution, 1787–1788* (2010)
- Michael Klarman, *The Framers' Coup* (2016)
- Ron Chernow, *Alexander Hamilton* (2004)
- Richard Beeman, *Plain, Honest Men* (2009)
- Gordon Wood, *The Creation of the American Republic, 1776–1787* (1969)
- Gordon Wood, *Empire of Liberty: A History of the Early Republic, 1789–1815* (2009)
- Bernard Bailyn, *The Ideological Origins of the American Revolution* (1967)
- Eric Nelson, *The Hebrew Republic* (2010)
- John Witte Jr., *The Reformation of Rights* (2007)
- Donald S. Lutz, *The Origins of American Constitutionalism* (1988)
- Donald S. Lutz, "The Relative Influence of European Writers on Late Eighteenth-Century American Political Thought," *American Political Science Review* 78, no. 1 (1984)
- Douglass Adair, "'That Politics May Be Reduced to a Science': David Hume, James Madison, and the Tenth Federalist," *Huntington Library Quarterly* 20, no. 4 (1957)
- Daniel Walker Howe, *What Hath God Wrought: The Transformation of America, 1815–1848* (2007)

---

## How to Read

The natural order is Part I → Part II → Part III, but each is self-contained. Part I gives you the room in 1787. Part II shows you what the room became over the next decade. Part III opens the bookshelves the men in the room were drawing from.

---

*Generated in conversation. Network graphs and prose composed by Claude (Anthropic), assembled at the direction of Timothy Hartzog.*
