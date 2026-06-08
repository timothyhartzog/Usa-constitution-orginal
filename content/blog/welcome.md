---
title: Welcome to the Constitution Research Platform
date: 2026-05-27
slug: welcome
tags: announcement, platform
---

# Welcome to the Constitution Research Platform

This is a WebAssembly-powered research workbench for exploring the U.S. Constitution, the writings of its framers, and 194 national constitutions from around the world. The entire app — search, citation graph, timeline, document reader, blog — runs in your browser.

## Live archive statistics

The platform indexes thousands of text chunks across every major founding-era source plus comparative constitutions. Here's what's loaded right now:

{{widget:stats}}

## Search any concept

Blog posts can embed live search widgets. Below is a real, running BM25 search for the phrase "due process":

{{widget:search query="due process" limit=4}}

Clicking through any result opens the annotated document reader, where inline references to clauses, essays, and founders are highlighted and cross-linked.

## See how a single idea connects the founders

The citation graph extracts mentions of clauses, essays, and founders from every chunk in the corpus. Here is the local neighborhood of Article I, Section 8 — the powers of Congress:

{{widget:mini_graph target="clause:I.8" links=8}}

## Compare across collections

How does the question of executive power look in the Constitution itself, versus the Federalist Papers, versus the Anti-Federalist objections?

{{widget:compare topic="executive power" collections="constitution,federalist_papers,anti_federalist"}}

## What you can do

- **Search** the entire corpus by topic, author, date, or collection
- **Graph** the citation network and click any node to see its incoming references
- **Timeline** the constitutional process from the failed Articles to the Bill of Rights
- **World map** to browse 194 constitutions, filter by region
- **Blog** with embedded interactive widgets (you're reading one now)
- **Compose** new posts in the live Markdown editor

Head over to [Search](/search), [Graph](/graph), or the [World](/world) page to start exploring.
