package com.gemwallet.android.features.transfer_amount.presents.dialogs

import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalFocusManager
import androidx.compose.ui.res.stringResource
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.gemwallet.android.domains.perpetual.autoclose.AutocloseEstimator
import com.gemwallet.android.domains.perpetual.autoclose.AutocloseField
import com.gemwallet.android.domains.perpetual.autoclose.AutocloseValidator
import com.gemwallet.android.ext.PerpetualFormatter
import com.gemwallet.android.features.transfer_amount.viewmodels.providers.AmountPerpetualProvider
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.buttons.MainActionButton
import com.gemwallet.android.ui.components.perpetual.AutocloseInputSection
import com.gemwallet.android.ui.components.perpetual.AutocloseSuggestionsBar
import com.gemwallet.android.ui.components.screen.ModalBottomSheet
import com.gemwallet.android.ui.models.perpetual.autoclose.AutocloseUIModel
import com.gemwallet.android.ui.models.perpetual.autoclose.AutocloseUIModelFactory
import com.gemwallet.android.ui.theme.Spacer16
import com.gemwallet.android.ui.theme.paddingDefault
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

    val direction = provider.direction
    val marketPrice = perpetual.price
    val assetDecimals = perpetual.asset.decimals
    val perpetualProvider = perpetual.provider

    var takeProfitText by remember { mutableStateOf(storedTakeProfit.orEmpty()) }
    var stopLossText by remember { mutableStateOf(storedStopLoss.orEmpty()) }
    var focused: TpslType? by remember { mutableStateOf(null) }
    val focusManager = LocalFocusManager.current

    val estimator = provider.estimatorFor(amount)
    val takeProfitField = buildField(TpslType.TakeProfit, takeProfitText, direction, marketPrice, estimator)
    val stopLossField = buildField(TpslType.StopLoss, stopLossText, direction, marketPrice, estimator)

    val activeField = focused?.let {
        when (it) {
            TpslType.TakeProfit -> takeProfitField
            TpslType.StopLoss -> stopLossField
        }
    }
    val activeText = focused?.let {
        when (it) {
            TpslType.TakeProfit -> takeProfitText
            TpslType.StopLoss -> stopLossText
        }
    }
    val confirmEnabled = (takeProfitText.toDoubleOrNull() != null && takeProfitField.error == null) ||
        (stopLossText.toDoubleOrNull() != null && stopLossField.error == null) ||
        takeProfitText.isEmpty() && stopLossText.isEmpty()

    ModalBottomSheet(
        isVisible = isVisible,
        onDismissRequest = onDismiss,
        title = stringResource(R.string.perpetual_auto_close),
    ) {
        Column(
            modifier = Modifier
                .fillMaxWidth()
                .padding(horizontal = paddingDefault),
        ) {
            AutocloseInputSection(
                field = takeProfitField,
                text = takeProfitText,
                onTextChanged = { takeProfitText = it },
                onFocusChanged = { hasFocus ->
                    if (hasFocus) focused = TpslType.TakeProfit
                    else if (focused == TpslType.TakeProfit) focused = null
                },
            )
            Spacer16()
            AutocloseInputSection(
                field = stopLossField,
                text = stopLossText,
                onTextChanged = { stopLossText = it },
                onFocusChanged = { hasFocus ->
                    if (hasFocus) focused = TpslType.StopLoss
                    else if (focused == TpslType.StopLoss) focused = null
                },
            )
            Spacer16()
            if (activeField != null && activeText.isNullOrEmpty()) {
                AutocloseSuggestionsBar(
                    suggestions = activeField.percentSuggestions,
                    onPercentSelected = { percent ->
                        val target = estimator.targetPriceFromRoe(percent, activeField.type)
                        val formatted = PerpetualFormatter.formatInputPrice(
                            provider = perpetualProvider,
                            price = target,
                            decimals = assetDecimals,
                        )
                        when (activeField.type) {
                            TpslType.TakeProfit -> takeProfitText = formatted
                            TpslType.StopLoss -> stopLossText = formatted
                        }
                    },
                    onDone = { focusManager.clearFocus() },
                )
            } else {
                MainActionButton(
                    title = stringResource(R.string.common_done),
                    enabled = confirmEnabled,
                    onClick = {
                        provider.setTakeProfit(takeProfitText.takeIf { it.isNotEmpty() && takeProfitField.error == null })
                        provider.setStopLoss(stopLossText.takeIf { it.isNotEmpty() && stopLossField.error == null })
                        onDismiss()
                    },
                )
            }
            Spacer16()
        }
    }

    LaunchedEffect(storedTakeProfit, storedStopLoss) {
        takeProfitText = storedTakeProfit.orEmpty()
        stopLossText = storedStopLoss.orEmpty()
    }
}

private fun buildField(
    type: TpslType,
    text: String,
    direction: PerpetualDirection,
    marketPrice: Double,
    estimator: AutocloseEstimator,
): AutocloseUIModel.Field {
    val price = text.toDoubleOrNull()
    val validator = AutocloseValidator(type = type, direction = direction, marketPrice = marketPrice)
    val field = AutocloseField(
        type = type,
        price = price,
        originalPrice = null,
        formattedPrice = null,
        error = validator.error(price),
        orderId = null,
    )
    return AutocloseUIModelFactory.createField(field = field, estimator = estimator)
}
