package com.gemwallet.android.ext

fun String.boldMarkdown() = "**$this**"

fun String.words(): List<String> = trim().split("\\s+".toRegex()).filter { it.isNotBlank() }
