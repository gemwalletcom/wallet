package com.gemwallet.android.features.assets.presents.banner

import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.sp
import com.gemwallet.android.ui.components.image.ListItemImageView
import com.gemwallet.android.ui.components.list_item.listItem
import com.gemwallet.android.ui.icons.AppIcons
import com.gemwallet.android.ui.localization.bannerDescription
import com.gemwallet.android.ui.localization.bannerTitle
import com.gemwallet.android.ui.models.ListPosition
import com.gemwallet.android.ui.style.image
import com.gemwallet.android.ui.theme.Spacer16
import com.gemwallet.android.ui.theme.paddingDefault
import com.gemwallet.android.ui.theme.paddingMiddle
import com.gemwallet.android.ui.theme.smallIconSize
import com.gemwallet.android.ui.theme.space2
import com.wallet.core.primitives.BannerEvent
import uniffi.gemstone.GemBannerContent
import uniffi.gemstone.GemBannerDestination
import uniffi.gemstone.GemBannerKey
import uniffi.gemstone.GemBannerRow
import uniffi.gemstone.GemBannerStyle

private val bannerEmojiFontSize = 32.sp

@Composable
fun Banner(banner: GemBannerRow, onSelect: (GemBannerDestination) -> Unit, onClose: (GemBannerKey) -> Unit, onBuy: () -> Unit = {}, onReceive: () -> Unit = {}) {
    val content = banner.content
    if (content.style == GemBannerStyle.WELCOME) {
        WelcomeBanner(content = content, onBuy = onBuy, onReceive = onReceive, onClose = { onClose(banner.key) })
        return
    }
    Box(
        modifier = Modifier.listItem(ListPosition.Single).clickable {
            content.destination?.let(onSelect)
        },
    ) {
        BannerText(
            content = content,
            onCancel = { onClose(banner.key) },
        )
    }
}

@Composable
private fun BannerText(content: GemBannerContent, onCancel: () -> Unit) {
    val context = LocalContext.current
    Box(modifier = Modifier.fillMaxWidth()) {
        Row(
            modifier = Modifier.fillMaxWidth(),
            verticalAlignment = Alignment.CenterVertically,
        ) {
            Spacer16()
            content.icon?.let { ListItemImageView(image = it.image()) }
            Spacer16()
            Column(
                modifier = Modifier
                    .weight(1f)
                    .padding(
                        top = paddingMiddle,
                        end = if (content.canClose) smallIconSize + paddingDefault else paddingDefault,
                        bottom = paddingMiddle,
                    ),
                verticalArrangement = Arrangement.Center,
            ) {
                content.title?.let { bannerTitle(context, it) }?.let { title ->
                    Text(
                        modifier = Modifier.fillMaxWidth(),
                        text = title,
                        maxLines = 1,
                        softWrap = false,
                        overflow = TextOverflow.Ellipsis,
                        style = MaterialTheme.typography.bodyLarge.copy(fontWeight = FontWeight.W500),
                        color = MaterialTheme.colorScheme.onSurface,
                    )
                }
                content.description?.let { bannerDescription(context, it) }?.let { subtitle ->
                    Spacer(modifier = Modifier.height(space2))
                    Text(
                        modifier = Modifier.padding(bottom = space2),
                        text = subtitle,
                        overflow = TextOverflow.Ellipsis,
                        color = MaterialTheme.colorScheme.secondary,
                        style = MaterialTheme.typography.bodyMedium,
                    )
                }
            }
        }
        if (content.canClose) {
            IconButton(
                modifier = Modifier
                    .align(Alignment.TopEnd)
                    .padding(top = paddingMiddle, end = paddingMiddle)
                    .size(smallIconSize),
                onClick = onCancel,
            ) {
                Icon(
                    imageVector = AppIcons.Close,
                    contentDescription = "cancel_banner",
                    tint = MaterialTheme.colorScheme.secondary,
                )
            }
        }
    }
}
