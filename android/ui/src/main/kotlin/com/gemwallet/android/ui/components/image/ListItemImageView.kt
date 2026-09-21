package com.gemwallet.android.ui.components.image

import androidx.compose.foundation.Image
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.unit.Dp
import com.gemwallet.android.domains.asset.icon
import com.gemwallet.android.ui.components.list_item.ListItemImage
import com.gemwallet.android.ui.components.list_item.ListItemImageStyle
import com.gemwallet.android.ui.components.list_item.ListItemSymbol
import com.gemwallet.android.ui.components.list_item.color
import com.gemwallet.android.ui.icons.AppIcons
import com.gemwallet.android.ui.theme.actionIconGlyphSize
import com.gemwallet.android.ui.theme.space12

@Composable
fun ListItemImageView(image: ListItemImage, modifier: Modifier = Modifier, style: ListItemImageStyle = image.style, size: Dp = style.size) {
    when (image) {
        is ListItemImage.Asset -> AsyncImage(
            model = image.assetId.iconModel(),
            modifier = modifier,
            size = size,
            placeholderText = image.assetId.icon().placeholder,
        )

        is ListItemImage.Url -> AsyncImage(model = image.url, modifier = modifier, size = size, placeholderText = image.placeholder)

        is ListItemImage.Stored -> AsyncImage(
            model = walletImageModel(LocalContext.current, image.name),
            modifier = modifier,
            size = size,
            placeholderText = image.placeholder,
        )

        is ListItemImage.Emoji -> EmojiView(
            emoji = image.glyph,
            modifier = modifier.size(size),
            background = image.backgroundColor?.let { Color(it) } ?: Color.Transparent,
            scale = AvatarScale.EMOJI,
        )

        is ListItemImage.Initials -> InitialsAvatar(text = image.text, size = size, modifier = modifier, placeholder = AppIcons.Person)

        is ListItemImage.Symbol -> if (style == ListItemImageStyle.Action) {
            Box(
                modifier = modifier
                    .size(size)
                    .background(color = MaterialTheme.colorScheme.primary, shape = RoundedCornerShape(space12)),
                contentAlignment = Alignment.Center,
            ) {
                Icon(
                    imageVector = image.symbol.vector(),
                    contentDescription = null,
                    modifier = Modifier.size(actionIconGlyphSize),
                    tint = MaterialTheme.colorScheme.onPrimary,
                )
            }
        } else {
            Icon(
                imageVector = image.symbol.vector(),
                contentDescription = null,
                modifier = modifier.size(size),
                tint = image.tint.color(),
            )
        }

        is ListItemImage.Drawable -> Image(
            painter = painterResource(image.id),
            contentDescription = null,
            modifier = if (style.isRounded) modifier.size(size).clip(CircleShape) else modifier.size(size),
        )
    }
}

@Composable
fun ListItemSymbol.vector(): ImageVector = when (this) {
    ListItemSymbol.Check -> AppIcons.Check
    ListItemSymbol.Paste -> AppIcons.ContentPaste
    ListItemSymbol.QrScanner -> AppIcons.QrCodeScanner
    ListItemSymbol.Pin -> AppIcons.PushPin
    ListItemSymbol.Unpin -> AppIcons.KeepOff
    ListItemSymbol.AddCircle -> AppIcons.AddCircleOutlined
    ListItemSymbol.Buy -> AppIcons.Buy
    ListItemSymbol.Swap -> AppIcons.SwapVert
    ListItemSymbol.Receive -> AppIcons.Receive
    ListItemSymbol.Add -> AppIcons.Add
    ListItemSymbol.Article -> AppIcons.Article
    ListItemSymbol.Error -> AppIcons.Error
    ListItemSymbol.CheckCircle -> AppIcons.CheckCircle
    ListItemSymbol.Notifications -> AppIcons.Notifications
    ListItemSymbol.NotificationsOutlined -> AppIcons.NotificationsOutlined
    ListItemSymbol.Warning -> AppIcons.Warning
    ListItemSymbol.CurrencyBitcoin -> AppIcons.CurrencyBitcoin
    ListItemSymbol.Close -> AppIcons.Close
}
