package com.gemwallet.android.ui.components.fields

import androidx.compose.foundation.text.input.InputTransformation
import androidx.compose.foundation.text.input.TextFieldBuffer
import uniffi.gemstone.GemNumberFormat
import java.text.DecimalFormatSymbols

object AmountInputTransformation : InputTransformation {
    private val format = GemNumberFormat(DecimalFormatSymbols.getInstance().decimalSeparator.toString())

    fun isValid(input: CharSequence, maximumFractionDigits: UInt? = null): Boolean =
        input.toString().let { format.sanitize(it, maximumFractionDigits, null) == it }

    override fun TextFieldBuffer.transformInput() {
        if (!isValid(asCharSequence())) {
            revertAllChanges()
        }
    }
}
