package com.gemwallet.android.features.banner.views

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
import androidx.compose.foundation.pager.HorizontalPager
import androidx.compose.foundation.pager.rememberPagerState
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.hilt.lifecycle.viewmodel.compose.hiltViewModel
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.gemwallet.android.ext.toIdentifier
import com.gemwallet.android.ui.components.image.IconWithBadge
import com.gemwallet.android.ui.components.list_item.listItem
import com.gemwallet.android.ui.icons.AppIcons
import com.gemwallet.android.ui.models.ListPosition
import com.gemwallet.android.ui.theme.Spacer16
import com.gemwallet.android.ui.theme.listItemIconSize
import com.gemwallet.android.ui.theme.paddingDefault
import com.gemwallet.android.ui.theme.paddingMiddle
import com.gemwallet.android.ui.theme.smallIconSize
import com.gemwallet.android.ui.theme.space2
import com.gemwallet.android.features.banner.viewmodels.BannersViewModel
import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.Banner

private val bannerEmojiFontSize = 32.sp

@Composable
fun BannersScene(
    asset: Asset?,
    onClick: (Banner) -> Unit,
    isGlobal: Boolean = false,
    viewModel: BannersViewModel = hiltViewModel(),
) {
    LaunchedEffect(asset?.id?.toIdentifier(), isGlobal) {
        viewModel.init(asset, isGlobal)
    }

    val banners by viewModel.banners.collectAsStateWithLifecycle()
    val pageState = rememberPagerState { banners.size }

    if (banners.isEmpty()) {
        return
    }
    HorizontalPager(pageState, pageSpacing = paddingDefault) { page ->
        val banner = banners[page]
        val model = bannerItemUIModel(banner, asset, viewModel::getActivationFee)
        Box(modifier = Modifier.listItem(ListPosition.Single).clickable { onClick(banner) }) {
            BannerText(
                model = model,
                onCancel = { viewModel.onCancel(banner) },
            )
        }
    }
}

@Composable
private fun BannerText(
    model: BannerItemUIModel,
    onCancel: () -> Unit,
) {
    Box(modifier = Modifier.fillMaxWidth()) {
        Row(
            modifier = Modifier.fillMaxWidth(),
            verticalAlignment = Alignment.CenterVertically,
        ) {
            Spacer16()
            BannerIconView(model.icon)
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
                Text(
                    modifier = Modifier.fillMaxWidth(),
                    text = model.title,
                    maxLines = 1,
                    softWrap = false,
                    overflow = TextOverflow.Ellipsis,
                    style = MaterialTheme.typography.bodyLarge.copy(fontWeight = FontWeight.W500),
                    color = MaterialTheme.colorScheme.onSurface,
                )
                Spacer(modifier = Modifier.height(space2))
                Text(
                    modifier = Modifier.padding(bottom = space2),
                    text = model.subtitle,
                    overflow = TextOverflow.Ellipsis,
                    color = MaterialTheme.colorScheme.secondary,
                    style = MaterialTheme.typography.bodyMedium,
                )
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

@Composable
private fun BannerIconView(icon: BannerIcon) {
    when (icon) {
        is BannerIcon.Emoji -> Text(text = icon.value, fontSize = bannerEmojiFontSize)
        is BannerIcon.Url -> IconWithBadge(icon = icon.value, placeholder = icon.value, size = listItemIconSize)
        is BannerIcon.Vector -> Icon(
            modifier = Modifier.size(listItemIconSize),
            imageVector = icon.image,
            contentDescription = null,
            tint = MaterialTheme.colorScheme.secondary,
        )
    }
}
