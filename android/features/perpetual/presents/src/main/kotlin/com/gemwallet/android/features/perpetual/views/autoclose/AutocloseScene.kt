package com.gemwallet.android.features.perpetual.views.autoclose

import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.imePadding
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.CenterAlignedTopAppBar
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.style.TextOverflow
import com.gemwallet.android.features.perpetual.views.components.PerpetualPositionItem
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.buttons.MainActionButton
import com.gemwallet.android.ui.components.list_item.property.PropertyItem
import com.gemwallet.android.ui.components.perpetual.AutocloseInputSection
import com.gemwallet.android.ui.components.perpetual.AutocloseSuggestionsBar
import com.gemwallet.android.ui.icons.AppIcons
import com.gemwallet.android.ui.models.ListPosition
import com.gemwallet.android.ui.models.perpetual.autoclose.AutocloseUIModel
import com.gemwallet.android.ui.theme.Spacer16
import com.gemwallet.android.ui.theme.paddingDefault
import com.wallet.core.primitives.TpslType

@OptIn(ExperimentalMaterial3Api::class)
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

    Column(
        modifier = Modifier
            .fillMaxWidth()
            .fillMaxHeight()
            .imePadding(),
    ) {
        CenterAlignedTopAppBar(
            title = {
                Text(
                    text = stringResource(R.string.perpetual_auto_close),
                    maxLines = 1,
                    overflow = TextOverflow.Ellipsis,
                )
            },
            navigationIcon = {
                IconButton(onClick = { onAction(AutocloseAction.Close) }) {
                    Icon(imageVector = AppIcons.Close, contentDescription = null)
                }
            },
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
                onTextChanged = { onAction(AutocloseAction.TakeProfitChanged(it)) },
                onFocusChanged = { focused ->
                    if (focused) focusedField = TpslType.TakeProfit
                    else if (focusedField == TpslType.TakeProfit) focusedField = null
                },
            )
            Spacer16()
            AutocloseInputSection(
                field = model.stopLoss,
                text = stopLossText,
                onTextChanged = { onAction(AutocloseAction.StopLossChanged(it)) },
                onFocusChanged = { focused ->
                    if (focused) focusedField = TpslType.StopLoss
                    else if (focusedField == TpslType.StopLoss) focusedField = null
                },
            )
            Spacer(Modifier.weight(1f))
            if (activeField != null && activeText.isEmpty()) {
                AutocloseSuggestionsBar(
                    suggestions = activeField.percentSuggestions,
                    onPercentSelected = { percent -> onAction(AutocloseAction.SelectPercent(activeField.type, percent)) },
                )
                Spacer16()
            }
            MainActionButton(
                title = stringResource(R.string.transfer_confirm),
                enabled = model.confirmEnabled,
                onClick = { onAction(AutocloseAction.Confirm) },
            )
            Spacer16()
        }
    }
}
