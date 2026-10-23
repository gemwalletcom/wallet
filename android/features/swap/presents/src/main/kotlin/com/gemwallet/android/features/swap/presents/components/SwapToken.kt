package com.gemwallet.android.features.swap.presents.components

import androidx.compose.animation.AnimatedContent
import androidx.compose.animation.EnterTransition
import androidx.compose.animation.core.snap
import androidx.compose.animation.core.tween
import androidx.compose.animation.fadeIn
import androidx.compose.animation.fadeOut
import androidx.compose.animation.togetherWith
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.text.BasicTextField
import androidx.compose.foundation.text.input.TextFieldLineLimits
import androidx.compose.foundation.text.input.TextFieldState
import androidx.compose.foundation.text.input.rememberTextFieldState
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.remember
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.focus.FocusRequester
import androidx.compose.ui.focus.focusRequester
import androidx.compose.ui.graphics.SolidColor
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.text.style.TextOverflow
import com.gemwallet.android.model.text
import com.gemwallet.android.ui.components.NumericText
import com.gemwallet.android.ui.components.clickable
import com.gemwallet.android.ui.components.fields.AmountInputTransformation
import com.gemwallet.android.ui.components.fields.decimalKeyboardOptions
import com.gemwallet.android.ui.components.image.AssetIcon
import com.gemwallet.android.ui.components.list_item.listItem
import com.gemwallet.android.ui.components.progress.CircularProgressIndicator20
import com.gemwallet.android.ui.icons.AppIcons
import com.gemwallet.android.ui.localization.string
import com.gemwallet.android.ui.models.ListPosition
import com.gemwallet.android.ui.theme.listItemIconSize
import com.gemwallet.android.ui.theme.paddingHalfSmall
import com.gemwallet.android.ui.theme.paddingMiddle
import com.gemwallet.android.ui.theme.paddingSmall
import com.gemwallet.android.ui.theme.smallPadding
import com.gemwallet.android.ui.theme.space2
import com.wallet.core.primitives.AssetData
import uniffi.gemstone.GemAssetIcon
import uniffi.gemstone.GemLocalizedText
import uniffi.gemstone.GemSwapSideInteraction
import uniffi.gemstone.GemSwapSideState

@Composable
internal fun SwapToken(
    item: AssetData?,
    side: GemSwapSideState,
    calculating: Boolean = false,
    quote: String? = null,
    isQuote: Boolean = false,
    state: TextFieldState = rememberTextFieldState(),
    onBalanceClick: () -> Unit,
    onAssetSelect: () -> Unit,
) {
    Box(
        modifier = Modifier
            .listItem(ListPosition.Single)
            .padding(horizontal = paddingMiddle, vertical = paddingMiddle)
            .fillMaxWidth(),
        contentAlignment = Alignment.CenterStart,
    ) {
        Row(
            modifier = Modifier
                .fillMaxWidth()
                .heightIn(min = listItemIconSize),
            verticalAlignment = Alignment.CenterVertically,
            horizontalArrangement = Arrangement.spacedBy(paddingSmall),
        ) {
            Column(
                modifier = Modifier.weight(1f),
                verticalArrangement = Arrangement.spacedBy(space2),
            ) {
                SwapItemInput(
                    calculating = calculating,
                    quote = quote,
                    isQuote = isQuote,
                    interaction = side.interaction,
                    state = state,
                    assetSelected = item != null,
                    placeholder = side.amountPlaceholder,
                )
                QuoteFade(isLoading = calculating) {
                    SwapEquivalent(equivalent = side.fiat?.text().orEmpty(), animated = isQuote)
                }
            }
            Column(
                horizontalAlignment = Alignment.End,
                verticalArrangement = Arrangement.spacedBy(space2),
            ) {
                SwapItemLotInfo(
                    title = side.title.string(LocalContext.current),
                    icon = side.icon,
                    enabled = side.interaction.isAssetSelectable,
                    onClick = onAssetSelect,
                )
                SwapBalance(
                    balance = side.balance,
                    interaction = side.interaction,
                    onBalanceClick = onBalanceClick,
                )
            }
        }
        if (calculating) {
            CircularProgressIndicator20()
        }
    }
}

@Composable
private fun SwapItemLotInfo(title: String, icon: GemAssetIcon?, enabled: Boolean, onClick: () -> Unit) {
    Row(
        modifier = Modifier
            .clickable(enabled = enabled, onClick = onClick)
            .smallPadding()
            .heightIn(min = listItemIconSize),
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.spacedBy(paddingSmall, Alignment.End),
    ) {
        icon?.let { AssetIcon(it) }
        Text(
            text = title,
            style = MaterialTheme.typography.titleMedium,
        )
        AssetPickerChevron()
    }
}

@Composable
private fun AssetPickerChevron() {
    Icon(
        imageVector = AppIcons.KeyboardArrowDown,
        contentDescription = null,
    )
}

@Composable
private fun SwapEquivalent(equivalent: String, animated: Boolean) {
    NumericText(
        modifier = Modifier.padding(vertical = paddingHalfSmall),
        text = equivalent,
        maxLines = 1,
        overflow = TextOverflow.Ellipsis,
        style = MaterialTheme.typography.bodySmall,
        animated = animated,
    )
}

@Composable
private fun SwapBalance(balance: GemLocalizedText?, interaction: GemSwapSideInteraction, onBalanceClick: () -> Unit) {
    Text(
        modifier = Modifier
            .clickable(
                enabled = balance != null && interaction.isBalanceActionEnabled,
                shape = MaterialTheme.shapes.small,
                onClick = onBalanceClick,
            )
            .smallPadding(),
        text = balance?.string(LocalContext.current).orEmpty(),
        minLines = 1,
        maxLines = 1,
        overflow = TextOverflow.Ellipsis,
        textAlign = TextAlign.End,
        style = MaterialTheme.typography.bodySmall,
    )
}

@Composable
private fun SwapItemInput(calculating: Boolean, quote: String?, isQuote: Boolean, interaction: GemSwapSideInteraction, assetSelected: Boolean, placeholder: String, state: TextFieldState = rememberTextFieldState()) {
    val focusRequester = remember { FocusRequester() }
    val amountTextStyle = MaterialTheme.typography.headlineSmall
    val inputTextStyle = amountTextStyle.copy(
        color = MaterialTheme.colorScheme.onSurface,
    )

    LaunchedEffect(assetSelected) {
        if (interaction.isAmountEditable && assetSelected) {
            focusRequester.requestFocus()
        }
    }
    Box(
        modifier = Modifier
            .fillMaxWidth()
            .padding(vertical = paddingHalfSmall)
            .heightIn(min = listItemIconSize),
        contentAlignment = Alignment.CenterStart,
    ) {
        if (isQuote) {
            QuoteFade(isLoading = calculating) { loading ->
                if (!loading) {
                    NumericText(
                        text = quote ?: placeholder,
                        style = amountTextStyle,
                        color = if (quote == null) MaterialTheme.colorScheme.secondary else MaterialTheme.colorScheme.onSurface,
                        maxLines = 1,
                    )
                }
            }
        } else {
            BasicTextField(
                modifier = Modifier.fillMaxWidth().focusRequester(focusRequester),
                state = state,
                inputTransformation = AmountInputTransformation(),
                textStyle = inputTextStyle,
                lineLimits = TextFieldLineLimits.SingleLine,
                cursorBrush = SolidColor(MaterialTheme.colorScheme.primary),
                keyboardOptions = decimalKeyboardOptions(),
                decorator = { innerTextField ->
                    if (placeholder.isNotEmpty() && state.text.isEmpty()) {
                        Text(
                            text = placeholder,
                            style = amountTextStyle,
                            color = MaterialTheme.colorScheme.secondary,
                        )
                    }
                    innerTextField()
                },
                readOnly = !interaction.isAmountEditable,
            )
        }
    }
}

@Composable
private fun QuoteFade(isLoading: Boolean, content: @Composable (Boolean) -> Unit) {
    AnimatedContent(
        targetState = isLoading,
        transitionSpec = { (if (targetState) EnterTransition.None else fadeIn(tween(QUOTE_FADE_IN_MS))) togetherWith fadeOut(snap()) using null },
        contentAlignment = Alignment.CenterStart,
        label = "quote_fade",
    ) { content(it) }
}

private const val QUOTE_FADE_IN_MS = 200
