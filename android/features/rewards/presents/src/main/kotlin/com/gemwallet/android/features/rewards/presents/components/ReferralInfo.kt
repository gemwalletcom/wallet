package com.gemwallet.android.features.rewards.presents.components

import androidx.compose.foundation.clickable
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
import com.gemwallet.android.features.rewards.viewmodels.models.RewardRedemptionUIModel
import com.gemwallet.android.features.rewards.viewmodels.models.RewardsSectionUIModel
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.list_item.ListItem
import com.gemwallet.android.ui.components.list_item.ListItemModel
import com.gemwallet.android.ui.components.list_item.SubheaderItem
import com.gemwallet.android.ui.components.list_item.property.DataBadgeChevron
import com.gemwallet.android.ui.components.list_item.property.itemsPositioned
import com.gemwallet.android.ui.models.ListPosition

internal fun LazyListScope.referralInfo(sections: List<RewardsSectionUIModel>, redemptions: List<RewardRedemptionUIModel>, onRedeem: (RewardRedemptionUIModel) -> Unit) {
    sections.forEach { section ->
        item {
            section.title?.let { SubheaderItem(it) }
            section.rows.forEachIndexed { index, row ->
                ListItem(model = row, listPosition = ListPosition.getPosition(index, section.rows.size))
            }
        }
    }

    if (redemptions.isNotEmpty()) {
        item { SubheaderItem(R.string.rewards_ways_spend_title) }
        itemsPositioned(redemptions) { position, item ->
            RewardRedemptionOptionItem(item, position) { onRedeem(item) }
        }
    }
}

@Composable
private fun RewardRedemptionOptionItem(item: RewardRedemptionUIModel, listPosition: ListPosition, onClick: () -> Unit) {
    var showConfirm by remember { mutableStateOf(false) }
    ListItem(
        model = item.model,
        listPosition = listPosition,
        modifier = Modifier.clickable { if (item.redemption.canRedeem) showConfirm = true else onClick() },
        accessory = { DataBadgeChevron() },
    )

    if (!showConfirm) return

    AlertDialog(
        onDismissRequest = { showConfirm = false },
        containerColor = MaterialTheme.colorScheme.background,
        text = {
            Text(
                text = item.confirmationMessage,
                style = MaterialTheme.typography.bodyLarge,
            )
        },
        confirmButton = {
            Button(
                {
                    onClick()
                    showConfirm = false
                },
            ) { Text(stringResource(R.string.transfer_confirm)) }
        },
        dismissButton = {
            Button(
                {
                    showConfirm = false
                },
            ) { Text(stringResource(R.string.common_cancel)) }
        },
    )
}
