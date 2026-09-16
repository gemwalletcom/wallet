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
import uniffi.gemstone.GemPercentageStyle
import com.gemwallet.android.domains.percentage.formatAsPercentage
import com.gemwallet.android.ext.asset
import com.gemwallet.android.model.AssetInfo
import com.gemwallet.android.model.ValueFormatter
import com.gemwallet.android.ui.icons.AppIcons
import com.gemwallet.android.domains.asset.subtitleSymbol
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
import uniffi.gemstone.GemStakeInfoRow
import uniffi.gemstone.GemStakeSection
import com.gemwallet.android.features.stake.presents.localization.stringRes
import com.gemwallet.android.features.stake.presents.components.stakeActions
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.Delegation
import uniffi.gemstone.GemValidatorRow
import java.math.BigInteger
import uniffi.gemstone.GemValueStyle

@Composable
internal fun StakeScene(
    inSync: Boolean,
    assetInfo: AssetInfo,
    actions: List<GemStakeActionItem>,
    rewardsText: String,
    delegations: List<Delegation>,
    validatorRows: Map<String, GemValidatorRow>,
    stakeInfoUrl: String?,
    lockTimeDays: Int?,
    minStakeAmount: BigInteger,
    sections: List<GemStakeSection>,
    infoRows: List<GemStakeInfoRow>,
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

                stakeInfoSection(assetInfo, infoRows, lockTimeDays, minStakeAmount)

                sections.forEach { section ->
                    item { SubheaderItem(section.stringRes()) }
                    when (section) {
                        GemStakeSection.MANAGE -> stakeActions(
                            actions = actions,
                            rewardsText = rewardsText,
                            assetId = assetInfo.id(),
                            amountAction = amountAction,
                            onRewards = { onAction(StakeSceneAction.ClaimRewards) },
                        )
                        GemStakeSection.RESOURCES -> energyItem(assetInfo.balance.metadata)
                        GemStakeSection.DELEGATIONS -> itemsIndexed(delegations) { index, item ->
                            DelegationItem(
                                assetInfo = assetInfo,
                                delegation = item,
                                validator = validatorRows[item.validator.id] ?: return@itemsIndexed,
                                listPosition = ListPosition.getPosition(index, delegations.size),
                                onClick = { onAction(StakeSceneAction.OpenDelegation(item)) }
                            )
                        }
                    }
                }

                if (!sections.contains(GemStakeSection.DELEGATIONS)) {
                    item {
                        Spacer(modifier = Modifier.height(paddingLarge))
                        EmptyContentView(type = EmptyContentType.Stake(symbol = assetInfo.asset.symbol))
                    }
                }
            }
        }
    }
}

private fun LazyListScope.stakeInfoSection(assetInfo: AssetInfo, rows: List<GemStakeInfoRow>, lockTimeDays: Int?, minStakeAmount: BigInteger) {
    val iconUrl = assetInfo.id().iconModel()
    itemsPositioned(rows) { position, row ->
        when (row) {
            GemStakeInfoRow.MINIMUM_AMOUNT -> PropertyItem(
                title = stringResource(row.stringRes(), ""),
                data = ValueFormatter(style = GemValueStyle.AUTO).string(minStakeAmount, assetInfo.asset.chain.asset()),
                listPosition = position,
            )
            GemStakeInfoRow.APR -> PropertyItem(
                title = stringResource(row.stringRes(), ""),
                data = (assetInfo.metadata.stakingApr ?: 0.0).formatAsPercentage(style = GemPercentageStyle.UNSIGNED),
                dataColor = MaterialTheme.colorScheme.tertiary,
                info = InfoSheetEntity.StakeAprInfo(icon = iconUrl),
                listPosition = position,
            )
            GemStakeInfoRow.LOCK_TIME -> PropertyItem(
                title = stringResource(row.stringRes()),
                data = formatDuration(Measure(lockTimeDays ?: 0, MeasureUnit.DAY)),
                info = InfoSheetEntity.StakeLockTimeInfo(icon = iconUrl),
                listPosition = position,
            )
        }
    }
}
