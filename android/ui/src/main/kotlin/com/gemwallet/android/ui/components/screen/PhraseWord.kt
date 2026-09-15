package com.gemwallet.android.ui.components.screen

import uniffi.gemstone.GemSecretPhraseRow
import uniffi.gemstone.GemSecretPhraseWord
import uniffi.gemstone.secretPhraseRows

data class PhraseWord(
    val index: Int,
    val word: String,
)

sealed interface PhraseRow {
    data class Pair(val left: PhraseWord, val right: PhraseWord) : PhraseRow
    data class Single(val word: PhraseWord) : PhraseRow
}

fun phraseRows(words: List<String>): List<PhraseRow> = secretPhraseRows(words).map { row ->
    when (row) {
        is GemSecretPhraseRow.Pair -> PhraseRow.Pair(left = row.left.toUi(), right = row.right.toUi())
        is GemSecretPhraseRow.Single -> PhraseRow.Single(word = row.word.toUi())
    }
}

private fun GemSecretPhraseWord.toUi() = PhraseWord(index = index.toInt(), word = word)
