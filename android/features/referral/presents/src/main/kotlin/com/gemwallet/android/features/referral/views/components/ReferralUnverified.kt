package com.gemwallet.android.features.referral.views.components

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.lazy.LazyListScope
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Info
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import com.gemwallet.android.features.referral.viewmodels.RewardsUIState
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.list_item.listItem
import com.gemwallet.android.ui.components.list_item.property.PropertyTitleText
import com.gemwallet.android.ui.models.ListPosition
import com.gemwallet.android.ui.theme.paddingDefault
import com.gemwallet.android.ui.theme.paddingHalfSmall
import com.gemwallet.android.ui.theme.pendingColor
import com.gemwallet.android.ui.theme.tinyIconSize

internal fun LazyListScope.referralUnverified(uiState: RewardsUIState) {
    if (!uiState.isUnverified) return
    item {
        Column(
            modifier = Modifier
                .fillMaxWidth()
                .listItem(ListPosition.Single).padding(paddingDefault),
            verticalArrangement = Arrangement.spacedBy(paddingHalfSmall)
        ) {
            PropertyTitleText(
                text = R.string.rewards_unverified_title,
                trailing = {
                    Icon(
                        modifier = Modifier.size(tinyIconSize),
                        imageVector = Icons.Default.Info,
                        tint = pendingColor,
                        contentDescription = "",
                    )
                }
            )
            Text(
                text = stringResource(R.string.rewards_unverified_description),
                color = MaterialTheme.colorScheme.secondary,
                style = MaterialTheme.typography.bodyMedium,
            )
        }
    }
}
