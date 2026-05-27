package com.gemwallet.android.features.transfer_amount.presents

import androidx.activity.compose.BackHandler
import androidx.compose.animation.AnimatedContent
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.hilt.lifecycle.viewmodel.compose.hiltViewModel
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.gemwallet.android.features.transfer_amount.viewmodels.AmountViewModel
import com.gemwallet.android.features.transfer_amount.viewmodels.providers.AmountStakeProvider
import com.gemwallet.android.model.ConfirmParams
import com.gemwallet.android.ui.components.animation.navigationSlideTransition
import com.gemwallet.android.ui.components.screen.LoadingScene

@Composable
fun AmountScreen(
    onCancel: () -> Unit,
    onConfirm: (ConfirmParams) -> Unit,
    viewModel: AmountViewModel = hiltViewModel(),
) {
    val provider = viewModel.provider
    val title = provider.title.asString()
    val assetInfo = provider.assetInfo.collectAsStateWithLifecycle().value ?: run {
        LoadingScene(title, onCancel)
        return
    }

    var isSelectValidator by remember { mutableStateOf(false) }
    val canPickValidator = provider is AmountStakeProvider &&
        provider.canSelectValidator.collectAsStateWithLifecycle().value
    BackHandler(isSelectValidator && canPickValidator) { isSelectValidator = false }

    val amountInputType by viewModel.amountInputType.collectAsStateWithLifecycle()
    val error by viewModel.amountError.collectAsStateWithLifecycle()
    val equivalent by viewModel.amountEquivalent.collectAsStateWithLifecycle()
    val available by viewModel.availableBalanceFormatted.collectAsStateWithLifecycle()
    val reserve by viewModel.reserveForFeeFormatted.collectAsStateWithLifecycle()
    val currency by viewModel.currency.collectAsStateWithLifecycle()

    AnimatedContent(
        isSelectValidator && canPickValidator,
        transitionSpec = { navigationSlideTransition(forward = targetState) },
        label = "amount-validator-pick",
    ) { showingPicker ->
        if (showingPicker && provider is AmountStakeProvider) {
            val validator by provider.validatorState.collectAsStateWithLifecycle()
            val source by provider.validatorSource.collectAsStateWithLifecycle()
            source?.let { resolved ->
                ValidatorsScreen(
                    source = resolved,
                    selectedValidatorId = validator?.id.orEmpty(),
                    onCancel = { isSelectValidator = false },
                    onSelect = {
                        provider.selectValidator(it)
                        isSelectValidator = false
                    },
                )
            }
        } else {
            AmountScene(
                title = title,
                amount = viewModel.amount,
                amountInputType = amountInputType,
                asset = assetInfo.asset,
                currency = currency,
                canSwitchInputType = provider.canSwitchInputType,
                readOnly = !provider.canChangeValue,
                showsAssetBalance = provider.showsAssetBalance,
                error = error,
                equivalent = equivalent,
                availableBalance = available,
                reserveForFee = reserve,
                onNext = { viewModel.onNext(onConfirm) },
                onInputAmount = viewModel::updateAmount,
                onInputTypeClick = viewModel::switchInputType,
                onMaxAmount = viewModel::onMaxAmount,
                onCancel = onCancel,
                additionParams = {
                    ProviderExtras(
                        provider = provider,
                        amount = viewModel.amount,
                        onPickValidator = { isSelectValidator = true },
                    )
                },
            )
        }
    }
}
