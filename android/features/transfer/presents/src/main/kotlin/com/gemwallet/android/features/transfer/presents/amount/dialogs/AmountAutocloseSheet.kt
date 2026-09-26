package com.gemwallet.android.features.transfer.presents.amount.dialogs

import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.imePadding
import androidx.compose.foundation.layout.padding
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.gemwallet.android.ext.toPrimitives
import com.gemwallet.android.features.transfer.viewmodels.amount.providers.AmountPerpetualProvider
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.PercentSuggestionsBar
import com.gemwallet.android.ui.components.buttons.MainActionButton
import com.gemwallet.android.ui.components.list_item.AssetListItem
import com.gemwallet.android.ui.components.list_item.GemListRowView
import com.gemwallet.android.ui.components.perpetual.AutocloseInputSection
import com.gemwallet.android.ui.components.screen.ModalBottomSheet
import com.gemwallet.android.ui.components.screen.SheetExpansion
import com.gemwallet.android.ui.models.ListPosition
import com.gemwallet.android.ui.models.buttonState
import com.gemwallet.android.ui.theme.Spacer16
import com.gemwallet.android.ui.theme.paddingDefault
import com.wallet.core.primitives.TpslType

@Composable
internal fun AmountAutocloseSheet(isVisible: Boolean, provider: AmountPerpetualProvider, amount: String, onDismiss: () -> Unit) {
    if (!isVisible) return
    if (provider.perpetual.collectAsStateWithLifecycle().value == null) {
        onDismiss()
        return
    }
    var focused: TpslType? by remember { mutableStateOf(null) }

    LaunchedEffect(Unit) { provider.onAutocloseOpened(amount) }
    val viewState = provider.autocloseViewState.collectAsStateWithLifecycle().value ?: return

    val activeField = when (focused) {
        TpslType.TakeProfit -> viewState.takeProfit
        TpslType.StopLoss -> viewState.stopLoss
        null -> null
    }
    ModalBottomSheet(
        isVisible = isVisible,
        onDismissRequest = onDismiss,
        expansion = SheetExpansion.Full,
        title = stringResource(R.string.perpetual_auto_close),
    ) {
        Column(
            modifier = Modifier
                .fillMaxWidth()
                .fillMaxHeight()
                .padding(horizontal = paddingDefault)
                .imePadding(),
        ) {
            provider.openPositionRow(amount)?.let { AssetListItem(row = it, listPosition = ListPosition.Single) }
            Spacer16()
            viewState.priceRows.forEachIndexed { index, row ->
                GemListRowView(row = row, listPosition = ListPosition.getPosition(index, viewState.priceRows.size))
            }
            Spacer16()
            AutocloseInputSection(
                field = viewState.takeProfit,
                onTextChanged = { provider.onAutocloseChanged(TpslType.TakeProfit, it) },
                onFocusChanged = { hasFocus ->
                    if (hasFocus) {
                        focused = TpslType.TakeProfit
                    } else if (focused == TpslType.TakeProfit) {
                        focused = null
                    }
                },
            )
            Spacer16()
            AutocloseInputSection(
                field = viewState.stopLoss,
                onTextChanged = { provider.onAutocloseChanged(TpslType.StopLoss, it) },
                onFocusChanged = { hasFocus ->
                    if (hasFocus) {
                        focused = TpslType.StopLoss
                    } else if (focused == TpslType.StopLoss) {
                        focused = null
                    }
                },
            )
            Spacer(Modifier.weight(1f))
            if (activeField != null && activeField.text.isEmpty()) {
                PercentSuggestionsBar(
                    suggestions = activeField.suggestions,
                    onPercentSelected = { percent -> provider.onAutoclosePercentSelected(activeField.tpslType.toPrimitives(), percent) },
                )
            } else {
                MainActionButton(
                    title = stringResource(R.string.common_done),
                    state = buttonState(enabled = viewState.confirmEnabled),
                    onClick = {
                        if (provider.onAutocloseSubmitted()) {
                            onDismiss()
                        }
                    },
                )
            }
            Spacer16()
        }
    }
}
