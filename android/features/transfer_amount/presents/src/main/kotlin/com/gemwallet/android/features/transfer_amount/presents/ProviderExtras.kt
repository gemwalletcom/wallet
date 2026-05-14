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
import com.gemwallet.android.features.transfer_amount.presents.dialogs.SelectLeverageDialog
import com.gemwallet.android.features.transfer_amount.viewmodels.providers.AmountDataProvider
import com.gemwallet.android.features.transfer_amount.viewmodels.providers.AmountPerpetualProvider
import com.gemwallet.android.features.transfer_amount.viewmodels.providers.AmountStakeProvider
import com.gemwallet.android.features.transfer_amount.viewmodels.providers.AmountTransferProvider
import com.gemwallet.android.model.AmountParams
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.TabsBar
import com.gemwallet.android.ui.components.clickable
import com.gemwallet.android.ui.components.list_item.SubheaderItem
import com.gemwallet.android.ui.components.list_item.property.DataBadgeChevron
import com.gemwallet.android.ui.components.list_item.property.PropertyDataText
import com.gemwallet.android.ui.components.list_item.property.PropertyItem
import com.gemwallet.android.ui.components.list_item.property.PropertyTitleText
import com.gemwallet.android.ui.components.list_item.property.PropertyValidatorItem
import com.gemwallet.android.ui.models.ListPosition
import com.wallet.core.primitives.Resource

@Composable
fun ProviderExtras(
    provider: AmountDataProvider,
    onPickValidator: () -> Unit,
) {
    Column {
        when (provider) {
            is AmountStakeProvider -> StakeProviderSection(provider, onPickValidator)
            is AmountPerpetualProvider -> PerpetualLeverageSection(provider)
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
        is AmountParams.Stake.Rewards -> StakeValidatorSection(provider, onPickValidator)
    }
}

@Composable
private fun StakeValidatorSection(provider: AmountStakeProvider, onPickValidator: () -> Unit) {
    val validator by provider.validatorState.collectAsStateWithLifecycle()
    val canSelectValidator by provider.canSelectValidator.collectAsStateWithLifecycle()
    validator?.let { current ->
        SubheaderItem(R.string.stake_validator)
        PropertyValidatorItem(
            validator = current,
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
        Text(stringResource(when (item) {
            Resource.Bandwidth -> R.string.stake_resource_bandwidth
            Resource.Energy -> R.string.stake_resource_energy
        }))
    }
}

@Composable
private fun PerpetualLeverageSection(provider: AmountPerpetualProvider) {
    var showLeverageSelect by remember { mutableStateOf(false) }
    val leverage by provider.leverage.collectAsStateWithLifecycle()
    val available by provider.availableLeverages.collectAsStateWithLifecycle()
    PropertyItem(
        modifier = Modifier.clickable { showLeverageSelect = true },
        title = { PropertyTitleText(R.string.perpetual_leverage) },
        data = { PropertyDataText("${leverage}x", badge = { DataBadgeChevron() }) },
        listPosition = ListPosition.Single,
    )
    SelectLeverageDialog(
        isVisible = showLeverageSelect,
        leverages = available,
        onDismiss = { showLeverageSelect = false },
        onSelect = provider::setLeverage,
    )
}
