package com.gemwallet.android.features.earn.delegation.presents

import androidx.compose.foundation.clickable
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.res.stringResource
import androidx.hilt.lifecycle.viewmodel.compose.hiltViewModel
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.gemwallet.android.features.earn.delegation.models.DelegationRowUIModel
import com.gemwallet.android.features.earn.delegation.viewmodels.DelegationViewModel
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

@Composable
fun DelegationScene(onAmount: AmountTransactionAction, onConfirm: ConfirmTransactionAction, onCancel: () -> Unit, viewModel: DelegationViewModel = hiltViewModel()) {
    val delegationInfo by viewModel.delegationInfo.collectAsStateWithLifecycle()
    val properties by viewModel.properties.collectAsStateWithLifecycle()
    val actions by viewModel.actions.collectAsStateWithLifecycle()
    val canClaimRewards by viewModel.canClaimRewards.collectAsStateWithLifecycle()

    if (delegationInfo == null) {
        LoadingScene(title = stringResource(id = R.string.transfer_stake_title), onCancel = onCancel)
        return
    }
    Scene(
        title = properties?.details?.title?.string(LocalContext.current) ?: stringResource(R.string.transfer_stake_title),
        onClose = onCancel,
    ) {
        LazyColumn {
            delegationInfo?.let { info ->
                item {
                    AmountListHead(
                        amount = info.cryptoFormatted,
                        equivalent = info.fiatFormatted,
                        icon = info.iconUrl,
                        iconPlaceholder = info.iconPlaceholder,
                    )
                }
            }
            properties?.let { properties ->
                itemsPositioned(properties.rows) { position, row ->
                    when (row) {
                        is DelegationRowUIModel.Row -> GemListRowView(row = row.row, listPosition = position)

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
            }

            if (actions.isNotEmpty()) {
                item { SubheaderItem(R.string.common_manage) }
            }
            itemsPositioned(actions) { position, item ->
                ListItem(
                    model = item.model,
                    listPosition = position,
                    modifier = Modifier.clickable { viewModel.onAction(item.action, onAmount, onConfirm) },
                    accessory = { DataBadgeChevron() },
                )
            }
        }
    }
}
