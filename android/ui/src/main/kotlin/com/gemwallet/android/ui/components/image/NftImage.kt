package com.gemwallet.android.ui.components.image

import androidx.compose.foundation.Image
import androidx.compose.runtime.Composable
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.remember
import androidx.compose.ui.Modifier
import androidx.compose.ui.layout.ContentScale
import coil3.compose.AsyncImagePainter
import coil3.compose.LocalPlatformContext
import coil3.compose.rememberAsyncImagePainter
import coil3.compose.rememberConstraintsSizeResolver
import coil3.request.ImageRequest

@Composable
fun NftImage(source: NftImageSource, modifier: Modifier = Modifier) {
    val (url, name) = source
    val context = LocalPlatformContext.current
    val sizeResolver = rememberConstraintsSizeResolver()
    val request = remember(context, url, sizeResolver) {
        ImageRequest.Builder(context).data(url.takeIf { it.isNotBlank() }).size(sizeResolver).build()
    }
    val painter = rememberAsyncImagePainter(model = request, contentScale = ContentScale.Crop)
    val state by painter.state.collectAsState()
    val sizedModifier = modifier.then(sizeResolver)

    when (state) {
        is AsyncImagePainter.State.Success -> Image(
            painter = painter,
            contentDescription = name,
            contentScale = ContentScale.Crop,
            modifier = sizedModifier,
        )

        is AsyncImagePainter.State.Loading -> NftImageLoading(modifier = sizedModifier)

        is AsyncImagePainter.State.Empty,
        is AsyncImagePainter.State.Error,
        -> NftImagePlaceholder(modifier = sizedModifier, name = name)
    }
}
