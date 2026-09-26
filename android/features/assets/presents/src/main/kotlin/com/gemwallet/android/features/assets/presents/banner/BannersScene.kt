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
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.sp
import com.gemwallet.android.ui.components.banner.BannerItemUIModel
import com.gemwallet.android.ui.components.banner.BannerRowUIModel
import com.gemwallet.android.ui.components.image.ListItemImageView
import com.gemwallet.android.ui.components.list_item.listItem
import com.gemwallet.android.ui.icons.AppIcons
import com.gemwallet.android.ui.models.ListPosition
import com.gemwallet.android.ui.theme.Spacer16
import com.gemwallet.android.ui.theme.paddingDefault
import com.gemwallet.android.ui.theme.paddingMiddle
import com.gemwallet.android.ui.theme.smallIconSize
import com.gemwallet.android.ui.theme.space2
import com.wallet.core.primitives.BannerEvent
import uniffi.gemstone.GemBannerDestination
import uniffi.gemstone.GemBannerKey
import uniffi.gemstone.GemBannerStyle

private val bannerEmojiFontSize = 32.sp

@Composable
fun BannerScene(banner: BannerRowUIModel, onSelect: (GemBannerDestination) -> Unit, onClose: (GemBannerKey) -> Unit, onBuy: () -> Unit = {}, onReceive: () -> Unit = {}) {
    val model = banner.model
    if (model.style == GemBannerStyle.WELCOME) {
        WelcomeBanner(model = model, onBuy = onBuy, onReceive = onReceive, onClose = { onClose(banner.key) })
        return
    }
    Box(
        modifier = Modifier.listItem(ListPosition.Single).clickable {
            model.destination?.let(onSelect)
        },
    ) {
        BannerText(
            model = model,
            onCancel = { onClose(banner.key) },
        )
    }
}

@Composable
private fun BannerText(model: BannerItemUIModel, onCancel: () -> Unit) {
    Box(modifier = Modifier.fillMaxWidth()) {
        Row(
            modifier = Modifier.fillMaxWidth(),
            verticalAlignment = Alignment.CenterVertically,
        ) {
            Spacer16()
            model.icon?.let { ListItemImageView(image = it) }
            Spacer16()
            Column(
                modifier = Modifier
                    .weight(1f)
                    .padding(
                        top = paddingMiddle,
                        end = if (model.canClose) smallIconSize + paddingDefault else paddingDefault,
                        bottom = paddingMiddle,
                    ),
                verticalArrangement = Arrangement.Center,
            ) {
                model.title?.let { title ->
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
                model.subtitle?.let { subtitle ->
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
        if (model.canClose) {
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
