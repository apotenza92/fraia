---
name: Knowledge request
description: Request a new Fraia knowledge topic, correction, or source-backed improvement.
title: "Knowledge: "
labels: [knowledge]
body:
  - type: markdown
    attributes:
      value: |
        Use this for Fraia wiki topic requests, source suggestions, or corrections. Do not upload copyrighted PDFs, copied source text, OCR output, or private screenshots.
  - type: textarea
    id: topic
    attributes:
      label: Topic or correction
      description: What should Fraia learn, improve, or correct?
    validations:
      required: true
  - type: textarea
    id: why
    attributes:
      label: Why it matters to Fraia
      description: How would this help scheme generation, modeling, diagnostics, explanations, or engineering workflows?
    validations:
      required: true
  - type: textarea
    id: sources
    attributes:
      label: Suggested original sources
      description: Provide URLs, titles, organizations/authors, page/section/figure references, and reliability/limits if known.
      placeholder: |
        - Organization/author, Title, URL or bibliographic locator, page/section, retrieved/consulted date, reliability/limits.
  - type: textarea
    id: scope
    attributes:
      label: Scope and cautions
      description: What is in scope, not in scope, jurisdiction/software limitations, or weak evidence?
  - type: checkboxes
    id: confirmation
    attributes:
      label: Contribution hygiene
      options:
        - label: I am not pasting copied source prose, OCR output, or raw excerpts.
          required: true
        - label: I am not uploading private/copyrighted PDFs, screenshots, or copied figures.
          required: true
