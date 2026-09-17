package com.gemwallet.android.ui.components.list_item

import androidx.annotation.DrawableRes
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.unit.Dp
import com.gemwallet.android.ui.components.InfoSheetEntity
import com.gemwallet.android.ui.components.image.ListItemImageView
import com.gemwallet.android.ui.components.list_item.property.PropertyDataText
import com.gemwallet.android.ui.components.list_item.property.PropertyItem
import com.gemwallet.android.ui.components.list_item.property.PropertyTitleText
import com.gemwallet.android.ui.components.progress.CircularProgressIndicator14
import com.gemwallet.android.ui.components.progress.CircularProgressIndicator16
import com.gemwallet.android.ui.models.ListPosition
import com.gemwallet.android.ui.theme.Spacer6
import com.gemwallet.android.ui.theme.Spacer8
import com.gemwallet.android.ui.theme.alpha10
import com.gemwallet.android.ui.theme.listItemIconSize
import com.gemwallet.android.ui.theme.paddingHalfSmall
import com.gemwallet.android.ui.theme.pendingColor
import com.gemwallet.android.ui.theme.secondaryFaded
import com.gemwallet.android.ui.theme.space2
import com.gemwallet.android.ui.theme.space6
import com.wallet.core.primitives.AssetId

data class ListItemModel(
    val title: String,
    val titleStyle: ListItemTextStyle = ListItemTextStyle.Body,
    val titleTag: String? = null,
    val titleTagStyle: ListItemTextStyle = ListItemTextStyle.Secondary,
    val titleTagType: ListItemTagType = ListItemTagType.None,
    val titleExtra: String? = null,
    val titleExtraStyle: ListItemTextStyle = ListItemTextStyle.Secondary,
    val subtitle: String? = null,
    val subtitleStyle: ListItemTextStyle = ListItemTextStyle.Secondary,
    val subtitleExtra: String? = null,
    val subtitleExtraStyle: ListItemTextStyle = ListItemTextStyle.Secondary,
    val subtitleTagType: ListItemTagType = ListItemTagType.None,
    val image: ListItemImage? = null,
    val info: InfoSheetEntity? = null,
)

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
}

sealed interface ListItemImage {
    data class Asset(val assetId: AssetId) : ListItemImage
    data class Url(val url: String, val placeholder: String? = null) : ListItemImage
    data class Stored(val name: String, val placeholder: String? = null) : ListItemImage
    data class Emoji(val glyph: String, val backgroundColor: Int? = null) : ListItemImage
    data class Initials(val text: String) : ListItemImage
    data class Symbol(val symbol: ListItemSymbol, val isFilled: Boolean = false) : ListItemImage
    data class Drawable(@DrawableRes val id: Int, val isRounded: Boolean = false) : ListItemImage
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
fun ListItem(
    model: ListItemModel,
    listPosition: ListPosition,
    modifier: Modifier = Modifier,
    minHeight: Dp = Dp.Unspecified,
    accessory: (@Composable () -> Unit)? = null,
) {
    if (model.image == null && model.titleExtra == null && model.titleTag == null && model.subtitleExtra == null) {
        PropertyItem(
            modifier = modifier,
            title = { PropertyTitleText(text = model.title, color = model.titleStyle.color(), info = model.info) },
            data = if (model.subtitle == null && accessory == null && model.subtitleTagType == ListItemTagType.None) {
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
        leading = model.image?.let { image -> { ListItemImageView(image = image, size = listItemIconSize) } },
        title = { ListItemTitleText(text = model.title, color = model.titleStyle.color(), titleBadge = model.titleTag?.let { { TitleTag(it, model.titleTagStyle, model.titleTagType) } }) },
        subtitle = model.titleExtra?.let { { ListItemSupportText(text = it, color = model.titleExtraStyle.color()) } },
        trailing = if (model.subtitle == null && model.subtitleExtra == null && accessory == null) {
            null
        } else {
            {
                if (model.subtitle != null || model.subtitleExtra != null) {
                    Column(horizontalAlignment = Alignment.End) {
                        model.subtitle?.let {
                            Text(
                                text = it,
                                style = MaterialTheme.typography.bodyLarge,
                                color = model.subtitleStyle.color(),
                                maxLines = 1,
                            )
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

private fun subtitleBadge(model: ListItemModel, accessory: (@Composable () -> Unit)?): (@Composable () -> Unit)? = when (model.subtitleTagType) {
    ListItemTagType.None -> accessory
    ListItemTagType.Progress -> {
        {
            SubtitleTag(model)
            accessory?.invoke()
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
        ListItemTagType.None -> Unit
    }
    when (style) {
        ListItemTextStyle.Primary,
        ListItemTextStyle.Positive,
        ListItemTextStyle.Negative,
        ListItemTextStyle.Warning -> Text(
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
        ListItemTextStyle.Body,
        ListItemTextStyle.Secondary,
        ListItemTextStyle.Faded -> Badge(text)
    }
}
