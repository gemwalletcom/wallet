package com.gemwallet.android.features.transfer_amount.presents

import androidx.compose.foundation.layout.Column
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.gemwallet.android.features.transfer_amount.presents.dialogs.AmountAutocloseSheet
import com.gemwallet.android.features.transfer_amount.presents.dialogs.SelectLeverageDialog
import com.gemwallet.android.features.transfer_amount.viewmodels.providers.AmountDataProvider
import com.gemwallet.android.features.transfer_amount.viewmodels.providers.AmountEarnProvider
import com.gemwallet.android.features.transfer_amount.viewmodels.providers.AmountPerpetualProvider
import com.gemwallet.android.features.transfer_amount.viewmodels.providers.AmountStakeProvider
import com.gemwallet.android.features.transfer_amount.viewmodels.providers.AmountTransferProvider
import com.gemwallet.android.model.AmountParams
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.TabsBar
import com.gemwallet.android.ui.components.clickable
import com.gemwallet.android.ui.components.list_item.ListItem
import com.gemwallet.android.ui.components.list_item.SubheaderItem
import com.gemwallet.android.ui.components.list_item.property.DataBadgeChevron
import com.gemwallet.android.ui.components.list_item.property.PropertyValidatorItem
import com.gemwallet.android.ui.components.list_item.uiModel
import com.gemwallet.android.ui.localization.stringRes
import com.gemwallet.android.ui.models.ListPosition
import com.wallet.core.primitives.Resource
import uniffi.gemstone.GemAmountType

@Composable
fun ProviderExtras(provider: AmountDataProvider, amount: String, onPickValidator: () -> Unit) {
    Column {
        when (provider) {
            is AmountStakeProvider -> StakeProviderSection(provider, onPickValidator)

            is AmountPerpetualProvider -> {
                PerpetualLeverageSection(provider)
                if (provider.showsAutoclose) {
                    PerpetualAutocloseSection(provider, amount)
                }
            }

            is AmountEarnProvider -> EarnProviderSection(provider)

            is AmountTransferProvider -> Unit
        }
    }
}

@Composable
private fun StakeProviderSection(provider: AmountStakeProvider, onPickValidator: () -> Unit) {
    when (provider.params) {
        is AmountParams.Stake.Freeze, is AmountParams.Stake.Unfreeze -> StakeResourceSection(provider)

        is AmountParams.Stake.Delegate,
        is AmountParams.Stake.Undelegate,
        is AmountParams.Stake.Redelegate,
        is AmountParams.Stake.Withdraw,
        is AmountParams.Stake.Rewards,
        -> StakeValidatorSection(provider, onPickValidator)
    }
}

@Composable
private fun EarnProviderSection(provider: AmountEarnProvider) {
    val amountType by provider.amountType.collectAsStateWithLifecycle()
    (amountType as? GemAmountType.Earn)?.let { earn ->
        SubheaderItem(R.string.common_provider)
        PropertyValidatorItem(
            validator = earn.provider.uiModel(),
            listPosition = ListPosition.Single,
        )
    }
}

@Composable
private fun StakeValidatorSection(provider: AmountStakeProvider, onPickValidator: () -> Unit) {
    val validator by provider.validatorState.collectAsStateWithLifecycle()
    val canSelectValidator by provider.canSelectValidator.collectAsStateWithLifecycle()
    validator?.let { current ->
        SubheaderItem(R.string.stake_validator)
        PropertyValidatorItem(
            validator = current.uiModel(),
            listPosition = ListPosition.Single,
            onClick = if (canSelectValidator) onPickValidator else null,
        )
    }
}

@Composable
private fun StakeResourceSection(provider: AmountStakeProvider) {
    val resource by provider.resource.collectAsStateWithLifecycle()
    TabsBar(
        tabs = listOf(Resource.Bandwidth, Resource.Energy),
        selected = resource,
        onSelect = provider::setResource,
    ) { item ->
        Text(stringResource(item.stringRes()))
    }
}

@Composable
private fun PerpetualLeverageSection(provider: AmountPerpetualProvider) {
    val state = provider.leverageState.collectAsStateWithLifecycle().value ?: return
    val model = provider.leverageListItem.collectAsStateWithLifecycle().value ?: return
    var showLeverageSelect by remember { mutableStateOf(false) }
    ListItem(
        model = model,
        listPosition = ListPosition.Single,
        modifier = Modifier.clickable { showLeverageSelect = true },
        accessory = { DataBadgeChevron() },
    )
    SelectLeverageDialog(
        isVisible = showLeverageSelect,
        leverages = state.options,
        selected = state.current,
        onDismiss = { showLeverageSelect = false },
        onSelect = provider::setLeverage,
    )
}

@Composable
private fun PerpetualAutocloseSection(provider: AmountPerpetualProvider, amount: String) {
    val model by provider.autocloseListItem.collectAsStateWithLifecycle()
    var sheetVisible by remember { mutableStateOf(false) }

    model?.let {
        ListItem(
            model = it,
            listPosition = ListPosition.Single,
            modifier = Modifier.clickable { sheetVisible = true },
            accessory = { DataBadgeChevron() },
        )
    }

    AmountAutocloseSheet(
        isVisible = sheetVisible,
        provider = provider,
        amount = amount,
        onDismiss = { sheetVisible = false },
    )
}
