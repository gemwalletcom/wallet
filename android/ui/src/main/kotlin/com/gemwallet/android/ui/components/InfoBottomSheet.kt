package com.gemwallet.android.ui.components

import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.platform.LocalUriHandler
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextAlign
import com.gemwallet.android.ui.components.buttons.MainActionButton
import com.gemwallet.android.ui.components.dialog.DialogBar
import com.gemwallet.android.ui.components.image.IconWithBadge
import com.gemwallet.android.ui.components.screen.ModalBottomSheet
import com.gemwallet.android.ui.components.screen.SheetExpansion
import com.gemwallet.android.ui.localization.label
import com.gemwallet.android.ui.localization.string
import com.gemwallet.android.ui.open
import com.gemwallet.android.ui.style.badgeIconModel
import com.gemwallet.android.ui.style.iconModel
import com.gemwallet.android.ui.style.placeholder
import com.gemwallet.android.ui.theme.Spacer16
import com.gemwallet.android.ui.theme.extraLargeIconSize
import com.gemwallet.android.ui.theme.paddingDefault
import com.gemwallet.android.ui.theme.paddingSmall
import uniffi.gemstone.GemConfirmErrorInfo
import uniffi.gemstone.GemInfoAction
import uniffi.gemstone.GemInfoImage
import uniffi.gemstone.GemInfoSheet
import uniffi.gemstone.GemInfoTopic
import uniffi.gemstone.Platform

internal val infoSheetIconSize = extraLargeIconSize

data class InfoSheetEntity(val sheet: GemInfoSheet, val onAction: ((GemInfoAction) -> Unit)? = null)

fun GemInfoTopic.infoSheet(onAction: ((GemInfoAction) -> Unit)? = null): InfoSheetEntity = InfoSheetEntity(sheet(Platform.ANDROID), onAction)

fun GemConfirmErrorInfo.infoSheet(onAction: ((GemInfoAction) -> Unit)? = null): InfoSheetEntity = InfoSheetEntity(sheet(Platform.ANDROID), onAction)

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun InfoBottomSheet(item: InfoSheetEntity?, onClose: () -> Unit) {
    val uriHandler = LocalUriHandler.current
    val context = LocalContext.current
    ModalBottomSheet(
        item = item,
        expansion = SheetExpansion.Full,
        containerColor = MaterialTheme.colorScheme.background,
        onDismissRequest = onClose,
    ) { shownItem ->
        val sheet = shownItem.sheet
        val onAction = shownItem.onAction
        val action = sheet.button(onAction != null)
        Column(
            modifier = Modifier
                .fillMaxWidth()
                .verticalScroll(rememberScrollState()),
            horizontalAlignment = Alignment.CenterHorizontally,
        ) {
            DialogBar(onDismissRequest = onClose)
            InfoSheetIcon(sheet.image)
            Spacer16()
            Text(
                modifier = Modifier
                    .fillMaxWidth()
                    .padding(horizontal = paddingDefault),
                text = parseMarkdownToAnnotatedString(sheet.title.string(context)),
                fontWeight = FontWeight.Bold,
                color = MaterialTheme.colorScheme.onSurface,
                style = MaterialTheme.typography.headlineMedium,
                textAlign = TextAlign.Center,
            )
            Text(
                modifier = Modifier.padding(vertical = paddingSmall, horizontal = paddingDefault),
                text = parseMarkdownToAnnotatedString(sheet.description.string(context)),
                color = MaterialTheme.colorScheme.secondary,
                style = MaterialTheme.typography.bodyLarge,
                textAlign = TextAlign.Center,
            )
            if (action != null) {
                Box(
                    modifier = Modifier
                        .fillMaxWidth()
                        .padding(vertical = paddingDefault, horizontal = paddingDefault),
                ) {
                    MainActionButton(
                        title = action.label(context),
                        onClick = {
                            onClose()
                            when (action) {
                                is GemInfoAction.LearnMore -> uriHandler.open(context, action.url)
                                else -> onAction?.invoke(action)
                            }
                        },
                    )
                }
            }
        }
    }
}

@Composable
private fun InfoSheetIcon(image: GemInfoImage) {
    Box(
        modifier = Modifier.size(infoSheetIconSize),
        contentAlignment = Alignment.Center,
    ) {
        IconWithBadge(
            icon = image.iconModel(),
            placeholder = image.placeholder,
            supportIcon = image.badgeIconModel(),
            size = infoSheetIconSize,
            badgeBackgroundColor = MaterialTheme.colorScheme.background,
        )
    }
}
