package com.gemwallet.android.features.referral.views.components

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.LazyListScope
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.Icon
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.tooling.preview.Preview
import com.gemwallet.android.features.referral.viewmodels.models.ReferralUIModel
import com.gemwallet.android.features.referral.views.previewRewardsState
import com.gemwallet.android.math.getRelativeDate
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.buttons.MainActionButton
import com.gemwallet.android.ui.components.list_item.ListItemSupportText
import com.gemwallet.android.ui.components.list_item.listItem
import com.gemwallet.android.ui.components.list_item.property.PropertyTitleText
import com.gemwallet.android.ui.icons.AppIcons
import com.gemwallet.android.ui.models.ListPosition
import com.gemwallet.android.ui.models.buttonState
import com.gemwallet.android.ui.theme.WalletTheme
import com.gemwallet.android.ui.theme.hairlineThickness
import com.gemwallet.android.ui.theme.paddingDefault
import com.gemwallet.android.ui.theme.paddingHalfSmall
import com.gemwallet.android.ui.theme.paddingSmall
import com.gemwallet.android.ui.theme.pendingColor
import com.gemwallet.android.ui.theme.tinyIconSize

internal fun LazyListScope.referralConfirmCode(uiState: ReferralUIModel, onConfirm: (String) -> Unit) {
    if (!uiState.hasPendingReferral) return
    val code = uiState.usedReferralCode ?: return
    val pendingDate = uiState.verifyAfter ?: return
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
                        imageVector = AppIcons.Info,
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
            HorizontalDivider(modifier = Modifier.padding(vertical = paddingSmall), thickness = hairlineThickness)
            MainActionButton(
                title = stringResource(R.string.transfer_confirm),
                state = buttonState(enabled = uiState.canActivatePendingReferral)
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
                pendingState(canActivate = false, verifyAfter = System.currentTimeMillis() + 86400000),
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
                pendingState(canActivate = true, verifyAfter = 0),
            ) {}
        }
    }
}

private fun pendingState(canActivate: Boolean, verifyAfter: Long) = previewRewardsState(
    hasReferralCode = true,
    hasUsedReferralCode = true,
    showsInfo = true,
    hasPendingReferral = true,
    canActivatePendingReferral = canActivate,
    referralCode = "some_code",
    usedReferralCode = "some_code_1",
    verifyAfter = verifyAfter,
)
