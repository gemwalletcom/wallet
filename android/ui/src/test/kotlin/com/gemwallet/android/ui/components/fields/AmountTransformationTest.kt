package com.gemwallet.android.ui.components.fields

import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.AnnotatedString
import com.gemwallet.android.ui.models.AmountInputType
import org.junit.Assert.assertEquals
import org.junit.Assert.assertTrue
import org.junit.Test

class AmountTransformationTest {

    @Test
    fun cryptoAmount_keepsEnteredValueAndSymbolWithoutEmptySpans() {
        val result = CryptoAmountTransformation(
            symbol = "ETH",
            inputType = AmountInputType.Crypto,
            color = Color.Gray,
        ).filter(AnnotatedString("12")).text

        assertEquals("12 ETH", result.text)
        assertTrue(result.spanStyles.none { it.start == it.end })
    }

    @Test
    fun fiatAmount_keepsEnteredValueAndSymbolWithoutEmptySpans() {
        val result = CryptoAmountTransformation(
            symbol = "${'$'}",
            inputType = AmountInputType.Fiat,
            color = Color.Gray,
        ).filter(AnnotatedString("12")).text

        assertEquals("${'$'} 12", result.text)
        assertTrue(result.spanStyles.none { it.start == it.end })
    }
}
