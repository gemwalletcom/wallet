package com.gemwallet.android.features.assets.presents.banner

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.Button
import androidx.compose.material3.ButtonDefaults
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.style.TextAlign
import com.gemwallet.android.ui.components.banner.BannerItemUIModel
import com.gemwallet.android.ui.components.buttons.secondaryActionButtonColors
import com.gemwallet.android.ui.components.image.ListItemImageView
import com.gemwallet.android.ui.components.list_item.listItem
import com.gemwallet.android.ui.icons.AppIcons
import com.gemwallet.android.ui.localization.stringRes
import com.gemwallet.android.ui.theme.paddingDefault
import com.gemwallet.android.ui.theme.paddingSmall
import uniffi.gemstone.GemBannerButton

@Composable
internal fun WelcomeBanner(model: BannerItemUIModel, onBuy: () -> Unit, onReceive: () -> Unit, onClose: () -> Unit) {
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
            model.icon?.let { ListItemImageView(image = it) }
            model.title?.let { title ->
                Text(
                    text = title,
                    style = MaterialTheme.typography.titleMedium,
                    color = MaterialTheme.colorScheme.onSurface,
                )
            }
            model.subtitle?.let { subtitle ->
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
                model.buttons.forEachIndexed { index, button ->
                    Button(
                        modifier = Modifier.weight(1f),
                        onClick = when (button) {
                            GemBannerButton.BUY -> onBuy
                            GemBannerButton.RECEIVE -> onReceive
                        },
                        colors = if (index == 0) ButtonDefaults.buttonColors() else secondaryActionButtonColors(),
                    ) {
                        Text(stringResource(button.stringRes()))
                    }
                }
            }
        }
        if (model.canClose) {
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
