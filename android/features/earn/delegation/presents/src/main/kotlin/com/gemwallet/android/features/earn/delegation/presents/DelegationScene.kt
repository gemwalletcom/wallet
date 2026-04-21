package com.gemwallet.android.features.earn.delegation.presents

import androidx.compose.foundation.clickable
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.platform.LocalUriHandler
import androidx.compose.ui.res.stringResource
import androidx.hilt.lifecycle.viewmodel.compose.hiltViewModel
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.list_head.AmountListHead
import com.gemwallet.android.ui.components.list_item.SubheaderItem
import com.gemwallet.android.ui.components.list_item.property.DataBadgeChevron
import com.gemwallet.android.ui.components.list_item.property.PropertyAssetBalanceItem
import com.gemwallet.android.ui.components.list_item.property.PropertyDataText
import com.gemwallet.android.ui.components.list_item.property.PropertyItem
import com.gemwallet.android.ui.components.list_item.property.PropertyTitleText
import com.gemwallet.android.ui.components.list_item.property.itemsPositioned
import com.gemwallet.android.ui.open
import com.gemwallet.android.ui.components.screen.LoadingScene
import com.gemwallet.android.ui.components.screen.Scene
import com.gemwallet.android.ui.models.actions.AmountTransactionAction
import com.gemwallet.android.ui.models.actions.ConfirmTransactionAction
import com.gemwallet.android.features.earn.delegation.models.DelegationActions
import com.gemwallet.android.features.earn.delegation.models.DelegationProperty
import com.gemwallet.android.features.earn.delegation.presents.components.DelegationState
import com.gemwallet.android.features.earn.delegation.presents.components.StakeApr
import com.gemwallet.android.features.earn.delegation.presents.components.TransactionStatus
import com.gemwallet.android.features.earn.delegation.viewmodels.DelegationViewModel

@Composable
fun DelegationScene(
    onAmount: AmountTransactionAction,
    onConfirm: ConfirmTransactionAction,
    onCancel: () -> Unit,
    viewModel: DelegationViewModel = hiltViewModel(),
) {
    val uiState by viewModel.uiState.collectAsStateWithLifecycle()
    val delegationInfo by viewModel.delegationInfo.collectAsStateWithLifecycle()
    val properties by viewModel.properties.collectAsStateWithLifecycle()
    val balances by viewModel.balances.collectAsStateWithLifecycle()
    val actions by viewModel.actions.collectAsStateWithLifecycle()
    val canClaimRewards by viewModel.canClaimRewards.collectAsStateWithLifecycle()
    val context = LocalContext.current
    val uriHandler = LocalUriHandler.current

    if (uiState == null) {
        LoadingScene(title = stringResource(id = R.string.transfer_stake_title), onCancel = onCancel)
        return
    }
    Scene(
        title = stringResource(R.string.transfer_stake_title),
        onClose = onCancel,
    ) {
        LazyColumn {
            delegationInfo?.let { info ->
                item {
                    AmountListHead(
                        amount = info.cryptoFormatted,
                        equivalent = info.fiatFormatted,
                        icon = info.iconUrl,
                    )
                }
            }
            itemsPositioned(properties) { position, item ->
                when (item) {
                    is DelegationProperty.Apr -> StakeApr(item.data, position)
                    is DelegationProperty.Name -> PropertyItem(
                        modifier = item.url?.let { url -> Modifier.clickable { uriHandler.open(context, url) } } ?: Modifier,
                        title = { PropertyTitleText(stringResource(R.string.stake_validator)) },
                        data = {
                            PropertyDataText(
                                text = item.data,
                                badge = item.url?.let { { DataBadgeChevron() } },
                            )
                        },
                        listPosition = position,
                    )
                    is DelegationProperty.State -> DelegationState(
                        item.state,
                        item.availableIn,
                        position
                    )
                    is DelegationProperty.TransactionStatus -> TransactionStatus(
                        item.state,
                        item.isActive,
                        position
                    )
                }
            }

            itemsPositioned(balances) { position, item ->
                val modifier = if (canClaimRewards) {
                    Modifier.clickable { viewModel.onClaimRewards(onConfirm) }
                } else {
                    Modifier
                }
                PropertyAssetBalanceItem(
                    model = item,
                    title = stringResource(R.string.stake_rewards),
                    modifier = modifier,
                    showChevron = canClaimRewards,
                    listPosition = position,
                )
            }

            if (actions.isNotEmpty()) {
                item { SubheaderItem(R.string.common_manage) }
            }
            itemsPositioned(actions) { position, item ->
                when (item) {
                    DelegationActions.RedelegateAction -> PropertyItem(R.string.transfer_redelegate_title, onClick = { viewModel.onRedelegate(onAmount) }, listPosition = position)
                    DelegationActions.StakeAction -> PropertyItem(R.string.transfer_stake_title, onClick = { viewModel.onStake(onAmount) }, listPosition = position)
                    DelegationActions.UnstakeAction -> PropertyItem(R.string.transfer_unstake_title, onClick = { viewModel.onUnstake(onAmount, onConfirm) }, listPosition = position)
                    DelegationActions.WithdrawalAction -> PropertyItem(R.string.transfer_withdraw_title, onClick = { viewModel.onWithdraw(onConfirm) }, listPosition = position)
                }
            }
        }
    }
}
