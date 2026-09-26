package com.gemwallet.android.ui.components.list_head

import androidx.compose.runtime.Composable
import androidx.compose.ui.platform.LocalContext
import com.gemwallet.android.ui.components.HideToggle
import com.gemwallet.android.ui.localization.string
import com.gemwallet.android.ui.style.textStyle
import uniffi.gemstone.GemValueHeader
import uniffi.gemstone.GemValueHeaderIcon

@Composable
fun ValueListHead(header: GemValueHeader, hideToggle: HideToggle? = null, onClick: (() -> Unit)? = null, onSubtitleClick: (() -> Unit)? = null, actions: (@Composable () -> Unit)? = null) {
    val context = LocalContext.current
    val subtitle = header.subtitle?.text?.string(context)
    val icon = header.icon
    AmountListHead(
        amount = header.title.string(context),
        hideToggle = hideToggle,
        equivalent = subtitle.takeIf { header.subtitleIcon == null },
        icon = when (icon) {
            is GemValueHeaderIcon.Asset -> icon.icon
            is GemValueHeaderIcon.Image -> icon.url
            null -> null
        },
        iconPlaceholder = (icon as? GemValueHeaderIcon.Image)?.placeholder,
        changedValue = subtitle.takeIf { header.subtitleIcon != null },
        changeStyle = header.subtitle?.tone?.textStyle() ?: com.gemwallet.android.ui.components.list_item.ListItemTextStyle.Secondary,
        onClick = onClick,
        onSubtitleClick = onSubtitleClick,
        actions = actions,
    )
}
