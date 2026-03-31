package com.gemwallet.android.ui.components.image

import androidx.compose.foundation.layout.size
import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.runtime.remember
import androidx.compose.ui.Modifier
import androidx.compose.ui.geometry.Size
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.graphics.vector.rememberVectorPainter
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.platform.LocalDensity
import androidx.compose.ui.text.rememberTextMeasurer
import androidx.compose.ui.unit.Dp
import coil3.compose.AsyncImage
import coil3.request.CachePolicy
import coil3.request.ImageRequest
import coil3.request.transformations
import coil3.transform.CircleCropTransformation
import coil3.transform.Transformation
import com.gemwallet.android.domains.asset.getIconUrl
import com.gemwallet.android.ui.theme.alpha50
import com.gemwallet.android.ui.theme.iconSize
import com.wallet.core.primitives.Asset

private val DefaultCircleCropTransformation = CircleCropTransformation()

@Composable
fun AsyncImage(
    model: Any?,
    size: Dp? = iconSize,
    modifier: Modifier = Modifier,
    contentDescription: String = "",
    placeholderText: String? = null,
    errorImageVector: ImageVector? = null,
    transformation: Transformation? = DefaultCircleCropTransformation
) {
    if (model == null) {
        return
    }
    val requestData = if (model is Asset) model.getIconUrl() else model
    val context = LocalContext.current
    val density = LocalDensity.current
    val placeholderColor = MaterialTheme.colorScheme.secondary.copy(alpha = alpha50)
    val textMeasurer = rememberTextMeasurer()
    val placeholderSize = size?.let { with(density) { Size(it.toPx(), it.toPx()) } } ?: Size.Unspecified
    val placeholder = remember(placeholderText, placeholderColor, placeholderSize) {
        if (placeholderText.isNullOrEmpty()) {
            null
        } else {
            TextPainter(
                circleColor = placeholderColor,
                textMeasurer = textMeasurer,
                text = placeholderText,
                circleSize = placeholderSize
            )
        }
    }

    val error = if (errorImageVector == null) {
        placeholder
    } else {
        rememberVectorPainter(image = errorImageVector)
    }
    val request = remember(context, requestData, transformation) {
        ImageRequest.Builder(context)
            .data(requestData)
            .diskCachePolicy(policy = CachePolicy.ENABLED)
            .networkCachePolicy(policy = CachePolicy.ENABLED)
            .apply {
                if (transformation != null) {
                    transformations(transformation)
                }
            }
            .build()
    }
    AsyncImage(
        model = request,
        placeholder = placeholder,
        error = error,
        contentDescription = contentDescription,
        modifier = size?.let { modifier.size(size) } ?: modifier,
    )
}

@Composable
fun AsyncImage(
    model: Asset,
    size: Dp = iconSize,
    modifier: Modifier = Modifier,
    placeholderText: String? = model.symbol,
    errorImageVector: ImageVector? = null,
) {
    AsyncImage(
        model = model.getIconUrl(),
        size = size,
        contentDescription = "asset_icon",
        modifier = modifier,
        placeholderText = placeholderText,
        errorImageVector = errorImageVector
    )
}
