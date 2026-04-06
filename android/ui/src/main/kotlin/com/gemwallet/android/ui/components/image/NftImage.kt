package com.gemwallet.android.ui.components.image

import androidx.compose.foundation.Image
import androidx.compose.runtime.Composable
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.layout.ContentScale
import coil3.compose.AsyncImagePainter
import coil3.compose.rememberAsyncImagePainter

@Composable
fun NftImage(
    source: NftImageSource,
    modifier: Modifier = Modifier,
) {
    val (url, name) = source
    val painter = rememberAsyncImagePainter(model = url.takeIf { it.isNotBlank() })
    val state by painter.state.collectAsState()

    when (state) {
        is AsyncImagePainter.State.Success -> Image(
            painter = painter,
            contentDescription = name,
            contentScale = ContentScale.Crop,
            modifier = modifier,
        )
        is AsyncImagePainter.State.Loading -> NftImageLoading(modifier = modifier)
        is AsyncImagePainter.State.Empty,
        is AsyncImagePainter.State.Error -> NftImagePlaceholder(modifier = modifier, name = name)
    }
}
