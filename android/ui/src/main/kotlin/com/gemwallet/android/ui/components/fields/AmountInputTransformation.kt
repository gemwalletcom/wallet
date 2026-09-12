package com.gemwallet.android.ui.components.fields

import androidx.compose.foundation.text.input.InputTransformation
import androidx.compose.foundation.text.input.TextFieldBuffer
import uniffi.gemstone.GemNumberSanitizer
import java.text.DecimalFormatSymbols

object AmountInputTransformation : InputTransformation {
    private val sanitizer = GemNumberSanitizer(DecimalFormatSymbols.getInstance().decimalSeparator.toString(), null, null)

    fun isValid(input: CharSequence): Boolean = input.toString().let { sanitizer.sanitize(it) == it }

    override fun TextFieldBuffer.transformInput() {
        if (!isValid(asCharSequence())) {
            revertAllChanges()
        }
    }
}
