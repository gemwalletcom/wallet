package com.gemwallet.android.features.perpetual.views.autoclose

import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalFocusManager
import androidx.compose.ui.res.stringResource
import androidx.hilt.lifecycle.viewmodel.compose.hiltViewModel
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.gemwallet.android.features.perpetual.viewmodels.AutocloseViewModel
import com.gemwallet.android.features.perpetual.views.components.PerpetualPositionItem
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.buttons.MainActionButton
import com.gemwallet.android.ui.components.list_item.property.PropertyItem
import com.gemwallet.android.ui.components.perpetual.AutocloseInputSection
import com.gemwallet.android.ui.components.perpetual.AutocloseSuggestionsBar
import com.gemwallet.android.ui.components.screen.ModalBottomSheet
import com.gemwallet.android.ui.models.ListPosition
import com.gemwallet.android.ui.models.actions.ConfirmTransactionAction
import com.gemwallet.android.ui.theme.Spacer16
import com.gemwallet.android.ui.theme.paddingDefault
import com.wallet.core.primitives.TpslType

@Composable
fun AutocloseSheet(
    isVisible: Boolean,
    confirmAction: ConfirmTransactionAction,
    onDismiss: () -> Unit,
    viewModel: AutocloseViewModel = hiltViewModel(),
) {
    val uiModel by viewModel.uiModel.collectAsStateWithLifecycle()
    val takeProfitText by viewModel.takeProfitText.collectAsStateWithLifecycle()
    val stopLossText by viewModel.stopLossText.collectAsStateWithLifecycle()

    var focusedField: TpslType? by remember { mutableStateOf(null) }
    val focusManager = LocalFocusManager.current

    val focusedText = focusedField?.let { type ->
        when (type) {
            TpslType.TakeProfit -> takeProfitText
            TpslType.StopLoss -> stopLossText
        }
    }
    val activeField = focusedField?.let { type ->
        when (type) {
            TpslType.TakeProfit -> uiModel?.takeProfit
            TpslType.StopLoss -> uiModel?.stopLoss
        }
    }

    ModalBottomSheet(
        isVisible = isVisible,
        onDismissRequest = onDismiss,
        title = stringResource(R.string.perpetual_auto_close),
    ) {
        val model = uiModel ?: return@ModalBottomSheet
        Column(
            modifier = Modifier
                .fillMaxWidth()
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
                onTextChanged = viewModel::onTakeProfitChanged,
                onFocusChanged = { focused ->
                    if (focused) focusedField = TpslType.TakeProfit
                    else if (focusedField == TpslType.TakeProfit) focusedField = null
                },
            )
            Spacer16()
            AutocloseInputSection(
                field = model.stopLoss,
                text = stopLossText,
                onTextChanged = viewModel::onStopLossChanged,
                onFocusChanged = { focused ->
                    if (focused) focusedField = TpslType.StopLoss
                    else if (focusedField == TpslType.StopLoss) focusedField = null
                },
            )
            Spacer16()
            if (activeField != null && focusedText.isNullOrEmpty()) {
                AutocloseSuggestionsBar(
                    suggestions = activeField.percentSuggestions,
                    onPercentSelected = { percent ->
                        viewModel.onPercentSelected(activeField.type, percent)
                    },
                    onDone = { focusManager.clearFocus() },
                )
            } else {
                MainActionButton(
                    title = stringResource(R.string.transfer_confirm),
                    enabled = model.confirmEnabled,
                    onClick = {
                        viewModel.onConfirm(confirmAction)
                        onDismiss()
                    },
                )
            }
            Spacer16()
        }
    }
}
