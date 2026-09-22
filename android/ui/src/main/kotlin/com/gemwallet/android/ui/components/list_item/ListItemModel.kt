package com.gemwallet.android.ui.components.list_item

import androidx.annotation.DrawableRes
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp
import com.gemwallet.android.ui.components.InfoSheetEntity
import com.gemwallet.android.ui.components.image.ListItemImageView
import com.gemwallet.android.ui.components.list_item.property.PropertyDataText
import com.gemwallet.android.ui.components.list_item.property.PropertyItem
import com.gemwallet.android.ui.components.list_item.property.PropertyTitleText
import com.gemwallet.android.ui.components.progress.CircularProgressIndicator14
import com.gemwallet.android.ui.components.progress.CircularProgressIndicator16
import com.gemwallet.android.ui.icons.AppIcons
import com.gemwallet.android.ui.models.ListPosition
import com.gemwallet.android.ui.theme.Spacer6
import com.gemwallet.android.ui.theme.Spacer8
import com.gemwallet.android.ui.theme.alpha10
import com.gemwallet.android.ui.theme.iconSize
import com.gemwallet.android.ui.theme.listItemIconSize
import com.gemwallet.android.ui.theme.paddingHalfSmall
import com.gemwallet.android.ui.theme.pendingColor
import com.gemwallet.android.ui.theme.secondaryFaded
import com.gemwallet.android.ui.theme.smallIconSize
import com.gemwallet.android.ui.theme.space2
import com.gemwallet.android.ui.theme.space6
import com.wallet.core.primitives.AssetId

data class ListItemModel(
    val title: String,
    val titleStyle: ListItemTextStyle = ListItemTextStyle.Body,
    val titleLineLimit: Int? = 1,
    val titleTag: String? = null,
    val titleTagStyle: ListItemTextStyle = ListItemTextStyle.Secondary,
    val titleTagType: ListItemTagType = ListItemTagType.None,
    val titleExtra: String? = null,
    val titleExtraStyle: ListItemTextStyle = ListItemTextStyle.Secondary,
    val titleExtraLineLimit: Int? = null,
    val subtitle: String? = null,
    val subtitleStyle: ListItemTextStyle = ListItemTextStyle.Secondary,
    val subtitleExtra: String? = null,
    val subtitleExtraStyle: ListItemTextStyle = ListItemTextStyle.Secondary,
    val subtitleSuffix: String? = null,
    val subtitleSuffixStyle: ListItemTextStyle = ListItemTextStyle.Secondary,
    val subtitleTagType: ListItemTagType = ListItemTagType.None,
    val image: ListItemImage? = null,
    val info: InfoSheetEntity? = null,
)

private val listItemTagIconSize = 18.dp

enum class ListItemTextStyle {
    Body,
    Secondary,
    Positive,
    Negative,
    Warning,
    Primary,
    Faded,
}

enum class ListItemSymbol {
    Check,
    Paste,
    QrScanner,
    Pin,
    Unpin,
    AddCircle,
    Buy,
    Swap,
    Receive,
    Add,
    Article,
    Error,
    CheckCircle,
    Notifications,
    NotificationsOutlined,
    Warning,
    CurrencyBitcoin,
    Close,
}

enum class ListItemTagType {
    None,
    Progress,
    Pending,
}

sealed interface ListItemImage {
    val style: ListItemImageStyle

    data class Asset(val assetId: AssetId) : ListItemImage {
        override val style: ListItemImageStyle = ListItemImageStyle.Avatar
    }

    data class Url(val url: String, val placeholder: String? = null) : ListItemImage {
        override val style: ListItemImageStyle = ListItemImageStyle.Avatar
    }

    data class Stored(val name: String, val placeholder: String? = null) : ListItemImage {
        override val style: ListItemImageStyle = ListItemImageStyle.Avatar
    }

    data class Emoji(val glyph: String, val backgroundColor: Int? = null) : ListItemImage {
        override val style: ListItemImageStyle = ListItemImageStyle.Avatar
    }

    data class Initials(val text: String) : ListItemImage {
        override val style: ListItemImageStyle = ListItemImageStyle.Avatar
    }

    data class Symbol(val symbol: ListItemSymbol, val tint: ListItemTextStyle = ListItemTextStyle.Body, override val style: ListItemImageStyle = ListItemImageStyle.Glyph) : ListItemImage

    data class Drawable(@DrawableRes val id: Int, override val style: ListItemImageStyle = ListItemImageStyle.Settings) : ListItemImage
}

enum class ListItemImageStyle(val size: Dp, val isRounded: Boolean = false) {
    Avatar(listItemIconSize, isRounded = true),
    Banner(listItemIconSize),
    Action(listItemIconSize),
    Settings(iconSize),
    Glyph(smallIconSize),
}

@Composable
fun ListItemTextStyle.color(): Color = when (this) {
    ListItemTextStyle.Body -> MaterialTheme.colorScheme.onSurface
    ListItemTextStyle.Secondary -> MaterialTheme.colorScheme.secondary
    ListItemTextStyle.Positive -> MaterialTheme.colorScheme.tertiary
    ListItemTextStyle.Negative -> MaterialTheme.colorScheme.error
    ListItemTextStyle.Warning -> pendingColor
    ListItemTextStyle.Primary -> MaterialTheme.colorScheme.primary
    ListItemTextStyle.Faded -> MaterialTheme.colorScheme.secondaryFaded
}

@Composable
fun ListItem(model: ListItemModel, listPosition: ListPosition, modifier: Modifier = Modifier, minHeight: Dp = Dp.Unspecified, accessory: (@Composable () -> Unit)? = null) {
    if (model.image == null && model.titleExtra == null && model.titleTag == null && model.subtitleExtra == null) {
        PropertyItem(
            modifier = modifier,
            title = { PropertyTitleText(text = model.title, color = model.titleStyle.color(), info = model.info) },
            data = if (model.subtitle == null && model.subtitleSuffix == null && accessory == null && model.subtitleTagType == ListItemTagType.None) {
                null
            } else {
                { PropertyDataText(text = model.subtitle ?: "", color = model.subtitleStyle.color(), badge = subtitleBadge(model, accessory)) }
            },
            listPosition = listPosition,
        )
        return
    }
    ListItem(
        modifier = modifier,
        listPosition = listPosition,
        minHeight = minHeight,
        leading = model.image?.let { image -> { ListItemImageView(image = image) } },
        title = {
            ListItemTitleText(
                text = model.title,
                color = model.titleStyle.color(),
                style = MaterialTheme.typography.bodyLarge,
                maxLines = model.titleLineLimit ?: Int.MAX_VALUE,
                titleBadge = model.titleTag?.let { { TitleTag(it, model.titleTagStyle, model.titleTagType) } },
            )
        },
        subtitle = model.titleExtra?.let {
            { ListItemSupportText(text = it, color = model.titleExtraStyle.color(), maxLines = model.titleExtraLineLimit ?: Int.MAX_VALUE) }
        },
        trailing = if (model.subtitle == null && model.subtitleExtra == null && accessory == null) {
            null
        } else {
            {
                if (model.subtitle != null || model.subtitleSuffix != null || model.subtitleExtra != null) {
                    Column(horizontalAlignment = Alignment.End) {
                        if (model.subtitle != null || model.subtitleSuffix != null) {
                            Row(verticalAlignment = Alignment.CenterVertically) {
                                model.subtitle?.let {
                                    Text(
                                        text = it,
                                        style = MaterialTheme.typography.bodyLarge,
                                        color = model.subtitleStyle.color(),
                                        maxLines = 1,
                                    )
                                }
                                model.subtitleSuffix?.let { SubtitleSuffix(it, model.subtitleSuffixStyle) }
                            }
                        }
                        model.subtitleExtra?.let { ListItemSupportText(text = it, color = model.subtitleExtraStyle.color()) }
                    }
                }
                SubtitleTag(model)
                accessory?.invoke()
            }
        },
    )
}

@Composable
private fun SubtitleSuffix(text: String, style: ListItemTextStyle) {
    Spacer(modifier = Modifier.width(paddingHalfSmall))
    Text(
        text = text,
        color = style.color(),
        style = MaterialTheme.typography.bodyLarge,
        maxLines = 1,
    )
}

private fun subtitleBadge(model: ListItemModel, accessory: (@Composable () -> Unit)?): (@Composable () -> Unit)? {
    if (model.subtitleSuffix == null && model.subtitleTagType == ListItemTagType.None && accessory == null) {
        return null
    }
    return {
        model.subtitleSuffix?.let { SubtitleSuffix(it, model.subtitleSuffixStyle) }
        when (model.subtitleTagType) {
            ListItemTagType.None -> accessory?.invoke()

            ListItemTagType.Progress, ListItemTagType.Pending -> {
                SubtitleTag(model)
                accessory?.invoke()
            }
        }
    }
}

@Composable
private fun SubtitleTag(model: ListItemModel) {
    when (model.subtitleTagType) {
        ListItemTagType.Progress -> {
            Spacer8()
            CircularProgressIndicator16(color = model.subtitleStyle.color())
        }

        ListItemTagType.Pending -> {
            Spacer8()
            Icon(
                imageVector = AppIcons.ClockBadgeExclamation,
                contentDescription = null,
                modifier = Modifier.size(listItemTagIconSize),
                tint = pendingColor,
            )
        }

        ListItemTagType.None -> Unit
    }
}

@Composable
private fun TitleTag(text: String, style: ListItemTextStyle, type: ListItemTagType) {
    when (type) {
        ListItemTagType.Progress -> {
            Spacer6()
            CircularProgressIndicator14()
            return
        }

        ListItemTagType.None, ListItemTagType.Pending -> Unit
    }
    when (style) {
        ListItemTextStyle.Primary -> Text(
            modifier = Modifier
                .padding(start = paddingHalfSmall)
                .background(
                    color = style.color().copy(alpha = alpha10),
                    shape = RoundedCornerShape(space6),
                )
                .padding(horizontal = space6, vertical = space2),
            text = text,
            color = style.color(),
            style = MaterialTheme.typography.bodyMedium,
        )

        ListItemTextStyle.Positive,
        ListItemTextStyle.Negative,
        ListItemTextStyle.Warning,
        -> Text(
            modifier = Modifier
                .padding(start = paddingHalfSmall)
                .background(
                    color = style.color().copy(alpha = alpha10),
                    shape = RoundedCornerShape(space6),
                )
                .padding(horizontal = paddingHalfSmall, vertical = space2),
            text = text,
            color = style.color(),
            maxLines = 1,
            overflow = TextOverflow.Ellipsis,
            style = MaterialTheme.typography.labelMedium,
        )

        ListItemTextStyle.Body,
        ListItemTextStyle.Secondary,
        ListItemTextStyle.Faded,
        -> Badge(text)
    }
}
