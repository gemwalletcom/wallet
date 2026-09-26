package com.gemwallet.android.features.transfer.presents.amount

import androidx.compose.foundation.layout.Column
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.res.stringResource
import com.gemwallet.android.ext.toPrimitives
import com.gemwallet.android.features.transfer.presents.amount.dialogs.SelectLeverageDialog
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.TabsBar
import com.gemwallet.android.ui.components.clickable
import com.gemwallet.android.ui.components.list_item.ListItem
import com.gemwallet.android.ui.components.list_item.ListItemModel
import com.gemwallet.android.ui.components.list_item.SubheaderItem
import com.gemwallet.android.ui.components.list_item.listItemModel
import com.gemwallet.android.ui.components.list_item.property.DataBadgeChevron
import com.gemwallet.android.ui.components.list_item.property.PropertyValidatorItem
import com.gemwallet.android.ui.localization.string
import com.gemwallet.android.ui.localization.stringRes
import com.gemwallet.android.ui.models.ListPosition
import com.gemwallet.android.ui.style.textStyle
import com.wallet.core.primitives.Resource
import uniffi.gemstone.GemAmountExtras
import uniffi.gemstone.GemAmountLeverage
import uniffi.gemstone.GemListRow

@Composable
fun ProviderExtras(extras: GemAmountExtras, onPickValidator: () -> Unit, onSelectResource: (Resource) -> Unit, onSelectLeverage: (UByte) -> Unit, onOpenAutoclose: () -> Unit) {
    Column {
        when (extras) {
            GemAmountExtras.None -> Unit

            is GemAmountExtras.Resources -> TabsBar(
                tabs = extras.options.map { it.toPrimitives() },
                selected = extras.selected.toPrimitives(),
                onSelect = onSelectResource,
            ) { item ->
                Text(stringResource(item.stringRes()))
            }

            is GemAmountExtras.Validator -> {
                SubheaderItem(R.string.stake_validator)
                PropertyValidatorItem(
                    validator = extras.row,
                    listPosition = ListPosition.Single,
                    onClick = if (extras.canSelect) onPickValidator else null,
                )
            }

            is GemAmountExtras.Provider -> {
                SubheaderItem(R.string.common_provider)
                PropertyValidatorItem(validator = extras.row, listPosition = ListPosition.Single)
            }

            is GemAmountExtras.Perpetual -> PerpetualSections(extras.leverage, extras.autoclose, onSelectLeverage, onOpenAutoclose)
        }
    }
}

@Composable
private fun PerpetualSections(leverage: GemAmountLeverage?, autoclose: GemListRow?, onSelectLeverage: (UByte) -> Unit, onOpenAutoclose: () -> Unit) {
    val context = LocalContext.current
    var showLeverageSelect by remember { mutableStateOf(false) }
    leverage?.let {
        ListItem(
            model = ListItemModel(
                title = stringResource(R.string.perpetual_leverage),
                subtitle = it.selection.selected.label.string(context),
                subtitleStyle = it.direction.toPrimitives().textStyle(),
            ),
            listPosition = ListPosition.Single,
            modifier = Modifier.clickable { showLeverageSelect = true },
            accessory = { DataBadgeChevron() },
        )
        SelectLeverageDialog(
            isVisible = showLeverageSelect,
            leverages = it.selection.options,
            selected = it.selection.selected,
            onDismiss = { showLeverageSelect = false },
            onSelect = onSelectLeverage,
        )
    }
    autoclose?.listItemModel(context)?.let { model ->
        ListItem(
            model = model,
            listPosition = ListPosition.Single,
            modifier = Modifier.clickable(onClick = onOpenAutoclose),
            accessory = { DataBadgeChevron() },
        )
    }
}
