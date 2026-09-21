package com.gemwallet.android.features.transfer_amount.presents.dialogs

import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.imePadding
import androidx.compose.foundation.layout.padding
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.gemwallet.android.ext.PerpetualFormatter
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.features.transfer_amount.viewmodels.providers.AmountPerpetualProvider
import com.gemwallet.android.math.parseInputNumberOrNull
import com.gemwallet.android.model.CurrencyFormatter
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.PercentSuggestionsBar
import com.gemwallet.android.ui.components.buttons.MainActionButton
import com.gemwallet.android.ui.components.list_item.ListItem
import com.gemwallet.android.ui.components.perpetual.AutocloseInputSection
import com.gemwallet.android.ui.components.screen.ModalBottomSheet
import com.gemwallet.android.ui.components.screen.SheetExpansion
import com.gemwallet.android.ui.models.ListPosition
import com.gemwallet.android.ui.models.buttonState
import com.gemwallet.android.ui.theme.Spacer16
import com.gemwallet.android.ui.theme.paddingDefault
import com.wallet.core.primitives.Currency
import com.wallet.core.primitives.TpslType
import uniffi.gemstone.AutocloseValidation

@Composable
internal fun AmountAutocloseSheet(isVisible: Boolean, provider: AmountPerpetualProvider, amount: String, onDismiss: () -> Unit) {
    if (!isVisible) return
    val perpetual = provider.perpetual.collectAsStateWithLifecycle().value ?: run {
        onDismiss()
        return
    }
    val storedTakeProfit by provider.takeProfit.collectAsStateWithLifecycle()
    val storedStopLoss by provider.stopLoss.collectAsStateWithLifecycle()
    val marketPriceListItem by provider.marketPriceListItem.collectAsStateWithLifecycle()

    val assetDecimals = perpetual.asset.decimals
    val perpetualProvider = perpetual.provider

    var takeProfitText by remember { mutableStateOf(storedTakeProfit.orEmpty()) }
    var stopLossText by remember { mutableStateOf(storedStopLoss.orEmpty()) }
    var submitAttempted by remember { mutableStateOf(false) }
    var focused: TpslType? by remember { mutableStateOf(null) }

    val estimator = provider.estimatorFor(amount)
    val takeProfitPrice = takeProfitText.parseInputNumberOrNull()?.toDouble()
    val stopLossPrice = stopLossText.parseInputNumberOrNull()?.toDouble()
    val takeProfitField = provider.autocloseField(TpslType.TakeProfit, amount, takeProfitPrice, submitAttempted)
    val stopLossField = provider.autocloseField(TpslType.StopLoss, amount, stopLossPrice, submitAttempted)

    val activeField = focused?.let {
        when (it) {
            TpslType.TakeProfit -> takeProfitField
            TpslType.StopLoss -> stopLossField
        }
    }
    val activeText = when (focused) {
        TpslType.TakeProfit -> takeProfitText
        TpslType.StopLoss -> stopLossText
        null -> ""
    }
    val isTakeProfitValid = takeProfitText.isEmpty() || takeProfitField.validation == AutocloseValidation.VALID
    val isStopLossValid = stopLossText.isEmpty() || stopLossField.validation == AutocloseValidation.VALID
    val hasInput = takeProfitPrice != null || stopLossPrice != null ||
        (takeProfitText.isEmpty() && stopLossText.isEmpty())
    val confirmEnabled = if (submitAttempted) isTakeProfitValid && isStopLossValid else hasInput

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
            provider.openPositionListItem(amount)?.let { ListItem(model = it, listPosition = ListPosition.Single) }
            Spacer16()
            marketPriceListItem?.let { ListItem(model = it, listPosition = ListPosition.Single) }
            Spacer16()
            AutocloseInputSection(
                field = takeProfitField,
                text = takeProfitText,
                onTextChanged = {
                    submitAttempted = false
                    takeProfitText = it
                },
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
                field = stopLossField,
                text = stopLossText,
                onTextChanged = {
                    submitAttempted = false
                    stopLossText = it
                },
                onFocusChanged = { hasFocus ->
                    if (hasFocus) {
                        focused = TpslType.StopLoss
                    } else if (focused == TpslType.StopLoss) {
                        focused = null
                    }
                },
            )
            Spacer(Modifier.weight(1f))
            if (activeField != null && activeText.isEmpty()) {
                PercentSuggestionsBar(
                    suggestions = activeField.percentSuggestions,
                    onPercentSelected = { percent ->
                        val target = estimator.targetPriceFromRoe(percent, activeField.type.toGem())
                        val formatted = PerpetualFormatter.formatInputPrice(
                            provider = perpetualProvider,
                            price = target,
                            decimals = assetDecimals,
                        )
                        submitAttempted = false
                        when (activeField.type) {
                            TpslType.TakeProfit -> takeProfitText = formatted
                            TpslType.StopLoss -> stopLossText = formatted
                        }
                    },
                )
            } else {
                MainActionButton(
                    title = stringResource(R.string.common_done),
                    state = buttonState(enabled = confirmEnabled),
                    onClick = {
                        submitAttempted = true
                        if (isTakeProfitValid && isStopLossValid) {
                            provider.setTakeProfit(takeProfitText.takeIf { it.isNotEmpty() })
                            provider.setStopLoss(stopLossText.takeIf { it.isNotEmpty() })
                            onDismiss()
                        }
                    },
                )
            }
            Spacer16()
        }
    }
}

private val usdFormatter = CurrencyFormatter(type = CurrencyFormatter.Type.Currency, currency = Currency.USD)
