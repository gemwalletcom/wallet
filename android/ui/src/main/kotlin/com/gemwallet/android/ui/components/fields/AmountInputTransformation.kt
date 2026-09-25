package com.gemwallet.android.ui.components.fields

import androidx.compose.foundation.text.input.InputTransformation
import androidx.compose.foundation.text.input.TextFieldBuffer
import androidx.compose.ui.text.TextRange
import com.gemwallet.android.math.numberFormat

fun sanitizeAmount(text: String, maximumFractionDigits: UInt?): String = numberFormat().sanitize(text, maximumFractionDigits, null)

fun amountCursor(cursor: Int, text: String, sanitized: String): Int = (cursor - (text.length - sanitized.length)).coerceIn(0, sanitized.length)

class AmountInputTransformation(private val maximumFractionDigits: UInt? = null) : InputTransformation {
    override fun TextFieldBuffer.transformInput() {
        val text = asCharSequence().toString()
        val sanitized = sanitizeAmount(text, maximumFractionDigits)
        if (sanitized != text) {
            val cursor = amountCursor(selection.end, text, sanitized)
            replace(0, length, sanitized)
            selection = TextRange(cursor)
        }
    }
}
