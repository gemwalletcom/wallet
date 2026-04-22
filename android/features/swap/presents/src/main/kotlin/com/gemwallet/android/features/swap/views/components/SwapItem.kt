package com.gemwallet.android.features.swap.views.components

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.RowScope
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.text.BasicTextField
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.foundation.text.input.TextFieldLineLimits
import androidx.compose.foundation.text.input.TextFieldState
import androidx.compose.foundation.text.input.clearText
import androidx.compose.foundation.text.input.rememberTextFieldState
import androidx.compose.foundation.text.input.setTextAndPlaceCursorAtEnd
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.KeyboardArrowDown
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.remember
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.focus.FocusRequester
import androidx.compose.ui.focus.focusRequester
import androidx.compose.ui.graphics.SolidColor
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.input.KeyboardType
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.text.style.TextOverflow
import com.gemwallet.android.domains.asset.availableBalance
import com.gemwallet.android.domains.asset.availableBalanceFormatted
import com.gemwallet.android.model.AssetInfo
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.clickable
import com.gemwallet.android.ui.components.image.AssetIcon
import com.gemwallet.android.ui.components.list_item.listItem
import com.gemwallet.android.ui.components.progress.CircularProgressIndicator16
import com.gemwallet.android.ui.models.ListPosition
import com.gemwallet.android.ui.theme.listItemIconSize
import com.gemwallet.android.ui.theme.paddingDefault
import com.gemwallet.android.ui.theme.paddingMiddle
import com.gemwallet.android.ui.theme.paddingSmall
import com.gemwallet.android.ui.theme.space2
import com.gemwallet.android.ui.theme.smallPadding
import com.gemwallet.android.features.swap.viewmodels.models.SwapItemInteraction
import com.wallet.core.primitives.Asset

@Composable
internal fun SwapItem(
    item: AssetInfo?,
    equivalent: String,
    calculating: Boolean = false,
    interaction: SwapItemInteraction,
    state: TextFieldState = rememberTextFieldState(),
    onAssetSelect: () -> Unit,
) {
    Column(
        modifier = Modifier
            .listItem(ListPosition.Single)
            .padding(horizontal = paddingDefault, vertical = paddingMiddle)
            .fillMaxWidth(),
        verticalArrangement = Arrangement.spacedBy(space2)
    ) {
        Row(
            modifier = Modifier
                .heightIn(min = listItemIconSize)
                .fillMaxWidth(),
            verticalAlignment = Alignment.CenterVertically,
        ) {
            SwapItemInput(
                calculating = calculating,
                interaction = interaction,
                state = state,
            )
            SwapItemLotInfo(
                asset = item?.asset,
                enabled = interaction.isAssetSelectable,
                onClick = onAssetSelect,
            )
        }
        SwapValues(
            calculating = calculating,
            equivalent = equivalent,
            balance = item?.availableBalanceFormatted,
            interaction = interaction,
        ) {
            state.clearText()
            state.setTextAndPlaceCursorAtEnd(item?.availableBalance.orEmpty())
        }
    }
}

@Composable
private fun SwapItemLotInfo(
    asset: Asset?,
    enabled: Boolean,
    onClick: () -> Unit
) {
    if (asset == null) {
        SelectAssetInfo(enabled, onClick)
    } else {
        AssetInfo(asset, enabled, onClick)
    }
}

@Composable
private fun SelectAssetInfo(enabled: Boolean, onClick: () -> Unit) {
    Row(
        modifier = Modifier.clickable(enabled = enabled, onClick = onClick).smallPadding(),
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.End,
    ) {
        Text(
            text = stringResource(R.string.assets_select_asset),
            style = MaterialTheme.typography.bodyLarge,
        )
        AssetPickerChevron()
    }
}

@Composable
private fun AssetInfo(
    asset: Asset,
    enabled: Boolean,
    onClick: () -> Unit
) {
    Row(
        modifier = Modifier.clickable(enabled = enabled, onClick = onClick).smallPadding(),
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.spacedBy(paddingSmall, Alignment.End),
    ) {
        AssetIcon(asset)
        Text(
            text = asset.symbol,
            style = MaterialTheme.typography.titleMedium,
        )
        AssetPickerChevron()
    }
}

@Composable
private fun AssetPickerChevron() {
    Icon(
        imageVector = Icons.Default.KeyboardArrowDown,
        contentDescription = null,
    )
}

@Composable
private fun SwapValues(
    calculating: Boolean,
    equivalent: String,
    balance: String?,
    interaction: SwapItemInteraction,
    onBalanceClick: () -> Unit,
) {
    Row(
        modifier = Modifier.fillMaxWidth(),
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.spacedBy(paddingSmall)
    ) {
        Text(
            modifier = Modifier.fillMaxWidth(0.5f),
            text = if (calculating) "" else equivalent,
            style = MaterialTheme.typography.bodySmall,
        )
        Box(modifier = Modifier.fillMaxWidth(1f)) {
            Text(
                modifier = Modifier
                    .align(Alignment.CenterEnd)
                    .clickable(
                        enabled = interaction.isBalanceActionEnabled,
                        shape = MaterialTheme.shapes.small,
                        onClick = onBalanceClick,
                    )
                    .smallPadding(),
                text = balance?.let { stringResource(id = R.string.transfer_balance, it) } ?: "",
                maxLines = 1,
                overflow = TextOverflow.Ellipsis,
                textAlign = TextAlign.End,
                style = MaterialTheme.typography.bodySmall,
            )
        }
    }
}

@Composable
private fun RowScope.SwapItemInput(
    calculating: Boolean,
    interaction: SwapItemInteraction,
    state: TextFieldState = rememberTextFieldState(),
) {
    val focusRequester = remember { FocusRequester() }
    val amountTextStyle = MaterialTheme.typography.headlineSmall
    val inputTextStyle = amountTextStyle.copy(
        color = MaterialTheme.colorScheme.onSurface
    )

    LaunchedEffect(Unit) {
        if (interaction.isAmountEditable) {
            focusRequester.requestFocus()
        }
    }
    Box(
        modifier = Modifier
            .weight(1f)
            .fillMaxWidth()
    ) {
        if (calculating) {
            CircularProgressIndicator16()
        } else {
            BasicTextField(
                modifier = Modifier.fillMaxWidth().focusRequester(focusRequester),
                state = state,
                textStyle = inputTextStyle,
                lineLimits = TextFieldLineLimits.SingleLine,
                cursorBrush = SolidColor(MaterialTheme.colorScheme.primary),
                keyboardOptions = KeyboardOptions.Default.copy(
                    autoCorrectEnabled = false,
                    keyboardType = KeyboardType.Decimal
                ),
                decorator = { innerTextField ->
                    if (state.text.isEmpty()) {
                        Text(
                            text = "0",
                            style = amountTextStyle,
                            color = MaterialTheme.colorScheme.secondary,
                        )
                    }
                    innerTextField()
                },
                readOnly = !interaction.isAmountEditable
            )
        }
    }
}
