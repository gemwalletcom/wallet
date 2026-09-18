package com.gemwallet.android.features.stake.viewmodels.models

import android.content.Context
import android.icu.util.Measure
import com.gemwallet.android.domains.asset.chain
import com.gemwallet.android.domains.duration.formatDuration
import com.gemwallet.android.domains.percentage.formatAsPercentage
import com.gemwallet.android.ext.asset
import com.gemwallet.android.features.stake.viewmodels.localization.stringRes
import com.gemwallet.android.model.AssetInfo
import com.gemwallet.android.model.ValueFormatter
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.InfoSheetEntity
import com.gemwallet.android.ui.components.image.iconModel
import com.gemwallet.android.ui.components.list_item.ListItemModel
import com.gemwallet.android.ui.components.list_item.ListItemTextStyle
import java.math.BigInteger
import uniffi.gemstone.GemDurationPart
import uniffi.gemstone.GemPercentageStyle
import uniffi.gemstone.GemStakeAction
import uniffi.gemstone.GemStakeActionItem
import uniffi.gemstone.GemStakeInfoRow
import uniffi.gemstone.GemValueStyle

enum class StakeAction { Stake, Freeze, Unfreeze, ClaimRewards }

data class StakeActionUIModel(
    val action: StakeAction,
    val isEnabled: Boolean,
    val requiresFrozenBalance: Boolean,
    val model: ListItemModel,
)

internal fun GemStakeActionItem.uiModel(context: Context, assetInfo: AssetInfo, rewardsText: String): StakeActionUIModel = StakeActionUIModel(
    action = when (action) {
        GemStakeAction.STAKE -> StakeAction.Stake
        GemStakeAction.FREEZE -> StakeAction.Freeze
        GemStakeAction.UNFREEZE -> StakeAction.Unfreeze
        GemStakeAction.CLAIM_REWARDS -> StakeAction.ClaimRewards
    },
    isEnabled = isEnabled,
    requiresFrozenBalance = requiresFrozenBalance,
    model = ListItemModel(
        title = context.getString(action.stringRes()),
        titleStyle = if (requiresFrozenBalance) ListItemTextStyle.Faded else ListItemTextStyle.Body,
        subtitle = rewardsText.takeIf { action == GemStakeAction.CLAIM_REWARDS },
        info = InfoSheetEntity.StakeFrozenRequired(assetInfo.id().iconModel()).takeIf { requiresFrozenBalance },
    ),
)

internal fun GemStakeInfoRow.listItem(context: Context, assetInfo: AssetInfo, lockTimeParts: List<GemDurationPart>, minStakeAmount: BigInteger): ListItemModel {
    val iconUrl = assetInfo.id().iconModel()
    return when (this) {
        GemStakeInfoRow.MINIMUM_AMOUNT -> ListItemModel(
            title = context.getString(stringRes(), ""),
            subtitle = ValueFormatter(style = GemValueStyle.AUTO).string(minStakeAmount, assetInfo.asset.chain.asset()),
        )
        GemStakeInfoRow.APR -> ListItemModel(
            title = context.getString(stringRes(), ""),
            subtitle = (assetInfo.metadata.stakingApr ?: 0.0).formatAsPercentage(style = GemPercentageStyle.UNSIGNED),
            subtitleStyle = ListItemTextStyle.Positive,
            info = InfoSheetEntity.StakeAprInfo(icon = iconUrl),
        )
        GemStakeInfoRow.LOCK_TIME -> ListItemModel(
            title = context.getString(stringRes()),
            subtitle = lockTimeParts.formatDuration(),
            info = InfoSheetEntity.StakeLockTimeInfo(icon = iconUrl),
        )
    }
}
