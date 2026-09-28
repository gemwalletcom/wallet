package com.gemwallet.android.ui.components.list_item.property

import androidx.annotation.DrawableRes
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
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.InfoBottomSheet
import com.gemwallet.android.ui.components.infoSheet
import com.gemwallet.android.ui.components.list_item.ListItem
import com.gemwallet.android.ui.components.list_item.ListItemModel
import com.gemwallet.android.ui.localization.labelRes
import com.gemwallet.android.ui.models.ListPosition
import com.gemwallet.android.ui.style.badgeIconRes
import com.gemwallet.android.ui.style.textStyle
import com.gemwallet.android.ui.theme.compactIconSize
import com.gemwallet.android.ui.theme.smallIconSize
import com.wallet.core.primitives.VerificationStatus
import uniffi.gemstone.GemInfoTopic

fun LazyListScope.verificationStatusItem(status: VerificationStatus, listPosition: ListPosition = ListPosition.Single) {
    if (status == VerificationStatus.Verified) {
        return
    }
    item {
        VerificationStatusItem(status, listPosition)
    }
}

@Composable
private fun VerificationStatusItem(status: VerificationStatus, listPosition: ListPosition) {
    val labelRes = status.labelRes() ?: return
    val badgeIconRes = status.badgeIconRes() ?: return
    val infoSheetEntity = GemInfoTopic.AssetStatus(status.toGem()).infoSheet()
    var showInfoSheet by remember { mutableStateOf(false) }

    ListItem(
        model = ListItemModel(
            title = stringResource(R.string.transaction_status),
            subtitle = stringResource(labelRes),
            subtitleStyle = status.textStyle(),
            info = infoSheetEntity,
        ),
        listPosition = listPosition,
        modifier = Modifier.clickable { showInfoSheet = true },
        accessory = {
            DataBadgeChevron {
                VerificationBadgeIcon(badgeIconRes)
            }
        },
    )

    if (showInfoSheet) {
        InfoBottomSheet(item = infoSheetEntity) {
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
