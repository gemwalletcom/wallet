package com.gemwallet.android.features.transfer.presents.amount

import androidx.activity.compose.BackHandler
import androidx.compose.animation.AnimatedContent
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.hilt.lifecycle.viewmodel.compose.hiltViewModel
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.gemwallet.android.domains.confirm.ConfirmTransferInput
import com.gemwallet.android.features.stake.presents.ValidatorSelectScene
import com.gemwallet.android.features.transfer.presents.amount.dialogs.AmountAutocloseSheet
import com.gemwallet.android.features.transfer.viewmodels.amount.AmountViewModel
import com.gemwallet.android.features.transfer.viewmodels.amount.models.AmountExtrasUIModel
import com.gemwallet.android.ui.components.animation.navigationSlideTransition
import com.gemwallet.android.ui.components.screen.LoadingScene
import com.wallet.core.primitives.AssetId

@Composable
fun AmountScreen(onCancel: () -> Unit, onConfirm: (ConfirmTransferInput) -> Unit, onBuy: (AssetId) -> Unit, viewModel: AmountViewModel = hiltViewModel()) {
    val uiState by viewModel.uiState.collectAsStateWithLifecycle()
    val asset = uiState.asset ?: run {
        LoadingScene(uiState.title, onCancel)
        return
    }

    var isSelectValidator by remember { mutableStateOf(false) }
    var showsAutoclose by remember { mutableStateOf(false) }
    val canPickValidator = (uiState.extras as? AmountExtrasUIModel.Validator)?.canSelect == true
    BackHandler(isSelectValidator && canPickValidator) { isSelectValidator = false }

    val validatorPicker by viewModel.validatorPicker.collectAsStateWithLifecycle()

    AnimatedContent(
        isSelectValidator && canPickValidator,
        transitionSpec = { navigationSlideTransition(forward = targetState) },
        label = "amount-validator-pick",
    ) { showingPicker ->
        if (showingPicker && validatorPicker != null) {
            validatorPicker?.let { picker ->
                ValidatorSelectScene(
                    selection = picker.selection,
                    selectedValidatorId = picker.selectedId,
                    onCancel = { isSelectValidator = false },
                    onSelect = {
                        viewModel.selectValidator(it)
                        isSelectValidator = false
                    },
                )
            }
        } else {
            AmountScene(
                title = uiState.title,
                amount = viewModel.amount,
                amountSymbol = uiState.amountSymbol,
                asset = asset,
                currency = viewModel.currency,
                canSwitchInputType = uiState.canSwitchInputType,
                readOnly = uiState.readOnly,
                focusesInput = uiState.focusesInput,
                usesWholeAmounts = uiState.usesWholeAmounts,
                showsAssetBalance = uiState.showsAssetBalance,
                error = uiState.error,
                errorTopic = uiState.errorTopic,
                equivalent = uiState.equivalent,
                availableBalance = uiState.availableBalance,
                reserveForFee = uiState.reserveForFee,
                buttonState = uiState.buttonState,
                onAction = { action ->
                    when (action) {
                        AmountAction.Next -> viewModel.onNext(onConfirm)
                        is AmountAction.SetAmount -> viewModel.updateAmount(action.amount)
                        AmountAction.SwitchInputType -> viewModel.switchInputType()
                        AmountAction.SetMaxAmount -> viewModel.onMaxAmount()
                        AmountAction.Buy -> onBuy(asset.id)
                        AmountAction.Cancel -> onCancel()
                    }
                },
                additionParams = {
                    ProviderExtras(
                        extras = uiState.extras,
                        onPickValidator = { isSelectValidator = true },
                        onSelectResource = viewModel::selectResource,
                        onSelectLeverage = viewModel::selectLeverage,
                        onOpenAutoclose = { showsAutoclose = true },
                    )
                },
            )
        }
    }

    viewModel.perpetualProvider?.let { perpetual ->
        AmountAutocloseSheet(
            isVisible = showsAutoclose,
            provider = perpetual,
            amount = viewModel.amount,
            onDismiss = { showsAutoclose = false },
        )
    }
}
