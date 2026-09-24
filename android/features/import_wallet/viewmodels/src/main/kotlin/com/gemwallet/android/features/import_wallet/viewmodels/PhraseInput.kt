package com.gemwallet.android.features.import_wallet.viewmodels

data class PhraseInput(val text: String, val cursor: Int) {
    private val start: Int = (cursor downTo 1).firstOrNull { text[it - 1].isWhitespace() } ?: 0
    private val end: Int = (cursor until text.length).firstOrNull { text[it].isWhitespace() } ?: text.length

    val word: String get() = text.substring(start, end)

    val wordCursor: Int get() = cursor - start

    fun completing(suggestion: String): PhraseInput {
        val rest = text.substring(end)
        val separator = rest.firstOrNull()?.takeIf { it.isWhitespace() }
        val head = text.substring(0, start) + suggestion + (separator ?: ' ')
        val tail = if (separator == null) rest else rest.drop(1)
        return PhraseInput(head + tail, head.length)
    }

    companion object {
        fun of(text: String, cursor: Int?): PhraseInput = PhraseInput(text, (cursor ?: text.length).coerceIn(0, text.length))
    }
}
