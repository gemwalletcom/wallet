package com.gemwallet.android.ui.components.fields

import androidx.compose.foundation.text.input.InputTransformation
import androidx.compose.foundation.text.input.TextFieldBuffer
import com.gemwallet.android.math.NumberSanitizer

object AmountInputTransformation : InputTransformation {
    private val sanitizer = NumberSanitizer()

    fun isValid(input: CharSequence): Boolean = input.toString().let { sanitizer.sanitize(it) == it }

    override fun TextFieldBuffer.transformInput() {
        if (!isValid(asCharSequence())) {
            revertAllChanges()
        }
    }
}
