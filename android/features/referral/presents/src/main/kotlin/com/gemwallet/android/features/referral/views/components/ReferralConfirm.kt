package com.gemwallet.android.features.referral.views.components

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.LazyListScope
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Info
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.Icon
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import com.gemwallet.android.math.getRelativeDate
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.buttons.MainActionButton
import com.gemwallet.android.ui.components.list_item.ListItemSupportText
import com.gemwallet.android.ui.components.list_item.listItem
import com.gemwallet.android.ui.components.list_item.property.PropertyTitleText
import com.gemwallet.android.ui.models.ListPosition
import com.gemwallet.android.ui.theme.WalletTheme
import com.gemwallet.android.ui.theme.paddingDefault
import com.gemwallet.android.ui.theme.paddingHalfSmall
import com.gemwallet.android.ui.theme.paddingSmall
import com.gemwallet.android.ui.theme.pendingColor
import com.gemwallet.android.ui.theme.tinyIconSize
import com.gemwallet.android.features.referral.viewmodels.RewardsUIState
import com.wallet.core.primitives.RewardStatus
import com.wallet.core.primitives.Rewards

internal fun LazyListScope.referralConfirmCode(rewards: Rewards, uiState: RewardsUIState, onConfirm: (String) -> Unit) {
    val code = rewards.usedReferralCode ?: return
    if (!uiState.hasPendingReferral) return
    val pendingDate = rewards.verifyAfter ?: return
    item {
        Column(
            modifier = Modifier
                .fillMaxWidth()
                .listItem(ListPosition.Single).padding(paddingDefault),
            verticalArrangement = Arrangement.spacedBy(paddingHalfSmall)
        ) {
            PropertyTitleText(
                text = R.string.rewards_pending_title,
                trailing = {
                    Icon(
                        modifier = Modifier.size(tinyIconSize),
                        imageVector = Icons.Default.Info,
                        tint = pendingColor,
                        contentDescription = "",
                    )
                }
            )
            ListItemSupportText(
                if (uiState.canActivatePendingReferral) {
                    stringResource(R.string.rewards_pending_description_ready)
                } else {
                    stringResource(R.string.rewards_pending_description, getRelativeDate(pendingDate))
                }
            )
            HorizontalDivider(modifier = Modifier.padding(vertical = paddingSmall), thickness = 0.5.dp)
            MainActionButton(
                title = stringResource(R.string.transfer_confirm),
                enabled = uiState.canActivatePendingReferral
            ) {
                onConfirm(code)
            }
        }
    }
}

@Preview
@Composable
private fun ReferralConfirmCodePendingPreview() {
    WalletTheme {
        LazyColumn {
            referralConfirmCode(
                Rewards(
                    referralCount = 0,
                    points = 0,
                    status = RewardStatus.Pending,
                    redemptionOptions = emptyList(),
                    code = "some_code",
                    usedReferralCode = "some_code_1",
                    verifyAfter = System.currentTimeMillis() + 86400000,
                ),
                RewardsUIState(canInvite = false, isUnverified = false, hasPendingReferral = true, canActivatePendingReferral = false),
            ) {}
        }
    }
}

@Preview
@Composable
private fun ReferralConfirmCodeReadyPreview() {
    WalletTheme {
        LazyColumn {
            referralConfirmCode(
                Rewards(
                    referralCount = 0,
                    points = 0,
                    status = RewardStatus.Pending,
                    redemptionOptions = emptyList(),
                    code = "some_code",
                    usedReferralCode = "some_code_1",
                    verifyAfter = 0,
                ),
                RewardsUIState(canInvite = false, isUnverified = false, hasPendingReferral = true, canActivatePendingReferral = true),
            ) {}
        }
    }
}
