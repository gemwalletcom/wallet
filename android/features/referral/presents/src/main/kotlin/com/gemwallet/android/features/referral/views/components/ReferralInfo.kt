package com.gemwallet.android.features.referral.views.components

import com.gemwallet.android.ext.toPrimitives
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.lazy.LazyListScope
import androidx.compose.material3.AlertDialog
import androidx.compose.material3.Button
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import com.gemwallet.android.model.text
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.image.AssetIcon
import com.gemwallet.android.ui.components.list_item.ListItemDefaults
import com.gemwallet.android.ui.components.list_item.SubheaderItem
import com.gemwallet.android.ui.components.list_item.property.DataBadgeChevron
import com.gemwallet.android.ui.components.list_item.property.PropertyDataText
import com.gemwallet.android.ui.components.list_item.property.PropertyItem
import com.gemwallet.android.ui.components.list_item.property.PropertyTitleText
import com.gemwallet.android.ui.components.list_item.property.itemsPositioned
import com.gemwallet.android.ui.models.ListPosition
import uniffi.gemstone.GemRewardsRedemption
import uniffi.gemstone.GemRewardsState
import uniffi.gemstone.RewardRedemptionOption

internal fun LazyListScope.referralInfo(
    uiState: GemRewardsState,
    onRedeem: (GemRewardsRedemption) -> Unit,
) {
    item {
        SubheaderItem(R.string.common_info)
        PropertyItem(
            title = R.string.rewards_my_referral_code,
            data = uiState.referralCode,
            listPosition = ListPosition.First
        )
        PropertyItem(
            title = R.string.rewards_referrals,
            data = uiState.referralCountText,
            listPosition = ListPosition.Middle
        )
        PropertyItem(
            title = R.string.rewards_points,
            data = uiState.pointsText,
            listPosition = ListPosition.Last
        )
    }

    val redemptions = uiState.redemptions
    if (redemptions.isNotEmpty()) {
        item { SubheaderItem(R.string.rewards_ways_spend_title) }
        itemsPositioned(redemptions) { position, item ->
            RewardRedemptionOptionItem(item, position) { onRedeem(item) }
        }
    }
}

@Composable
private fun RewardRedemptionOptionItem(
    redemption: GemRewardsRedemption,
    listPosition: ListPosition = ListPosition.Middle,
    onClick: () -> Unit
) {
    val option = redemption.option
    val asset = option.asset ?: return
    var showConfirm by remember { mutableStateOf(false) }
    PropertyItem(
        modifier = Modifier
            .heightIn(min = ListItemDefaults.defaultMinHeight)
            .clickable { showConfirm = true },
        title = {
            PropertyTitleText(
                text = stringResource(R.string.rewards_ways_spend_asset_title, redemption.value.text()),
                trailing = { AssetIcon(asset.toPrimitives()) },
            )
        },
        data = {
            PropertyDataText(
                text = redemption.pointsText,
                badge = { DataBadgeChevron() },
            )
        },
        listPosition = listPosition,
    )

    if (!showConfirm) return

    AlertDialog(
        onDismissRequest = { showConfirm = false },
        containerColor = MaterialTheme.colorScheme.background,
        text = {
            Text(
                text = redemption.confirmationMessage(),
                style = MaterialTheme.typography.bodyLarge,
            )
        },
        confirmButton = {
            Button(
                {
                    onClick()
                    showConfirm = false
                }
            ) { Text(stringResource(R.string.transfer_confirm)) }
        },
        dismissButton = {
            Button(
                {
                    showConfirm = false
                }
            ) { Text(stringResource(R.string.common_cancel)) }
        }
    )
}

@Composable
private fun GemRewardsRedemption.confirmationMessage(): String =
    stringResource(R.string.rewards_confirm_redeem, value.text(), pointsText)

