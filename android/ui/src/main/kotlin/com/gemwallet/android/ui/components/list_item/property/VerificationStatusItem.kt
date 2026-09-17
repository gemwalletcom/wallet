package com.gemwallet.android.ui.components.list_item.property

import androidx.annotation.DrawableRes
import androidx.annotation.StringRes
import androidx.compose.foundation.Image
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.lazy.LazyListScope
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.res.stringResource
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.InfoBottomSheet
import com.gemwallet.android.ui.components.InfoSheetEntity
import com.gemwallet.android.ui.components.list_item.ListItem
import com.gemwallet.android.ui.components.list_item.ListItemModel
import com.gemwallet.android.ui.models.ListPosition
import com.gemwallet.android.ui.style.textStyle
import com.gemwallet.android.ui.theme.compactIconSize
import com.gemwallet.android.ui.theme.smallIconSize
import com.wallet.core.primitives.VerificationStatus

fun LazyListScope.verificationStatusItem(
    status: VerificationStatus,
    listPosition: ListPosition = ListPosition.Single,
) {
    if (status == VerificationStatus.Verified) {
        return
    }
    item {
        VerificationStatusItem(status, listPosition)
    }
}

@Composable
private fun VerificationStatusItem(
    status: VerificationStatus,
    listPosition: ListPosition,
) {
    val display = status.display() ?: return
    var showInfoSheet by remember { mutableStateOf(false) }

    ListItem(
        model = ListItemModel(
            title = stringResource(R.string.transaction_status),
            subtitle = stringResource(display.labelRes),
            subtitleStyle = status.textStyle(),
            info = display.infoSheetEntity,
        ),
        listPosition = listPosition,
        modifier = Modifier.clickable { showInfoSheet = true },
        accessory = {
            DataBadgeChevron {
                VerificationBadgeIcon(display.badgeIconRes)
            }
        },
    )

    if (showInfoSheet) {
        InfoBottomSheet(item = display.infoSheetEntity) {
            showInfoSheet = false
        }
    }
}

@Composable
private fun VerificationBadgeIcon(@DrawableRes icon: Int) {
    Box(
        modifier = Modifier.size(smallIconSize),
        contentAlignment = Alignment.Center,
    ) {
        Image(
            painter = painterResource(icon),
            contentDescription = null,
            modifier = Modifier.size(compactIconSize),
        )
    }
}

private class VerificationStatusDisplay(
    @param:StringRes val labelRes: Int,
    @param:DrawableRes val badgeIconRes: Int,
    val infoSheetEntity: InfoSheetEntity,
)

private fun VerificationStatus.display(): VerificationStatusDisplay? = when (this) {
    VerificationStatus.Verified -> null
    VerificationStatus.Unverified -> VerificationStatusDisplay(
        labelRes = R.string.asset_verification_unverified,
        badgeIconRes = R.drawable.unverified,
        infoSheetEntity = InfoSheetEntity.AssetStatusUnverifiedInfo,
    )
    VerificationStatus.Suspicious -> VerificationStatusDisplay(
        labelRes = R.string.asset_verification_suspicious,
        badgeIconRes = R.drawable.suspicious,
        infoSheetEntity = InfoSheetEntity.AssetStatusSuspiciousInfo,
    )
}
