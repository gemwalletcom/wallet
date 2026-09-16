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
import com.gemwallet.android.features.earn.delegation.presents.localization.stringRes
import uniffi.gemstone.GemDelegationAction
import uniffi.gemstone.GemDelegationRow
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
    val delegationInfo by viewModel.delegationInfo.collectAsStateWithLifecycle()
    val properties by viewModel.properties.collectAsStateWithLifecycle()
    val actions by viewModel.actions.collectAsStateWithLifecycle()
    val canClaimRewards by viewModel.canClaimRewards.collectAsStateWithLifecycle()
    val context = LocalContext.current
    val uriHandler = LocalUriHandler.current

    if (delegationInfo == null) {
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
                        iconPlaceholder = info.iconPlaceholder,
                    )
                }
            }
            properties?.let { properties ->
                itemsPositioned(properties.rows) { position, row ->
                    when (row) {
                        GemDelegationRow.APR -> StakeApr(properties.validator, position)
                        GemDelegationRow.PROVIDER -> PropertyItem(
                            modifier = properties.validatorUrl?.let { url -> Modifier.clickable { uriHandler.open(context, url) } } ?: Modifier,
                            title = { PropertyTitleText(stringResource(R.string.stake_validator)) },
                            data = {
                                PropertyDataText(
                                    text = properties.validatorName,
                                    badge = properties.validatorUrl?.let { { DataBadgeChevron() } },
                                )
                            },
                            listPosition = position,
                        )
                        GemDelegationRow.COMPLETION_DATE -> properties.status.completion?.let {
                            DelegationState(it, properties.availableIn, position)
                        }
                        GemDelegationRow.STATUS -> TransactionStatus(properties.status, position)
                        GemDelegationRow.REWARDS -> PropertyAssetBalanceItem(
                            model = properties.rewards,
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
                PropertyItem(
                    action = item.stringRes(),
                    onClick = {
                        when (item) {
                            GemDelegationAction.REDELEGATE -> viewModel.onRedelegate(onAmount)
                            GemDelegationAction.STAKE -> viewModel.onStake(onAmount)
                            GemDelegationAction.UNSTAKE -> viewModel.onUnstake(onAmount, onConfirm)
                            GemDelegationAction.WITHDRAW -> viewModel.onWithdraw(onAmount, onConfirm)
                            GemDelegationAction.DEPOSIT -> viewModel.onDeposit(onAmount)
                        }
                    },
                    listPosition = position,
                )
            }
        }
    }
}
