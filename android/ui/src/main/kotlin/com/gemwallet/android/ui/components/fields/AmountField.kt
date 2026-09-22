package com.gemwallet.android.ui.components.fields

import androidx.compose.foundation.clickable
import androidx.compose.foundation.interaction.MutableInteractionSource
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.ColumnScope
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.text.BasicTextField
import androidx.compose.foundation.text.KeyboardActions
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.SolidColor
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.text.AnnotatedString
import androidx.compose.ui.text.SpanStyle
import androidx.compose.ui.text.TextRange
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.buildAnnotatedString
import androidx.compose.ui.text.input.ImeAction
import androidx.compose.ui.text.input.KeyboardType
import androidx.compose.ui.text.input.PlatformImeOptions
import androidx.compose.ui.text.input.OffsetMapping
import androidx.compose.ui.text.input.TextFieldValue
import androidx.compose.ui.text.input.TransformedText
import androidx.compose.ui.text.input.VisualTransformation
import androidx.compose.ui.text.style.TextAlign
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.icons.AppIcons
import com.gemwallet.android.ui.theme.compactIconSize
import com.gemwallet.android.ui.theme.paddingSmall
import com.gemwallet.android.ui.theme.secondaryFaded

fun decimalKeyboardOptions(keyboardType: KeyboardType = KeyboardType.Decimal, imeAction: ImeAction = ImeAction.Default) = KeyboardOptions(
    autoCorrectEnabled = false,
    keyboardType = keyboardType,
    imeAction = imeAction,
    platformImeOptions = PlatformImeOptions(privateImeOptions = "disableToolbar=true"),
)

@Composable
fun ColumnScope.AmountField(
    amount: String,
    symbol: AmountSymbolUIModel,
    equivalent: String,
    onValueChange: (String) -> Unit,
    onNext: () -> Unit,
    modifier: Modifier = Modifier,
    onInputTypeClick: (() -> Unit)? = null,
    readOnly: Boolean = false,
    keyboardType: KeyboardType = KeyboardType.Decimal,
    maximumFractionDigits: UInt? = null,
    error: String,
    errorInfo: (() -> Unit)? = null,
    textStyle: TextStyle = MaterialTheme.typography.displaySmall,
    transformation: AmountTransformation = CryptoAmountTransformation(symbol.symbol, symbol.placement, MaterialTheme.colorScheme.secondary),
) {
    val interactionSource = remember { MutableInteractionSource() }
    var fieldValue by remember { mutableStateOf(TextFieldValue(amount, TextRange(amount.length))) }
    val displayed = if (fieldValue.text == amount) fieldValue else TextFieldValue(amount, TextRange(amount.length))

    BasicTextField(
        modifier = modifier,
        value = displayed,
        onValueChange = { newValue ->
            val sanitized = sanitizeAmount(newValue.text, maximumFractionDigits)
            fieldValue = if (sanitized == newValue.text) newValue else TextFieldValue(sanitized, TextRange(amountCursor(newValue.selection.end, newValue.text, sanitized)))
            if (sanitized != amount) onValueChange(sanitized)
        },
        visualTransformation = transformation,
        maxLines = 1,
        textStyle = textStyle.copy(
            textAlign = TextAlign.Center,
            color = MaterialTheme.colorScheme.onSurface,
        ),
        keyboardOptions = decimalKeyboardOptions(keyboardType, ImeAction.Next),
        keyboardActions = KeyboardActions(
            onNext = { onNext() },
        ),
        interactionSource = interactionSource,
        cursorBrush = SolidColor(MaterialTheme.colorScheme.primary),
        readOnly = readOnly,
    )
    Spacer(modifier = Modifier.height(paddingSmall))
    if (equivalent.isNotEmpty()) {
        Row(
            modifier = if (onInputTypeClick == null) {
                Modifier
            } else {
                Modifier.clickable(
                    interactionSource = null,
                    indication = null,
                    onClick = onInputTypeClick,
                )
            },
            verticalAlignment = Alignment.CenterVertically,
            horizontalArrangement = Arrangement.spacedBy(paddingSmall),
        ) {
            Text(
                text = equivalent,
                color = MaterialTheme.colorScheme.secondary,
            )
            onInputTypeClick?.let {
                Icon(
                    modifier = Modifier.size(compactIconSize),
                    painter = painterResource(R.drawable.amount_switch),
                    tint = MaterialTheme.colorScheme.secondary,
                    contentDescription = null,
                )
            }
        }
    }
    error.takeIf { it.isNotEmpty() }?.run {
        Row(
            verticalAlignment = Alignment.CenterVertically,
            horizontalArrangement = Arrangement.spacedBy(paddingSmall),
        ) {
            errorInfo?.let { onInfo ->
                Icon(
                    modifier = Modifier
                        .size(compactIconSize)
                        .clip(CircleShape)
                        .clickable(onClick = onInfo),
                    imageVector = AppIcons.InfoOutlined,
                    contentDescription = null,
                    tint = MaterialTheme.colorScheme.secondaryFaded,
                )
            }
            Text(
                text = error,
                color = MaterialTheme.colorScheme.error,
            )
        }
    }
}

class CryptoAmountTransformation(symbol: String, placement: AmountSymbolPlacement, color: Color) : AmountTransformation(placement, symbol, color) {

    override fun transformText(text: AnnotatedString): AnnotatedString {
        val zeroValue = if (text.isEmpty()) "0" else ""
        val info = buildAnnotatedString {
            when (placement) {
                AmountSymbolPlacement.Trailing -> {
                    append(zeroValue)
                    append(" ")
                    append(symbol)
                    addPlaceholderStyle(
                        zeroValue = zeroValue,
                        color = color,
                        start = 0,
                        end = zeroValue.length,
                    )
                }

                AmountSymbolPlacement.Leading -> {
                    append(symbol)
                    append(" ")
                    append(zeroValue)
                    addPlaceholderStyle(
                        zeroValue = zeroValue,
                        color = color,
                        start = symbol.length,
                        end = symbol.length + zeroValue.length + 1,
                    )
                }
            }
        }
        return when (placement) {
            AmountSymbolPlacement.Trailing -> text + info
            AmountSymbolPlacement.Leading -> info + text
        }
    }

    override fun convertToOriginal(text: AnnotatedString, offset: Int): Int = when (placement) {
        AmountSymbolPlacement.Trailing -> if (offset > text.text.length) text.text.length else offset
        AmountSymbolPlacement.Leading -> if (offset > text.text.length) 0 else text.text.length
    }
}

abstract class AmountTransformation(val placement: AmountSymbolPlacement, val symbol: String, val color: Color) : VisualTransformation {

    override fun filter(text: AnnotatedString): TransformedText {
        val result = transformText(text)
        val offsetMapping = object : OffsetMapping {
            override fun originalToTransformed(offset: Int): Int = offset + when (placement) {
                AmountSymbolPlacement.Trailing -> 0
                AmountSymbolPlacement.Leading -> symbol.length + 1 + if (text.isEmpty()) 1 else 0
            }

            override fun transformedToOriginal(offset: Int): Int = convertToOriginal(text, offset)
        }
        // Add formatting
        return TransformedText(result, offsetMapping)
    }

    abstract fun transformText(text: AnnotatedString): AnnotatedString

    abstract fun convertToOriginal(text: AnnotatedString, offset: Int): Int
}

private fun AnnotatedString.Builder.addPlaceholderStyle(zeroValue: String, color: Color, start: Int, end: Int) {
    if (zeroValue.isEmpty()) {
        return
    }
    addStyle(
        SpanStyle(color = color),
        start = start,
        end = end,
    )
}
