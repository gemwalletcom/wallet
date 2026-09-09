@file:OptIn(ExperimentalMaterial3Api::class)

package com.gemwallet.android.features.stake.presents

import com.gemwallet.android.ui.components.image.iconModel
import android.icu.util.Measure
import android.icu.util.MeasureUnit
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.LazyListScope
import androidx.compose.foundation.lazy.itemsIndexed
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.platform.LocalUriHandler
import androidx.compose.ui.res.stringResource
import com.gemwallet.android.domains.asset.chain
import com.gemwallet.android.domains.duration.formatDuration
import com.gemwallet.android.domains.percentage.PercentageFormatterStyle
import com.gemwallet.android.domains.percentage.formatAsPercentage
import com.gemwallet.android.ext.asset
import com.gemwallet.android.model.AssetInfo
import com.gemwallet.android.model.ValueFormatter
import com.gemwallet.android.ui.icons.AppIcons
import com.gemwallet.android.ui.models.subtitleSymbol
import com.gemwallet.android.ui.open
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.theme.paddingLarge
import com.gemwallet.android.ui.components.empty.EmptyContentType
import com.gemwallet.android.ui.components.empty.EmptyContentView
import com.gemwallet.android.ui.components.InfoSheetEntity
import com.gemwallet.android.ui.components.list_head.CenteredListHead
import com.gemwallet.android.ui.components.list_head.HeaderIcon
import com.gemwallet.android.ui.components.list_item.DelegationItem
import com.gemwallet.android.ui.components.list_item.SubheaderItem
import com.gemwallet.android.ui.components.list_item.energyItem
import com.gemwallet.android.ui.components.list_item.property.PropertyItem
import com.gemwallet.android.ui.components.list_item.property.itemsPositioned
import com.gemwallet.android.ui.components.screen.PullToRefreshBox
import com.gemwallet.android.ui.components.screen.Scene
import com.gemwallet.android.ui.models.ListPosition
import com.gemwallet.android.ui.models.actions.AmountTransactionAction
import uniffi.gemstone.GemStakeActionItem
import com.gemwallet.android.features.stake.presents.components.stakeActions
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.Delegation
import java.math.BigInteger

@Composable
internal fun StakeScene(
    inSync: Boolean,
    assetInfo: AssetInfo,
    actions: List<GemStakeActionItem>,
    rewardsText: String,
    delegations: List<Delegation>,
    stakeInfoUrl: String?,
    lockTimeDays: Int?,
    minStakeAmount: BigInteger,
    amountAction: AmountTransactionAction,
    onAction: (StakeSceneAction) -> Unit,
) {
    val context = LocalContext.current
    val uriHandler = LocalUriHandler.current

    Scene(
        title = stringResource(id = R.string.transfer_stake_title),
        onClose = { onAction(StakeSceneAction.Cancel) },
        actions = {
            stakeInfoUrl?.let { url ->
                IconButton(onClick = { uriHandler.open(context, url) }) {
                    Icon(imageVector = AppIcons.InfoOutlined, contentDescription = null)
                }
            }
        },
    ) {
        PullToRefreshBox(
            isRefreshing = inSync,
            onRefresh = { onAction(StakeSceneAction.Refresh) },
        ) {
            LazyColumn(modifier = Modifier.fillMaxSize()) {
                item {
                    CenteredListHead(
                        title = assetInfo.asset.name,
                        subtitle = assetInfo.asset.subtitleSymbol,
                        leading = { HeaderIcon(assetInfo.asset) },
                    )
                }

                stakeInfoSection(assetInfo, lockTimeDays, minStakeAmount)

                stakeActions(
                    actions = actions,
                    rewardsText = rewardsText,
                    assetId = assetInfo.id(),
                    amountAction = amountAction,
                    onRewards = { onAction(StakeSceneAction.ClaimRewards) },
                )

                energyItem(assetInfo.balance.metadata)

                if (delegations.isEmpty()) {
                    item {
                        Spacer(modifier = Modifier.height(paddingLarge))
                        EmptyContentView(type = EmptyContentType.Stake(symbol = assetInfo.asset.symbol))
                    }
                } else {
                    item { SubheaderItem(R.string.stake_delegations) }
                    itemsIndexed(delegations) { index, item ->
                        DelegationItem(
                            assetInfo = assetInfo,
                            delegation = item,
                            listPosition = ListPosition.getPosition(index, delegations.size),
                            onClick = { onAction(StakeSceneAction.OpenDelegation(item)) }
                        )
                    }
                }
            }
        }
    }
}

private sealed interface StakeInfoRow {
    data class MinAmount(val value: BigInteger, val chain: Chain) : StakeInfoRow
    data class Apr(val value: Double, val iconUrl: Any?) : StakeInfoRow
    data class LockTime(val days: Int, val iconUrl: Any?) : StakeInfoRow
}

private fun LazyListScope.stakeInfoSection(assetInfo: AssetInfo, lockTimeDays: Int?, minStakeAmount: BigInteger) {
    val iconUrl = assetInfo.id().iconModel()
    val rows = listOfNotNull(
        StakeInfoRow.Apr(assetInfo.metadata.stakingApr ?: 0.0, iconUrl),
        lockTimeDays?.let { StakeInfoRow.LockTime(it, iconUrl) },
        minStakeAmount.takeIf { it > BigInteger.ZERO }?.let { StakeInfoRow.MinAmount(it, assetInfo.asset.chain) },
    )
    itemsPositioned(rows) { position, row ->
        when (row) {
            is StakeInfoRow.MinAmount -> PropertyItem(
                title = stringResource(id = R.string.stake_minimum_amount, ""),
                data = ValueFormatter(style = ValueFormatter.Style.Full)
                    .string(row.value, row.chain.asset()),
                listPosition = position,
            )
            is StakeInfoRow.Apr -> PropertyItem(
                title = stringResource(id = R.string.stake_apr, ""),
                data = row.value.formatAsPercentage(style = PercentageFormatterStyle.PercentSignLess),
                dataColor = MaterialTheme.colorScheme.tertiary,
                info = InfoSheetEntity.StakeAprInfo(icon = row.iconUrl),
                listPosition = position,
            )
            is StakeInfoRow.LockTime -> PropertyItem(
                title = stringResource(id = R.string.stake_lock_time),
                data = formatDuration(Measure(row.days, MeasureUnit.DAY)),
                info = InfoSheetEntity.StakeLockTimeInfo(icon = row.iconUrl),
                listPosition = position,
            )
        }
    }
}
