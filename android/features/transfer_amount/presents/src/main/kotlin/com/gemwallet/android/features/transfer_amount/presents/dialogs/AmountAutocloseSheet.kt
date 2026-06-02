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
import com.gemwallet.android.domains.perpetual.autoclose.AutocloseError
import com.gemwallet.android.domains.perpetual.autoclose.AutocloseEstimator
import com.gemwallet.android.domains.perpetual.autoclose.AutocloseField
import com.gemwallet.android.domains.perpetual.autoclose.AutocloseValidator
import com.gemwallet.android.ext.PerpetualFormatter
import com.gemwallet.android.features.transfer_amount.viewmodels.providers.AmountPerpetualProvider
import com.gemwallet.android.math.parseNumberOrNull
import com.gemwallet.android.model.CurrencyFormatter
import com.gemwallet.android.model.NumericFormatter
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.buttons.MainActionButton
import com.gemwallet.android.ui.components.list_item.property.PropertyItem
import com.gemwallet.android.ui.components.perpetual.AutocloseInputSection
import com.gemwallet.android.ui.components.perpetual.AutocloseSuggestionsBar
import com.gemwallet.android.ui.components.screen.ModalBottomSheet
import com.gemwallet.android.ui.models.ListPosition
import com.gemwallet.android.ui.models.perpetual.autoclose.AutocloseUIModel
import com.gemwallet.android.ui.models.perpetual.autoclose.AutocloseUIModelFactory
import com.gemwallet.android.ui.theme.Spacer16
import com.gemwallet.android.ui.theme.paddingDefault
import com.wallet.core.primitives.Currency
import com.wallet.core.primitives.PerpetualDirection
import com.wallet.core.primitives.TpslType

@Composable
internal fun AmountAutocloseSheet(
    isVisible: Boolean,
    provider: AmountPerpetualProvider,
    amount: String,
    onDismiss: () -> Unit,
) {
    if (!isVisible) return
    val perpetual = provider.perpetual.collectAsStateWithLifecycle().value ?: run {
        onDismiss()
        return
    }
    val storedTakeProfit by provider.takeProfit.collectAsStateWithLifecycle()
    val storedStopLoss by provider.stopLoss.collectAsStateWithLifecycle()
    val leverageState by provider.leverageState.collectAsStateWithLifecycle()

    val direction = provider.direction
    val marketPrice = perpetual.price
    val assetDecimals = perpetual.asset.decimals
    val perpetualProvider = perpetual.provider
    val leverage = leverageState?.current ?: 1
    val marketPriceText = usdFormatter.string(marketPrice)
    val sizeText = usdFormatter.string((amount.parseNumberOrNull()?.toDouble() ?: 0.0) * leverage)

    var takeProfitText by remember { mutableStateOf(storedTakeProfit.orEmpty()) }
    var stopLossText by remember { mutableStateOf(storedStopLoss.orEmpty()) }
    var submitAttempted by remember { mutableStateOf(false) }
    var focused: TpslType? by remember { mutableStateOf(null) }

    val numericFormatter = remember { NumericFormatter() }
    val estimator = provider.estimatorFor(amount)
    val takeProfitPrice = numericFormatter.double(takeProfitText)
    val stopLossPrice = numericFormatter.double(stopLossText)
    val takeProfitRawError = AutocloseValidator(TpslType.TakeProfit, direction, marketPrice).error(takeProfitPrice)
    val stopLossRawError = AutocloseValidator(TpslType.StopLoss, direction, marketPrice).error(stopLossPrice)
    val takeProfitField = buildField(TpslType.TakeProfit, takeProfitPrice, takeProfitRawError, estimator, submitAttempted)
    val stopLossField = buildField(TpslType.StopLoss, stopLossPrice, stopLossRawError, estimator, submitAttempted)

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
    val isTakeProfitValid = takeProfitText.isEmpty() || takeProfitRawError == null
    val isStopLossValid = stopLossText.isEmpty() || stopLossRawError == null
    val hasInput = takeProfitPrice != null || stopLossPrice != null ||
        (takeProfitText.isEmpty() && stopLossText.isEmpty())
    val confirmEnabled = if (submitAttempted) isTakeProfitValid && isStopLossValid else hasInput

    ModalBottomSheet(
        isVisible = isVisible,
        onDismissRequest = onDismiss,
        skipPartiallyExpanded = true,
        title = stringResource(R.string.perpetual_auto_close),
    ) {
        Column(
            modifier = Modifier
                .fillMaxWidth()
                .fillMaxHeight()
                .padding(horizontal = paddingDefault)
                .imePadding(),
        ) {
            OpenPositionItem(
                asset = perpetual.asset,
                direction = direction,
                leverage = leverage,
                sizeText = sizeText,
                listPosition = ListPosition.Single,
            )
            Spacer16()
            PropertyItem(
                title = stringResource(R.string.perpetual_market_price),
                data = marketPriceText,
                listPosition = ListPosition.Single,
            )
            Spacer16()
            AutocloseInputSection(
                field = takeProfitField,
                text = takeProfitText,
                onTextChanged = {
                    submitAttempted = false
                    takeProfitText = it
                },
                onFocusChanged = { hasFocus ->
                    if (hasFocus) focused = TpslType.TakeProfit
                    else if (focused == TpslType.TakeProfit) focused = null
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
                    if (hasFocus) focused = TpslType.StopLoss
                    else if (focused == TpslType.StopLoss) focused = null
                },
            )
            Spacer(Modifier.weight(1f))
            if (activeField != null && activeText.isEmpty()) {
                AutocloseSuggestionsBar(
                    suggestions = activeField.percentSuggestions,
                    onPercentSelected = { percent ->
                        val target = estimator.targetPriceFromRoe(percent, activeField.type)
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
                Spacer16()
            }
            MainActionButton(
                title = stringResource(R.string.common_done),
                enabled = confirmEnabled,
                onClick = {
                    submitAttempted = true
                    if (isTakeProfitValid && isStopLossValid) {
                        provider.setTakeProfit(takeProfitText.takeIf { it.isNotEmpty() })
                        provider.setStopLoss(stopLossText.takeIf { it.isNotEmpty() })
                        onDismiss()
                    }
                },
            )
            Spacer16()
        }
    }
}

private val usdFormatter = CurrencyFormatter(type = CurrencyFormatter.Type.Currency, currency = Currency.USD)

private fun buildField(
    type: TpslType,
    price: Double?,
    error: AutocloseError?,
    estimator: AutocloseEstimator,
    showErrors: Boolean,
): AutocloseUIModel.Field {
    val field = AutocloseField(
        type = type,
        price = price,
        originalPrice = null,
        formattedPrice = null,
        error = error,
        orderId = null,
    )
    return AutocloseUIModelFactory.createField(field = field, estimator = estimator, showErrors = showErrors)
}
