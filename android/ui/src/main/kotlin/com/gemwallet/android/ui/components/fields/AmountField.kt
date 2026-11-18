package com.gemwallet.android.ui.components.fields

import androidx.compose.foundation.clickable
import androidx.compose.foundation.interaction.MutableInteractionSource
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.ColumnScope
import androidx.compose.foundation.layout.IntrinsicSize
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.text.BasicTextField
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.foundation.text.input.OutputTransformation
import androidx.compose.foundation.text.input.TextFieldLineLimits
import androidx.compose.foundation.text.input.TextFieldState
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.CompositionLocalProvider
import androidx.compose.runtime.remember
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.SolidColor
import androidx.compose.ui.platform.LocalLayoutDirection
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.input.ImeAction
import androidx.compose.ui.text.input.KeyboardType
import androidx.compose.ui.text.input.PlatformImeOptions
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.unit.LayoutDirection
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.icons.AppIcons
import com.gemwallet.android.ui.theme.compactIconSize
import com.gemwallet.android.ui.theme.paddingDefault
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
    state: TextFieldState,
    symbol: AmountSymbolUIModel,
    equivalent: String,
    onNext: () -> Unit,
    modifier: Modifier = Modifier,
    onInputTypeClick: (() -> Unit)? = null,
    readOnly: Boolean = false,
    keyboardType: KeyboardType = KeyboardType.Decimal,
    maximumFractionDigits: Int? = null,
    error: String,
    errorInfo: (() -> Unit)? = null,
    textStyle: TextStyle = MaterialTheme.typography.displaySmall,
) {
    val interactionSource = remember { MutableInteractionSource() }

    CompositionLocalProvider(LocalLayoutDirection provides LayoutDirection.Ltr) {
        Row(
            modifier = Modifier.fillMaxWidth(),
            verticalAlignment = Alignment.CenterVertically,
            horizontalArrangement = Arrangement.spacedBy(paddingSmall, Alignment.CenterHorizontally),
        ) {
            if (symbol.placement == AmountSymbolPlacement.Leading) {
                AmountSymbol(symbol.symbol, textStyle)
            }
            BasicTextField(
                modifier = modifier.weight(1f, fill = false).width(IntrinsicSize.Min),
                state = state,
                readOnly = readOnly,
                inputTransformation = AmountInputTransformation(maximumFractionDigits),
                outputTransformation = ZeroPlaceholder,
                lineLimits = TextFieldLineLimits.SingleLine,
                textStyle = textStyle.copy(
                    textAlign = TextAlign.Center,
                    color = if (state.text.isEmpty()) MaterialTheme.colorScheme.secondary else MaterialTheme.colorScheme.onSurface,
                ),
                keyboardOptions = decimalKeyboardOptions(keyboardType, ImeAction.Next),
                onKeyboardAction = { onNext() },
                interactionSource = interactionSource,
                cursorBrush = SolidColor(MaterialTheme.colorScheme.primary),
            )
            if (symbol.placement == AmountSymbolPlacement.Trailing) {
                AmountSymbol(symbol.symbol, textStyle)
            }
        }
    }
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
                modifier = Modifier.testTag("amount_equivalent"),
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
            modifier = Modifier.padding(horizontal = paddingDefault),
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
                    contentDescription = stringResource(R.string.common_learn_more),
                    tint = MaterialTheme.colorScheme.secondaryFaded,
                )
            }
            Text(
                text = error,
                color = MaterialTheme.colorScheme.error,
                textAlign = TextAlign.Center,
            )
        }
    }
}

@Composable
private fun AmountSymbol(symbol: String, textStyle: TextStyle) {
    Text(
        text = symbol,
        style = textStyle,
        color = MaterialTheme.colorScheme.onSurface,
        maxLines = 1,
    )
}

private val ZeroPlaceholder = OutputTransformation {
    if (length == 0) {
        append("0")
    }
}
