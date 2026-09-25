package com.gemwallet.android.features.perpetual.views.autoclose

import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.WindowInsets
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.material3.SnackbarHostState
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.res.stringResource
import com.gemwallet.android.ext.toPrimitives
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.PercentSuggestionsBar
import com.gemwallet.android.ui.components.buttons.MainActionButton
import com.gemwallet.android.ui.components.isKeyboardVisible
import com.gemwallet.android.ui.components.list_item.AssetListItem
import com.gemwallet.android.ui.components.list_item.GemListRowView
import com.gemwallet.android.ui.components.perpetual.AutocloseInputSection
import com.gemwallet.android.ui.components.screen.MainActionWidth
import com.gemwallet.android.ui.components.screen.Scene
import com.gemwallet.android.ui.models.ListPosition
import com.gemwallet.android.ui.models.buttonState
import com.gemwallet.android.ui.theme.Spacer16
import com.gemwallet.android.ui.theme.paddingDefault
import com.gemwallet.android.ui.theme.paddingSmall
import com.gemwallet.android.ui.theme.sceneContentPadding
import com.wallet.core.primitives.TpslType
import uniffi.gemstone.GemAssetItemRow
import uniffi.gemstone.GemAutocloseViewState

@Composable
internal fun AutocloseScene(model: GemAutocloseViewState, positionRow: GemAssetItemRow?, takeProfitText: String, stopLossText: String, snackbar: SnackbarHostState, onAction: (AutocloseAction) -> Unit) {
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
    val isPercentBarVisible = WindowInsets.isKeyboardVisible && activeField != null && activeText.isEmpty()

    Scene(
        title = stringResource(R.string.perpetual_auto_close),
        onClose = { onAction(AutocloseAction.Close) },
        snackbar = snackbar,
        closeIcon = true,
        mainActionWidth = if (isPercentBarVisible) MainActionWidth.FillWidth else MainActionWidth.Constrained,
        mainActionPadding = PaddingValues(
            horizontal = sceneContentPadding(),
            vertical = if (isPercentBarVisible) paddingSmall else paddingDefault,
        ),
        mainAction = {
            if (isPercentBarVisible) {
                PercentSuggestionsBar(
                    suggestions = activeField.suggestions,
                    onPercentSelected = { percent -> onAction(AutocloseAction.SelectPercent(activeField.tpslType.toPrimitives(), percent)) },
                )
            } else {
                MainActionButton(
                    title = stringResource(R.string.transfer_confirm),
                    state = buttonState(enabled = model.confirmEnabled),
                    onClick = { onAction(AutocloseAction.Confirm) },
                )
            }
        },
    ) {
        LazyColumn {
            item {
                positionRow?.let { AssetListItem(row = it, listPosition = ListPosition.Single) }
                Spacer16()
            }
            item {
                model.priceRows.forEachIndexed { index, row ->
                    GemListRowView(row = row, listPosition = ListPosition.getPosition(index, model.priceRows.size))
                }
                Spacer16()
            }
            item {
                AutocloseInputSection(
                    field = model.takeProfit,
                    text = takeProfitText,
                    onTextChanged = { onAction(AutocloseAction.TakeProfitChanged(it)) },
                    onFocusChanged = { focused ->
                        if (focused) {
                            focusedField = TpslType.TakeProfit
                        } else if (focusedField == TpslType.TakeProfit) {
                            focusedField = null
                        }
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
                        if (focused) {
                            focusedField = TpslType.StopLoss
                        } else if (focusedField == TpslType.StopLoss) {
                            focusedField = null
                        }
                    },
                )
            }
        }
    }
}
