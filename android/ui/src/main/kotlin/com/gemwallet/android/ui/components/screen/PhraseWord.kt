package com.gemwallet.android.ui.components.screen

import androidx.compose.runtime.Immutable
import uniffi.gemstone.GemSecretPhraseRow
import uniffi.gemstone.secretPhraseRows

data class PhraseWord(val index: Int, val word: String)

@Immutable
sealed interface PhraseRow {
    data class Pair(val left: PhraseWord, val right: PhraseWord) : PhraseRow
    data class Single(val word: PhraseWord) : PhraseRow
}

fun phraseRows(words: List<String>): List<PhraseRow> = phraseRows(secretPhraseRows(words.size), words)

fun phraseRows(rows: List<GemSecretPhraseRow>, words: List<String>): List<PhraseRow> = rows.map { row ->
    when (row) {
        is GemSecretPhraseRow.Pair -> PhraseRow.Pair(left = words.phraseWord(row.left), right = words.phraseWord(row.right))
        is GemSecretPhraseRow.Single -> PhraseRow.Single(word = words.phraseWord(row.index))
    }
}

private fun List<String>.phraseWord(index: Int) = PhraseWord(index = index, word = this[index])
