package com.gemwallet.android.features.perpetual.views.autoclose

import androidx.compose.foundation.layout.ExperimentalLayoutApi
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.WindowInsets
import androidx.compose.foundation.layout.isImeVisible
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.res.stringResource
import com.gemwallet.android.features.perpetual.views.components.PerpetualPositionItem
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.PercentSuggestionsBar
import com.gemwallet.android.ui.components.buttons.MainActionButton
import com.gemwallet.android.ui.components.list_item.property.PropertyItem
import com.gemwallet.android.ui.components.perpetual.AutocloseInputSection
import com.gemwallet.android.ui.components.screen.MainActionWidth
import com.gemwallet.android.ui.components.screen.Scene
import com.gemwallet.android.ui.models.ListPosition
import com.gemwallet.android.ui.models.perpetual.autoclose.AutocloseUIModel
import com.gemwallet.android.ui.theme.Spacer16
import com.gemwallet.android.ui.theme.paddingDefault
import com.gemwallet.android.ui.theme.paddingSmall
import com.gemwallet.android.ui.theme.sceneContentPadding
import com.wallet.core.primitives.TpslType

@OptIn(ExperimentalLayoutApi::class)
@Composable
internal fun AutocloseScene(
    model: AutocloseUIModel,
    takeProfitText: String,
    stopLossText: String,
    onAction: (AutocloseAction) -> Unit,
) {
    var focusedField: TpslType? by remember { mutableStateOf(null) }

    val activeField = focusedField?.let { type ->
        when (type) {
            TpslType.TakeProfit -> model.takeProfit
            TpslType.StopLoss -> model.stopLoss
        }
    }
    val activeText = when (focusedField) {
        TpslType.TakeProfit -> takeProfitText
        TpslType.StopLoss -> stopLossText
        null -> ""
    }
    val isPercentBarVisible = WindowInsets.isImeVisible && activeField != null && activeText.isEmpty()

    Scene(
        title = stringResource(R.string.perpetual_auto_close),
        onClose = { onAction(AutocloseAction.Close) },
        closeIcon = true,
        mainActionWidth = if (isPercentBarVisible) MainActionWidth.FillWidth else MainActionWidth.Constrained,
        mainActionPadding = PaddingValues(
            horizontal = sceneContentPadding(),
            vertical = if (isPercentBarVisible) paddingSmall else paddingDefault,
        ),
        mainAction = {
            if (isPercentBarVisible) {
                PercentSuggestionsBar(
                    suggestions = activeField.percentSuggestions,
                    onPercentSelected = { percent -> onAction(AutocloseAction.SelectPercent(activeField.type, percent)) },
                )
            } else {
                MainActionButton(
                    title = stringResource(R.string.transfer_confirm),
                    state = model.buttonState,
                    onClick = { onAction(AutocloseAction.Confirm) },
                )
            }
        },
    ) {
        LazyColumn {
            item {
                PerpetualPositionItem(
                    data = model.position,
                    listPosition = ListPosition.Single,
                )
                Spacer16()
            }
            item {
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
            }
            item {
                AutocloseInputSection(
                    field = model.takeProfit,
                    text = takeProfitText,
                    onTextChanged = { onAction(AutocloseAction.TakeProfitChanged(it)) },
                    onFocusChanged = { focused ->
                        if (focused) focusedField = TpslType.TakeProfit
                        else if (focusedField == TpslType.TakeProfit) focusedField = null
                    },
                )
                Spacer16()
            }
            item {
                AutocloseInputSection(
                    field = model.stopLoss,
                    text = stopLossText,
                    onTextChanged = { onAction(AutocloseAction.StopLossChanged(it)) },
                    onFocusChanged = { focused ->
                        if (focused) focusedField = TpslType.StopLoss
                        else if (focusedField == TpslType.StopLoss) focusedField = null
                    },
                )
            }
        }
    }
}
