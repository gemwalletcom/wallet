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
import com.gemwallet.android.features.transfer_amount.presents.dialogs.SelectLeverageDialog
import com.gemwallet.android.features.transfer_amount.viewmodels.models.AmountExtrasUIModel
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.TabsBar
import com.gemwallet.android.ui.components.clickable
import com.gemwallet.android.ui.components.list_item.ListItem
import com.gemwallet.android.ui.components.list_item.SubheaderItem
import com.gemwallet.android.ui.components.list_item.property.DataBadgeChevron
import com.gemwallet.android.ui.components.list_item.property.PropertyValidatorItem
import com.gemwallet.android.ui.localization.stringRes
import com.gemwallet.android.ui.models.ListPosition
import com.wallet.core.primitives.Resource

@Composable
fun ProviderExtras(extras: AmountExtrasUIModel, onPickValidator: () -> Unit, onSelectResource: (Resource) -> Unit, onSelectLeverage: (Int) -> Unit, onOpenAutoclose: () -> Unit) {
    Column {
        when (extras) {
            AmountExtrasUIModel.None -> Unit

            is AmountExtrasUIModel.Resources -> TabsBar(
                tabs = extras.options,
                selected = extras.selected,
                onSelect = onSelectResource,
            ) { item ->
                Text(stringResource(item.stringRes()))
            }

            is AmountExtrasUIModel.Validator -> {
                SubheaderItem(R.string.stake_validator)
                PropertyValidatorItem(
                    validator = extras.row,
                    listPosition = ListPosition.Single,
                    onClick = if (extras.canSelect) onPickValidator else null,
                )
            }

            is AmountExtrasUIModel.EarnProvider -> {
                SubheaderItem(R.string.common_provider)
                PropertyValidatorItem(validator = extras.row, listPosition = ListPosition.Single)
            }

            is AmountExtrasUIModel.Perpetual -> PerpetualSections(extras, onSelectLeverage, onOpenAutoclose)
        }
    }
}

@Composable
private fun PerpetualSections(extras: AmountExtrasUIModel.Perpetual, onSelectLeverage: (Int) -> Unit, onOpenAutoclose: () -> Unit) {
    var showLeverageSelect by remember { mutableStateOf(false) }
    extras.leverage?.let { model ->
        ListItem(
            model = model,
            listPosition = ListPosition.Single,
            modifier = Modifier.clickable { showLeverageSelect = true },
            accessory = { DataBadgeChevron() },
        )
        SelectLeverageDialog(
            isVisible = showLeverageSelect,
            leverages = extras.leverages,
            selected = extras.selectedLeverage,
            onDismiss = { showLeverageSelect = false },
            onSelect = onSelectLeverage,
        )
    }
    extras.autoclose?.let { model ->
        ListItem(
            model = model,
            listPosition = ListPosition.Single,
            modifier = Modifier.clickable(onClick = onOpenAutoclose),
            accessory = { DataBadgeChevron() },
        )
    }
}
