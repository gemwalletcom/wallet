package com.gemwallet.android.ext

fun String.boldMarkdown() = "**$this**"

fun String.truncate(first: Int = 6, last: Int = 6): String {
    if (length <= first + last) return this
    return "${take(first)}...${takeLast(last)}"
}

fun String.words(): List<String> {
    val words = mutableListOf<String>()
    val word = StringBuilder()
    for (char in this) {
        when {
            !char.isWhitespace() -> word.append(char)
            word.isNotEmpty() -> {
                words.add(word.toString())
                word.clear()
            }
        }
    }
    if (word.isNotEmpty()) {
        words.add(word.toString())
    }
    return words
}
