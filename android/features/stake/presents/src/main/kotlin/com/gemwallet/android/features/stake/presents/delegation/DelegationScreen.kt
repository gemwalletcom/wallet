package com.gemwallet.android.features.stake.presents.delegation

import androidx.compose.foundation.clickable
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.res.stringResource
import androidx.hilt.lifecycle.viewmodel.compose.hiltViewModel
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.gemwallet.android.features.stake.viewmodels.delegation.DelegationViewModel
import com.gemwallet.android.features.stake.viewmodels.delegation.models.DelegationRowUIModel
import com.gemwallet.android.model.text
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.list_head.AmountListHead
import com.gemwallet.android.ui.components.list_item.GemListRowView
import com.gemwallet.android.ui.components.list_item.ListItem
import com.gemwallet.android.ui.components.list_item.SubheaderItem
import com.gemwallet.android.ui.components.list_item.property.DataBadgeChevron
import com.gemwallet.android.ui.components.list_item.property.PropertyAssetBalanceItem
import com.gemwallet.android.ui.components.list_item.property.itemsPositioned
import com.gemwallet.android.ui.components.screen.LoadingScene
import com.gemwallet.android.ui.components.screen.Scene
import com.gemwallet.android.ui.localization.string
import com.gemwallet.android.ui.models.actions.AmountTransactionAction
import com.gemwallet.android.ui.models.actions.ConfirmTransactionAction
import com.wallet.core.primitives.ChainAddress

@Composable
fun DelegationScreen(onAmount: AmountTransactionAction, onConfirm: ConfirmTransactionAction, onOpenAddress: (ChainAddress) -> Unit, onCancel: () -> Unit, viewModel: DelegationViewModel = hiltViewModel()) {
    val current by viewModel.properties.collectAsStateWithLifecycle()
    val properties = current

    if (properties == null) {
        LoadingScene(title = stringResource(id = R.string.transfer_stake_title), onCancel = onCancel)
        return
    }
    val details = properties.details
    val canClaimRewards = details.claim != null
    Scene(
        title = details.title.string(LocalContext.current),
        onClose = onCancel,
    ) {
        LazyColumn {
            item {
                AmountListHead(
                    amount = details.balance.text(),
                    equivalent = details.fiat?.text().orEmpty(),
                    icon = details.header.validator.imageUrl,
                    iconPlaceholder = details.header.validator.placeholder,
                )
            }
            itemsPositioned(properties.rows) { position, row ->
                when (row) {
                    is DelegationRowUIModel.Row -> GemListRowView(
                        row = row.row,
                        listPosition = position,
                        onSelectAddress = { onOpenAddress(ChainAddress(properties.asset.id.chain, it)) },
                    )

                    DelegationRowUIModel.Rewards -> PropertyAssetBalanceItem(
                        asset = properties.asset,
                        amount = properties.details.rewards ?: return@itemsPositioned,
                        fiat = properties.details.rewardsFiat,
                        title = stringResource(R.string.stake_rewards),
                        modifier = if (canClaimRewards) Modifier.clickable { viewModel.onClaimRewards(onConfirm) } else Modifier,
                        showChevron = canClaimRewards,
                        listPosition = position,
                    )
                }
            }

            if (properties.actions.isNotEmpty()) {
                item { SubheaderItem(R.string.common_manage) }
            }
            itemsPositioned(properties.actions) { position, item ->
                ListItem(
                    model = item.model,
                    listPosition = position,
                    modifier = Modifier.clickable { viewModel.onAction(item.destination, onAmount, onConfirm) },
                    accessory = { DataBadgeChevron() },
                )
            }
        }
    }
}
