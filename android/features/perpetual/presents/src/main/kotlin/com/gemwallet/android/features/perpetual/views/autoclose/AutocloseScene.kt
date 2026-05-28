package com.gemwallet.android.features.perpetual.views.autoclose

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
import androidx.compose.ui.platform.LocalFocusManager
import androidx.compose.ui.res.stringResource
import com.gemwallet.android.features.perpetual.views.components.PerpetualPositionItem
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.buttons.MainActionButton
import com.gemwallet.android.ui.components.dialog.DialogBar
import com.gemwallet.android.ui.components.list_item.property.PropertyItem
import com.gemwallet.android.ui.components.perpetual.AutocloseInputSection
import com.gemwallet.android.ui.components.perpetual.AutocloseSuggestionsBar
import com.gemwallet.android.ui.models.ListPosition
import com.gemwallet.android.ui.models.perpetual.autoclose.AutocloseUIModel
import com.gemwallet.android.ui.theme.Spacer16
import com.gemwallet.android.ui.theme.paddingDefault
import com.wallet.core.primitives.TpslType

@Composable
internal fun AutocloseScene(
    model: AutocloseUIModel,
    takeProfitText: String,
    stopLossText: String,
    onTakeProfitChanged: (String) -> Unit,
    onStopLossChanged: (String) -> Unit,
    onPercentSelected: (TpslType, Int) -> Unit,
    onConfirm: () -> Unit,
    onClose: () -> Unit,
) {
    var focusedField: TpslType? by remember { mutableStateOf(null) }
    val focusManager = LocalFocusManager.current

    val activeField = focusedField?.let { type ->
        when (type) {
            TpslType.TakeProfit -> model.takeProfit
            TpslType.StopLoss -> model.stopLoss
        }
    }

    Column(
        modifier = Modifier
            .fillMaxWidth()
            .fillMaxHeight()
            .imePadding(),
    ) {
        DialogBar(
            onDismissRequest = onClose,
            title = stringResource(R.string.perpetual_auto_close),
        )
        Column(
            modifier = Modifier
                .fillMaxWidth()
                .weight(1f)
                .padding(horizontal = paddingDefault),
        ) {
            PerpetualPositionItem(
                data = model.position,
                listPosition = ListPosition.Single,
            )
            Spacer16()
            PropertyItem(
                title = stringResource(R.string.perpetual_entry_price),
                data = model.entryPriceText,
                listPosition = ListPosition.First,
            )
            PropertyItem(
                title = stringResource(R.string.perpetual_market_price),
                data = model.marketPriceText,
                listPosition = ListPosition.Last,
            )
            Spacer16()
            AutocloseInputSection(
                field = model.takeProfit,
                text = takeProfitText,
                onTextChanged = onTakeProfitChanged,
                onFocusChanged = { focused ->
                    if (focused) focusedField = TpslType.TakeProfit
                    else if (focusedField == TpslType.TakeProfit) focusedField = null
                },
            )
            Spacer16()
            AutocloseInputSection(
                field = model.stopLoss,
                text = stopLossText,
                onTextChanged = onStopLossChanged,
                onFocusChanged = { focused ->
                    if (focused) focusedField = TpslType.StopLoss
                    else if (focusedField == TpslType.StopLoss) focusedField = null
                },
            )
            Spacer(Modifier.weight(1f))
            if (activeField != null) {
                AutocloseSuggestionsBar(
                    suggestions = activeField.percentSuggestions,
                    onPercentSelected = { percent -> onPercentSelected(activeField.type, percent) },
                    onDone = { focusManager.clearFocus() },
                )
                Spacer16()
            }
            MainActionButton(
                title = stringResource(R.string.transfer_confirm),
                enabled = model.confirmEnabled,
                onClick = onConfirm,
            )
            Spacer16()
        }
    }
}
