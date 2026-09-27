package com.gemwallet.android.features.assets.presents.banner

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.Button
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.style.TextAlign
import com.gemwallet.android.ui.components.image.ListItemImageView
import com.gemwallet.android.ui.components.list_item.listItem
import com.gemwallet.android.ui.icons.AppIcons
import com.gemwallet.android.ui.localization.bannerDescription
import com.gemwallet.android.ui.localization.bannerTitle
import com.gemwallet.android.ui.localization.stringRes
import com.gemwallet.android.ui.style.colors
import com.gemwallet.android.ui.style.image
import com.gemwallet.android.ui.theme.paddingDefault
import com.gemwallet.android.ui.theme.paddingSmall
import uniffi.gemstone.GemBannerButton
import uniffi.gemstone.GemBannerContent

@Composable
internal fun WelcomeBanner(content: GemBannerContent, onBuy: () -> Unit, onReceive: () -> Unit, onClose: () -> Unit) {
    val context = LocalContext.current
    Box(
        modifier = Modifier.fillMaxWidth().listItem(),
    ) {
        Column(
            modifier = Modifier
                .fillMaxWidth()
                .padding(paddingDefault),
            horizontalAlignment = Alignment.CenterHorizontally,
            verticalArrangement = Arrangement.spacedBy(paddingSmall),
        ) {
            content.icon?.let { ListItemImageView(image = it.image()) }
            content.title?.let { bannerTitle(context, it) }?.let { title ->
                Text(
                    text = title,
                    style = MaterialTheme.typography.titleMedium,
                    color = MaterialTheme.colorScheme.onSurface,
                )
            }
            content.description?.let { bannerDescription(context, it) }?.let { subtitle ->
                Text(
                    text = subtitle,
                    style = MaterialTheme.typography.bodyLarge,
                    color = MaterialTheme.colorScheme.onBackground,
                    textAlign = TextAlign.Center,
                )
            }
            Row(
                modifier = Modifier.fillMaxWidth(),
                verticalAlignment = Alignment.CenterVertically,
                horizontalArrangement = Arrangement.spacedBy(paddingDefault),
            ) {
                content.buttons.forEach { button ->
                    Button(
                        modifier = Modifier.weight(1f),
                        onClick = when (button) {
                            GemBannerButton.BUY -> onBuy
                            GemBannerButton.RECEIVE -> onReceive
                        },
                        colors = button.colors(),
                    ) {
                        Text(stringResource(button.stringRes()))
                    }
                }
            }
        }
        if (content.canClose) {
            IconButton(
                modifier = Modifier.align(Alignment.TopEnd),
                onClick = onClose,
            ) {
                Icon(
                    imageVector = AppIcons.Close,
                    contentDescription = null,
                )
            }
        }
    }
}
